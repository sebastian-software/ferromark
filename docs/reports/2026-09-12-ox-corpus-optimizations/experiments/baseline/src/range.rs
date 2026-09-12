//! Compact range representation for zero-copy text references.
//!
//! Uses `u32` offsets to save memory (8 bytes vs 16 for usize pair).
//! Supports documents up to [`MAX_INPUT_BYTES`] bytes in size.

use std::fmt;

/// Largest input size supported by ferromark's compact source positions.
///
/// One byte is reserved so one-based line and column values remain
/// representable even when every byte is a newline or a single-byte scalar.
pub const MAX_INPUT_BYTES: usize = u32::MAX as usize - 1;

/// Error returned when an input cannot be represented by [`Range`] offsets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InputSizeError {
    input_len: usize,
}

impl InputSizeError {
    /// The rejected input length in bytes.
    #[must_use]
    pub const fn input_len(self) -> usize {
        self.input_len
    }
}

impl fmt::Display for InputSizeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "input is {} bytes, exceeding the maximum supported size of {MAX_INPUT_BYTES} bytes",
            self.input_len
        )
    }
}

impl std::error::Error for InputSizeError {}

/// Validate that an input length fits every ferromark source offset.
///
/// This is useful for callers of infallible legacy APIs, which panic when the
/// input is too large to represent. APIs that can return an error use the same
/// validation internally.
pub fn validate_input_size(input_len: usize) -> Result<(), InputSizeError> {
    if input_len <= MAX_INPUT_BYTES {
        Ok(())
    } else {
        Err(InputSizeError { input_len })
    }
}

#[inline]
pub(crate) fn assert_input_size(input_len: usize) {
    if let Err(error) = validate_input_size(input_len) {
        panic!("{error}");
    }
}

/// Convert a source offset after the input-size invariant has been checked.
#[inline]
pub(crate) fn offset_to_u32(offset: usize) -> u32 {
    u32::try_from(offset).expect("source offset exceeds the supported u32 range")
}

/// Compact range into an input buffer.
///
/// Fits 8 ranges per 64-byte L1 cache line.
///
/// # Example
/// ```
/// use ferromark::Range;
///
/// let input = b"Hello, World!";
/// let range = Range::new(0, 5);
/// assert_eq!(range.slice(input), b"Hello");
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
#[repr(C)]
pub struct Range {
    /// Inclusive byte offset where the range starts.
    pub start: u32,
    /// Exclusive byte offset where the range ends.
    pub end: u32,
}

// Compile-time size verification
const _: () = assert!(std::mem::size_of::<Range>() == 8);

impl Range {
    /// Create a new range.
    #[inline]
    pub const fn new(start: u32, end: u32) -> Self {
        Self { start, end }
    }

    /// Create a range from usize values.
    ///
    /// # Panics
    /// Panics if either value exceeds [`u32::MAX`].
    #[inline]
    pub fn from_usize(start: usize, end: usize) -> Self {
        Self {
            start: offset_to_u32(start),
            end: offset_to_u32(end),
        }
    }

    /// Create an empty range at a position.
    #[inline]
    pub const fn empty_at(pos: u32) -> Self {
        Self {
            start: pos,
            end: pos,
        }
    }

    /// Get the slice this range refers to.
    #[inline]
    pub fn slice<'a>(&self, input: &'a [u8]) -> &'a [u8] {
        &input[self.start as usize..self.end as usize]
    }

    /// Get the slice as a string, validating its UTF-8 encoding.
    #[inline]
    pub fn slice_str<'a>(&self, input: &'a [u8]) -> Result<&'a str, std::str::Utf8Error> {
        std::str::from_utf8(self.slice(input))
    }

    /// Get the slice as a string, validating its UTF-8 encoding.
    ///
    /// This compatibility alias is equivalent to [`Self::slice_str`].
    #[inline]
    pub fn try_slice_str<'a>(&self, input: &'a [u8]) -> Result<&'a str, std::str::Utf8Error> {
        self.slice_str(input)
    }

    /// Length of the range in bytes.
    #[inline]
    pub const fn len(&self) -> u32 {
        self.end - self.start
    }

    /// Check if the range is empty.
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.start == self.end
    }

    /// Start position as usize.
    #[inline]
    pub const fn start_usize(&self) -> usize {
        self.start as usize
    }

    /// End position as usize.
    #[inline]
    pub const fn end_usize(&self) -> usize {
        self.end as usize
    }

    /// Length as usize.
    #[inline]
    pub const fn len_usize(&self) -> usize {
        (self.end - self.start) as usize
    }

    /// Check if this range contains a position.
    #[inline]
    pub const fn contains(&self, pos: u32) -> bool {
        pos >= self.start && pos < self.end
    }

    /// Extend the end of this range.
    #[inline]
    pub fn extend_to(&mut self, new_end: u32) {
        debug_assert!(new_end >= self.end);
        self.end = new_end;
    }

    /// Create a subrange within this range.
    #[inline]
    pub const fn subrange(&self, offset: u32, len: u32) -> Self {
        debug_assert!(offset + len <= self.len());
        Self {
            start: self.start + offset,
            end: self.start + offset + len,
        }
    }
}

impl From<std::ops::Range<u32>> for Range {
    #[inline]
    fn from(r: std::ops::Range<u32>) -> Self {
        Self::new(r.start, r.end)
    }
}

impl From<std::ops::Range<usize>> for Range {
    #[inline]
    fn from(r: std::ops::Range<usize>) -> Self {
        Self::from_usize(r.start, r.end)
    }
}

impl From<Range> for std::ops::Range<usize> {
    #[inline]
    fn from(r: Range) -> Self {
        r.start_usize()..r.end_usize()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_range_size() {
        assert_eq!(std::mem::size_of::<Range>(), 8);
    }

    #[test]
    fn input_size_boundary_is_accepted() {
        assert!(validate_input_size(MAX_INPUT_BYTES).is_ok());
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn input_size_rejection_reports_the_actual_length() {
        let too_large = MAX_INPUT_BYTES + 1;
        let error = validate_input_size(too_large).unwrap_err();

        assert_eq!(error.input_len(), too_large);
        assert_eq!(
            error.to_string(),
            "input is 4294967295 bytes, exceeding the maximum supported size of 4294967294 bytes"
        );
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    #[should_panic(expected = "exceeding the maximum supported size")]
    fn legacy_input_guard_rejects_oversized_lengths() {
        assert_input_size(MAX_INPUT_BYTES + 1);
    }

    #[test]
    fn test_range_new() {
        let r = Range::new(10, 20);
        assert_eq!(r.start, 10);
        assert_eq!(r.end, 20);
        assert_eq!(r.len(), 10);
        assert!(!r.is_empty());
    }

    #[test]
    fn test_range_empty() {
        let r = Range::empty_at(5);
        assert_eq!(r.start, 5);
        assert_eq!(r.end, 5);
        assert_eq!(r.len(), 0);
        assert!(r.is_empty());
    }

    #[test]
    fn test_range_slice() {
        let input = b"Hello, World!";
        let r = Range::new(0, 5);
        assert_eq!(r.slice(input), b"Hello");

        let r2 = Range::new(7, 12);
        assert_eq!(r2.slice(input), b"World");
    }

    #[test]
    fn slice_str_rejects_invalid_utf8() {
        let input = [0xff];
        let range = Range::new(0, 1);

        assert!(range.slice_str(&input).is_err());
        assert!(range.try_slice_str(&input).is_err());
    }

    #[test]
    fn test_range_from_usize() {
        let r = Range::from_usize(100, 200);
        assert_eq!(r.start, 100);
        assert_eq!(r.end, 200);
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    #[should_panic(expected = "source offset exceeds the supported u32 range")]
    fn range_from_usize_rejects_truncation() {
        let _ = Range::from_usize(u32::MAX as usize + 1, u32::MAX as usize + 1);
    }

    #[test]
    fn test_range_contains() {
        let r = Range::new(10, 20);
        assert!(!r.contains(9));
        assert!(r.contains(10));
        assert!(r.contains(15));
        assert!(r.contains(19));
        assert!(!r.contains(20));
    }

    #[test]
    fn test_range_subrange() {
        let r = Range::new(100, 200);
        let sub = r.subrange(10, 20);
        assert_eq!(sub.start, 110);
        assert_eq!(sub.end, 130);
    }

    #[test]
    fn test_range_from_std_range() {
        let r: Range = (10u32..20u32).into();
        assert_eq!(r.start, 10);
        assert_eq!(r.end, 20);

        let r2: Range = (10usize..20usize).into();
        assert_eq!(r2.start, 10);
        assert_eq!(r2.end, 20);
    }

    #[test]
    fn test_range_extend() {
        let mut r = Range::new(10, 20);
        r.extend_to(30);
        assert_eq!(r.end, 30);
    }

    #[test]
    fn test_cache_line_fit() {
        // 8 ranges should fit in a 64-byte L1 cache line
        assert!(std::mem::size_of::<[Range; 8]>() <= 64);
    }
}
