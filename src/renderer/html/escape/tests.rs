//! Differential tests: every escape path must agree with a plain
//! byte-at-a-time reference, including the vector classifiers.

use super::url::tests::reference_url;
use super::*;

/// Straightforward byte-at-a-time escaper the SWAR scanner must match.
///
/// Every escaped byte is ASCII, so copying the rest through verbatim at
/// the byte level keeps multi-byte sequences intact.
fn reference(s: &str, flags: &[u8; 256], table: &[&'static str; 256]) -> String {
    let mut out: Vec<u8> = Vec::new();
    for byte in s.bytes() {
        if flags[byte as usize] != 0 {
            out.extend_from_slice(table[byte as usize].as_bytes());
        } else {
            out.push(byte);
        }
    }
    String::from_utf8(out).expect("escaping only rewrites ASCII bytes")
}

fn check(s: &str) {
    let mut text = String::new();
    write_escaped_into(&mut text, s);
    assert_eq!(
        text,
        reference(s, &ESCAPE_FLAG, &ESCAPE_TABLE),
        "text escape: {s:?}"
    );

    let mut url = String::new();
    write_url_escaped_into(&mut url, s);
    assert_eq!(url, reference_url(s), "url escape: {s:?}");

    let mut attributes = String::from("prefix:");
    let mut expected = attributes.clone();
    for ch in s.chars() {
        match ch {
            '&' => expected.push_str("&amp;"),
            '<' => expected.push_str("&lt;"),
            '>' => expected.push_str("&gt;"),
            '"' => expected.push_str("&quot;"),
            '\'' => expected.push_str("&#39;"),
            '\r' => expected.push_str("&#13;"),
            '\n' => expected.push_str("&#10;"),
            _ => expected.push(ch),
        }
    }
    write_attribute_escaped_into(&mut attributes, s);
    assert_eq!(attributes, expected, "attribute escape: {s:?}");
}

#[test]
fn matches_reference_on_fixtures() {
    for case in [
        "",
        "a",
        "&",
        "<>",
        "plain ascii text with no escapes at all",
        "&<>\"'",
        "a&b<c>d\"e'f",
        "exactly-8b",
        "seven77",
        "&&&&&&&&&&&&&&&&",
        "trailing escape &",
        "& leading escape",
        // The folded masks admit no extra bytes, but these neighbours are
        // the ones a sloppy fold would leak: 0x21 0x23 0x25 0x3D 0x3F.
        "!#%=?",
        "!#%=? &<>\"' !#%=?",
        "\r\n",
        "\n\r\r\n\n",
        "日本語\r\n<&>\"'\n🙂\r",
    ] {
        check(case);
    }
}

#[test]
fn runs_of_every_length_are_copied_intact() {
    for len in 0..=200 {
        let run: String = (0..len)
            .map(|i| char::from(b'a' + u8::try_from(i % 26).unwrap()))
            .collect();
        for (prefix, suffix) in [("", ""), ("&", ""), ("", "<"), ("'", "\"")] {
            let source = format!("{prefix}{run}{suffix}");
            check(&source);
            let mut out = String::from("existing-content");
            write_escaped_into(&mut out, &source);
            assert_eq!(
                out,
                format!(
                    "existing-content{}",
                    reference(&source, &ESCAPE_FLAG, &ESCAPE_TABLE)
                )
            );
        }
    }
}

#[test]
fn matches_reference_on_borrow_propagation_shapes() {
    // `has_zero` can flag a 0x01 lane that borrowed from a real match
    // below it. These interleavings put such bytes directly after a match
    // inside the same word, which is where a wrong lane index would show.
    for filler in ["!", "#", "%", "=", "?", "\x01", "\x00"] {
        for needle in ["&", "<", ">", "\"", "'", " "] {
            for lead in 0..9 {
                let mut s = "x".repeat(lead);
                s.push_str(needle);
                for _ in 0..8 {
                    s.push_str(filler);
                }
                s.push_str(needle);
                check(&s);
            }
        }
    }
}

#[test]
fn matches_reference_across_the_overlapping_tail_read() {
    // The sub-word tail is covered by re-reading the last eight bytes
    // with the already-cleared lanes masked off, which is exactly where
    // a masked-off match can borrow into the lane above it. The pairs
    // below are the ones that can do it: after the mask folds, `<`/`>`
    // leave a `0x01` lane on a following `=` or `?`, and `"` on a
    // following `#`. One is placed at every offset of every length so
    // each lands on both sides of the tail boundary.
    for pair in ["<=", "<?", ">=", ">?", "\"#"] {
        for len in 2..40usize {
            for at in 0..len - 1 {
                let mut s = "x".repeat(len);
                s.replace_range(at..at + 2, pair);
                check(&s);
            }
        }
    }
}

#[test]
fn matches_reference_for_every_byte_value_at_every_offset() {
    // The vector paths classify from nibble pairs rather than the flag
    // table, so a wrong entry would admit or drop a byte the table
    // disagrees with. Check all 256 values at every offset of a buffer
    // long enough to cross the 16-byte step and land in the overlapping
    // tail, and at lengths below one vector so the word path is covered
    // too.
    for value in 0..=255u8 {
        for len in [1usize, 7, 8, 15, 16, 17, 33] {
            for at in 0..len {
                let mut buffer = vec![b'x'; len];
                buffer[at] = value;
                check(&String::from_utf8_lossy(&buffer));
            }
        }
    }
}

#[test]
fn matches_reference_on_pseudorandom_bytes() {
    // xorshift over the printable-plus-needles range; deterministic so a
    // failure is reproducible.
    let mut state = 0x2545_F491_4F6C_DD1Du64;
    let alphabet: Vec<u8> = (0x20u8..0x7f).chain(*b"&<>\"' \r\n").collect();
    for len in 0..200 {
        let mut s = String::with_capacity(len);
        for _ in 0..len {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            s.push(alphabet[(state % alphabet.len() as u64) as usize] as char);
        }
        check(&s);
    }
}

#[test]
fn matches_reference_on_multibyte_utf8() {
    check("日本語のテキスト");
    check("emoji 🎉 and <tags> mixed");
    check("café & naïve — \"quoted\"");
}

#[test]
fn matches_reference_across_the_short_run_copy_boundaries() {
    // `push_run` copies runs of 16 bytes or fewer with overlapping loads
    // instead of `memmove`, switching strategy at 1, 4, 8 and 16 bytes. A
    // run is the gap between two escapes, so bracketing a plain run of
    // every length puts each of those boundaries through the mid-loop copy,
    // the leading copy and the trailing copy in turn.
    for len in 0..=40usize {
        let run = "x".repeat(len);
        check(&run);
        check(&format!("&{run}"));
        check(&format!("{run}&"));
        check(&format!("&{run}&"));
        check(&format!("<{run}>{run}\""));
    }
}

#[test]
fn short_run_copy_keeps_multibyte_sequences_intact() {
    // The overlapping copy is byte-oriented, so anything that split a
    // multi-byte sequence would surface as invalid UTF-8 rather than a
    // wrong escape. Sweep lengths that straddle every branch boundary.
    for filler in ["é", "→", "🙂", "日本語"] {
        for count in 0..=12usize {
            let run = filler.repeat(count);
            check(&run);
            check(&format!("&{run}<"));
        }
    }
}

/// Whether this target scans inputs of 16 bytes or more with a vector
/// classifier (NEON or SSE2) rather than the word scan alone.
const VECTOR_SCAN: bool = cfg!(any(
    target_arch = "aarch64",
    all(target_arch = "x86_64", target_feature = "sse2")
));

/// Checks one escaper's scanner against its flag table, which is the
/// definition of its needle set.
///
/// `first_flagged` must report the first flagged byte at or after `from`
/// (or the length), and the vector path on its own must answer the same for
/// every input of at least 16 bytes and step aside below that. Every length
/// up to 200 is scanned clean from every start, then with one of `members`
/// at each 16-byte step edge and anywhere in the final 17 bytes, where the
/// overlapping tail re-reads what the loop cleared — starting before, at
/// and just after the member, so a member before `from` must be ignored —
/// and finally made of nothing but one member.
pub(super) fn assert_scan_matches_flags(
    mask_of: impl Fn(u64) -> u64 + Copy,
    flags: &[u8; 256],
    needles: impl NeedleSet,
    members: &[u8],
) {
    const FILLER: u8 = b'x';
    assert_eq!(flags[usize::from(FILLER)], 0);
    let check = |bytes: &[u8], from: usize| {
        let len = bytes.len();
        let expected = (from..len)
            .find(|&at| flags[usize::from(bytes[at])] != 0)
            .unwrap_or(len);
        assert_eq!(
            first_flagged(bytes, from, mask_of, flags, needles),
            expected,
            "scan from {from} of {bytes:?}"
        );
        assert_eq!(
            first_flagged_simd(bytes, from, needles),
            (VECTOR_SCAN && len >= 16).then_some(expected),
            "vector scan from {from} of {bytes:?}"
        );
    };
    for len in 0..=200usize {
        let mut buffer = vec![FILLER; len];
        for from in 0..=len {
            check(&buffer, from);
        }
        for at in (0..len).filter(|&at| at % 16 == 0 || at % 16 == 15 || at + 17 >= len) {
            for &member in members {
                assert_ne!(flags[usize::from(member)], 0, "{member:#04x}");
                buffer[at] = member;
                for from in [0, at.saturating_sub(1), at, at + 1] {
                    check(&buffer, from);
                }
            }
            buffer[at] = FILLER;
        }
        for &member in members {
            let dense = vec![member; len];
            for from in 0..=len {
                check(&dense, from);
            }
        }
    }
}

/// Checks a needle set's SSE2 classifier against its flag table: every
/// byte value in every lane, once among non-members and once among members,
/// and every run of sixteen consecutive values, so each value also meets
/// every lane between its neighbours.
#[cfg(all(target_arch = "x86_64", target_feature = "sse2"))]
#[allow(unsafe_code)]
pub(super) fn assert_lanes_match_flags(needles: impl NeedleSet, flags: &[u8; 256]) {
    let check = |bytes: [u8; 16]| {
        // SAFETY: an unaligned load of exactly the sixteen bytes of `bytes`.
        let vector = unsafe { std::arch::x86_64::_mm_loadu_si128(bytes.as_ptr().cast()) };
        let expected = (0..16)
            .filter(|&lane| flags[usize::from(bytes[lane])] != 0)
            .fold(0u32, |mask, lane| mask | 1 << lane);
        assert_eq!(needles.lanes(vector), expected, "lanes of {bytes:02x?}");
    };
    let member = (0..=255u8).find(|&b| flags[usize::from(b)] != 0).unwrap();
    let non_member = (0..=255u8).find(|&b| flags[usize::from(b)] == 0).unwrap();
    for background in [non_member, member] {
        for value in 0..=255u8 {
            for lane in 0..16 {
                let mut bytes = [background; 16];
                bytes[lane] = value;
                check(bytes);
            }
        }
    }
    for start in 0..=255u8 {
        check(std::array::from_fn(|lane| {
            start.wrapping_add(u8::try_from(lane).unwrap())
        }));
    }
}

#[test]
fn text_scan_matches_flag_table_across_vector_steps_and_tails() {
    assert_scan_matches_flags(escape_mask, &ESCAPE_FLAG, EscapeNeedles, b"&<>\"'");
}

#[cfg(all(target_arch = "x86_64", target_feature = "sse2"))]
#[test]
fn text_sse2_classifier_matches_flag_table_in_every_lane() {
    assert_lanes_match_flags(EscapeNeedles, &ESCAPE_FLAG);
}

#[test]
fn every_needle_matches_reference_across_vector_steps_and_tails() {
    // Each byte either escaper replaces, and UTF-8 sequences of two, three
    // and four bytes, which URL escaping percent encodes, placed at the
    // 16-byte step edges and anywhere in the final vector of every length
    // up to 200: the vector loop, its overlapping tail and the word scan
    // below 16 bytes each meet every needle, both when a scan starts at
    // the beginning and when it resumes after a replacement.
    for needle in [
        "&", "<", ">", "\"", "'", " ", "[", "\\", "]", "`", "é", "中", "🙂",
    ] {
        for len in needle.len()..=200 {
            let room = len - needle.len();
            for at in (0..=room).filter(|&at| at % 16 == 0 || at % 16 == 15 || at + 17 >= len) {
                let mut source = "x".repeat(at);
                source.push_str(needle);
                source.push_str(&"y".repeat(room - at));
                check(&source);
            }
        }
    }
}
