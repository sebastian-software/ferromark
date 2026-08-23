//! SIMD helpers for inline parsing.
//!
//! AArch64 uses NEON and x86-64 uses its baseline SSE2 support. Other
//! architectures keep the scalar fallback in the callers.

#[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
use std::arch::aarch64::*;

#[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
#[inline]
#[target_feature(enable = "neon")]
unsafe fn any_eq_mask(v: uint8x16_t, bytes: &[u8]) -> uint8x16_t {
    let mut mask = vdupq_n_u8(0);
    for &b in bytes {
        let m = vceqq_u8(v, vdupq_n_u8(b));
        mask = vorrq_u8(mask, m);
    }
    mask
}

#[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
#[inline]
#[target_feature(enable = "neon")]
unsafe fn mask_has_any(mask: uint8x16_t) -> bool {
    vmaxvq_u8(mask) != 0
}

#[cfg(any(
    target_arch = "x86_64",
    all(target_arch = "aarch64", target_feature = "neon")
))]
#[inline]
fn is_inline_special<const HIGHLIGHT: bool, const SUPERSCRIPT: bool>(b: u8) -> bool {
    matches!(
        b,
        b'*' | b'_' | b'`' | b'[' | b']' | b'<' | b'\\' | b'\n' | b'~' | b'$'
    ) || (HIGHLIGHT && b == b'=')
        || (SUPERSCRIPT && b == b'^')
}

#[cfg(any(test, all(target_arch = "aarch64", target_feature = "neon")))]
#[inline]
fn is_mark_special<const HIGHLIGHT: bool, const SUPERSCRIPT: bool>(b: u8) -> bool {
    matches!(
        b,
        b'`' | b'*' | b'_' | b'\\' | b'\n' | b'[' | b']' | b'<' | b'~' | b'$'
    ) || (HIGHLIGHT && b == b'=')
        || (SUPERSCRIPT && b == b'^')
}

/// SIMD-accelerated check for inline specials.
/// Returns Some(result) if SIMD path was used, otherwise None.
#[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
#[target_feature(enable = "neon")]
pub unsafe fn has_inline_specials_simd<const HIGHLIGHT: bool, const SUPERSCRIPT: bool>(
    input: &[u8],
) -> Option<bool> {
    let len = input.len();
    let mut pos = 0usize;
    while pos + 16 <= len {
        unsafe {
            let v = vld1q_u8(input.as_ptr().add(pos));
            let mask = if HIGHLIGHT && SUPERSCRIPT {
                any_eq_mask(v, b"*_`[]<\\\n~$=^")
            } else if HIGHLIGHT {
                any_eq_mask(v, b"*_`[]<\\\n~$=")
            } else if SUPERSCRIPT {
                any_eq_mask(v, b"*_`[]<\\\n~$^")
            } else {
                any_eq_mask(v, b"*_`[]<\\\n~$")
            };
            if mask_has_any(mask) {
                return Some(true);
            }
        }
        pos += 16;
    }
    // Fallback for tail.
    for &b in &input[pos..] {
        if is_inline_special::<HIGHLIGHT, SUPERSCRIPT>(b) {
            return Some(true);
        }
    }
    Some(false)
}

/// SIMD scan for next inline mark special used by mark collection.
/// Advances `pos` to the end of SIMD-scanned region if no hit.
#[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
#[target_feature(enable = "neon")]
pub unsafe fn next_mark_special_simd<const HIGHLIGHT: bool, const SUPERSCRIPT: bool>(
    text: &[u8],
    pos: &mut usize,
) -> Option<usize> {
    let len = text.len();
    let mut p = *pos;
    while p + 16 <= len {
        unsafe {
            let v = vld1q_u8(text.as_ptr().add(p));
            let mask = if HIGHLIGHT && SUPERSCRIPT {
                any_eq_mask(v, b"`*_\\\n[]<~$=^")
            } else if HIGHLIGHT {
                any_eq_mask(v, b"`*_\\\n[]<~$=")
            } else if SUPERSCRIPT {
                any_eq_mask(v, b"`*_\\\n[]<~$^")
            } else {
                any_eq_mask(v, b"`*_\\\n[]<~$")
            };
            if mask_has_any(mask) {
                // Find first match within the chunk.
                for i in 0..16 {
                    if is_mark_special::<HIGHLIGHT, SUPERSCRIPT>(text[p + i]) {
                        return Some(p + i);
                    }
                }
            }
        }
        p += 16;
    }
    *pos = p;
    None
}

/// Bitmask of lanes equal to an inline or mark special byte.
#[cfg(target_arch = "x86_64")]
#[inline]
fn special_mask_sse2<const HIGHLIGHT: bool, const SUPERSCRIPT: bool>(
    v: std::arch::x86_64::__m128i,
) -> u32 {
    use std::arch::x86_64::*;

    // SAFETY: SSE2 is part of the x86-64 baseline. `v` was loaded from a
    // valid 16-byte input range by the caller; these operations do not impose
    // additional alignment or pointer requirements.
    unsafe {
        let mut mask = _mm_cmpeq_epi8(v, _mm_set1_epi8(b'*' as i8));
        for byte in b"_`[]<\\\n~$" {
            mask = _mm_or_si128(mask, _mm_cmpeq_epi8(v, _mm_set1_epi8(*byte as i8)));
        }
        if HIGHLIGHT {
            mask = _mm_or_si128(mask, _mm_cmpeq_epi8(v, _mm_set1_epi8(b'=' as i8)));
        }
        if SUPERSCRIPT {
            mask = _mm_or_si128(mask, _mm_cmpeq_epi8(v, _mm_set1_epi8(b'^' as i8)));
        }
        _mm_movemask_epi8(mask) as u32
    }
}

/// SSE2-accelerated check for inline specials on x86-64.
///
/// SSE2 is guaranteed by the x86-64 architecture, so unlike optional x86
/// features this path does not require runtime feature detection.
#[cfg(target_arch = "x86_64")]
#[inline]
pub unsafe fn has_inline_specials_simd<const HIGHLIGHT: bool, const SUPERSCRIPT: bool>(
    input: &[u8],
) -> Option<bool> {
    use std::arch::x86_64::_mm_loadu_si128;

    let mut pos = 0;
    while pos + 16 <= input.len() {
        // SAFETY: `pos + 16 <= input.len()` proves that this unaligned
        // 16-byte load is in bounds. SSE2 is guaranteed on x86-64.
        let vector = unsafe { _mm_loadu_si128(input.as_ptr().add(pos).cast()) };
        if special_mask_sse2::<HIGHLIGHT, SUPERSCRIPT>(vector) != 0 {
            return Some(true);
        }
        pos += 16;
    }

    Some(
        input[pos..]
            .iter()
            .copied()
            .any(is_inline_special::<HIGHLIGHT, SUPERSCRIPT>),
    )
}

/// SSE2 scan for the next inline mark special used by mark collection.
#[cfg(target_arch = "x86_64")]
#[inline]
pub unsafe fn next_mark_special_simd<const HIGHLIGHT: bool, const SUPERSCRIPT: bool>(
    text: &[u8],
    pos: &mut usize,
) -> Option<usize> {
    use std::arch::x86_64::_mm_loadu_si128;

    let mut scan = *pos;
    while scan + 16 <= text.len() {
        // SAFETY: `scan + 16 <= text.len()` proves that this unaligned
        // 16-byte load is in bounds. SSE2 is guaranteed on x86-64.
        let vector = unsafe { _mm_loadu_si128(text.as_ptr().add(scan).cast()) };
        let mask = special_mask_sse2::<HIGHLIGHT, SUPERSCRIPT>(vector);
        if mask != 0 {
            return Some(scan + mask.trailing_zeros() as usize);
        }
        scan += 16;
    }
    *pos = scan;
    None
}

#[cfg(not(any(
    target_arch = "x86_64",
    all(target_arch = "aarch64", target_feature = "neon")
)))]
#[allow(dead_code)]
pub fn has_inline_specials_simd<const HIGHLIGHT: bool, const SUPERSCRIPT: bool>(
    _input: &[u8],
) -> Option<bool> {
    None
}

#[cfg(not(any(
    target_arch = "x86_64",
    all(target_arch = "aarch64", target_feature = "neon")
)))]
#[allow(dead_code)]
pub fn next_mark_special_simd<const HIGHLIGHT: bool, const SUPERSCRIPT: bool>(
    _text: &[u8],
    _pos: &mut usize,
) -> Option<usize> {
    None
}

#[cfg(all(
    test,
    any(
        target_arch = "x86_64",
        all(target_arch = "aarch64", target_feature = "neon")
    )
))]
mod tests {
    use super::*;

    fn scalar_inline<const HIGHLIGHT: bool, const SUPERSCRIPT: bool>(input: &[u8]) -> bool {
        input
            .iter()
            .copied()
            .any(is_inline_special::<HIGHLIGHT, SUPERSCRIPT>)
    }

    fn scalar_mark<const HIGHLIGHT: bool, const SUPERSCRIPT: bool>(input: &[u8]) -> Option<usize> {
        input
            .iter()
            .position(|&byte| is_mark_special::<HIGHLIGHT, SUPERSCRIPT>(byte))
    }

    fn assert_inline_parity<const HIGHLIGHT: bool, const SUPERSCRIPT: bool>() {
        for len in [0, 1, 15, 16, 17, 31, 32, 33, 63, 64, 65] {
            for byte in 0..=u8::MAX {
                for special_pos in 0..=len {
                    let mut input = vec![b'a'; len];
                    if special_pos < len {
                        input[special_pos] = byte;
                    }
                    // SAFETY: this test only compiles for NEON-enabled AArch64 or
                    // x86-64, whose SIMD path has no caller-controlled precondition.
                    let simd =
                        unsafe { has_inline_specials_simd::<HIGHLIGHT, SUPERSCRIPT>(&input) }
                            .expect("SIMD implementation is compiled for this target");
                    assert_eq!(simd, scalar_inline::<HIGHLIGHT, SUPERSCRIPT>(&input));
                }
            }
        }
    }

    fn assert_mark_parity<const HIGHLIGHT: bool, const SUPERSCRIPT: bool>() {
        for len in [0, 1, 15, 16, 17, 31, 32, 33, 63, 64, 65] {
            for byte in 0..=u8::MAX {
                for special_pos in 0..=len {
                    let mut input = vec![b'a'; len];
                    if special_pos < len {
                        input[special_pos] = byte;
                    }
                    let mut pos = 0;
                    // SAFETY: see `assert_inline_parity`.
                    let simd = unsafe {
                        next_mark_special_simd::<HIGHLIGHT, SUPERSCRIPT>(&input, &mut pos)
                    };
                    let scanned_len = input.len() / 16 * 16;
                    let scalar = scalar_mark::<HIGHLIGHT, SUPERSCRIPT>(&input[..scanned_len]);
                    assert_eq!(simd, scalar);
                    if scalar.is_none() {
                        assert_eq!(pos, scanned_len);
                    }
                }
            }
        }
    }

    #[test]
    fn inline_specials_match_scalar_at_vector_boundaries() {
        assert_inline_parity::<false, false>();
        assert_inline_parity::<true, false>();
        assert_inline_parity::<false, true>();
        assert_inline_parity::<true, true>();
    }

    #[test]
    fn mark_specials_match_scalar_at_vector_boundaries() {
        assert_mark_parity::<false, false>();
        assert_mark_parity::<true, false>();
        assert_mark_parity::<false, true>();
        assert_mark_parity::<true, true>();
    }
}
