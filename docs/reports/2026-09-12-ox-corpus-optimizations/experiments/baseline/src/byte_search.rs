//! Shared implementation for the unpublished `ferro-byte-search` crate.
//!
//! Kept in Ferromark's package so published releases embed the exact same
//! implementation without an unpublished dependency. No Markdown semantics.

/// An allocation-free, reusable set of bytes to search for.
///
/// `N` is the number of supplied bytes, including duplicates. Empty sets never
/// match. NUL and all other byte values are supported. Small fixed sets benefit
/// from constant propagation and SIMD; no UTF-8 or alignment is required.
#[derive(Clone, Copy, Debug)]
pub struct ByteSet<const N: usize> {
    // Scalar targets use only the membership table.
    #[cfg_attr(
        not(any(
            target_arch = "x86_64",
            all(target_arch = "aarch64", target_feature = "neon")
        )),
        allow(dead_code)
    )]
    bytes: [u8; N],
    membership: [bool; 256],
    #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
    low: [u8; 16],
    #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
    high: [u8; 16],
    #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
    nibble_valid: bool,
}

impl<const N: usize> ByteSet<N> {
    /// Copy a byte array into a reusable set; usable in a `const` declaration.
    #[inline]
    pub const fn new(bytes: &[u8; N]) -> Self {
        let mut membership = [false; 256];
        let mut i = 0;
        while i < N {
            membership[bytes[i] as usize] = true;
            i += 1;
        }
        #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
        let (low, high, nibble_valid) = {
            // Each high nibble selects a row of allowed low nibbles. Assign
            // one bit to each distinct nonempty row (at most eight), sharing
            // bits only for identical rows. The two table lookups intersect
            // exactly for members of the set, including arbitrary non-ASCII
            // bytes. More than eight distinct rows use byte comparisons.
            let mut rows = [0u16; 16];
            let mut i = 0;
            while i < N {
                rows[(bytes[i] >> 4) as usize] |= 1u16 << (bytes[i] & 15);
                i += 1;
            }
            let mut low = [0u8; 16];
            let mut high = [0u8; 16];
            let mut groups = 0;
            let mut h = 0;
            let mut valid = true;
            while h < 16 {
                if rows[h] != 0 {
                    let mut previous = 0;
                    while previous < h && rows[previous] != rows[h] {
                        previous += 1;
                    }
                    if previous < h {
                        high[h] = high[previous];
                    } else if groups < 8 {
                        let bit = 1u8 << groups;
                        groups += 1;
                        high[h] = bit;
                        let mut l = 0;
                        while l < 16 {
                            if rows[h] & (1 << l) != 0 {
                                low[l] |= bit;
                            }
                            l += 1;
                        }
                    } else {
                        valid = false;
                    }
                }
                h += 1;
            }
            (low, high, valid)
        };
        Self {
            bytes: *bytes,
            membership,
            #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
            low,
            #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
            high,
            #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
            nibble_valid,
        }
    }

    #[inline(always)]
    fn matches(&self, byte: u8) -> bool {
        self.membership[byte as usize]
    }

    /// Return the byte offset of the first match, or `None`.
    #[inline]
    pub fn find(&self, input: &[u8]) -> Option<usize> {
        if N == 0 || input.is_empty() {
            return None;
        }
        #[cfg(any(
            target_arch = "x86_64",
            all(target_arch = "aarch64", target_feature = "neon")
        ))]
        {
            let len = input.len();
            if len < 16 {
                // Short NEON loads need a padded copy; scalar lookup avoids
                // that setup. Large sets also favor scalar lookup on SSE2.
                if N > 5 || cfg!(target_arch = "aarch64") {
                    return self.find_scalar(input);
                }
                let mut padded = [0; 16];
                padded[..len].copy_from_slice(input);
                if self.chunk_has_match(&padded) {
                    // Only inspect real input bytes: NUL in the padding must
                    // never appear as a match beyond the end of the slice.
                    return self.find_scalar(input);
                }
                return None;
            }
            let mut pos = 0;
            while len - pos >= 16 {
                #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
                {
                    if let Some(i) = self.first_in_chunk(&input[pos..pos + 16]) {
                        return Some(pos + i);
                    }
                }
                #[cfg(target_arch = "x86_64")]
                {
                    if self.chunk_has_match(&input[pos..]) {
                        return self.first_in_chunk(&input[pos..pos + 16]).map(|i| pos + i);
                    }
                }
                pos += 16;
            }
            if pos < len && self.chunk_has_match(&input[len - 16..]) {
                return self.find_scalar(&input[pos..]).map(|i| pos + i);
            }
            None
        }
        #[cfg(not(any(
            target_arch = "x86_64",
            all(target_arch = "aarch64", target_feature = "neon")
        )))]
        self.find_scalar(input)
    }

    /// Whether the input contains at least one byte from the set.
    ///
    /// Does not locate the matching lane when a SIMD chunk has a match.
    #[inline]
    pub fn contains_any(&self, input: &[u8]) -> bool {
        if N == 0 {
            return false;
        }
        #[allow(unused_mut)] // Only the SIMD configurations advance this cursor.
        let mut pos = 0;
        #[cfg(any(
            target_arch = "x86_64",
            all(target_arch = "aarch64", target_feature = "neon")
        ))]
        while input.len() - pos >= 16 {
            if self.chunk_has_match(&input[pos..]) {
                return true;
            }
            pos += 16;
        }
        self.find_scalar(&input[pos..]).is_some()
    }

    #[inline]
    fn find_scalar(&self, input: &[u8]) -> Option<usize> {
        input.iter().position(|&byte| self.matches(byte))
    }

    #[cfg(target_arch = "x86_64")]
    #[inline]
    fn chunk_mask(&self, input: &[u8]) -> u32 {
        use core::arch::x86_64::*;
        assert!(input.len() >= 16);
        // SAFETY: the safe wrapper proves a readable 16-byte range. The load
        // is unaligned; SSE2 is guaranteed on x86-64. No pointer escapes.
        unsafe {
            let v = _mm_loadu_si128(input.as_ptr().cast());
            let mut mask = _mm_setzero_si128();
            for &byte in &self.bytes {
                mask = _mm_or_si128(mask, _mm_cmpeq_epi8(v, _mm_set1_epi8(byte as i8)));
            }
            _mm_movemask_epi8(mask) as u32
        }
    }

    #[cfg(target_arch = "x86_64")]
    #[inline]
    fn chunk_has_match(&self, input: &[u8]) -> bool {
        self.chunk_mask(input) != 0
    }

    #[cfg(target_arch = "x86_64")]
    #[inline]
    fn first_in_chunk(&self, input: &[u8]) -> Option<usize> {
        let mask = self.chunk_mask(input);
        (mask != 0).then(|| mask.trailing_zeros() as usize)
    }

    #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
    #[inline]
    fn chunk_has_match(&self, input: &[u8]) -> bool {
        use core::arch::aarch64::*;
        // SAFETY: NEON is enabled by the target-feature cfg.
        unsafe { vmaxvq_u8(self.neon_matches(input)) != 0 }
    }

    #[cfg(all(
        target_arch = "aarch64",
        target_feature = "neon",
        target_endian = "little"
    ))]
    #[inline]
    fn first_in_chunk(&self, input: &[u8]) -> Option<usize> {
        use core::arch::aarch64::*;
        let lanes = self.neon_matches(input);
        // Narrow each pair of 0x00/0xff lanes into two four-bit groups.
        // This preserves lane order and avoids rescanning a matching chunk.
        // SAFETY: NEON is enabled by the target-feature cfg.
        let mask = unsafe {
            vget_lane_u64(
                vreinterpret_u64_u8(vshrn_n_u16::<4>(vreinterpretq_u16_u8(lanes))),
                0,
            )
        };
        (mask != 0).then(|| (mask.trailing_zeros() / 4) as usize)
    }

    #[cfg(all(
        target_arch = "aarch64",
        target_feature = "neon",
        target_endian = "big"
    ))]
    #[inline]
    fn first_in_chunk(&self, input: &[u8]) -> Option<usize> {
        // The packed integer mask relies on little-endian lane order.
        self.find_scalar(input)
    }

    #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
    #[inline]
    fn neon_matches(&self, input: &[u8]) -> core::arch::aarch64::uint8x16_t {
        use core::arch::aarch64::*;
        assert!(input.len() >= 16);
        // SAFETY: a complete readable vector is available. NEON is guaranteed
        // by the target-feature cfg; vld1q_u8 accepts unaligned input. The
        // lookup arrays each contain a complete readable 16-byte vector.
        unsafe {
            let v = vld1q_u8(input.as_ptr());
            if self.nibble_valid {
                let lo = vqtbl1q_u8(vld1q_u8(self.low.as_ptr()), vandq_u8(v, vdupq_n_u8(15)));
                let hi = vqtbl1q_u8(vld1q_u8(self.high.as_ptr()), vshrq_n_u8::<4>(v));
                vtstq_u8(lo, hi)
            } else {
                let mut mask = vdupq_n_u8(0);
                for &byte in &self.bytes {
                    mask = vorrq_u8(mask, vceqq_u8(v, vdupq_n_u8(byte)));
                }
                mask
            }
        }
    }
}
