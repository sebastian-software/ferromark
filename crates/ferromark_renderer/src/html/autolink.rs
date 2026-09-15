//! Bare-URL autolink scanner.
//!
//! Text nodes can optionally turn recognized URL prefixes into anchors. The scanner
//! first indexes possible leading bytes so most prose is skipped without repeated
//! prefix checks, then validates word boundaries and trims punctuation around matches.

use super::options::AutolinkPatterns;

mod index;

pub(in crate::html) use index::FirstByteIndex;

/// Scans `s` from `from` for the next position that begins one of the
/// registered URL prefixes at a word boundary, and returns the
/// `(match_start, url_end)` byte range with trailing punctuation trimmed.
///
/// The boundary rule mirrors common autolinkers: a match is only accepted
/// when the preceding byte (if any) is not an ASCII alphanumeric — so
/// `"see http://x"` matches but `"shttp://x"` doesn't. The URL extends to
/// the next whitespace, `<`, `>`, `"`, `'`, or backtick, and we then strip
/// trailing `.,;:!?` plus an unbalanced `)`, `]`, or `}`.
///
/// `index` skips ahead to the next byte that could start a pattern, so the
/// per-byte boundary and prefix checks below only run at real candidates
/// rather than across every byte of non-URL prose.
pub(super) fn find_autolink_match(
    s: &str,
    from: usize,
    patterns: AutolinkPatterns<'_>,
    index: &FirstByteIndex,
) -> Option<(usize, usize)> {
    match patterns {
        AutolinkPatterns::Defaults(patterns) => find_autolink_match_in(s, from, patterns, index),
        AutolinkPatterns::Custom(patterns) => find_autolink_match_in(s, from, patterns, index),
    }
}

fn find_autolink_match_in<P: AsRef<str>>(
    s: &str,
    from: usize,
    patterns: &[P],
    index: &FirstByteIndex,
) -> Option<(usize, usize)> {
    let bytes = s.as_bytes();
    let mut base = from;
    while base < bytes.len() {
        let rel = index.next(&bytes[base..])?;
        let i = base + rel;
        // One-byte look-ahead: most prose hits on a frequent candidate byte
        // ("the", "words"…) die here before any prefix comparison.
        if index.rejects_second(bytes[i], bytes.get(i + 1)) {
            base = i + 1;
            continue;
        }
        // Word boundary: the previous byte must not be ASCII alphanumeric.
        let is_boundary = i == 0 || !bytes[i - 1].is_ascii_alphanumeric();
        if is_boundary {
            for pat in patterns {
                let pat = pat.as_ref();
                let pat_bytes = pat.as_bytes();
                if pat_bytes.is_empty() {
                    continue;
                }
                if i + pat_bytes.len() <= bytes.len()
                    && bytes[i..i + pat_bytes.len()].eq_ignore_ascii_case(pat_bytes)
                {
                    let url_start = i;
                    let url_end = scan_url_end(s, i + pat_bytes.len());
                    // Require at least one byte beyond the scheme/prefix
                    // so `"http://"` on its own isn't auto-linked.
                    if url_end == i + pat_bytes.len() {
                        continue;
                    }
                    return Some((url_start, trim_trailing_punct(bytes, url_start, url_end)));
                }
            }
        }
        base = i + 1;
    }
    None
}

/// Extends a URL from `from` to the offset where it stops.
///
/// Non-ASCII characters normally belong to the URL — an IRI carries them
/// verbatim (`/wiki/日本語`) — with CJK sentence punctuation the exception.
fn scan_url_end(s: &str, from: usize) -> usize {
    let bytes = s.as_bytes();
    // An IRI host can start with Unicode immediately after its scheme. Avoid
    // SIMD setup when there is no ASCII prefix to skip.
    let mut end = if bytes.get(from).is_some_and(u8::is_ascii) {
        scan_ascii_url_prefix(bytes, from)
    } else {
        from
    };
    if end >= bytes.len() || bytes[end].is_ascii() {
        return end;
    }

    // A wide ASCII pass stops at the first high-bit byte. Resume the original
    // Unicode-aware walk there so IRI characters remain part of the URL while
    // CJK/fullwidth sentence punctuation still terminates it.
    while end < bytes.len() {
        let byte = bytes[end];
        if byte.is_ascii() {
            if !is_url_byte(byte) {
                break;
            }
            end += 1;
            continue;
        }
        let Some(ch) = s.get(end..).and_then(|rest| rest.chars().next()) else {
            break;
        };
        if ends_url(ch) {
            break;
        }
        end += ch.len_utf8();
    }
    end
}

#[cfg(target_arch = "aarch64")]
#[allow(unsafe_code)]
#[inline]
fn scan_ascii_url_prefix(bytes: &[u8], from: usize) -> usize {
    use std::arch::aarch64::*;

    let end = bytes.len();
    let mut i = from;
    // SAFETY: full vectors are loaded only when 16 bytes remain; the overlapping
    // tail starts at len - 16 and masks positions before the unconsumed cursor.
    // AArch64 guarantees NEON support. No pointer outlives the borrowed slice.
    unsafe {
        let high_bit = vdupq_n_u8(0x7f);
        let classify = |v: uint8x16_t| {
            let mut stop = vcgtq_u8(v, high_bit);
            for terminator in [b' ', b'\t', b'\n', b'\r', b'<', b'>', b'"', b'\'', b'`'] {
                stop = vorrq_u8(stop, vceqq_u8(v, vdupq_n_u8(terminator)));
            }
            let stop = vtstq_u8(stop, stop);
            let narrow = vshrn_n_u16(vreinterpretq_u16_u8(stop), 4);
            vget_lane_u64(vreinterpret_u64_u8(narrow), 0)
        };

        while i + 16 <= end {
            let mask = classify(vld1q_u8(bytes.as_ptr().add(i)));
            if mask != 0 {
                return i + (mask.trailing_zeros() / 4) as usize;
            }
            i += 16;
        }
        if i < end && end >= 16 {
            let base = end - 16;
            let mask =
                classify(vld1q_u8(bytes.as_ptr().add(base))) & (u64::MAX << ((i - base) * 4));
            if mask != 0 {
                return base + (mask.trailing_zeros() / 4) as usize;
            }
            return end;
        }
    }
    scan_ascii_url_prefix_scalar(bytes, i)
}

#[cfg(not(target_arch = "aarch64"))]
#[inline]
fn scan_ascii_url_prefix(bytes: &[u8], from: usize) -> usize {
    scan_ascii_url_prefix_scalar(bytes, from)
}

#[inline]
fn scan_ascii_url_prefix_scalar(bytes: &[u8], from: usize) -> usize {
    let end = bytes.len();
    let mut i = from;
    while i + 8 <= end {
        let chunk = &bytes[i..i + 8];
        let mut stop = 0u8;
        for (offset, &byte) in chunk.iter().enumerate() {
            if !byte.is_ascii() || !is_url_byte(byte) {
                stop |= 1 << offset;
            }
        }
        if stop != 0 {
            return i + stop.trailing_zeros() as usize;
        }
        i += 8;
    }
    while i < end && bytes[i].is_ascii() && is_url_byte(bytes[i]) {
        i += 1;
    }
    i
}

#[inline]
fn is_url_byte(byte: u8) -> bool {
    !matches!(
        byte,
        b' ' | b'\t' | b'\n' | b'\r' | b'<' | b'>' | b'"' | b'\'' | b'`'
    )
}

/// Punctuation that ends a bare URL the way ASCII whitespace does.
///
/// Trailing-punctuation handling is defined for ASCII only, and CJK prose
/// puts no space between a URL and the `。` that closes the sentence — so
/// without this the rest of the sentence is swallowed into the link.
///
/// Mirrored by the GFM autolink scan in `ferromark_parser`.
const fn ends_url(ch: char) -> bool {
    matches!(
        ch,
        // CJK symbols and punctuation: the ideographic space, 、。〈〉《》
        // 「」『』【】 and friends.
        '\u{3000}'..='\u{303F}'
        // Fullwidth ！＂＃＄％＆＇（）＊＋，－．／
        | '\u{FF01}'..='\u{FF0F}'
        // Fullwidth ：；＜＝＞？＠
        | '\u{FF1A}'..='\u{FF20}'
        // Fullwidth ［＼］＾＿｀
        | '\u{FF3B}'..='\u{FF40}'
        // Fullwidth ｛｜｝～｟｠ and halfwidth ｡｢｣､･
        | '\u{FF5B}'..='\u{FF65}'
    )
}

fn trim_trailing_punct(bytes: &[u8], start: usize, mut end: usize) -> usize {
    // Count lazily: the common case has no closing bracket. Each bracket type
    // is counted at most once, then updated as its trailing closers are removed.
    // This bounds trimming to three linear scans even for mixed bracket runs.
    let mut counts: [Option<(usize, usize)>; 3] = [None; 3];
    while end > start {
        match bytes[end - 1] {
            b'.' | b',' | b';' | b':' | b'!' | b'?' => end -= 1,
            close @ (b')' | b']' | b'}') => {
                let (slot, open) = match close {
                    b')' => (0, b'('),
                    b']' => (1, b'['),
                    _ => (2, b'{'),
                };
                let (opens, closes) = counts[slot].get_or_insert_with(|| {
                    let mut opens = 0;
                    let mut closes = 0;
                    for &byte in &bytes[start..end] {
                        if byte == open {
                            opens += 1;
                        } else if byte == close {
                            closes += 1;
                        }
                    }
                    (opens, closes)
                });
                if *closes > *opens {
                    *closes -= 1;
                    end -= 1;
                } else {
                    break;
                }
            }
            _ => break,
        }
    }
    end
}

#[cfg(test)]
mod trimming_tests {
    use super::trim_trailing_punct;

    fn original(bytes: &[u8], start: usize, mut end: usize) -> usize {
        while end > start {
            let b = bytes[end - 1];
            match b {
                b'.' | b',' | b';' | b':' | b'!' | b'?' => end -= 1,
                b')' | b']' | b'}' => {
                    let (open, close) = match b {
                        b')' => (b'(', b')'),
                        b']' => (b'[', b']'),
                        _ => (b'{', b'}'),
                    };
                    // Strip the closing bracket only when it has no unmatched
                    // partner inside the URL — a single pass over the slice is
                    // simpler than two `filter().count()` walks and avoids the
                    // `naive_bytecount` clippy lint.
                    let mut opens = 0usize;
                    let mut closes = 0usize;
                    for &x in &bytes[start..end - 1] {
                        if x == open {
                            opens += 1;
                        } else if x == close {
                            closes += 1;
                        }
                    }
                    if closes >= opens {
                        end -= 1;
                    } else {
                        break;
                    }
                }
                _ => break,
            }
        }
        end
    }

    #[test]
    fn matches_original_for_exhaustive_bracket_and_punctuation_tails() {
        const ALPHABET: &[u8] = b"()[]{}.!a";
        let mut bytes = [0u8; 9];
        for length in 0..=6u32 {
            for mut code in 0..ALPHABET.len().pow(length) {
                bytes[..3].copy_from_slice(b"x:/");
                for byte in &mut bytes[3..3 + length as usize] {
                    *byte = ALPHABET[code % ALPHABET.len()];
                    code /= ALPHABET.len();
                }
                let input = &bytes[..3 + length as usize];
                for start in [0, 3] {
                    assert_eq!(
                        trim_trailing_punct(input, start, input.len()),
                        original(input, start, input.len()),
                        "{input:?}, start {start}"
                    );
                }
            }
        }
    }

    #[test]
    fn matches_original_for_long_unbalanced_and_unicode_urls() {
        let mut bytes =
            b"prefix https://example.org/\xe6\x97\xa5\xe6\x9c\xac\xe8\xaa\x9e/([{".to_vec();
        for i in 0..4_096 {
            bytes.push(b")]}.!?"[i % 6]);
        }
        for end in (8..bytes.len()).step_by(37) {
            assert_eq!(
                trim_trailing_punct(&bytes, 7, end),
                original(&bytes, 7, end)
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scalar_oracle(s: &str, from: usize) -> usize {
        let bytes = s.as_bytes();
        let mut end = from;
        while end < bytes.len() {
            let byte = bytes[end];
            if byte.is_ascii() {
                if !is_url_byte(byte) {
                    break;
                }
                end += 1;
                continue;
            }
            let Some(ch) = s.get(end..).and_then(|rest| rest.chars().next()) else {
                break;
            };
            if ends_url(ch) {
                break;
            }
            end += ch.len_utf8();
        }
        end
    }

    #[test]
    fn matches_scalar_for_every_byte_at_every_offset() {
        for value in 0u8..=0x7f {
            for marker_at in 0..40 {
                let mut bytes = [b'x'; 40];
                bytes[marker_at] = value;
                let text = std::str::from_utf8(&bytes).unwrap();
                for from in 0..=bytes.len() {
                    assert_eq!(scan_url_end(text, from), scalar_oracle(text, from));
                }
            }
        }
    }

    #[test]
    fn matches_scalar_for_ascii_tails_and_terminators() {
        let terminators = [b' ', b'\t', b'\n', b'\r', b'<', b'>', b'"', b'\'', b'`'];
        for len in 0..=48 {
            for &terminator in &terminators {
                let mut bytes = [b'a'; 48];
                if len > 0 {
                    bytes[len - 1] = terminator;
                }
                let text = std::str::from_utf8(&bytes[..len]).unwrap();
                for from in 0..=len {
                    assert_eq!(scan_url_end(text, from), scalar_oracle(text, from));
                }
            }
        }
    }

    #[test]
    fn preserves_mixed_utf8_and_cjk_boundaries() {
        for text in [
            "https://example.test/wiki/日本語。next",
            "https://example.test/é中/path`tail",
            "https://example.test/中\u{3000}next",
            "https://example.test/日本語/fullwidth：tail",
        ] {
            for from in 0..=text.len() {
                assert_eq!(
                    scan_url_end(text, from),
                    scalar_oracle(text, from),
                    "text {text:?}, from {from}"
                );
            }
        }
    }
}
