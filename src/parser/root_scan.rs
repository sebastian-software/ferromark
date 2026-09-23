//! One pass over the root body for the two searches that precede parsing.
//!
//! Before block parsing starts, the root parser asks the whole body two
//! questions that an ordinary document answers with "nowhere": where its
//! first NUL byte is, which [`source_normalization`](super::source_normalization)
//! has to replace, and where its first `]:` is, which the definition pre-pass
//! needs before it looks for a single opener (see [`prepass`](super::prepass)).
//! Each search used to walk to the end of such a document on its own.
//!
//! On aarch64 one NEON loop answers both. Every 16-byte block is compared for
//! NUL and for `]`, and a second load one byte further on compares the byte
//! after each lane for `:`, so a `]:` split across two blocks belongs to the
//! block that holds its `]`. Four blocks share one branch, and their NUL test
//! folds into a lane-wise minimum. The loop stops at the first byte that is a
//! NUL or starts a `]:`; only a body that holds a `]:` then searches its rest
//! for NUL, with `memchr`. The pre-pass's third probe, for any `[`, only
//! matters once a `]:` is known, so it stays behind this scan and runs only
//! then. Other targets keep the two separate searches: `memchr`'s own vector
//! paths are what a portable word scan would have to beat, and no speedup is
//! established there.
//!
//! The answer describes the unmodified body. A body that holds a NUL is
//! rewritten by normalization, which moves every later offset, so the caller
//! keeps only the NUL's position and the pre-pass searches the rewritten
//! source itself. Parsers that never run the pre-pass — sub-parsers, the
//! definition pass, and documents with neither link references nor footnotes
//! enabled — do not run this scan; normalization keeps its plain `memchr`.

#[cfg(target_arch = "aarch64")]
use memchr::memchr;

#[cfg(any(test, not(target_arch = "aarch64")))]
use super::prepass::DEFINITION_CLOSER;

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
    #[cfg(target_arch = "aarch64")]
    {
        scan_fused(body)
    }
    #[cfg(not(target_arch = "aarch64"))]
    {
        scan_separately(body)
    }
}

/// The two searches the fused scan replaces, one after the other: the path on
/// targets without the vector loop, and the tests' reference for it.
#[cfg(any(test, not(target_arch = "aarch64")))]
fn scan_separately(body: &[u8]) -> RootScan {
    match memchr::memchr(0, body) {
        Some(nul) => RootScan::Nul(nul),
        None => RootScan::Clean {
            first_closer: DEFINITION_CLOSER.find(body),
        },
    }
}

#[cfg(target_arch = "aarch64")]
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
    let ptr = bytes.as_ptr();
    let mut at = 0;
    // SAFETY: a block at `from` loads `from..from + 16`, and its lookahead
    // loads `from + 1..from + 17`. The grouped loop runs only while the
    // lookahead of its fourth block ends in bounds (`at + 65 <= len`), and the
    // block loop only while its own does (`at + 17 <= len`).
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
    }
    first_nul_or_closer_scalar(bytes, at)
}

/// [`first_nul_or_closer`] from `from`, one byte at a time: the tail of the
/// vector loop, shorter than one block and its lookahead byte.
#[cfg(target_arch = "aarch64")]
fn first_nul_or_closer_scalar(bytes: &[u8], from: usize) -> Option<usize> {
    let mut at = from;
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
        #[cfg(target_arch = "aarch64")]
        {
            let first_event =
                (0..body.len()).find(|&at| body[at] == 0 || body[at..].starts_with(b"]:"));
            assert_eq!(
                super::first_nul_or_closer(body),
                first_event,
                "vector loop on {shown:?}"
            );
        }
    }

    /// Lengths that cover the scalar tail, one block with and without its
    /// lookahead byte, and several four-block groups with a remainder.
    const LENGTHS: std::ops::RangeInclusive<usize> = 0..=160;

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
