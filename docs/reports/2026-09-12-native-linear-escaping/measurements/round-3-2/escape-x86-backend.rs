#[cfg(target_arch = "x86_64")]
#[inline(never)]
fn first_escape_x86<const ATTR: bool>(input: &[u8]) -> Option<usize> {
    let found = if std::is_x86_feature_detected!("avx2") {
        // SAFETY: the runtime check includes OS support for AVX state.
        unsafe { first_escape_avx2::<ATTR>(input) }
    } else {
        // SAFETY: SSE2 is part of the x86-64 baseline.
        unsafe { first_escape_sse2::<ATTR>(input) }
    };
    #[cfg(test)]
    tests::record_long_search(input.len(), found);
    found
}

#[cfg(target_arch = "x86_64")]
macro_rules! escape_vector_mask {
    ($v:expr, $attr:ident, $and:ident, $eq:ident, $or:ident, $clear4:ident, $clear2:ident, $quote:ident, $less:ident, $apos:ident) => {{
        let v = $v;
        // Clearing bit 2 pairs '"' with '&'; clearing bit 1 pairs '<' with '>'.
        // All other bits must still match, so no other byte can be accepted.
        let common = $or($eq($and(v, $clear4), $quote), $eq($and(v, $clear2), $less));
        if $attr { $or(common, $eq(v, $apos)) } else { common }
    }};
}

#[cfg(target_arch = "x86_64")]
macro_rules! define_escape_scan {
    ($name:ident, $feature:literal, $width:literal, $load:ident, $set:ident, $and:ident, $eq:ident, $or:ident, $mask:ident) => {
        /// # Safety
        /// The caller must establish support for the named CPU feature.
        #[target_feature(enable = $feature)]
        unsafe fn $name<const ATTR: bool>(input: &[u8]) -> Option<usize> {
            use core::arch::x86_64::*;
            // SAFETY: the caller establishes the CPU feature. Each load below
            // is guarded by a complete readable vector or four-vector range.
            // The final overlapping vector ends exactly at input.len().
            unsafe {
                let clear4 = $set(!4_i8);
                let clear2 = $set(!2_i8);
                let quote = $set(b'"' as i8);
                let less = $set(b'<' as i8);
                let apos = $set(b'\'' as i8);
                let mut pos = 0;
                // Avoid four-vector setup when an escape occurs immediately.
                if input.len() >= $width {
                    let v = $load(input.as_ptr().cast());
                    let bits = $mask(escape_vector_mask!(v, ATTR, $and, $eq, $or, clear4, clear2, quote, less, apos)) as u32;
                    if bits != 0 { return Some(bits.trailing_zeros() as usize); }
                    pos = $width;
                }
                while input.len() - pos >= 4 * $width {
                    let a = escape_vector_mask!($load(input.as_ptr().add(pos).cast()), ATTR, $and, $eq, $or, clear4, clear2, quote, less, apos);
                    let b = escape_vector_mask!($load(input.as_ptr().add(pos + $width).cast()), ATTR, $and, $eq, $or, clear4, clear2, quote, less, apos);
                    let c = escape_vector_mask!($load(input.as_ptr().add(pos + 2 * $width).cast()), ATTR, $and, $eq, $or, clear4, clear2, quote, less, apos);
                    let d = escape_vector_mask!($load(input.as_ptr().add(pos + 3 * $width).cast()), ATTR, $and, $eq, $or, clear4, clear2, quote, less, apos);
                    if $mask($or($or(a,b),$or(c,d))) != 0 {
                        for (i, lanes) in [a,b,c,d].into_iter().enumerate() {
                            let bits = $mask(lanes) as u32;
                            if bits != 0 { return Some(pos + i * $width + bits.trailing_zeros() as usize); }
                        }
                    }
                    pos += 4 * $width;
                }
                while input.len() - pos >= $width {
                    let v = $load(input.as_ptr().add(pos).cast());
                    let bits = $mask(escape_vector_mask!(v, ATTR, $and, $eq, $or, clear4, clear2, quote, less, apos)) as u32;
                    if bits != 0 { return Some(pos + bits.trailing_zeros() as usize); }
                    pos += $width;
                }
                if pos < input.len() {
                    if input.len() >= $width {
                        let last = input.len() - $width;
                        let v = $load(input.as_ptr().add(last).cast());
                        let bits = $mask(escape_vector_mask!(v, ATTR, $and, $eq, $or, clear4, clear2, quote, less, apos)) as u32;
                        // Overlapping bytes have already been found clean.
                        if bits != 0 { return Some(last + bits.trailing_zeros() as usize); }
                    } else {
                        return first_escape_in_set::<ATTR>(input);
                    }
                }
                None
            }
        }
    };
}

#[cfg(target_arch = "x86_64")]
define_escape_scan!(first_escape_sse2, "sse2", 16, _mm_loadu_si128, _mm_set1_epi8, _mm_and_si128, _mm_cmpeq_epi8, _mm_or_si128, _mm_movemask_epi8);
#[cfg(target_arch = "x86_64")]
define_escape_scan!(first_escape_avx2, "avx2", 32, _mm256_loadu_si256, _mm256_set1_epi8, _mm256_and_si256, _mm256_cmpeq_epi8, _mm256_or_si256, _mm256_movemask_epi8);
