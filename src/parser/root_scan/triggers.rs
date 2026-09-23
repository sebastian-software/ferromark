//! The root scan, collecting the GFM autolink pre-flight's trigger offsets on
//! the way.
//!
//! The pre-flight (`gfm_autolink::may_contain_autolink`) asks every block's
//! inline content for three needles: an `@`, a `://`, and a `www.`. It used to
//! answer by walking the content once more after the inline parse had read
//! it. The root scan already reads every byte of the body before block parsing
//! starts, and in prose the needles are rare, so it can write down where they
//! are for a few more vector operations per block: the pre-flight then visits
//! only the recorded offsets that fall inside a block's content (see
//! `gfm_autolink::AutolinkTriggers`).
//!
//! A trigger is any byte that [`is_autolink_trigger`] accepts: every `@`, and
//! every `:` or `w` that a `.` or `/` follows. That keeps every `:` that starts
//! a `://` and every `w` that ends a `www.`, and leaves out the colons of
//! ordinary prose and code (`Note:`, `a::b`, `key: value`), which are most of
//! them. The pre-flight's `mailto:` and `xmpp:` rule needs every colon, but
//! only matters when the content also holds an `@`; the pre-flight hands such
//! content to the full pass.
//!
//! The recorded offsets describe the unmodified body, like the rest of the
//! root scan's answer, and are only kept for a body without NUL.

use crate::allocator::Vec;

use super::RootScan;

/// Whether `byte` is an autolink trigger, given the byte after it (NUL past
/// the end of the body).
///
/// `next | 1 == b'/'` holds for exactly `.` (0x2E) and `/` (0x2F), so the pair
/// test costs one compare in the vector loop. The two extra pairs it lets in,
/// `:.` and `w/`, are rare and cost only an entry the pre-flight skips.
#[inline]
pub(in crate::parser) const fn is_autolink_trigger(byte: u8, next: u8) -> bool {
    byte == b'@' || ((byte == b':' || byte == b'w') && next | 1 == b'/')
}

/// [`super::scan`] over `body`, pushing the offset of every autolink trigger
/// onto `triggers` in ascending order.
///
/// When the answer is [`RootScan::Nul`], `triggers` holds an arbitrary prefix
/// of the offsets and has to be dropped: normalization rewrites that body.
/// Offsets are stored as `u32`, so `body` must be shorter than 4 GiB.
#[inline]
pub(in crate::parser) fn scan_collecting_triggers(
    body: &[u8],
    triggers: &mut Vec<'_, u32>,
) -> RootScan {
    debug_assert!(u32::try_from(body.len()).is_ok());
    #[cfg(target_arch = "aarch64")]
    {
        if body.len() >= 16 {
            return neon::scan(body, triggers);
        }
    }
    scan_bytewise(body, triggers)
}

/// The collecting scan one byte at a time: the answer for a body shorter than
/// one vector, the tests' reference for the vector loop, and the whole scan
/// on other targets, where only the tests ask for it.
fn scan_bytewise(body: &[u8], triggers: &mut Vec<'_, u32>) -> RootScan {
    let mut first_closer = None;
    for (at, &byte) in body.iter().enumerate() {
        if byte == 0 {
            return RootScan::Nul(at);
        }
        let next = body.get(at + 1).copied().unwrap_or(0);
        if first_closer.is_none() && byte == b']' && next == b':' {
            first_closer = Some(at);
        }
        if is_autolink_trigger(byte, next) {
            triggers.push(at as u32);
        }
    }
    RootScan::Clean { first_closer }
}

/// The NEON loop of the root scan (see the parent module) with the trigger
/// test added to every block.
///
/// The trigger test is eight vector operations per 16 bytes: compares for `@`,
/// `:` and `w`, one for the byte after each lane (`next | 1 == b'/'`, on the
/// lookahead vector the loop already loads for its `]:` test), and four to
/// combine them. Four blocks still share one branch: it is taken for a group
/// that holds a NUL, a `]:` or a trigger, and only the first two leave the
/// loop.
///
/// The scan runs in two phases. Until the first `]:` it stops at a NUL or at
/// that `]:`; after it, only at a NUL, where the plain root scan hands the rest
/// of the body to `memchr`. A phase that stops in a block has recorded the
/// triggers before that block and none in it, so the next phase starts at that
/// block and nothing is recorded twice.
#[cfg(target_arch = "aarch64")]
#[allow(unsafe_code)]
mod neon {
    use std::arch::aarch64::*;

    use crate::allocator::Vec;

    use super::super::RootScan;

    /// Where one phase stopped.
    enum Stop {
        /// A NUL at this offset, the first in the body.
        Nul(usize),
        /// The first `]:` of the body starts at this offset.
        Closer(usize),
        /// The end of the body.
        End,
    }

    pub(super) fn scan(body: &[u8], triggers: &mut Vec<'_, u32>) -> RootScan {
        let mut at = 0;
        let first_closer = match phase::<true>(body, &mut at, triggers) {
            Stop::Nul(nul) => return RootScan::Nul(nul),
            Stop::End => return RootScan::Clean { first_closer: None },
            Stop::Closer(closer) => closer,
        };
        match phase::<false>(body, &mut at, triggers) {
            Stop::Nul(nul) => RootScan::Nul(nul),
            Stop::Closer(_) | Stop::End => RootScan::Clean {
                first_closer: Some(first_closer),
            },
        }
    }

    /// Pushes `from` plus the index of every lane set in `mask`, a lane mask
    /// with four bits per lane.
    #[inline]
    fn record(mask: u64, from: usize, triggers: &mut Vec<'_, u32>) {
        let mut bits = mask & 0x8888_8888_8888_8888;
        while bits != 0 {
            triggers.push((from + (bits.trailing_zeros() / 4) as usize) as u32);
            bits &= bits - 1;
        }
    }

    /// Scans from `*at`, recording triggers, until a NUL, the first `]:`
    /// (with `CLOSER` only) or the end of the body. On a stop, `*at` is the
    /// start of the block that holds it, whose triggers are not recorded yet.
    fn phase<const CLOSER: bool>(body: &[u8], at: &mut usize, triggers: &mut Vec<'_, u32>) -> Stop {
        let len = body.len();
        let ptr = body.as_ptr();
        debug_assert!(len >= 16 && *at < len);
        // SAFETY: a block at `from` loads `from..from + 16`, and its lookahead
        // loads `from + 1..from + 17`. The grouped loop runs only while the
        // lookahead of its fourth block ends in bounds (`at + 65 <= len`), and
        // the block loop only while its own does (`at + 17 <= len`). The last
        // vector loads `len - 16..len`, and `len >= 16`.
        unsafe {
            let close = vdupq_n_u8(b']');
            let colon = vdupq_n_u8(b':');
            let at_sign = vdupq_n_u8(b'@');
            let letter_w = vdupq_n_u8(b'w');
            let one = vdupq_n_u8(1);
            let slash = vdupq_n_u8(b'/');
            // Four bits per lane, so the lowest set bit names the first lane.
            let lanes = |mask: uint8x16_t| {
                vget_lane_u64(
                    vreinterpret_u64_u8(vshrn_n_u16(vreinterpretq_u16_u8(mask), 4)),
                    0,
                )
            };
            // 0xFF in lane `j` when `here[j]` is a trigger and `next[j]` the
            // byte after it (see `is_autolink_trigger`).
            let trigger = |here: uint8x16_t, next: uint8x16_t| {
                vorrq_u8(
                    vceqq_u8(here, at_sign),
                    vandq_u8(
                        vorrq_u8(vceqq_u8(here, colon), vceqq_u8(here, letter_w)),
                        vceqq_u8(vorrq_u8(next, one), slash),
                    ),
                )
            };
            // 0xFF in lane `j` when `here[j]` starts a `]:`.
            let closer = |here: uint8x16_t, next: uint8x16_t| {
                vandq_u8(vceqq_u8(here, close), vceqq_u8(next, colon))
            };
            // 0xFF in lane `j` when `here[j]` stops this phase.
            let stops = |here: uint8x16_t, next: uint8x16_t| {
                let nul = vceqzq_u8(here);
                if CLOSER {
                    vorrq_u8(nul, closer(here, next))
                } else {
                    nul
                }
            };
            // The offset a stop mask names, as a phase result.
            let stop_at = |first: usize| {
                if body[first] == 0 {
                    Stop::Nul(first)
                } else {
                    Stop::Closer(first)
                }
            };

            while *at + 65 <= len {
                let from = *at;
                let a = vld1q_u8(ptr.add(from));
                let b = vld1q_u8(ptr.add(from + 16));
                let c = vld1q_u8(ptr.add(from + 32));
                let d = vld1q_u8(ptr.add(from + 48));
                let next_a = vld1q_u8(ptr.add(from + 1));
                let next_b = vld1q_u8(ptr.add(from + 17));
                let next_c = vld1q_u8(ptr.add(from + 33));
                let next_d = vld1q_u8(ptr.add(from + 49));
                // A NUL anywhere in the group is the minimum of its lane.
                let nul = vceqzq_u8(vminq_u8(vminq_u8(a, b), vminq_u8(c, d)));
                let stop = if CLOSER {
                    vorrq_u8(
                        nul,
                        vorrq_u8(
                            vorrq_u8(closer(a, next_a), closer(b, next_b)),
                            vorrq_u8(closer(c, next_c), closer(d, next_d)),
                        ),
                    )
                } else {
                    nul
                };
                let hit_a = trigger(a, next_a);
                let hit_b = trigger(b, next_b);
                let hit_c = trigger(c, next_c);
                let hit_d = trigger(d, next_d);
                let any = vorrq_u8(
                    stop,
                    vorrq_u8(vorrq_u8(hit_a, hit_b), vorrq_u8(hit_c, hit_d)),
                );
                if lanes(any) != 0 {
                    if lanes(stop) != 0 {
                        // The block loop below names the stop and records the
                        // triggers of the blocks in front of it.
                        break;
                    }
                    record(lanes(hit_a), from, triggers);
                    record(lanes(hit_b), from + 16, triggers);
                    record(lanes(hit_c), from + 32, triggers);
                    record(lanes(hit_d), from + 48, triggers);
                }
                *at += 64;
            }
            while *at + 17 <= len {
                let here = vld1q_u8(ptr.add(*at));
                let next = vld1q_u8(ptr.add(*at + 1));
                let stop = lanes(stops(here, next));
                if stop != 0 {
                    return stop_at(*at + (stop.trailing_zeros() / 4) as usize);
                }
                record(lanes(trigger(here, next)), *at, triggers);
                *at += 16;
            }
            // One vector over the last 16 bytes answers for the 1 to 16 bytes
            // the loops left, as in the plain scan. The lanes in front of `at`
            // hold no stop, and their triggers are recorded already. Nothing
            // follows the last lane, so a zero lane stands in for its
            // lookahead: its byte starts no `]:` and is no trigger.
            let base = len - 16;
            let here = vld1q_u8(ptr.add(base));
            let next = vextq_u8::<1>(here, vdupq_n_u8(0));
            let stop = lanes(stops(here, next));
            if stop != 0 {
                return stop_at(base + (stop.trailing_zeros() / 4) as usize);
            }
            let seen = *at - base;
            record(
                lanes(trigger(here, next)) & (u64::MAX << (4 * seen)),
                base,
                triggers,
            );
            *at = len;
            Stop::End
        }
    }
}

#[cfg(test)]
mod tests {
    // Owned buffers keep the test oracle independent of production storage.
    #![allow(
        clippy::disallowed_macros,
        clippy::disallowed_methods,
        clippy::disallowed_types
    )]

    use crate::allocator::Allocator;

    use super::super::{RootScan, scan};
    use super::{is_autolink_trigger, scan_bytewise, scan_collecting_triggers};

    /// The collecting scan's definition, one byte at a time: the plain root
    /// scan's answer, and every trigger offset of a body without NUL.
    fn oracle(body: &[u8]) -> (RootScan, Option<Vec<u32>>) {
        let answer = scan(body);
        let triggers = matches!(answer, RootScan::Clean { .. }).then(|| {
            (0..body.len())
                .filter(|&at| is_autolink_trigger(body[at], body.get(at + 1).copied().unwrap_or(0)))
                .map(|at| at as u32)
                .collect()
        });
        (answer, triggers)
    }

    #[track_caller]
    fn check(body: &[u8]) {
        let (expected, expected_triggers) = oracle(body);
        let shown = String::from_utf8_lossy(body);
        let allocator = Allocator::new();
        for (label, collect) in [
            (
                "collecting scan",
                scan_collecting_triggers
                    as fn(&[u8], &mut crate::allocator::Vec<'_, u32>) -> RootScan,
            ),
            ("bytewise scan", scan_bytewise),
        ] {
            let mut triggers = allocator.new_vec();
            let answer = collect(body, &mut triggers);
            assert_eq!(answer, expected, "{label} of {shown:?}");
            if let Some(expected_triggers) = &expected_triggers {
                assert_eq!(
                    triggers.as_slice(),
                    expected_triggers.as_slice(),
                    "{label} triggers of {shown:?}"
                );
            }
        }
    }

    #[test]
    fn the_trigger_bytes() {
        for byte in 0..=u8::MAX {
            for next in 0..=u8::MAX {
                let expected =
                    byte == b'@' || (matches!(byte, b':' | b'w') && matches!(next, b'.' | b'/'));
                assert_eq!(
                    is_autolink_trigger(byte, next),
                    expected,
                    "{byte:#x} {next:#x}"
                );
            }
        }
    }

    #[test]
    fn every_short_body_over_the_deciding_bytes() {
        // Exhaustive over the bytes that decide an answer, up to six bytes.
        let alphabet = [b'x', b']', b':', 0, b'@', b'w', b'.', b'/'];
        for len in 0..=6u32 {
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

    /// Lengths that cover bodies shorter than one vector, one block with and
    /// without its lookahead byte, several four-block groups, and the last
    /// vector after every number of bytes the loops leave.
    const LENGTHS: std::ops::RangeInclusive<usize> = 0..=200;

    #[test]
    fn a_needle_at_every_offset() {
        let needles: [&[u8]; 12] = [
            b"@", b":/", b"://", b"w.", b"www.", b":.", b"w/", b"]:", b"\0", b"]", b":", b"w",
        ];
        for len in LENGTHS {
            for filler in [b'x', b'w', b':', b'.', b'/'] {
                for needle in needles {
                    for at in 0..=len.saturating_sub(needle.len()) {
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
    fn a_closer_or_nul_among_triggers() {
        // Triggers before, inside and after the block that stops the first
        // phase, and a NUL before or after the first `]:`.
        for len in LENGTHS {
            for stop in 0..len.saturating_sub(1) {
                for stopper in [&b"]:"[..], b"\0"] {
                    let mut body: Vec<u8> = (0..len)
                        .map(|at| if at % 7 == 3 { b'@' } else { b'w' })
                        .collect();
                    for at in (0..len).filter(|at| at % 11 == 5) {
                        body[at] = b'.';
                    }
                    body[stop..stop + stopper.len()].copy_from_slice(stopper);
                    check(&body);
                    if stop + 9 < len {
                        body[stop + 7] = 0;
                        check(&body);
                    }
                }
            }
        }
    }

    #[test]
    fn generated_mixes() {
        let tokens = [
            "@",
            ":",
            "/",
            ".",
            "w",
            "www.",
            "://",
            "http://a.b",
            "]:",
            "]",
            "[",
            "\0",
            "\n",
            " ",
            "x",
            "word ",
            "é",
            "🙂",
            "w/o",
            "a:.b",
            "user@example.com",
            "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
        ];
        let mut state = 0x5851_f42d_4c95_7f2du64;
        let mut next = move || {
            state = state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            (state >> 33) as usize
        };
        for _ in 0..6000 {
            let len = next() % 80;
            // A NUL ends the collection, so most mixes leave it out.
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

    #[test]
    fn long_bodies() {
        let prose = "Plain prose with [links](https://example.com) and `code: spans`, \
                     now with www.example.org and a@b.c.\n";
        let long = prose.repeat(512);
        check(long.as_bytes());
        check(format!("{long}]:").as_bytes());
        check(format!("{long}\0").as_bytes());
        check(format!("[a]: /url\n{long}").as_bytes());
        check(format!("{long}[a]: /url\n{long}").as_bytes());
        check(format!("{long}[a]: /url\n{long}\0{long}").as_bytes());
    }
}
