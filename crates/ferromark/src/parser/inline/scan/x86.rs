//! x86-64 SSSE3 and AVX2 backends, called after runtime feature detection.

use super::scalar::{next_inline_special_options_scalar, next_inline_special_scalar};
use super::{HIGH_NIBBLE, LOW_NIBBLE, selected_marker_tables};

/// SSSE3 counterpart of the NEON classifier. `pshufb` is the same 16-entry
/// table lookup `vqtbl1q_u8` performs, so both paths run the identical
/// classifier and are covered by the same differential tests.
///
/// # Safety
///
/// The caller must have verified SSSE3 support. Every load is bounded by
/// the length checks around it.
#[allow(unsafe_code)]
#[target_feature(enable = "ssse3")]
unsafe fn next_special_ssse3_with_tables(
    bytes: &[u8],
    from: usize,
    low_table: &[u8; 16],
    high_table: &[u8; 16],
) -> usize {
    use std::arch::x86_64::*;
    let end = bytes.len();
    let mut i = from;
    unsafe {
        let low = _mm_loadu_si128(low_table.as_ptr().cast());
        let high = _mm_loadu_si128(high_table.as_ptr().cast());
        let nibble = _mm_set1_epi8(0x0F);
        let classify = |v: __m128i| {
            // `_mm_srli_epi16` shifts 16-bit lanes, so the neighbouring
            // byte's low bits ride along; masking leaves the high nibble.
            let lo = _mm_shuffle_epi8(low, _mm_and_si128(v, nibble));
            let hi = _mm_shuffle_epi8(high, _mm_and_si128(_mm_srli_epi16(v, 4), nibble));
            let m = _mm_and_si128(lo, hi);
            // `movemask` of "lane is zero" sets a bit per *clean* byte, so
            // the complement's lowest set bit is the first marker.
            // `movemask` fills only the low 16 bits, so the cast is exact
            // and the complement below stays inside them.
            let clean = _mm_movemask_epi8(_mm_cmpeq_epi8(m, _mm_setzero_si128()));
            u32::from_ne_bytes((!clean & 0xFFFF).to_ne_bytes())
        };
        while i + 32 <= end {
            let m0 = classify(_mm_loadu_si128(bytes.as_ptr().add(i).cast()));
            if m0 != 0 {
                return i + m0.trailing_zeros() as usize;
            }
            let m1 = classify(_mm_loadu_si128(bytes.as_ptr().add(i + 16).cast()));
            if m1 != 0 {
                return i + 16 + m1.trailing_zeros() as usize;
            }
            i += 32;
        }
        while i + 16 <= end {
            let flagged = classify(_mm_loadu_si128(bytes.as_ptr().add(i).cast()));
            if flagged != 0 {
                return i + flagged.trailing_zeros() as usize;
            }
            i += 16;
        }
        if i < end && end >= 16 {
            let base = end - 16;
            let flagged = classify(_mm_loadu_si128(bytes.as_ptr().add(base).cast()))
                & (u32::MAX << (i - base));
            if flagged != 0 {
                return base + flagged.trailing_zeros() as usize;
            }
            return end;
        }
    }
    next_inline_special_scalar(bytes, i)
}

/// # Safety
/// The caller must have verified SSSE3 support.
#[allow(unsafe_code)]
#[inline]
pub(super) unsafe fn next_special_ssse3(bytes: &[u8], from: usize) -> usize {
    // SAFETY: the caller guarantees SSSE3; the backend bounds every load.
    unsafe { next_special_ssse3_with_tables(bytes, from, &LOW_NIBBLE, &HIGH_NIBBLE) }
}

/// # Safety
/// The caller must have verified SSSE3 support.
#[allow(unsafe_code)]
#[inline]
pub(super) unsafe fn next_special_ssse3_options(bytes: &[u8], from: usize, options: u8) -> usize {
    if bytes.len().saturating_sub(from) < 16 {
        return next_inline_special_options_scalar(bytes, from, options);
    }
    let (low, high) = selected_marker_tables(options);
    // SAFETY: the caller guarantees SSSE3; the backend bounds every load.
    unsafe { next_special_ssse3_with_tables(bytes, from, low, high) }
}

/// AVX2 sibling: the same nibble classifier, 32 bytes at a time.
///
/// `vpshufb` is per 128-bit lane, so broadcasting the 16-entry tables into
/// both lanes of a YMM register classifies 32 bytes with one shuffle pair.
///
/// # Safety
///
/// The caller must have verified AVX2 support. Every load is bounded by
/// the length checks around it.
#[allow(unsafe_code)]
#[target_feature(enable = "avx2")]
unsafe fn next_special_avx2_with_tables(
    bytes: &[u8],
    from: usize,
    low_table: &[u8; 16],
    high_table: &[u8; 16],
) -> usize {
    use std::arch::x86_64::*;
    let end = bytes.len();
    let mut i = from;
    unsafe {
        let low = _mm256_broadcastsi128_si256(_mm_loadu_si128(low_table.as_ptr().cast()));
        let high = _mm256_broadcastsi128_si256(_mm_loadu_si128(high_table.as_ptr().cast()));
        let nibble = _mm256_set1_epi8(0x0F);
        let classify = |v: __m256i| {
            let lo = _mm256_shuffle_epi8(low, _mm256_and_si256(v, nibble));
            let hi = _mm256_shuffle_epi8(high, _mm256_and_si256(_mm256_srli_epi16(v, 4), nibble));
            let m = _mm256_and_si256(lo, hi);
            let clean = _mm256_movemask_epi8(_mm256_cmpeq_epi8(m, _mm256_setzero_si256()));
            u32::from_ne_bytes((!clean).to_ne_bytes())
        };
        while i + 32 <= end {
            let flagged = classify(_mm256_loadu_si256(bytes.as_ptr().add(i).cast()));
            if flagged != 0 {
                return i + flagged.trailing_zeros() as usize;
            }
            i += 32;
        }
        if i < end && end >= 32 {
            let base = end - 32;
            let flagged = classify(_mm256_loadu_si256(bytes.as_ptr().add(base).cast()))
                & (u32::MAX << (i - base));
            if flagged != 0 {
                return base + flagged.trailing_zeros() as usize;
            }
            return end;
        }
    }
    next_inline_special_scalar(bytes, i)
}

/// # Safety
/// The caller must have verified AVX2 support.
#[allow(unsafe_code)]
#[inline]
pub(super) unsafe fn next_special_avx2(bytes: &[u8], from: usize) -> usize {
    // SAFETY: the caller guarantees AVX2; the backend bounds every load.
    unsafe { next_special_avx2_with_tables(bytes, from, &LOW_NIBBLE, &HIGH_NIBBLE) }
}

/// # Safety
/// The caller must have verified AVX2 support.
#[allow(unsafe_code)]
#[inline]
pub(super) unsafe fn next_special_avx2_options(bytes: &[u8], from: usize, options: u8) -> usize {
    if bytes.len().saturating_sub(from) < 32 {
        return next_inline_special_options_scalar(bytes, from, options);
    }
    let (low, high) = selected_marker_tables(options);
    // SAFETY: the caller guarantees AVX2; the backend bounds every load.
    unsafe { next_special_avx2_with_tables(bytes, from, low, high) }
}
