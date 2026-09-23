//! Vector classifiers for the escape scanners.
//!
//! Each escaper's needle set has one vector form per target, selected at
//! compile time through [`NeedleSet`]:
//!
//! - **aarch64** looks bytes up in a pair of 16-entry nibble tables with
//!   `vqtbl1q_u8`.
//! - **x86-64** compares against the needles with SSE2, which every x86-64
//!   processor has. The table lookup's x86 counterpart, `pshufb`, needs
//!   SSSE3, which the x86-64 baseline lacks; behind run-time detection it
//!   sits in a `#[target_feature]` function that a baseline build cannot
//!   inline into the escape loop, so every scan pays a detection check and a
//!   call. The SSE2 classifiers are plain `#[inline]` code, like the NEON
//!   one.
//!
//! Both forms answer each lane exactly and are covered by the differential
//! tests in the parent module. Other targets use the word scan alone.

/// One escaper's needle set, in the vector form this target scans with.
///
/// Implemented by a zero-sized marker per escaper, so each escape loop is
/// monomorphized with its classifier inlined.
pub(super) trait NeedleSet: Copy {
    /// The nibble tables for the NEON lookup.
    #[cfg(target_arch = "aarch64")]
    fn nibbles(self) -> &'static NibbleTables;

    /// Needle lanes of one 16-byte vector: bit `k` of the result is set iff
    /// byte `k` of `v` is in the set. Bits 16 and up are always clear.
    #[cfg(all(target_arch = "x86_64", target_feature = "sse2"))]
    fn lanes(self, v: std::arch::x86_64::__m128i) -> u32;
}

/// The HTML text set: `&`, `<`, `>`, `"`, `'`.
#[derive(Clone, Copy)]
pub(super) struct EscapeNeedles;

/// The URL attribute set: space, `"`, `&`, `<`, `>`, `[`, `\`, `]`, backtick,
/// and every byte at or above `0x80`.
#[derive(Clone, Copy)]
pub(super) struct UrlEscapeNeedles;

impl NeedleSet for EscapeNeedles {
    #[cfg(target_arch = "aarch64")]
    #[inline]
    fn nibbles(self) -> &'static NibbleTables {
        &ESCAPE_NIBBLES
    }

    /// The fold of the parent module's `escape_mask`, sixteen lanes at a
    /// time: forcing bit 0 on maps `&`/`'` onto `0x27` and forcing bit 1
    /// on maps `<`/`>` onto `0x3E`, and neither admits any other byte, so
    /// three compares cover five needles. Every compare needs its lane to
    /// equal an ASCII value, so bytes at or above `0x80` never match.
    #[cfg(all(target_arch = "x86_64", target_feature = "sse2"))]
    #[allow(unsafe_code)]
    #[inline]
    fn lanes(self, v: std::arch::x86_64::__m128i) -> u32 {
        use std::arch::x86_64::*;
        // SAFETY: the `cfg` above admits only builds with SSE2 enabled, and
        // these intrinsics compute on registers only.
        let mask = unsafe {
            let amp_apos =
                _mm_cmpeq_epi8(_mm_or_si128(v, _mm_set1_epi8(0x01)), _mm_set1_epi8(0x27));
            let angle = _mm_cmpeq_epi8(_mm_or_si128(v, _mm_set1_epi8(0x02)), _mm_set1_epi8(0x3E));
            let quote = _mm_cmpeq_epi8(v, _mm_set1_epi8(0x22));
            _mm_movemask_epi8(_mm_or_si128(_mm_or_si128(amp_apos, angle), quote))
        };
        // `movemask` fills only the low 16 bits, so the value is never
        // negative and the conversion is exact.
        mask.cast_unsigned()
    }
}

impl NeedleSet for UrlEscapeNeedles {
    #[cfg(target_arch = "aarch64")]
    #[inline]
    fn nibbles(self) -> &'static NibbleTables {
        &URL_ESCAPE_NIBBLES
    }

    /// Five tests and the sign bit:
    ///
    /// - forcing bit 1 on maps space/`"` onto `0x22` and `<`/`>` onto
    ///   `0x3E`, admitting nothing else, so one OR serves two compares
    ///   (the same folds as the parent module's `url_escape_mask`);
    /// - `&` and backtick are exact compares;
    /// - `[`, `\` and `]` are the consecutive bytes `0x5B..=0x5D`: adding
    ///   `0x25` moves exactly them onto `0x80..=0x82`, the three smallest
    ///   signed bytes, which one signed compare below `i8::MIN + 3` picks
    ///   out (the add wraps, so every other byte lands elsewhere);
    /// - every compare leaves `0xFF` or `0x00` in its lane, so ORing `v`
    ///   itself in before `movemask` adds exactly the lanes whose bit 7 is
    ///   set: the non-ASCII bytes.
    #[cfg(all(target_arch = "x86_64", target_feature = "sse2"))]
    #[allow(unsafe_code)]
    #[inline]
    fn lanes(self, v: std::arch::x86_64::__m128i) -> u32 {
        use std::arch::x86_64::*;
        // SAFETY: the `cfg` above admits only builds with SSE2 enabled, and
        // these intrinsics compute on registers only.
        let mask = unsafe {
            let folded = _mm_or_si128(v, _mm_set1_epi8(0x02));
            let space_quote = _mm_cmpeq_epi8(folded, _mm_set1_epi8(0x22));
            let angle = _mm_cmpeq_epi8(folded, _mm_set1_epi8(0x3E));
            let amp = _mm_cmpeq_epi8(v, _mm_set1_epi8(0x26));
            let backtick = _mm_cmpeq_epi8(v, _mm_set1_epi8(0x60));
            let brackets = _mm_cmplt_epi8(
                _mm_add_epi8(v, _mm_set1_epi8(0x25)),
                _mm_set1_epi8(i8::MIN + 3),
            );
            _mm_movemask_epi8(_mm_or_si128(
                _mm_or_si128(
                    _mm_or_si128(space_quote, angle),
                    _mm_or_si128(amp, backtick),
                ),
                _mm_or_si128(brackets, v),
            ))
        };
        // `movemask` fills only the low 16 bits, so the value is never
        // negative and the conversion is exact.
        mask.cast_unsigned()
    }
}

/// Nibble-pair classifier tables for the NEON path, one pair per escaper.
///
/// A byte is flagged iff `low[b & 0x0F] & high[b >> 4]` is nonzero. Both
/// needle sets split cleanly by high nibble. HTML escaping uses two row
/// bits, and every byte >= 0x80 maps to a zero high entry. URL escaping uses
/// four bits to include brackets, backslash, and backtick, plus a fifth bit
/// shared by all eight high rows so that every non-ASCII byte is flagged for
/// percent encoding.
///
/// Sixteen entries is what a vector shuffle holds: `vqtbl1q_u8` classifies
/// all sixteen lanes in one instruction.
#[cfg(target_arch = "aarch64")]
pub(super) struct NibbleTables {
    pub(super) low: [u8; 16],
    pub(super) high: [u8; 16],
}

#[cfg(target_arch = "aarch64")]
const ESCAPE_NIBBLES: NibbleTables = NibbleTables {
    //          0     1     2     3  4  5     6     7  8  9  A  B     C  D     E  F
    low: [
        0, 0, 0x01, 0, 0, 0, 0x01, 0x01, 0, 0, 0, 0, 0x02, 0, 0x02, 0,
    ],
    high: [0, 0, 0x01, 0x02, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
};

#[cfg(target_arch = "aarch64")]
const URL_ESCAPE_NIBBLES: NibbleTables = NibbleTables {
    // 0x01: space/quote/&; 0x02: angle brackets; 0x04: brackets/backslash;
    // 0x08: backtick; 0x10: any low nibble in a high row >= 8 (non-ASCII).
    // The separate bits prevent overlap between high-nibble rows.
    low: [
        0x19, 0x10, 0x11, 0x10, 0x10, 0x10, 0x11, 0x10, 0x10, 0x10, 0x10, 0x14, 0x16, 0x14, 0x12,
        0x10,
    ],
    high: [
        0, 0, 0x01, 0x02, 0, 0x04, 0x08, 0, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10,
    ],
};

/// Offset of the first flagged byte in `bytes[from..]`, or `None` when the
/// caller should fall through to the word scan (input shorter than one
/// vector, or no vector path on this target).
#[cfg(target_arch = "aarch64")]
#[allow(unsafe_code)]
#[inline]
fn first_flagged_vector(bytes: &[u8], from: usize, tables: &NibbleTables) -> Option<usize> {
    use std::arch::aarch64::*;
    let len = bytes.len();
    if len < 16 {
        return None;
    }
    let mut i = from;
    unsafe {
        let low = vld1q_u8(tables.low.as_ptr());
        let high = vld1q_u8(tables.high.as_ptr());
        let nibble = vdupq_n_u8(0x0F);
        let classify = |v: uint8x16_t| {
            let lo = vqtbl1q_u8(low, vandq_u8(v, nibble));
            let hi = vqtbl1q_u8(high, vshrq_n_u8(v, 4));
            let m = vtstq_u8(lo, hi);
            vget_lane_u64(
                vreinterpret_u64_u8(vshrn_n_u16(vreinterpretq_u16_u8(m), 4)),
                0,
            )
        };
        while i + 16 <= len {
            let mask = classify(vld1q_u8(bytes.as_ptr().add(i)));
            if mask != 0 {
                return Some(i + (mask.trailing_zeros() / 4) as usize);
            }
            i += 16;
        }
        if i < len {
            // Overlapping tail: re-read the last vector and drop the lanes
            // the loop already cleared. `vtstq_u8` gives exact per-lane
            // answers, so no borrow can leak across the mask.
            let base = len - 16;
            let mask =
                classify(vld1q_u8(bytes.as_ptr().add(base))) & (u64::MAX << ((i - base) * 4));
            if mask != 0 {
                return Some(base + (mask.trailing_zeros() / 4) as usize);
            }
        }
    }
    Some(len)
}

/// SSE2 sibling of [`first_flagged_vector`], with the same contract, loop
/// and overlapping tail.
///
/// Inlined into the escape loop, which calls it once per replaced byte.
/// There is no feature to detect and no table to load: the classifier's
/// constants are splats, which the compiler loads into registers once per
/// escaped string, ahead of the loop, and again only after the loop's
/// out-of-line calls (a long run copy, buffer growth). A scan that stops in
/// its first vector therefore costs one load, the compares and a
/// `movemask`. Hoisting the constants further by hand would gain nothing:
/// every vector register is caller-saved on System V, so they would be
/// spilled around the same calls.
#[cfg(all(target_arch = "x86_64", target_feature = "sse2"))]
#[allow(unsafe_code)]
#[inline]
fn first_flagged_sse2(bytes: &[u8], from: usize, needles: impl NeedleSet) -> Option<usize> {
    use std::arch::x86_64::_mm_loadu_si128;
    let len = bytes.len();
    if len < 16 {
        return None;
    }
    let mut i = from;
    // SAFETY: every 16-byte load is bounded by the `i + 16 <= len` check or
    // reads the final full vector at `len - 16`, which exists because
    // `len >= 16`. `_mm_loadu_si128` has no alignment requirement.
    unsafe {
        while i + 16 <= len {
            let mask = needles.lanes(_mm_loadu_si128(bytes.as_ptr().add(i).cast()));
            if mask != 0 {
                return Some(i + mask.trailing_zeros() as usize);
            }
            i += 16;
        }
        if i < len {
            // Overlapping tail, as in the NEON scan: `i > len - 16` here, so
            // the shift is 1..=15 and drops the lanes before `i`, which the
            // loop already cleared or which lie before `from`. Each lane's
            // answer is exact, so nothing leaks across the mask.
            let base = len - 16;
            let mask = needles.lanes(_mm_loadu_si128(bytes.as_ptr().add(base).cast()))
                & (u32::MAX << (i - base));
            if mask != 0 {
                return Some(base + mask.trailing_zeros() as usize);
            }
        }
    }
    Some(len)
}

/// Offset of the first byte of `needles` in `bytes[from..]` (`bytes.len()`
/// when there is none), or `None` when the caller should fall through to the
/// word scan: the input is shorter than one vector, or this target has no
/// vector path.
#[inline]
pub(super) fn first_flagged_simd(
    bytes: &[u8],
    from: usize,
    needles: impl NeedleSet,
) -> Option<usize> {
    #[cfg(target_arch = "aarch64")]
    {
        first_flagged_vector(bytes, from, needles.nibbles())
    }
    #[cfg(all(target_arch = "x86_64", target_feature = "sse2"))]
    {
        first_flagged_sse2(bytes, from, needles)
    }
    #[cfg(not(any(
        target_arch = "aarch64",
        all(target_arch = "x86_64", target_feature = "sse2")
    )))]
    {
        let _ = (bytes, from, needles);
        None
    }
}
