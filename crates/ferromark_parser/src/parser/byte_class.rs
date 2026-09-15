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

/// An ASCII byte set with a scalar flag table and vector nibble tables.
pub(in crate::parser) struct ByteClass {
    flags: [u8; 256],
    #[cfg_attr(not(target_arch = "aarch64"), allow(dead_code))]
    low: [u8; 16],
    #[cfg_attr(not(target_arch = "aarch64"), allow(dead_code))]
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
        #[cfg(target_arch = "aarch64")]
        {
            // One vector is the breakeven against the table walk; shorter
            // remainders would pay a 16-byte overlapping reload.
            if bytes.len() - from >= 16 {
                return self.first_in_neon(bytes, from);
            }
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
}

#[cfg(test)]
mod tests {
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
        }
        let empty = ByteClass::from_flags([0; 256]);
        assert_eq!(
            empty.first_in(b"ordinary text and []", 0),
            b"ordinary text and []".len()
        );
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
