//! Recording where the autolink trigger bytes of a byte range are.
//!
//! A trigger is any byte that [`is_autolink_trigger`] accepts: every `@`, and
//! every `:` or `w` that a `.` or `/` follows. That keeps every `:` that starts
//! a `://` and every `w` that ends a `www.`, and leaves out the colons of
//! ordinary prose and code (`Note:`, `a::b`, `key: value`), which are most of
//! them.

use crate::allocator::Vec;

/// Whether `byte` is an autolink trigger, given the byte after it (NUL past
/// the end of the body).
///
/// `next | 1 == b'/'` holds for exactly `.` (0x2E) and `/` (0x2F), so the pair
/// test costs one compare in the vector loop. The two extra pairs it lets in,
/// `:.` and `w/`, are rare and cost only an entry the pre-flight skips.
#[inline]
pub(super) const fn is_autolink_trigger(byte: u8, next: u8) -> bool {
    byte == b'@' || ((byte == b':' || byte == b'w') && next | 1 == b'/')
}

/// Pushes the offset of every trigger byte in `body[from..to]` onto `offsets`,
/// in ascending order. The byte after `to - 1` is read from `body` when it
/// has one, so a trigger's pair can straddle `to`.
///
/// `body` must be shorter than 4 GiB: offsets are stored as `u32`.
#[inline]
pub(super) fn record(body: &[u8], from: usize, to: usize, offsets: &mut Vec<'_, u32>) {
    debug_assert!(from <= to && to <= body.len());
    debug_assert!(u32::try_from(body.len()).is_ok());
    #[cfg(target_arch = "aarch64")]
    {
        if to >= 16 {
            neon::record(body, from, to, offsets);
            return;
        }
    }
    record_bytewise(body, from, to, offsets);
}

/// [`record`] one byte at a time: the answer for a range that ends before the
/// body's 16th byte, the tests' reference for the vector loop, and the whole
/// recorder on other targets, where only the tests ask for it.
fn record_bytewise(body: &[u8], from: usize, to: usize, offsets: &mut Vec<'_, u32>) {
    for at in from..to {
        if is_autolink_trigger(body[at], body.get(at + 1).copied().unwrap_or(0)) {
            offsets.push(at as u32);
        }
    }
}

/// The NEON recorder: eight vector operations per 16 bytes (compares for `@`,
/// `:` and `w`, one for the byte after each lane, `next | 1 == b'/'`, and four
/// to combine them), and one branch per 64 bytes, taken only for a group that
/// holds a trigger.
#[cfg(target_arch = "aarch64")]
#[allow(unsafe_code)]
mod neon {
    use std::arch::aarch64::*;

    use crate::allocator::Vec;

    /// Pushes `from` plus the index of every lane set in `mask`, a lane mask
    /// with four bits per lane.
    #[inline]
    fn push_lanes(mask: u64, from: usize, offsets: &mut Vec<'_, u32>) {
        let mut bits = mask & 0x8888_8888_8888_8888;
        while bits != 0 {
            offsets.push((from + (bits.trailing_zeros() / 4) as usize) as u32);
            bits &= bits - 1;
        }
    }

    pub(super) fn record(body: &[u8], from: usize, to: usize, offsets: &mut Vec<'_, u32>) {
        let len = body.len();
        let ptr = body.as_ptr();
        debug_assert!(from <= to && to <= len && to >= 16);
        let mut at = from;
        // SAFETY: a block at `at` loads `at..at + 16`, and its lookahead loads
        // `at + 1..at + 17`. The grouped loop runs only while the lookahead of
        // its fourth block ends in bounds (`at + 65 <= len`), and the block
        // loop only while its own does (`at + 17 <= len`). The last vector
        // loads `to - 16..to`, with `to >= 16`, and its lookahead
        // `to - 15..to + 1` only when `to < len`.
        unsafe {
            let at_sign = vdupq_n_u8(b'@');
            let colon = vdupq_n_u8(b':');
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
            while at + 64 <= to && at + 65 <= len {
                let hit_a = trigger(vld1q_u8(ptr.add(at)), vld1q_u8(ptr.add(at + 1)));
                let hit_b = trigger(vld1q_u8(ptr.add(at + 16)), vld1q_u8(ptr.add(at + 17)));
                let hit_c = trigger(vld1q_u8(ptr.add(at + 32)), vld1q_u8(ptr.add(at + 33)));
                let hit_d = trigger(vld1q_u8(ptr.add(at + 48)), vld1q_u8(ptr.add(at + 49)));
                if lanes(vorrq_u8(vorrq_u8(hit_a, hit_b), vorrq_u8(hit_c, hit_d))) != 0 {
                    push_lanes(lanes(hit_a), at, offsets);
                    push_lanes(lanes(hit_b), at + 16, offsets);
                    push_lanes(lanes(hit_c), at + 32, offsets);
                    push_lanes(lanes(hit_d), at + 48, offsets);
                }
                at += 64;
            }
            while at + 16 <= to && at + 17 <= len {
                let hit = trigger(vld1q_u8(ptr.add(at)), vld1q_u8(ptr.add(at + 1)));
                push_lanes(lanes(hit), at, offsets);
                at += 16;
            }
            if at < to {
                // One vector over the last 16 bytes of the range answers for
                // the 1 to 16 bytes the loops left; its lanes in front of `at`
                // are recorded already, or lie before `from`. At the end of
                // the body a zero lane stands in for the byte after the last.
                let base = to - 16;
                let here = vld1q_u8(ptr.add(base));
                let next = if to < len {
                    vld1q_u8(ptr.add(base + 1))
                } else {
                    vextq_u8::<1>(here, vdupq_n_u8(0))
                };
                let seen = at - base;
                push_lanes(
                    lanes(trigger(here, next)) & (u64::MAX << (4 * seen)),
                    base,
                    offsets,
                );
            }
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

    use super::{is_autolink_trigger, record, record_bytewise};

    /// Every trigger offset in `body[from..to]`, from the definition.
    fn oracle(body: &[u8], from: usize, to: usize) -> Vec<u32> {
        (from..to)
            .filter(|&at| is_autolink_trigger(body[at], body.get(at + 1).copied().unwrap_or(0)))
            .map(|at| at as u32)
            .collect()
    }

    #[track_caller]
    fn check(body: &[u8], from: usize, to: usize) {
        let expected = oracle(body, from, to);
        let allocator = Allocator::new();
        let mut offsets = allocator.new_vec();
        record(body, from, to, &mut offsets);
        let shown = String::from_utf8_lossy(body);
        assert_eq!(
            offsets.as_slice(),
            expected.as_slice(),
            "{from}..{to} of {shown:?}"
        );
        let mut offsets = allocator.new_vec();
        record_bytewise(body, from, to, &mut offsets);
        assert_eq!(
            offsets.as_slice(),
            expected.as_slice(),
            "bytewise {from}..{to}"
        );
    }

    /// Every range of `body`, when it is short, or every range between the
    /// points where the vector loops change step: the body's ends, one
    /// vector in from each, its middle, and `extra`.
    fn check_ranges(body: &[u8], extra: &[usize]) {
        let len = body.len();
        if len <= 40 {
            for from in 0..=len {
                for to in from..=len {
                    check(body, from, to);
                }
            }
            return;
        }
        let mut points = vec![0, 1, 15, 16, 17, len / 2, len - 17, len - 16, len - 1, len];
        points.extend(extra.iter().flat_map(|&at| {
            [
                at.saturating_sub(16),
                at.saturating_sub(1),
                at,
                at + 1,
                at + 16,
            ]
        }));
        points.retain(|&at| at <= len);
        points.sort_unstable();
        points.dedup();
        for &from in &points {
            for &to in points.iter().filter(|&&to| to >= from) {
                check(body, from, to);
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
    fn every_range_of_short_bodies_over_the_deciding_bytes() {
        let alphabet = [b'x', b'@', b':', b'w', b'.', b'/'];
        for len in 0..=5u32 {
            for mut code in 0..alphabet.len().pow(len) {
                let mut body = Vec::new();
                for _ in 0..len {
                    body.push(alphabet[code % alphabet.len()]);
                    code /= alphabet.len();
                }
                check_ranges(&body, &[]);
            }
        }
    }

    #[test]
    fn a_needle_at_every_offset() {
        let needles: [&[u8]; 9] = [
            b"@", b":/", b"://", b"w.", b"www.", b":.", b"w/", b":", b"w",
        ];
        for len in [15, 16, 17, 31, 33, 63, 64, 65, 66, 80, 127, 128, 129, 150] {
            for filler in [b'x', b'w', b':', b'.'] {
                for needle in needles {
                    for at in 0..=len - needle.len() {
                        let mut body = vec![filler; len];
                        body[at..at + needle.len()].copy_from_slice(needle);
                        check_ranges(&body, &[at, at + needle.len()]);
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
        for _ in 0..3000 {
            let len = next() % 40;
            let mut body = String::new();
            for _ in 0..len {
                body.push_str(tokens[next() % tokens.len()]);
            }
            let points = [next() % (body.len() + 1), next() % (body.len() + 1)];
            check_ranges(body.as_bytes(), &points);
        }
    }
}
