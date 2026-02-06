//! SIMD helpers for inline parsing (AArch64 NEON).

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

#[inline]
fn is_inline_special(b: u8) -> bool {
    matches!(b, b'*' | b'_' | b'`' | b'[' | b']' | b'<' | b'&' | b'\\' | b'\n')
}

#[inline]
fn is_mark_special(b: u8) -> bool {
    matches!(b, b'`' | b'*' | b'_' | b'\\' | b'\n' | b'[' | b']' | b'<' | b'&')
}

/// SIMD-accelerated check for inline specials.
/// Returns Some(result) if SIMD path was used, otherwise None.
#[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
#[target_feature(enable = "neon")]
pub unsafe fn has_inline_specials_simd(input: &[u8]) -> Option<bool> {
    let len = input.len();
    let mut pos = 0usize;
    while pos + 16 <= len {
        unsafe {
            let v = vld1q_u8(input.as_ptr().add(pos));
            let mask = any_eq_mask(v, b"*_`[]<&\\\n");
            if mask_has_any(mask) {
                return Some(true);
            }
        }
        pos += 16;
    }
    // Fallback for tail.
    for &b in &input[pos..] {
        if is_inline_special(b) {
            return Some(true);
        }
    }
    Some(false)
}

/// SIMD scan for next inline mark special used by mark collection.
/// Advances `pos` to the end of SIMD-scanned region if no hit.
#[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
#[target_feature(enable = "neon")]
pub unsafe fn next_mark_special_simd(text: &[u8], pos: &mut usize) -> Option<usize> {
    let len = text.len();
    let mut p = *pos;
    while p + 16 <= len {
        unsafe {
            let v = vld1q_u8(text.as_ptr().add(p));
            let mask = any_eq_mask(v, b"`*_\\\n[]<&");
            if mask_has_any(mask) {
                // Find first match within the chunk.
                for i in 0..16 {
                    if is_mark_special(text[p + i]) {
                        *pos = p + 16;
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

#[cfg(not(all(target_arch = "aarch64", target_feature = "neon")))]
pub fn has_inline_specials_simd(_input: &[u8]) -> Option<bool> {
    None
}

#[cfg(not(all(target_arch = "aarch64", target_feature = "neon")))]
pub fn next_mark_special_simd(_text: &[u8], _pos: &mut usize) -> Option<usize> {
    None
}
