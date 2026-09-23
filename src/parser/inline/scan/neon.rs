//! AArch64 NEON backend for the shared nibble classifier.

use super::super::gfm_autolink::AutolinkFacts;
use super::scalar::{
    next_inline_special_options_scalar, next_marker_tracking_scalar, visit_autolink_triggers_scalar,
};
use super::{
    HIGH_NIBBLE, INLINE_SPECIAL, LOW_NIBBLE, is_autolink_trigger, selected_marker_tables,
    selected_tracking_tables,
};

#[allow(unsafe_code)]
#[inline]
fn next_special_neon_with_tables(
    bytes: &[u8],
    from: usize,
    low_table: &[u8; 16],
    high_table: &[u8; 16],
) -> usize {
    use std::arch::aarch64::*;
    let end = bytes.len();
    let mut i = from;
    unsafe {
        let low = vld1q_u8(low_table.as_ptr());
        let high = vld1q_u8(high_table.as_ptr());
        let nibble = vdupq_n_u8(0x0F);
        let classify = |v: uint8x16_t| {
            let lo = vqtbl1q_u8(low, vandq_u8(v, nibble));
            let hi = vqtbl1q_u8(high, vshrq_n_u8(v, 4));
            // `vtstq_u8` turns the per-lane AND into the all-ones/all-zeros
            // form the nibble-narrowing mask extraction needs.
            let m = vtstq_u8(lo, hi);
            let narrow = vshrn_n_u16(vreinterpretq_u16_u8(m), 4);
            vget_lane_u64(vreinterpret_u64_u8(narrow), 0)
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
            // the loops already cleared. `vtstq_u8` is exact per lane, so
            // masking off processed bytes cannot leak a match into a
            // neighbour the way a SWAR zero-test can.
            let base = end - 16;
            let mask =
                classify(vld1q_u8(bytes.as_ptr().add(base))) & (u64::MAX << ((i - base) * 4));
            if mask != 0 {
                return base + (mask.trailing_zeros() / 4) as usize;
            }
            return end;
        }
    }
    while i < end && INLINE_SPECIAL[bytes[i] as usize] == 0 {
        i += 1;
    }
    i
}

#[inline]
pub(super) fn next_special_neon(bytes: &[u8], from: usize) -> usize {
    next_special_neon_with_tables(bytes, from, &LOW_NIBBLE, &HIGH_NIBBLE)
}

#[inline]
pub(super) fn next_special_neon_options(bytes: &[u8], from: usize, options: u8) -> usize {
    if bytes.len().saturating_sub(from) < 16 {
        return next_inline_special_options_scalar(bytes, from, options);
    }
    let (low, high) = selected_marker_tables(options);
    next_special_neon_with_tables(bytes, from, low, high)
}

/// One bit per lane of a nibble-narrowed mask, so `bits & (bits - 1)`
/// clears exactly one lane.
const LANE_BITS: u64 = 0x8888_8888_8888_8888;

/// Walks the stop lanes of one vector at `base` in order: autolink triggers
/// the watermark has not passed are visited, and the first marker is
/// returned with the watermark moved behind it.
///
/// A stop lane is a marker exactly when its byte is not a trigger — the
/// tracking tables add only `@` and `:` to the marker set, the pair test only
/// `.`, and no marker is any of the three.
#[inline]
fn resolve_stops(bytes: &[u8], base: usize, mask: u64, facts: &mut AutolinkFacts) -> Option<usize> {
    let mut lanes = mask & LANE_BITS;
    while lanes != 0 {
        let at = base + (lanes.trailing_zeros() / 4) as usize;
        if !is_autolink_trigger(bytes[at]) {
            facts.advance_seen(at + 1);
            return Some(at);
        }
        if at >= facts.seen() {
            facts.visit(bytes, at);
        }
        lanes &= lanes - 1;
    }
    None
}

/// The tracking form of [`next_special_neon_with_tables`]: the same marker
/// position, found by the same nibble classifier, while every autolink
/// trigger before it is visited on the way.
///
/// `@` and `:` ride in the classifier tables as extra stops, so they cost
/// nothing until one occurs. The `w.` pair needs the byte in front of every
/// lane, which `vextq_u8` takes from the previous vector (`carry`, whose last
/// lane is primed with the byte before the scan start). Those five vector
/// operations are the price the marker loop pays for making the pre-flight's
/// own pass over this text unnecessary.
///
/// The caller has visited every trigger before `from` (`from <= seen`).
#[allow(unsafe_code)]
#[inline]
pub(super) fn next_marker_tracking_neon(
    bytes: &[u8],
    from: usize,
    options: u8,
    facts: &mut AutolinkFacts,
) -> usize {
    use std::arch::aarch64::*;
    let end = bytes.len();
    if end < 16 {
        return next_marker_tracking_scalar(bytes, from, options, facts);
    }
    let (low_table, high_table) = selected_tracking_tables(options);
    let mut i = from;
    // SAFETY: NEON is part of the aarch64 baseline, and every load below
    // reads 16 bytes at an offset the surrounding bounds keep within
    // `bytes`: `i + 16 <= end` in the loops, `end - 16` in the tail.
    unsafe {
        let low = vld1q_u8(low_table.as_ptr());
        let high = vld1q_u8(high_table.as_ptr());
        let nibble = vdupq_n_u8(0x0F);
        let dot = vdupq_n_u8(b'.');
        let w = vdupq_n_u8(b'w');
        let classify = |v: uint8x16_t, prev: uint8x16_t| {
            let lo = vqtbl1q_u8(low, vandq_u8(v, nibble));
            let hi = vqtbl1q_u8(high, vshrq_n_u8(v, 4));
            let stops = vorrq_u8(
                vtstq_u8(lo, hi),
                vandq_u8(vceqq_u8(v, dot), vceqq_u8(prev, w)),
            );
            let narrow = vshrn_n_u16(vreinterpretq_u16_u8(stops), 4);
            vget_lane_u64(vreinterpret_u64_u8(narrow), 0)
        };
        // A vector whose last lane is the byte in front of `at`, so that
        // `vextq_u8(carry, v, 15)` lines every lane up with its predecessor.
        // Nothing precedes the content; a `.` at 0 cannot end `www.`.
        let before = |at: usize| vdupq_n_u8(if at > 0 { bytes[at - 1] } else { 0 });
        let mut carry = before(i);
        while i + 32 <= end {
            let v0 = vld1q_u8(bytes.as_ptr().add(i));
            let v1 = vld1q_u8(bytes.as_ptr().add(i + 16));
            let m0 = classify(v0, vextq_u8(carry, v0, 15));
            if m0 != 0
                && let Some(stop) = resolve_stops(bytes, i, m0, facts)
            {
                return stop;
            }
            let m1 = classify(v1, vextq_u8(v0, v1, 15));
            if m1 != 0
                && let Some(stop) = resolve_stops(bytes, i + 16, m1, facts)
            {
                return stop;
            }
            carry = v1;
            i += 32;
        }
        while i + 16 <= end {
            let v = vld1q_u8(bytes.as_ptr().add(i));
            let mask = classify(v, vextq_u8(carry, v, 15));
            if mask != 0
                && let Some(stop) = resolve_stops(bytes, i, mask, facts)
            {
                return stop;
            }
            carry = v;
            i += 16;
        }
        if i < end {
            // Overlapping tail, as in the plain scan: the lanes before `i`
            // were classified by the loops above or lie before `from`, and
            // their triggers were visited either way.
            let base = end - 16;
            let v = vld1q_u8(bytes.as_ptr().add(base));
            let mask = classify(v, vextq_u8(before(base), v, 15)) & (u64::MAX << ((i - base) * 4));
            if mask != 0
                && let Some(stop) = resolve_stops(bytes, base, mask, facts)
            {
                return stop;
            }
        }
    }
    facts.advance_seen(end);
    end
}

/// Visits every autolink trigger in `from..to` with the trigger test alone,
/// for the bytes a construct consumed without the marker scan classifying
/// them. The bytes around the range are read as context only.
///
/// `.` is too common to visit on its own, so the `www.` needle is caught by
/// its last two bytes, as in the marker scan; `AutolinkFacts::visit` checks
/// the rest. The pair test drops only `.` lanes not preceded by `w`, and
/// such a `.` cannot end a `www.`.
#[allow(unsafe_code)]
pub(super) fn visit_autolink_triggers_neon(
    bytes: &[u8],
    from: usize,
    to: usize,
    facts: &mut AutolinkFacts,
) {
    use std::arch::aarch64::*;
    if to - from < 16 {
        visit_autolink_triggers_scalar(bytes, from, to, facts);
        return;
    }
    // SAFETY: NEON is part of the aarch64 baseline; `to <= bytes.len()` and
    // every load reads 16 bytes at `i` with `i + 16 <= to`, or at `to - 16`,
    // which is at least `from` here.
    unsafe {
        let at_sign = vdupq_n_u8(b'@');
        let colon = vdupq_n_u8(b':');
        let dot = vdupq_n_u8(b'.');
        let w = vdupq_n_u8(b'w');
        let triggers = |v: uint8x16_t, prev: uint8x16_t| {
            let lanes = vorrq_u8(
                vorrq_u8(vceqq_u8(v, at_sign), vceqq_u8(v, colon)),
                vandq_u8(vceqq_u8(v, dot), vceqq_u8(prev, w)),
            );
            let narrow = vshrn_n_u16(vreinterpretq_u16_u8(lanes), 4);
            vget_lane_u64(vreinterpret_u64_u8(narrow), 0)
        };
        let before = |at: usize| vdupq_n_u8(if at > 0 { bytes[at - 1] } else { 0 });
        let visit_lanes = |facts: &mut AutolinkFacts, base: usize, mask: u64| {
            let mut lanes = mask & LANE_BITS;
            while lanes != 0 {
                facts.visit(bytes, base + (lanes.trailing_zeros() / 4) as usize);
                lanes &= lanes - 1;
            }
        };
        let mut i = from;
        let mut carry = before(i);
        while i + 16 <= to {
            let v = vld1q_u8(bytes.as_ptr().add(i));
            let mask = triggers(v, vextq_u8(carry, v, 15));
            if mask != 0 {
                visit_lanes(facts, i, mask);
            }
            carry = v;
            i += 16;
        }
        if i < to {
            let base = to - 16;
            let v = vld1q_u8(bytes.as_ptr().add(base));
            let mask = triggers(v, vextq_u8(before(base), v, 15)) & (u64::MAX << ((i - base) * 4));
            if mask != 0 {
                visit_lanes(facts, base, mask);
            }
        }
    }
}
