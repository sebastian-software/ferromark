//! Byte searches over the short slices the inline scanners work on.
//!
//! Link destinations, titles, and labels are a few dozen bytes at most: the
//! median destination in an encyclopedia article does not fill one vector.
//! `memchr` is tuned for the opposite case. Below a vector it drops into a
//! byte-at-a-time walk behind a call the inline path repeats for every link,
//! and that fallback is visible in profiles of link-dense documents.
//!
//! These helpers run the same eight-byte SWAR (SIMD-within-a-register) word
//! test the pre-pass line scanner uses — see `line_scan.rs` for the shape of
//! the trick — and hand anything a vector wide or wider straight to `memchr`,
//! which wins from there on. The result is the `memchr` result for every
//! input; only the route to it changes.

const ONES: u64 = 0x0101_0101_0101_0101;
const HIGH: u64 = 0x8080_8080_8080_8080;

/// Length at which `memchr`'s vector scan takes over. One vector is its own
/// breakeven: below it `memchr` walks bytes, above it the word test loses.
const VECTOR: usize = 16;

/// Sets `0x80` in every byte lane of `word` that is zero.
///
/// A lane holding `0x01` can also light up when a lower lane borrowed into
/// it, so callers must only consume the *lowest* set bit: a spurious lane can
/// only sit above a genuine zero lane. That still holds after several of
/// these masks are combined, because the genuine lane that caused a borrow is
/// itself set in the same mask and sits lower.
#[inline]
const fn has_zero(word: u64) -> u64 {
    word.wrapping_sub(ONES) & !word & HIGH
}

/// `byte` repeated into every lane of a word.
#[inline]
fn splat(byte: u8) -> u64 {
    // `0xFF * ONES` is exactly `u64::MAX`, so no byte overflows the product.
    u64::from(byte) * ONES
}

/// Offset of the first `needle`, like [`memchr::memchr`].
#[inline]
pub(in crate::parser) fn find(needle: u8, bytes: &[u8]) -> Option<usize> {
    if bytes.len() >= VECTOR {
        return memchr::memchr(needle, bytes);
    }
    let lanes = splat(needle);
    scan_short(bytes, |word| has_zero(word ^ lanes), |byte| byte == needle)
}

/// Offset of the first `first` or `second`, like [`memchr::memchr2`].
#[inline]
pub(in crate::parser) fn find2(first: u8, second: u8, bytes: &[u8]) -> Option<usize> {
    if bytes.len() >= VECTOR {
        return memchr::memchr2(first, second, bytes);
    }
    let (first_lanes, second_lanes) = (splat(first), splat(second));
    scan_short(
        bytes,
        |word| has_zero(word ^ first_lanes) | has_zero(word ^ second_lanes),
        |byte| byte == first || byte == second,
    )
}

/// Offset of the first `first`, `second`, or `third`, like
/// [`memchr::memchr3`].
#[inline]
pub(in crate::parser) fn find3(first: u8, second: u8, third: u8, bytes: &[u8]) -> Option<usize> {
    if bytes.len() >= VECTOR {
        return memchr::memchr3(first, second, third, bytes);
    }
    let (first_lanes, second_lanes, third_lanes) = (splat(first), splat(second), splat(third));
    scan_short(
        bytes,
        |word| {
            has_zero(word ^ first_lanes)
                | has_zero(word ^ second_lanes)
                | has_zero(word ^ third_lanes)
        },
        |byte| byte == first || byte == second || byte == third,
    )
}

/// One word test followed by the remainder. Callers keep this under a vector,
/// so the word loop runs at most once.
#[inline]
fn scan_short(
    bytes: &[u8],
    word_test: impl Fn(u64) -> u64,
    byte_test: impl Fn(u8) -> bool,
) -> Option<usize> {
    let mut i = 0;
    while i + 8 <= bytes.len() {
        let mut chunk = [0u8; 8];
        chunk.copy_from_slice(&bytes[i..i + 8]);
        let mask = word_test(u64::from_le_bytes(chunk));
        if mask != 0 {
            return Some(i + (mask.trailing_zeros() / 8) as usize);
        }
        i += 8;
    }
    while i < bytes.len() {
        if byte_test(bytes[i]) {
            return Some(i);
        }
        i += 1;
    }
    None
}

#[cfg(test)]
mod tests {
    // Owned buffers keep the test oracle independent of parser storage.
    #![allow(
        clippy::disallowed_macros,
        clippy::disallowed_methods,
        clippy::disallowed_types
    )]

    use super::{find, find2, find3};

    /// The needle sets the inline scanners actually search for, plus the
    /// values a lane can borrow from or into and one above the high bit.
    const NEEDLES: [(u8, u8, u8); 3] =
        [(b'\\', b'&', b'`'), (b'[', b'*', b'_'), (0x00, 0x01, 0xFF)];

    #[track_caller]
    fn check(bytes: &[u8]) {
        for (first, second, third) in NEEDLES {
            assert_eq!(
                find(first, bytes),
                memchr::memchr(first, bytes),
                "find {first:#x} over {bytes:?}"
            );
            assert_eq!(
                find2(first, second, bytes),
                memchr::memchr2(first, second, bytes),
                "find2 {first:#x},{second:#x} over {bytes:?}"
            );
            assert_eq!(
                find3(first, second, third, bytes),
                memchr::memchr3(first, second, third, bytes),
                "find3 {first:#x},{second:#x},{third:#x} over {bytes:?}"
            );
        }
    }

    #[test]
    fn short_scan_matches_memchr_for_every_byte_value_and_position() {
        // Lengths on both sides of the word step, the vector threshold, and
        // the tail that follows each of them.
        for len in [0, 1, 2, 3, 7, 8, 9, 15, 16, 17, 23, 24, 31, 40] {
            let mut buffer = vec![b'u'; len];
            for value in 0..=255u8 {
                for at in 0..len {
                    buffer[at] = value;
                    check(&buffer);
                    buffer[at] = b'u';
                }
            }
        }
    }

    #[test]
    fn short_scan_matches_memchr_across_the_word_and_vector_boundaries() {
        // Fillers that can borrow into the lane above them, so a spurious
        // `0x01` lane would have to be reported instead of the real match.
        for filler in [b'u', 0x00, 0x01, 0x02] {
            for len in 0..=64usize {
                let mut buffer = vec![filler; len];
                check(&buffer);
                for at in 0..len {
                    for value in [b'\\', b'[', 0x00, 0xFF] {
                        buffer[at] = value;
                        check(&buffer);
                        buffer[at] = filler;
                    }
                }
            }
        }
    }

    #[test]
    fn short_scan_reports_the_first_of_several_matches() {
        // Two needles in one word: only the lowest lane of the combined mask
        // is trustworthy, and it has to be the earlier of the two.
        for len in 0..=24usize {
            for first_at in 0..len {
                for second_at in first_at..len {
                    for (early, late) in [(b'\\', b'&'), (b'&', b'\\'), (b'\\', b'\\')] {
                        let mut buffer = vec![0x01u8; len];
                        buffer[first_at] = early;
                        buffer[second_at] = late;
                        check(&buffer);
                    }
                }
            }
        }
    }
}
