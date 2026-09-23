//! One pass over the root body for the two searches that precede parsing.
//!
//! Before block parsing starts, the root parser asks the whole body two
//! questions that an ordinary document answers with "nowhere": where its
//! first NUL byte is, which [`source_normalization`](super::source_normalization)
//! has to replace, and where its first `]:` is, which the definition pre-pass
//! needs before it looks for a single opener (see [`prepass`](super::prepass)).
//! Each search used to walk to the end of such a document on its own.
//!
//! On aarch64 and x86-64 one vector loop answers both. Every 16-byte block is
//! compared for NUL and for `]`, and a second load one byte further on
//! compares the byte after each lane for `:`, so a `]:` split across two
//! blocks belongs to the block that holds its `]`. Four blocks share one
//! branch, and their NUL test folds into a lane-wise unsigned minimum. The
//! loop stops at the first byte that is a NUL or starts a `]:`; only a body
//! that holds a `]:` then searches its rest for NUL, with `memchr`. The
//! pre-pass's third probe, for any `[`, only matters once a `]:` is known, so
//! it stays behind this scan and runs only then.
//!
//! aarch64 runs the loop with NEON. x86-64 runs the same loop with SSE2,
//! which is part of the x86-64 baseline, or with 32-byte AVX2 blocks where the
//! CPU reports AVX2 at run time: published builds target the baseline, and
//! `is_x86_feature_detected!` caches its answer. Other targets do not run this
//! scan at all: the root parse keeps its plain NUL `memchr`, and the pre-pass
//! keeps its `[` probe in front of the `]:` search, so a body without `[` is
//! never searched for `]:`. `memchr`'s own vector paths are what a portable
//! word scan would have to beat, and no speedup is established there.
//!
//! The 1 to 16 bytes the loops leave (1 to 32 with AVX2) are answered by one
//! vector over the last 16 (32) bytes of the body, as `memchr` finishes its
//! own searches, rather than byte by byte: on a comment-sized body those few
//! trailing bytes are a large share of the whole scan. Only a body shorter
//! than one 16-byte vector is walked one byte at a time; the AVX2 loop hands a
//! body shorter than its own vector to the SSE2 one.
//!
//! The answer describes the unmodified body. A body that holds a NUL is
//! rewritten by normalization, which moves every later offset, so the caller
//! keeps only the NUL's position and the pre-pass searches the rewritten
//! source itself. Parsers that never run the pre-pass — sub-parsers, the
//! definition pass, and documents with neither link references nor footnotes
//! enabled — do not run this scan; normalization keeps its plain `memchr`.

#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
use memchr::memchr;

#[cfg(any(test, not(any(target_arch = "aarch64", target_arch = "x86_64"))))]
use super::prepass::DEFINITION_CLOSER;

/// Whether this target runs the fused scan. Where it does not, the root parse
/// keeps its plain NUL search, and the pre-pass its own `[` probe in front of
/// its `]:` search.
pub(super) const FUSED: bool = cfg!(any(target_arch = "aarch64", target_arch = "x86_64"));

/// What one pass over the root body found.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum RootScan {
    /// The body holds a NUL, the first one at this offset. Normalization
    /// rewrites the body, so nothing else the scan saw applies to the result.
    Nul(usize),
    /// The body holds no NUL, so normalization leaves its bytes as they are.
    /// Its first `]:` starts at `first_closer`, if it holds one.
    Clean { first_closer: Option<usize> },
}

/// What the definition pre-pass is handed about the first `]:` of the source
/// it runs on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum DefinitionCloser {
    /// Nothing searched for it; the pre-pass runs its own searches.
    Unscanned,
    /// The source holds no `]:`.
    Absent,
    /// The first `]:` starts at this offset.
    At(usize),
}

/// The first NUL and the first `]:` of `body`, in one pass where the target
/// has one.
#[inline]
pub(super) fn scan(body: &[u8]) -> RootScan {
    #[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
    {
        scan_fused(body)
    }
    #[cfg(not(any(target_arch = "aarch64", target_arch = "x86_64")))]
    {
        scan_separately(body)
    }
}

/// The two searches the fused scan replaces, one after the other: the tests'
/// reference for it, and the answer on targets without the vector loop, where
/// only the tests ask.
#[cfg(any(test, not(any(target_arch = "aarch64", target_arch = "x86_64"))))]
fn scan_separately(body: &[u8]) -> RootScan {
    match memchr::memchr(0, body) {
        Some(nul) => RootScan::Nul(nul),
        None => RootScan::Clean {
            first_closer: DEFINITION_CLOSER.find(body),
        },
    }
}

#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
#[inline]
fn scan_fused(body: &[u8]) -> RootScan {
    let Some(at) = first_nul_or_closer(body) else {
        return RootScan::Clean { first_closer: None };
    };
    if body[at] == 0 {
        return RootScan::Nul(at);
    }
    // `at` starts the first `]:`, and no NUL comes before it. The rest of the
    // body only has to be searched for NUL.
    let rest = at + 2;
    match memchr(0, &body[rest..]) {
        Some(nul) => RootScan::Nul(rest + nul),
        None => RootScan::Clean {
            first_closer: Some(at),
        },
    }
}

/// Offset of the first byte of `bytes` that is a NUL or starts a `]:`.
#[cfg(target_arch = "aarch64")]
#[allow(unsafe_code)]
fn first_nul_or_closer(bytes: &[u8]) -> Option<usize> {
    use std::arch::aarch64::*;
    let len = bytes.len();
    if len < 16 {
        return first_nul_or_closer_short(bytes);
    }
    let ptr = bytes.as_ptr();
    let mut at = 0;
    // SAFETY: a block at `from` loads `from..from + 16`, and its lookahead
    // loads `from + 1..from + 17`. The grouped loop runs only while the
    // lookahead of its fourth block ends in bounds (`at + 65 <= len`), and the
    // block loop only while its own does (`at + 17 <= len`). The last vector
    // loads `len - 16..len`, and `len >= 16` was checked above.
    unsafe {
        let close = vdupq_n_u8(b']');
        let colon = vdupq_n_u8(b':');
        // 0xFF in lane `j` when `here[j]` is `]` and the byte after it is `:`.
        let closers = |here: uint8x16_t, from: usize| {
            vandq_u8(
                vceqq_u8(here, close),
                vceqq_u8(vld1q_u8(ptr.add(from + 1)), colon),
            )
        };
        // Four bits per lane, so the lowest set bit names the first lane.
        let lanes = |mask: uint8x16_t| {
            vget_lane_u64(
                vreinterpret_u64_u8(vshrn_n_u16(vreinterpretq_u16_u8(mask), 4)),
                0,
            )
        };
        while at + 65 <= len {
            let a = vld1q_u8(ptr.add(at));
            let b = vld1q_u8(ptr.add(at + 16));
            let c = vld1q_u8(ptr.add(at + 32));
            let d = vld1q_u8(ptr.add(at + 48));
            let closer = vorrq_u8(
                vorrq_u8(closers(a, at), closers(b, at + 16)),
                vorrq_u8(closers(c, at + 32), closers(d, at + 48)),
            );
            // A NUL anywhere in the group is the minimum of its lane.
            let nul = vceqzq_u8(vminq_u8(vminq_u8(a, b), vminq_u8(c, d)));
            if lanes(vorrq_u8(closer, nul)) != 0 {
                // The block loop below names the byte within these four.
                break;
            }
            at += 64;
        }
        while at + 17 <= len {
            let here = vld1q_u8(ptr.add(at));
            let mask = lanes(vorrq_u8(vceqzq_u8(here), closers(here, at)));
            if mask != 0 {
                return Some(at + (mask.trailing_zeros() / 4) as usize);
            }
            at += 16;
        }
        // One vector over the last 16 bytes answers for the 1 to 16 bytes the
        // loops left. It re-reads bytes the loops already cleared, which hold
        // no NUL and start no `]:`, so its first set lane is at or past `at`
        // without a mask. Nothing follows its last lane, so a zero lane stands
        // in for that lookahead: a `]` in the final byte starts no `]:`.
        let base = len - 16;
        let here = vld1q_u8(ptr.add(base));
        let next = vextq_u8::<1>(here, vdupq_n_u8(0));
        let mask = lanes(vorrq_u8(
            vceqzq_u8(here),
            vandq_u8(vceqq_u8(here, close), vceqq_u8(next, colon)),
        ));
        if mask == 0 {
            None
        } else {
            Some(base + (mask.trailing_zeros() / 4) as usize)
        }
    }
}

/// Offset of the first byte of `bytes` that is a NUL or starts a `]:`: the
/// NEON loop's answer, in 32-byte AVX2 blocks where the CPU has AVX2 and in
/// 16-byte SSE2 blocks otherwise.
#[cfg(target_arch = "x86_64")]
#[inline]
fn first_nul_or_closer(bytes: &[u8]) -> Option<usize> {
    if std::arch::is_x86_feature_detected!("avx2") {
        // SAFETY: guarded by the detection above.
        #[allow(unsafe_code)]
        unsafe {
            first_nul_or_closer_avx2(bytes)
        }
    } else {
        first_nul_or_closer_sse2(bytes)
    }
}

/// [`first_nul_or_closer`] in 16-byte SSE2 blocks, the NEON loop lane for
/// lane. SSE2 is part of the x86-64 baseline, so it needs no detection.
#[cfg(target_arch = "x86_64")]
#[allow(unsafe_code)]
fn first_nul_or_closer_sse2(bytes: &[u8]) -> Option<usize> {
    use std::arch::x86_64::*;
    let len = bytes.len();
    if len < 16 {
        return first_nul_or_closer_short(bytes);
    }
    let ptr = bytes.as_ptr();
    let mut at = 0;
    // SAFETY: a block at `from` loads `from..from + 16`, and its lookahead
    // loads `from + 1..from + 17`. The grouped loop runs only while the
    // lookahead of its fourth block ends in bounds (`at + 65 <= len`), and the
    // block loop only while its own does (`at + 17 <= len`). The last vector
    // loads `len - 16..len`, and `len >= 16` was checked above.
    unsafe {
        let zero = _mm_setzero_si128();
        let close = _mm_set1_epi8(b']'.cast_signed());
        let colon = _mm_set1_epi8(b':'.cast_signed());
        let load = |from: usize| _mm_loadu_si128(ptr.add(from).cast());
        // 0xFF in lane `j` when `here[j]` is `]` and the byte after it is `:`.
        let closers = |here: __m128i, from: usize| {
            _mm_and_si128(
                _mm_cmpeq_epi8(here, close),
                _mm_cmpeq_epi8(load(from + 1), colon),
            )
        };
        // One bit per lane, so the lowest set bit names the first lane.
        let lanes = |mask: __m128i| _mm_movemask_epi8(mask).cast_unsigned();
        while at + 65 <= len {
            let a = load(at);
            let b = load(at + 16);
            let c = load(at + 32);
            let d = load(at + 48);
            let closer = _mm_or_si128(
                _mm_or_si128(closers(a, at), closers(b, at + 16)),
                _mm_or_si128(closers(c, at + 32), closers(d, at + 48)),
            );
            // A NUL anywhere in the group is the minimum of its lane. The
            // minimum is unsigned: a signed one would let a byte from 0x80 up
            // hide a NUL in the same lane of another block.
            let nul = _mm_cmpeq_epi8(_mm_min_epu8(_mm_min_epu8(a, b), _mm_min_epu8(c, d)), zero);
            if lanes(_mm_or_si128(closer, nul)) != 0 {
                // The block loop below names the byte within these four.
                break;
            }
            at += 64;
        }
        while at + 17 <= len {
            let here = load(at);
            let mask = lanes(_mm_or_si128(_mm_cmpeq_epi8(here, zero), closers(here, at)));
            if mask != 0 {
                return Some(at + mask.trailing_zeros() as usize);
            }
            at += 16;
        }
        // One vector over the last 16 bytes answers for the 1 to 16 bytes the
        // loops left, as on aarch64.
        let base = len - 16;
        let here = load(base);
        last_vector(
            base,
            lanes(_mm_cmpeq_epi8(here, zero)),
            lanes(_mm_cmpeq_epi8(here, close)),
            lanes(_mm_cmpeq_epi8(here, colon)),
        )
    }
}

/// [`first_nul_or_closer`] in 32-byte AVX2 blocks: the SSE2 loop at twice the
/// width, again four blocks to a group. A body shorter than one such vector
/// takes the SSE2 loop.
///
/// # Safety
///
/// The caller must have verified AVX2 support.
#[cfg(target_arch = "x86_64")]
#[allow(unsafe_code)]
#[target_feature(enable = "avx2")]
unsafe fn first_nul_or_closer_avx2(bytes: &[u8]) -> Option<usize> {
    use std::arch::x86_64::*;
    let len = bytes.len();
    if len < 32 {
        return first_nul_or_closer_sse2(bytes);
    }
    let ptr = bytes.as_ptr();
    let mut at = 0;
    // SAFETY: a block at `from` loads `from..from + 32`, and its lookahead
    // loads `from + 1..from + 33`. The grouped loop runs only while the
    // lookahead of its fourth block ends in bounds (`at + 129 <= len`), and
    // the block loop only while its own does (`at + 33 <= len`). The last
    // vector loads `len - 32..len`, and `len >= 32` was checked above.
    unsafe {
        let zero = _mm256_setzero_si256();
        let close = _mm256_set1_epi8(b']'.cast_signed());
        let colon = _mm256_set1_epi8(b':'.cast_signed());
        let load = |from: usize| _mm256_loadu_si256(ptr.add(from).cast());
        let closers = |here: __m256i, from: usize| {
            _mm256_and_si256(
                _mm256_cmpeq_epi8(here, close),
                _mm256_cmpeq_epi8(load(from + 1), colon),
            )
        };
        let lanes = |mask: __m256i| _mm256_movemask_epi8(mask).cast_unsigned();
        while at + 129 <= len {
            let a = load(at);
            let b = load(at + 32);
            let c = load(at + 64);
            let d = load(at + 96);
            let closer = _mm256_or_si256(
                _mm256_or_si256(closers(a, at), closers(b, at + 32)),
                _mm256_or_si256(closers(c, at + 64), closers(d, at + 96)),
            );
            let nul = _mm256_cmpeq_epi8(
                _mm256_min_epu8(_mm256_min_epu8(a, b), _mm256_min_epu8(c, d)),
                zero,
            );
            if lanes(_mm256_or_si256(closer, nul)) != 0 {
                break;
            }
            at += 128;
        }
        while at + 33 <= len {
            let here = load(at);
            let mask = lanes(_mm256_or_si256(
                _mm256_cmpeq_epi8(here, zero),
                closers(here, at),
            ));
            if mask != 0 {
                return Some(at + mask.trailing_zeros() as usize);
            }
            at += 32;
        }
        let base = len - 32;
        let here = load(base);
        last_vector(
            base,
            lanes(_mm256_cmpeq_epi8(here, zero)),
            lanes(_mm256_cmpeq_epi8(here, close)),
            lanes(_mm256_cmpeq_epi8(here, colon)),
        )
    }
}

/// The answer of the x86 loops' last vector, which starts at `base` and ends
/// the body, from its lane bits for NUL, `]` and `:`.
///
/// Like the NEON one, it re-reads bytes the loops already cleared, which hold
/// no NUL and start no `]:`, so its first set lane is at or past where they
/// stopped without a mask. AVX2 has no byte shift across its two halves, so
/// the lookahead is taken from the bits instead: shifting the `:` bits down by
/// one lines each lane up with the byte after it. The bit shifted in above
/// the last lane is zero and stands in for the byte that does not follow the
/// body, so a `]` in the final byte starts no `]:`.
#[cfg(target_arch = "x86_64")]
#[inline]
fn last_vector(base: usize, nul: u32, close: u32, colon: u32) -> Option<usize> {
    let mask = nul | (close & (colon >> 1));
    if mask == 0 {
        None
    } else {
        Some(base + mask.trailing_zeros() as usize)
    }
}

/// [`first_nul_or_closer`] for a body shorter than one vector, one byte at a
/// time.
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
fn first_nul_or_closer_short(bytes: &[u8]) -> Option<usize> {
    let mut at = 0;
    while at < bytes.len() {
        match bytes[at] {
            0 => return Some(at),
            b']' if bytes.get(at + 1) == Some(&b':') => return Some(at),
            _ => at += 1,
        }
    }
    None
}

#[cfg(test)]
mod tests {
    // Owned buffers keep the test oracle independent of production storage.
    #![allow(
        clippy::disallowed_macros,
        clippy::disallowed_methods,
        clippy::disallowed_types
    )]

    use super::{RootScan, scan, scan_separately};

    /// The scan's definition, one byte at a time and without `memchr`.
    fn oracle(body: &[u8]) -> RootScan {
        if let Some(nul) = body.iter().position(|&byte| byte == 0) {
            return RootScan::Nul(nul);
        }
        RootScan::Clean {
            first_closer: body.windows(2).position(|pair| pair == b"]:"),
        }
    }

    #[track_caller]
    fn check(body: &[u8]) {
        let expected = oracle(body);
        let shown = String::from_utf8_lossy(body);
        assert_eq!(scan(body), expected, "scan of {shown:?}");
        assert_eq!(
            scan_separately(body),
            expected,
            "separate searches of {shown:?}"
        );
        #[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
        {
            let first_event =
                (0..body.len()).find(|&at| body[at] == 0 || body[at..].starts_with(b"]:"));
            assert_eq!(
                super::first_nul_or_closer(body),
                first_event,
                "vector loop on {shown:?}"
            );
            // Both x86 widths, whichever one the dispatch above picked.
            #[cfg(target_arch = "x86_64")]
            {
                assert_eq!(
                    super::first_nul_or_closer_sse2(body),
                    first_event,
                    "SSE2 loop on {shown:?}"
                );
                if std::arch::is_x86_feature_detected!("avx2") {
                    // SAFETY: guarded by the detection above.
                    #[allow(unsafe_code)]
                    let avx2 = unsafe { super::first_nul_or_closer_avx2(body) };
                    assert_eq!(avx2, first_event, "AVX2 loop on {shown:?}");
                }
            }
        }
    }

    /// Lengths that cover bodies shorter than one vector, one block with and
    /// without its lookahead byte, several four-block groups of 16 bytes and
    /// one of 32, and the last vector after every number of bytes the loops of
    /// either width leave.
    const LENGTHS: std::ops::RangeInclusive<usize> = 0..=200;

    #[test]
    fn tiny_bodies() {
        for body in [
            &b""[..],
            b"]",
            b":",
            b"[",
            b"\0",
            b"]:",
            b":]",
            b"]]:",
            b"]\0:",
            b"\0]:",
            b"]:\0",
            b"[]:",
            b"]:[",
            b"] :",
            b"]\n:",
        ] {
            check(body);
        }
    }

    #[test]
    fn every_short_body_over_the_deciding_bytes() {
        // Exhaustive over the bytes that decide an answer, up to seven bytes.
        let alphabet = [b'x', b']', b':', 0];
        for len in 0..=7u32 {
            for mut code in 0..alphabet.len().pow(len) {
                let mut body = Vec::new();
                for _ in 0..len {
                    body.push(alphabet[code % alphabet.len()]);
                    code /= alphabet.len();
                }
                check(&body);
            }
        }
    }

    #[test]
    fn a_closer_at_every_offset_across_block_and_group_boundaries() {
        for len in LENGTHS {
            for filler in [b'x', b':', b']'] {
                for at in 0..len.saturating_sub(1) {
                    let mut body = vec![filler; len];
                    body[at] = b']';
                    body[at + 1] = b':';
                    check(&body);
                    // A second closer further on must not win.
                    if at + 5 < len {
                        body[at + 3] = b']';
                        body[at + 4] = b':';
                        check(&body);
                    }
                }
            }
        }
    }

    #[test]
    fn the_last_vector_answers_for_every_tail_length() {
        // From 16 bytes on, a body ends on one vector over its last 16 bytes
        // (32 with AVX2, from 32 bytes on), however many of them the loops
        // before it already cleared. Put each needle in each of the last 33
        // positions, behind every filler, up to the final byte, which has no
        // lookahead byte after it. The lengths reach past two 128-byte AVX2
        // groups.
        for len in 16..=300usize {
            for filler in [b'x', b']', b':'] {
                for at in len.saturating_sub(33)..len {
                    for needle in [&b"\0"[..], b"]:", b"]", b":"] {
                        if at + needle.len() > len {
                            continue;
                        }
                        let mut body = vec![filler; len];
                        body[at..at + needle.len()].copy_from_slice(needle);
                        check(&body);
                    }
                }
            }
        }
    }

    #[test]
    fn every_prefix_of_a_comment_sized_body() {
        // A real sub-kilobyte comment without `[`, `]:` or NUL: the scan runs
        // to its end, so every prefix ends on a different tail length.
        let comment = "Does this also apply when the connection is already open? I can \
                       still reproduce the original behavior after refreshing the page, \
                       but only on the first request.\n";
        for end in 0..=comment.len() {
            let prefix = &comment.as_bytes()[..end];
            check(prefix);
            for needle in [&b"]:"[..], b"\0", b"]"] {
                let mut body = prefix.to_vec();
                body.extend_from_slice(needle);
                check(&body);
            }
        }
    }

    #[test]
    fn a_split_or_reversed_pair_is_no_closer() {
        for len in LENGTHS {
            for at in 0..len {
                // `]` alone, the last byte included, and `:` alone, the first
                // byte included.
                let mut body = vec![b'x'; len];
                body[at] = b']';
                check(&body);
                body[at] = b':';
                check(&body);
                // `:]`, and `]` and `:` with a byte between them.
                if at + 1 < len {
                    body[at + 1] = b']';
                    check(&body);
                }
                if at + 2 < len {
                    body[at] = b']';
                    body[at + 1] = b' ';
                    body[at + 2] = b':';
                    check(&body);
                }
            }
        }
    }

    #[test]
    fn a_nul_at_every_offset_before_and_after_a_closer() {
        for len in LENGTHS {
            for nul in 0..len {
                let mut body = vec![b'x'; len];
                body[nul] = 0;
                check(&body);
                // A closer before the NUL, after it, and around it.
                for closer in [0, nul.saturating_sub(2), nul + 1, len.saturating_sub(2)] {
                    let mut with_closer = body.clone();
                    if closer + 1 < len && closer != nul && closer + 1 != nul {
                        with_closer[closer] = b']';
                        with_closer[closer + 1] = b':';
                    }
                    check(&with_closer);
                }
                // A `]` right before the NUL, and a `:` right after it.
                if nul > 0 {
                    body[nul - 1] = b']';
                }
                if nul + 1 < len {
                    body[nul + 1] = b':';
                }
                check(&body);
            }
        }
    }

    #[test]
    fn a_nul_at_every_offset_among_high_bytes() {
        // Four blocks fold their NUL test into one lane-wise minimum, which
        // has to be unsigned: bytes from 0x80 up in the other blocks' lanes
        // must not hide the NUL.
        for len in LENGTHS {
            for filler in [0x80, 0xBF, 0xFF] {
                for nul in 0..len {
                    let mut body = vec![filler; len];
                    body[nul] = 0;
                    check(&body);
                }
            }
        }
    }

    #[test]
    fn multibyte_text_around_the_needles() {
        for piece in ["é", "🙂", "\u{fffd}", "\u{feff}"] {
            for lead in 0..40 {
                for shape in ["]:", "]", ":", "\0", "[", "]é:", ":]"] {
                    let body = format!("{}{shape}{piece}", piece.repeat(lead));
                    check(body.as_bytes());
                    let body = format!("{}{piece}]{piece}:{shape}", "x".repeat(lead));
                    check(body.as_bytes());
                }
            }
        }
    }

    #[test]
    fn long_bodies() {
        let prose = "Plain prose with [links](/url) and `code: spans`, no definition.\n";
        let long = prose.repeat(1024);
        check(long.as_bytes());
        check(format!("{long}]").as_bytes());
        check(format!("{long}]:").as_bytes());
        check(format!("{long}\0").as_bytes());
        check(format!("{long}[a]: /url\n{long}\0").as_bytes());
        check(format!("[a]: /url\n{long}").as_bytes());
        check(format!("\0{long}[a]: /url\n").as_bytes());
    }

    #[test]
    fn generated_mixes() {
        let tokens = [
            "[",
            "]",
            ":",
            "]:",
            "\0",
            "\n",
            "\r\n",
            " ",
            "x",
            "word ",
            "é",
            "🙂",
            "\u{feff}",
            "\u{fffd}",
            "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
        ];
        let mut state = 0x9e37_79b9_7f4a_7c15u64;
        let mut next = move || {
            state = state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1);
            (state >> 33) as usize
        };
        for _ in 0..6000 {
            let len = next() % 80;
            // The NUL and `]:` tokens stop the scan, so most mixes leave them
            // out and are walked further, often to the end.
            let rare = next() % 4 == 0;
            let mut body = String::new();
            for _ in 0..len {
                let token = tokens[next() % tokens.len()];
                if rare || !matches!(token, "\0" | "]:") {
                    body.push_str(token);
                }
            }
            check(body.as_bytes());
        }
    }
}
