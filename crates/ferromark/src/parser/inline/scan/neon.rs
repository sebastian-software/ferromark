//! AArch64 NEON backend for the shared nibble classifier.

use super::scalar::next_inline_special_options_scalar;
use super::{HIGH_NIBBLE, INLINE_SPECIAL, LOW_NIBBLE, selected_marker_tables};

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
