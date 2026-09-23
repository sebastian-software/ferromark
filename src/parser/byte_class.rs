//! ASCII byte-set classifiers for the parser's stop-byte scans.
//!
//! Several inline scanners walk a run of ordinary bytes until one of a
//! handful of ASCII bytes needs a decision: link destinations stop at
//! whitespace, control bytes, backslashes, and parentheses; bracket bodies
//! stop at escapes, code spans, raw HTML, and the brackets themselves. A
//! [`ByteClass`] describes such a set twice: as a 256-entry flag table, which
//! is the definition and serves short remainders, and as the nibble-pair
//! tables a vector shuffle needs. Both are derived from the same flags at
//! compile time, so they cannot disagree.
//!
//! The nibble form works because every set here splits by high nibble into a
//! few rows: each row with members owns one bit in the high table, and the
//! low table holds, per low nibble, the bits of the rows in which that nibble
//! is a member. A byte is in the set iff `LOW[b & 0x0F] & HIGH[b >> 4]` is
//! nonzero, which is exact for sets with at most eight rows. Bytes at or
//! above `0x80` are never members: their rows map to zero.
//!
//! The vector scans run that lookup sixteen lanes at a time: `vqtbl1q_u8` on
//! aarch64, and `pshufb` on x86-64, where AVX2 broadcasts the tables into
//! both 128-bit lanes to classify 32 bytes per shuffle pair. SSSE3 and AVX2
//! are not in the x86-64 baseline, so they are detected at run time, and a
//! machine without either keeps the table walk.

/// An ASCII byte set with a scalar flag table and vector nibble tables.
pub(in crate::parser) struct ByteClass {
    flags: [u8; 256],
    #[cfg_attr(
        not(any(target_arch = "aarch64", target_arch = "x86_64")),
        allow(dead_code)
    )]
    low: [u8; 16],
    #[cfg_attr(
        not(any(target_arch = "aarch64", target_arch = "x86_64")),
        allow(dead_code)
    )]
    high: [u8; 16],
}

impl ByteClass {
    /// Builds the classifier from a flag table: `flags[b] != 0` marks `b` as
    /// a member. Members must be ASCII and occupy at most eight high-nibble
    /// rows; both are checked at compile time.
    pub(in crate::parser) const fn from_flags(flags: [u8; 256]) -> Self {
        let mut low = [0u8; 16];
        let mut high = [0u8; 16];
        let mut bit = 1u8;
        let mut row = 0;
        while row < 8 {
            let mut members = 0u16;
            let mut nibble = 0;
            while nibble < 16 {
                if flags[row * 16 + nibble] != 0 {
                    members |= 1 << nibble;
                }
                nibble += 1;
            }
            if members != 0 {
                assert!(
                    bit != 0,
                    "a byte class may use at most eight high-nibble rows"
                );
                high[row] = bit;
                let mut nibble = 0;
                while nibble < 16 {
                    if members & (1 << nibble) != 0 {
                        low[nibble] |= bit;
                    }
                    nibble += 1;
                }
                bit = if bit == 0x80 { 0 } else { bit << 1 };
            }
            row += 1;
        }
        let mut byte = 0x80;
        while byte < 256 {
            assert!(flags[byte] == 0, "byte class members must be ASCII");
            byte += 1;
        }
        Self { flags, low, high }
    }

    /// Whether `byte` is a member.
    #[inline]
    pub(in crate::parser) const fn contains(&self, byte: u8) -> bool {
        self.flags[byte as usize] != 0
    }

    /// Offset of the first member at or after `from`, or `bytes.len()` when
    /// the remainder holds none.
    #[inline]
    pub(in crate::parser) fn first_in(&self, bytes: &[u8], from: usize) -> usize {
        // One vector is the breakeven against the table walk; shorter
        // remainders would pay a 16-byte overlapping reload.
        #[cfg(target_arch = "aarch64")]
        if bytes.len() - from >= 16 {
            return self.first_in_neon(bytes, from);
        }
        #[cfg(target_arch = "x86_64")]
        if bytes.len() - from >= 16 {
            return self.first_in_x86(bytes, from);
        }
        self.first_in_scalar(bytes, from)
    }

    #[inline]
    fn first_in_scalar(&self, bytes: &[u8], from: usize) -> usize {
        let mut i = from;
        while i < bytes.len() && !self.contains(bytes[i]) {
            i += 1;
        }
        i
    }

    #[cfg(target_arch = "aarch64")]
    #[allow(unsafe_code)]
    #[inline]
    fn first_in_neon(&self, bytes: &[u8], from: usize) -> usize {
        use std::arch::aarch64::*;
        let end = bytes.len();
        let mut i = from;
        // SAFETY: every 16-byte load is bounded by an explicit `i + 16 <= end`
        // check or reads the final full vector at `end - 16`, and the caller
        // guaranteed `end - from >= 16`.
        unsafe {
            let low = vld1q_u8(self.low.as_ptr());
            let high = vld1q_u8(self.high.as_ptr());
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
            while i + 16 <= end {
                let mask = classify(vld1q_u8(bytes.as_ptr().add(i)));
                if mask != 0 {
                    return i + (mask.trailing_zeros() / 4) as usize;
                }
                i += 16;
            }
            if i < end {
                // Overlapping tail: re-read the last vector and drop the lanes
                // the loop already cleared. `vtstq_u8` is exact per lane.
                let base = end - 16;
                let mask =
                    classify(vld1q_u8(bytes.as_ptr().add(base))) & (u64::MAX << ((i - base) * 4));
                if mask != 0 {
                    return base + (mask.trailing_zeros() / 4) as usize;
                }
            }
        }
        end
    }

    /// Picks the widest x86-64 vector scan this machine supports. The
    /// caller guarantees `bytes.len() - from >= 16`.
    ///
    /// The published addons target the x86-64 baseline, which has neither
    /// SSSE3 nor AVX2, so both are detected rather than assumed;
    /// `is_x86_feature_detected!` caches its answer after the first call.
    #[cfg(target_arch = "x86_64")]
    #[allow(unsafe_code)]
    #[inline]
    fn first_in_x86(&self, bytes: &[u8], from: usize) -> usize {
        if std::arch::is_x86_feature_detected!("avx2") {
            // SAFETY: AVX2 was detected above, and the caller guaranteed
            // `bytes.len() - from >= 16`.
            unsafe { self.first_in_avx2(bytes, from) }
        } else if std::arch::is_x86_feature_detected!("ssse3") {
            // SAFETY: SSSE3 was detected above, and the caller guaranteed
            // `bytes.len() - from >= 16`.
            unsafe { self.first_in_ssse3(bytes, from) }
        } else {
            self.first_in_scalar(bytes, from)
        }
    }

    /// SSSE3 counterpart of the NEON scan: `pshufb` is the same 16-entry
    /// lookup as `vqtbl1q_u8`, run in the same loop with the same
    /// overlapping tail.
    ///
    /// # Safety
    ///
    /// The caller must have verified SSSE3 support and guarantee
    /// `bytes.len() - from >= 16`.
    #[cfg(target_arch = "x86_64")]
    #[allow(unsafe_code)]
    #[target_feature(enable = "ssse3")]
    unsafe fn first_in_ssse3(&self, bytes: &[u8], from: usize) -> usize {
        use std::arch::x86_64::*;
        let end = bytes.len();
        let mut i = from;
        // SAFETY: every 16-byte load is bounded by an explicit `i + 16 <= end`
        // check or reads the final full vector at `end - 16`, and the caller
        // guaranteed `end - from >= 16`.
        unsafe {
            let low = _mm_loadu_si128(self.low.as_ptr().cast());
            let high = _mm_loadu_si128(self.high.as_ptr().cast());
            let nibble = _mm_set1_epi8(0x0F);
            let classify = |v: __m128i| {
                // Both shuffle indices are masked to 0..=15, so `pshufb`
                // never takes its zeroing branch (index bit 7 set) and each
                // lookup is the plain table read the scalar identity makes.
                // `_mm_srli_epi16` shifts 16-bit lanes; the mask drops the
                // neighbouring byte's bits that ride along.
                let lo = _mm_shuffle_epi8(low, _mm_and_si128(v, nibble));
                let hi = _mm_shuffle_epi8(high, _mm_and_si128(_mm_srli_epi16(v, 4), nibble));
                // `movemask` of "lane is zero" sets one of the low 16 bits
                // per non-member; the complement, kept inside those bits,
                // has one bit per member.
                let clean =
                    _mm_movemask_epi8(_mm_cmpeq_epi8(_mm_and_si128(lo, hi), _mm_setzero_si128()));
                u32::from_ne_bytes((!clean & 0xFFFF).to_ne_bytes())
            };
            while i + 16 <= end {
                let mask = classify(_mm_loadu_si128(bytes.as_ptr().add(i).cast()));
                if mask != 0 {
                    return i + mask.trailing_zeros() as usize;
                }
                i += 16;
            }
            if i < end {
                // Overlapping tail: re-read the last vector and drop the lanes
                // the loop already cleared. Each lane's answer is exact, so
                // nothing leaks across the mask.
                let base = end - 16;
                let mask = classify(_mm_loadu_si128(bytes.as_ptr().add(base).cast()))
                    & (u32::MAX << (i - base));
                if mask != 0 {
                    return base + mask.trailing_zeros() as usize;
                }
            }
        }
        end
    }

    /// AVX2 sibling of [`Self::first_in_ssse3`], 32 bytes per step.
    ///
    /// `vpshufb` looks up within each 128-bit lane, so the 16-entry tables
    /// are broadcast into both. A remainder of 16 to 31 bytes takes one
    /// overlapping 32-byte load when the slice is long enough; only a slice
    /// shorter than 32 bytes hands over to the SSSE3 scan.
    ///
    /// # Safety
    ///
    /// The caller must have verified AVX2 support and guarantee
    /// `bytes.len() - from >= 16`.
    #[cfg(target_arch = "x86_64")]
    #[allow(unsafe_code)]
    #[target_feature(enable = "avx2")]
    unsafe fn first_in_avx2(&self, bytes: &[u8], from: usize) -> usize {
        use std::arch::x86_64::*;
        let end = bytes.len();
        if end < 32 {
            // SAFETY: AVX2 implies SSSE3, and the caller's length guarantee
            // is the one the SSSE3 scan needs.
            return unsafe { self.first_in_ssse3(bytes, from) };
        }
        let mut i = from;
        // SAFETY: every 32-byte load is bounded by an explicit `i + 32 <= end`
        // check or reads the final full vector at `end - 32`, which exists
        // because `end >= 32`.
        unsafe {
            let low = _mm256_broadcastsi128_si256(_mm_loadu_si128(self.low.as_ptr().cast()));
            let high = _mm256_broadcastsi128_si256(_mm_loadu_si128(self.high.as_ptr().cast()));
            let nibble = _mm256_set1_epi8(0x0F);
            let classify = |v: __m256i| {
                let lo = _mm256_shuffle_epi8(low, _mm256_and_si256(v, nibble));
                let hi =
                    _mm256_shuffle_epi8(high, _mm256_and_si256(_mm256_srli_epi16(v, 4), nibble));
                let clean = _mm256_movemask_epi8(_mm256_cmpeq_epi8(
                    _mm256_and_si256(lo, hi),
                    _mm256_setzero_si256(),
                ));
                u32::from_ne_bytes((!clean).to_ne_bytes())
            };
            while i + 32 <= end {
                let mask = classify(_mm256_loadu_si256(bytes.as_ptr().add(i).cast()));
                if mask != 0 {
                    return i + mask.trailing_zeros() as usize;
                }
                i += 32;
            }
            if i < end {
                // Overlapping tail, as in the SSSE3 scan. When fewer than 32
                // bytes remained from the start, the masked lanes also cover
                // the bytes before `from`.
                let base = end - 32;
                let mask = classify(_mm256_loadu_si256(bytes.as_ptr().add(base).cast()))
                    & (u32::MAX << (i - base));
                if mask != 0 {
                    return base + mask.trailing_zeros() as usize;
                }
            }
        }
        end
    }
}

/// The differential checks are shared: each module that owns a production
/// class runs [`tests::assert_backends_match_flags`] on it.
#[cfg(test)]
pub(in crate::parser) mod tests {
    // Owned strings keep the test oracle independent of production arena storage.
    #![allow(
        clippy::disallowed_macros,
        clippy::disallowed_methods,
        clippy::disallowed_types
    )]

    use super::ByteClass;

    const fn flags(members: &[u8]) -> [u8; 256] {
        let mut table = [0u8; 256];
        let mut i = 0;
        while i < members.len() {
            table[members[i] as usize] = 1;
            i += 1;
        }
        table
    }

    static SAMPLE: ByteClass = ByteClass::from_flags(flags(b"\\`<[]"));
    static DENSE: ByteClass = ByteClass::from_flags({
        let mut table = [0u8; 256];
        let mut byte = 0;
        while byte < 0x21 {
            table[byte] = 1;
            byte += 1;
        }
        table[0x7F] = 1;
        table[b'(' as usize] = 1;
        table[b')' as usize] = 1;
        table[b'\\' as usize] = 1;
        table
    });

    fn scalar_first(class: &ByteClass, bytes: &[u8], from: usize) -> usize {
        bytes[from..]
            .iter()
            .position(|&b| class.contains(b))
            .map_or(bytes.len(), |o| from + o)
    }

    /// Starting offsets for the remainder sweep: both sides of the 16- and
    /// 32-byte steps, so the vector tails see every overlap with the start.
    const STARTS: [usize; 12] = [0, 1, 2, 3, 8, 15, 16, 17, 24, 31, 32, 33];
    /// Longest remainder the sweep checks after each start.
    const MAX_REMAINDER: usize = 200;

    /// One way to run the scan, and the shortest remainder it accepts.
    struct Backend {
        name: &'static str,
        min_remaining: usize,
        scan: fn(&ByteClass, &[u8], usize) -> usize,
    }

    /// Every scan this machine can run: the dispatcher, the table walk, and
    /// each vector path on its own, so an AVX2 host checks SSSE3 as well.
    /// Vector paths are gated on the same run-time detection production uses.
    fn backends() -> Vec<Backend> {
        let mut backends = vec![
            Backend {
                name: "first_in",
                min_remaining: 0,
                scan: ByteClass::first_in,
            },
            Backend {
                name: "scalar",
                min_remaining: 0,
                scan: ByteClass::first_in_scalar,
            },
        ];
        #[cfg(target_arch = "aarch64")]
        backends.push(Backend {
            name: "neon",
            min_remaining: 16,
            scan: ByteClass::first_in_neon,
        });
        #[cfg(target_arch = "x86_64")]
        {
            if std::arch::is_x86_feature_detected!("ssse3") {
                backends.push(Backend {
                    name: "ssse3",
                    min_remaining: 16,
                    scan: |class, bytes, from| {
                        // SAFETY: SSSE3 was detected, and every caller skips
                        // remainders below `min_remaining`.
                        #[allow(unsafe_code)]
                        unsafe {
                            class.first_in_ssse3(bytes, from)
                        }
                    },
                });
            }
            if std::arch::is_x86_feature_detected!("avx2") {
                backends.push(Backend {
                    name: "avx2",
                    min_remaining: 16,
                    scan: |class, bytes, from| {
                        // SAFETY: AVX2 was detected, and every caller skips
                        // remainders below `min_remaining`.
                        #[allow(unsafe_code)]
                        unsafe {
                            class.first_in_avx2(bytes, from)
                        }
                    },
                });
            }
        }
        backends
    }

    /// Asserts that every backend in `backends` that accepts the remainder
    /// finds `expected`.
    fn assert_all(
        backends: &[Backend],
        class: &ByteClass,
        bytes: &[u8],
        from: usize,
        expected: usize,
        case: std::fmt::Arguments<'_>,
    ) {
        for backend in backends {
            if bytes.len() - from >= backend.min_remaining {
                assert_eq!(
                    (backend.scan)(class, bytes, from),
                    expected,
                    "{}: {case} (len {}, from {from})",
                    backend.name,
                    bytes.len()
                );
            }
        }
    }

    /// Checks every scan backend on this machine against the flag table.
    ///
    /// Three sweeps: all 256 byte values in every lane position of inputs up
    /// to three AVX2 vectors long; every remainder from 0 to 200 bytes with
    /// the first member at every position, from several starting offsets,
    /// over noise that cycles through every non-member (non-ASCII included)
    /// and with members filling everything before the start, so a vector
    /// tail that let an earlier lane through would report it; and seeded
    /// inputs of mixed density, which put several members in one vector.
    pub(in crate::parser) fn assert_backends_match_flags(class: &ByteClass) {
        let backends = backends();
        let members: Vec<u8> = (0..=255u8).filter(|&b| class.contains(b)).collect();
        let others: Vec<u8> = (0..=255u8).filter(|&b| !class.contains(b)).collect();
        assert!(!members.is_empty(), "production classes have members");
        for byte in 0..=255u8 {
            let lo = class.low[usize::from(byte & 0x0F)];
            let hi = class.high[usize::from(byte >> 4)];
            assert_eq!(
                lo & hi != 0,
                class.contains(byte),
                "nibble tables disagree on {byte:#04x}"
            );
        }

        let filler = others
            .iter()
            .copied()
            .find(u8::is_ascii_alphanumeric)
            .unwrap_or(others[0]);
        let mut buffer = [filler; 96];
        for value in 0..=255u8 {
            let member = class.contains(value);
            for len in 0..=buffer.len() {
                for at in 0..len {
                    buffer[at] = value;
                    let expected = if member { at } else { len };
                    assert_all(
                        &backends,
                        class,
                        &buffer[..len],
                        0,
                        expected,
                        format_args!("byte {value:#04x} at {at}"),
                    );
                    buffer[at] = filler;
                }
            }
        }

        let mut buffer = [0u8; 33 + MAX_REMAINDER];
        for from in STARTS {
            for remaining in 0..=MAX_REMAINDER {
                let len = from + remaining;
                for hit in 0..=remaining {
                    for (k, byte) in buffer[..len].iter_mut().enumerate() {
                        *byte = if k < from {
                            members[k % members.len()]
                        } else {
                            others[(k * 7 + hit) % others.len()]
                        };
                    }
                    if hit < remaining {
                        buffer[from + hit] = members[(from + hit) % members.len()];
                    }
                    assert_all(
                        &backends,
                        class,
                        &buffer[..len],
                        from,
                        from + hit,
                        format_args!("first member {hit} past the start"),
                    );
                }
            }
        }

        assert_seeded_inputs(&backends, class, 20_000);
    }

    /// Compares every backend with the independent oracle on seeded inputs
    /// whose member density runs from one byte in two to one in 256.
    fn assert_seeded_inputs(backends: &[Backend], class: &ByteClass, cases: usize) {
        let members: Vec<u8> = (0..=255u8).filter(|&b| class.contains(b)).collect();
        let others: Vec<u8> = (0..=255u8).filter(|&b| !class.contains(b)).collect();
        let mut state = 0x9E37_79B9_7F4A_7C15u64;
        let mut next = move || {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            state
        };
        let mut buffer = [0u8; 256];
        for case in 0..cases {
            let len = next() as usize % (buffer.len() + 1);
            let from = next() as usize % (len + 1);
            let sparsity = 2u64 << (case % 8);
            for byte in &mut buffer[..len] {
                let roll = next();
                let pick = (roll >> 16) as usize;
                *byte = if !members.is_empty() && roll % sparsity == 0 {
                    members[pick % members.len()]
                } else {
                    others[pick % others.len()]
                };
            }
            let bytes = &buffer[..len];
            assert_all(
                backends,
                class,
                bytes,
                from,
                scalar_first(class, bytes, from),
                format_args!("seeded case {case}"),
            );
        }
    }

    fn check_class(class: &ByteClass) {
        for byte in 0..=255u8 {
            let expected = class.contains(byte);
            let lo = class.low[(byte & 0x0F) as usize];
            let hi = class.high[(byte >> 4) as usize];
            assert_eq!(
                lo & hi != 0,
                expected,
                "nibble tables disagree on {byte:#x}"
            );
        }
        for value in 0..=255u8 {
            for offset in 0..=40 {
                for tail in [0, 1, 7, 8, 15, 16, 17, 31, 32, 33] {
                    let mut input = vec![b'a'; offset];
                    input.push(value);
                    input.extend(std::iter::repeat_n(b'z', tail));
                    for from in [0, offset.min(1), offset / 2] {
                        assert_eq!(
                            class.first_in(&input, from),
                            scalar_first(class, &input, from),
                            "value {value:#x} offset {offset} tail {tail} from {from}"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn runtime_classes_preserve_arbitrary_ascii_membership() {
        // Production classes are const-evaluated. Exercise the constructor
        // contract independently of those few fixed sets, including all rows.
        for modulus in [1, 2, 3, 7, 17, 127, 128] {
            let mut flags = [0; 256];
            for (byte, flag) in flags.iter_mut().enumerate().take(128) {
                *flag = u8::from(byte % modulus == 0);
            }
            let class = ByteClass::from_flags(flags);
            for byte in 0..=255u8 {
                let expected = byte < 128 && usize::from(byte) % modulus == 0;
                assert_eq!(class.contains(byte), expected);
                assert_eq!(
                    class.low[usize::from(byte & 15)] & class.high[usize::from(byte >> 4)] != 0,
                    expected
                );
            }
            // These classes use up to all eight row bits of the vector tables.
            assert_seeded_inputs(&backends(), &class, 2_000);
        }
        let empty = ByteClass::from_flags([0; 256]);
        assert_eq!(
            empty.first_in(b"ordinary text and []", 0),
            b"ordinary text and []".len()
        );
        assert_seeded_inputs(&backends(), &empty, 2_000);
    }

    #[test]
    fn vector_backends_match_flags_on_local_classes() {
        assert_backends_match_flags(&SAMPLE);
        assert_backends_match_flags(&DENSE);
    }

    #[test]
    #[should_panic(expected = "byte class members must be ASCII")]
    fn runtime_classes_reject_non_ascii_members() {
        let mut flags = [0; 256];
        flags[128] = 1;
        let _ = ByteClass::from_flags(flags);
    }

    #[test]
    fn nibble_tables_match_flags_for_every_byte() {
        check_class(&SAMPLE);
        check_class(&DENSE);
    }

    #[test]
    fn first_in_finds_the_earliest_member() {
        let bytes = b"abcdefghijklmnopqrstuvwxyz[0123456789]";
        assert_eq!(SAMPLE.first_in(bytes, 0), 26);
        assert_eq!(SAMPLE.first_in(bytes, 27), 37);
        assert_eq!(SAMPLE.first_in(bytes, 38), 38);
        assert_eq!(SAMPLE.first_in(b"", 0), 0);
        assert_eq!(SAMPLE.first_in(b"plain", 5), 5);
    }
}
