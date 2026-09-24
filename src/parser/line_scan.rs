//! Newline scanning for the document pre-pass.
//!
//! The pre-pass walks every line of the document — blanks and one-word lines
//! included — so it asks "where does this line end" tens of thousands of times.
//! At that access pattern a `memchr` call spends most of its time on per-call
//! setup rather than on scanning: walking lines is ~90% of the pre-pass.
//!
//! Short tails and portable targets use a SWAR (SIMD-within-a-register) word
//! test, which beats `memchr` by a third on the short-line case that dominated
//! the original measurement. aarch64 additionally runs a 32-byte NEON scan on
//! the rest: rust-book-style prose (median line ~57 bytes, nearly half of
//! lines 64+) pays four to eight SWAR iterations per line, and those documents
//! are exactly the ones that cannot bail out of the pre-pass because they
//! hold link reference definitions. x86-64 runs the same scan with SSE2,
//! which is part of its baseline, but out of line (see `line_end_x86`).
//!
//! That advantage is specific to this access pattern. Block parsing walks the
//! long prose lines of a paragraph, where `memchr`'s wider SIMD step overtakes
//! an eight-byte word test — converting those call sites measured 4-11% slower,
//! so they deliberately keep using `memchr`.

const ONES: u64 = 0x0101_0101_0101_0101;
const HIGH: u64 = 0x8080_8080_8080_8080;
const NEWLINES: u64 = (b'\n' as u64) * ONES;
const RETURNS: u64 = (b'\r' as u64) * ONES;

/// Sets `0x80` in every byte lane of `word` that is zero.
///
/// A lane holding `0x01` can also light up when a lower lane borrowed into it,
/// so callers must only consume the *lowest* set bit: a spurious lane can only
/// sit above a genuine zero lane, which means the lowest set bit is always a
/// true match.
#[inline]
const fn has_zero(word: u64) -> u64 {
    word.wrapping_sub(ONES) & !word & HIGH
}

/// Byte offset of the line terminator that starts at `from`, or `bytes.len()`
/// when the last line is unterminated.
#[inline]
pub(in crate::parser) fn line_end(bytes: &[u8], from: usize) -> usize {
    #[cfg(target_arch = "aarch64")]
    {
        // One NEON vector is the breakeven against SWAR. Shorter remainders
        // stay on the word test so a 5-byte last line does not pay a 16-byte
        // overlapping reload.
        if bytes.len() - from >= 16 {
            return line_end_neon(bytes, from);
        }
    }
    #[cfg(all(target_arch = "x86_64", target_feature = "sse2"))]
    {
        line_end_x86(bytes, from)
    }
    #[cfg(not(all(target_arch = "x86_64", target_feature = "sse2")))]
    {
        line_end_swar(bytes, from)
    }
}

/// [`line_end`] on x86-64: the aarch64 dispatch with [`line_end_sse2`] in
/// place of the NEON scan, kept out of line.
///
/// `line_end` is `#[inline]` and reached from dozens of block-parser sites.
/// Inlined into each of them the way the NEON scan is, the SSE2 loop adds
/// 63–70 instructions per site to the x86-64 release build (+3,100 in the
/// parser, +874 in `parse_document` alone). Out of line, each site keeps
/// only a call, and the word scan the sites used to inline moves here as
/// well: it serves only scans that start in a document's last 15 bytes, so
/// it no longer occupies code and registers in every caller.
#[cfg(all(target_arch = "x86_64", target_feature = "sse2"))]
#[inline(never)]
fn line_end_x86(bytes: &[u8], from: usize) -> usize {
    // The NEON threshold, for the same reason.
    if bytes.len() - from >= 16 {
        return line_end_sse2(bytes, from);
    }
    line_end_swar(bytes, from)
}

#[cfg(target_arch = "aarch64")]
#[allow(unsafe_code)]
#[inline]
fn line_end_neon(bytes: &[u8], from: usize) -> usize {
    use std::arch::aarch64::*;
    let end = bytes.len();
    let mut i = from;
    unsafe {
        let nl = vdupq_n_u8(b'\n');
        let cr = vdupq_n_u8(b'\r');
        let classify = |v: uint8x16_t| {
            let m = vorrq_u8(vceqq_u8(v, nl), vceqq_u8(v, cr));
            vget_lane_u64(
                vreinterpret_u64_u8(vshrn_n_u16(vreinterpretq_u16_u8(m), 4)),
                0,
            )
        };
        while i + 32 <= end {
            let m0 = classify(vld1q_u8(bytes.as_ptr().add(i)));
            if m0 != 0 {
                return i + (m0.trailing_zeros() / 4) as usize;
            }
            let m1 = classify(vld1q_u8(bytes.as_ptr().add(i + 16)));
            if m1 != 0 {
                return i + 16 + (m1.trailing_zeros() / 4) as usize;
            }
            i += 32;
        }
        while i + 16 <= end {
            let mask = classify(vld1q_u8(bytes.as_ptr().add(i)));
            if mask != 0 {
                return i + (mask.trailing_zeros() / 4) as usize;
            }
            i += 16;
        }
        if i < end && end >= 16 {
            // Overlapping tail: re-read the last vector and drop the lanes
            // the loops already cleared. `vceqq_u8` is exact per lane.
            let base = end - 16;
            let mask =
                classify(vld1q_u8(bytes.as_ptr().add(base))) & (u64::MAX << ((i - base) * 4));
            if mask != 0 {
                return base + (mask.trailing_zeros() / 4) as usize;
            }
            return end;
        }
    }
    while i < end && !is_line_ending_byte(bytes[i]) {
        i += 1;
    }
    i
}

/// SSE2 sibling of `line_end_neon` for x86-64, where SSE2 is part of the
/// baseline, so nothing is detected at run time. Needs `bytes.len() >= 16`;
/// `line_end_x86` calls it only while at least one vector remains after
/// `from`.
///
/// The steps mirror the NEON scan: 32 bytes per iteration, then at most one
/// 16-byte step, then one overlapping load of the last vector for the
/// remaining 1–15 bytes. The two halves of a 32-byte step fold into one
/// 32-bit mask, so the step takes a single branch.
#[cfg(all(target_arch = "x86_64", target_feature = "sse2"))]
#[allow(unsafe_code)]
#[inline]
fn line_end_sse2(bytes: &[u8], from: usize) -> usize {
    use std::arch::x86_64::{
        _mm_cmpeq_epi8, _mm_loadu_si128, _mm_movemask_epi8, _mm_or_si128, _mm_set1_epi8,
    };
    let end = bytes.len();
    debug_assert!(from <= end && end >= 16);
    let mut i = from;
    // SAFETY: the `cfg` above admits only builds with SSE2 enabled. Every
    // 16-byte load starts at an offset `at` with `at + 16 <= end`: the loop
    // conditions bound the stepped loads, and the tail load starts at
    // `end - 16`, which exists because the caller guarantees `end >= 16`.
    // `_mm_loadu_si128` has no alignment requirement.
    unsafe {
        let nl = _mm_set1_epi8(b'\n'.cast_signed());
        let cr = _mm_set1_epi8(b'\r'.cast_signed());
        // Bit `k` is set iff byte `at + k` is `\n` or `\r`. `movemask` fills
        // only the low 16 bits, so the value is never negative and the
        // conversion is exact.
        let lanes = |at: usize| {
            let v = _mm_loadu_si128(bytes.as_ptr().add(at).cast());
            _mm_movemask_epi8(_mm_or_si128(_mm_cmpeq_epi8(v, nl), _mm_cmpeq_epi8(v, cr)))
                .cast_unsigned()
        };
        while i + 32 <= end {
            let mask = lanes(i) | (lanes(i + 16) << 16);
            if mask != 0 {
                return i + mask.trailing_zeros() as usize;
            }
            i += 32;
        }
        if i + 16 <= end {
            let mask = lanes(i);
            if mask != 0 {
                return i + mask.trailing_zeros() as usize;
            }
            i += 16;
        }
        if i < end {
            // Overlapping tail: `end - 16 < i < end` here, so the shift is
            // 1..=15 and drops the lanes before `i`, which the steps already
            // cleared or which lie before `from`. Each lane's compare is
            // exact, so nothing leaks across the mask.
            let base = end - 16;
            let mask = lanes(base) & (u32::MAX << (i - base));
            if mask != 0 {
                return base + mask.trailing_zeros() as usize;
            }
            return end;
        }
    }
    i
}

/// Portable eight-byte word scan used for short remainders, and for every
/// input on targets without a vector scan.
#[inline]
fn line_end_swar(bytes: &[u8], from: usize) -> usize {
    let end = bytes.len();
    let mut i = from;

    while i + 8 <= end {
        let word = u64::from_le_bytes(copy_eight(bytes, i));
        let mask = has_zero(word ^ NEWLINES) | has_zero(word ^ RETURNS);
        if mask != 0 {
            return i + (mask.trailing_zeros() / 8) as usize;
        }
        i += 8;
    }

    while i < end && !is_line_ending_byte(bytes[i]) {
        i += 1;
    }
    i
}

/// Byte offset of the next line's start — just past LF, CRLF, or CR, or
/// `bytes.len()` at end of input.
#[inline]
pub(in crate::parser) fn next_line_start(bytes: &[u8], from: usize) -> usize {
    let end = line_end(bytes, from);
    line_terminator_end(bytes, end)
}

#[inline]
pub(in crate::parser) fn line_terminator_end(bytes: &[u8], line_end: usize) -> usize {
    if line_end >= bytes.len() {
        return line_end;
    }
    if bytes[line_end] == b'\r' && bytes.get(line_end + 1) == Some(&b'\n') {
        line_end + 2
    } else {
        line_end + 1
    }
}

#[inline]
pub(in crate::parser) fn is_line_ending_byte(byte: u8) -> bool {
    matches!(byte, b'\n' | b'\r')
}

#[inline]
fn copy_eight(bytes: &[u8], from: usize) -> [u8; 8] {
    let mut chunk = [0u8; 8];
    chunk.copy_from_slice(&bytes[from..from + 8]);
    chunk
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::disallowed_macros,
        clippy::disallowed_methods,
        clippy::disallowed_types
    )]

    use super::*;

    #[test]
    fn matches_memchr_at_every_offset() {
        // Newlines at every position across the word boundary and the tail, so
        // both the SWAR lane index and the scalar remainder get exercised.
        for len in 0..40usize {
            for newline_at in 0..len {
                let mut buffer = [b'x'; 40];
                buffer[newline_at] = b'\n';
                let bytes = &buffer[..len];
                for from in 0..=len {
                    let expected =
                        memchr::memchr2(b'\n', b'\r', &bytes[from..]).map_or(len, |off| from + off);
                    assert_eq!(
                        line_end(bytes, from),
                        expected,
                        "len {len}, newline at {newline_at}, from {from}"
                    );
                    let expected_start = line_terminator_end(bytes, expected);
                    assert_eq!(next_line_start(bytes, from), expected_start);
                }
            }
        }
    }

    #[test]
    fn matches_memchr_with_no_newline() {
        let buffer = [b'x'; 40];
        for len in 0..=buffer.len() {
            let bytes = &buffer[..len];
            for from in 0..=len {
                assert_eq!(line_end(bytes, from), len);
                assert_eq!(next_line_start(bytes, from), len);
            }
        }
    }

    #[test]
    fn matches_memchr_on_borrow_propagation_shapes() {
        // `0x0B` is the byte whose lane holds `0x01` after the newline xor, so
        // it is the one that can light up spuriously behind a real newline.
        for filler in [0x0Bu8, 0x00, 0x01, b'x'] {
            for lead in 0..12usize {
                let mut buffer = [filler; 40];
                buffer[lead] = b'\n';
                let bytes = &buffer[..];
                let expected = memchr::memchr2(b'\n', b'\r', bytes).unwrap_or(bytes.len());
                assert_eq!(
                    line_end(bytes, 0),
                    expected,
                    "filler {filler:#x}, newline at {lead}"
                );
            }
        }
    }

    #[test]
    fn matches_memchr_on_long_lines() {
        // 32-byte NEON loop + overlapping 16-byte tail: newlines past the
        // first vector, and unterminated 80-byte last lines.
        for len in 16..96usize {
            for newline_at in 0..len {
                let mut buffer = [b'x'; 96];
                buffer[newline_at] = b'\n';
                let bytes = &buffer[..len];
                for from in 0..=len {
                    let expected =
                        memchr::memchr2(b'\n', b'\r', &bytes[from..]).map_or(len, |off| from + off);
                    assert_eq!(
                        line_end(bytes, from),
                        expected,
                        "len {len}, newline at {newline_at}, from {from}"
                    );
                }
            }
            let unterminated = [b'x'; 96];
            let bytes = &unterminated[..len];
            for from in 0..=len {
                assert_eq!(
                    line_end(bytes, from),
                    len,
                    "unterminated len {len} from {from}"
                );
            }
        }
    }

    #[test]
    fn matches_memchr_on_multiline_walk() {
        let source = "alpha\nbeta\n\ngamma delta epsilon\n\tindented\nlast line without newline";
        let bytes = source.as_bytes();
        // Walk both scanners in lockstep so a divergence fails on the line it
        // happens, rather than as a mismatch between two collected lists.
        let mut pos = 0;
        let mut expected_pos = 0;
        let mut lines = 0;
        while expected_pos < bytes.len() {
            let expected = memchr::memchr2(b'\n', b'\r', &bytes[expected_pos..])
                .map_or(bytes.len(), |off| expected_pos + off);
            assert_eq!(line_end(bytes, pos), expected, "line {lines}");
            pos = next_line_start(bytes, pos);
            expected_pos = line_terminator_end(bytes, expected);
            assert_eq!(pos, expected_pos, "line {lines}");
            lines += 1;
        }
        assert_eq!(lines, 6);
    }

    #[test]
    fn recognizes_crlf_and_lone_cr_line_endings() {
        for (source, expected) in [
            ("a\r\nb\nc\rd", &["a", "b", "c", "d"][..]),
            ("\r\n\n\r", &["", "", ""][..]),
        ] {
            let bytes = source.as_bytes();
            let mut pos = 0;
            let mut line_index = 0;
            while pos < bytes.len() {
                let end = line_end(bytes, pos);
                assert_eq!(&source[pos..end], expected[line_index]);
                line_index += 1;
                pos = next_line_start(bytes, pos);
            }
            assert_eq!(line_index, expected.len());
        }
    }

    /// The definition every scanner must meet, for each start at once:
    /// `answers[from]` is the offset of the first `\n` or `\r` at or after
    /// `from`, or `bytes.len()` when there is none.
    fn first_terminators(bytes: &[u8]) -> Vec<usize> {
        let mut answers = vec![bytes.len(); bytes.len() + 1];
        for at in (0..bytes.len()).rev() {
            answers[at] = if is_line_ending_byte(bytes[at]) {
                at
            } else {
                answers[at + 1]
            };
        }
        answers
    }

    /// Checks the scans from `from` against `expected`: `line_end` and
    /// `next_line_start` on every target, and on x86-64 the SSE2 scan on its
    /// own for every input of at least one vector. That includes starts in
    /// the last 15 bytes, which `line_end` leaves to the word scan, so the
    /// tail's mask must drop the lanes before `from`.
    fn check_from(bytes: &[u8], from: usize, expected: usize) {
        assert_eq!(
            line_end(bytes, from),
            expected,
            "line_end from {from} of {bytes:02x?}"
        );
        assert_eq!(
            next_line_start(bytes, from),
            line_terminator_end(bytes, expected),
            "next_line_start from {from} of {bytes:02x?}"
        );
        #[cfg(all(target_arch = "x86_64", target_feature = "sse2"))]
        {
            if bytes.len() >= 16 {
                assert_eq!(
                    line_end_sse2(bytes, from),
                    expected,
                    "SSE2 from {from} of {bytes:02x?}"
                );
            }
        }
    }

    /// [`check_from`] from every start in `bytes`.
    fn check_every_start(bytes: &[u8]) {
        for (from, &expected) in first_terminators(bytes).iter().enumerate() {
            check_from(bytes, from, expected);
        }
    }

    #[test]
    fn matches_definition_with_terminators_at_every_offset() {
        // LF, CR and CRLF at every offset of every length up to 200, scanned
        // from every start: before the terminator the 32-byte step (either
        // half), the 16-byte step or the overlapping tail finds it, at it the
        // scan stops at once, and past it the scan must ignore it, including
        // lanes of the overlapping tail that the steps already cleared.
        for len in 0..=200usize {
            let mut buffer = vec![b'x'; len];
            check_every_start(&buffer);
            for terminator in [&b"\n"[..], b"\r", b"\r\n"] {
                let Some(last) = len.checked_sub(terminator.len()) else {
                    continue;
                };
                for at in 0..=last {
                    buffer[at..at + terminator.len()].copy_from_slice(terminator);
                    check_every_start(&buffer);
                    buffer[at..at + terminator.len()].fill(b'x');
                }
            }
        }
    }

    #[test]
    fn matches_definition_on_terminator_only_input() {
        for len in 0..=200usize {
            for unit in [&b"\n"[..], b"\r", b"\r\n"] {
                let bytes: Vec<u8> = unit.iter().copied().cycle().take(len).collect();
                check_every_start(&bytes);
            }
        }
    }

    #[test]
    fn matches_definition_for_every_byte_value_in_every_lane() {
        // Every byte value in every lane of the 32-byte step (both halves),
        // the 16-byte step and the overlapping tail: 32 bytes take one
        // 32-byte step, 47 add the tail, and 48 add the 16-byte step. The
        // backgrounds are non-terminators close to `\n` and `\r`: a plain
        // letter, `0x0B` between the two, and `0x8A`/`0x8D`, which differ
        // from them only in the high bit that every non-ASCII byte sets.
        for background in [b'x', 0x0B, 0x8A, 0x8D] {
            for len in [32usize, 47, 48] {
                let mut buffer = vec![background; len];
                for at in 0..len {
                    for value in 0..=u8::MAX {
                        buffer[at] = value;
                        let found = if is_line_ending_byte(value) { at } else { len };
                        check_from(&buffer, 0, found);
                        check_from(&buffer, at, found);
                        check_from(&buffer, at + 1, len);
                    }
                    buffer[at] = background;
                }
            }
        }
    }

    #[test]
    fn matches_definition_on_non_ascii_lines() {
        // Lines of up to 80 bytes built from 1- to 4-byte UTF-8 sequences,
        // ended in turn by LF, CRLF and CR, with an unterminated last line,
        // scanned from every start and walked line by line.
        let mut source = String::new();
        for len in 0..=80usize {
            let line_start = source.len();
            for ch in "aé中🙂".chars().cycle() {
                if source.len() - line_start + ch.len_utf8() > len {
                    break;
                }
                source.push(ch);
            }
            source.push_str(["\n", "\r\n", "\r"][len % 3]);
        }
        source.push_str("last line 🙂 without a terminator, long enough for a vector");
        let bytes = source.as_bytes();
        check_every_start(bytes);

        let answers = first_terminators(bytes);
        let mut pos = 0;
        let mut lines = 0;
        while pos < bytes.len() {
            let end = line_end(bytes, pos);
            assert_eq!(end, answers[pos], "line {lines}");
            assert!(source.is_char_boundary(end), "line {lines}");
            pos = next_line_start(bytes, pos);
            lines += 1;
        }
        assert_eq!(lines, 82);
    }
}
