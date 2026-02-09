//! Compact range representation for zero-copy text references.
//!
//! Uses `u32` offsets to save memory (8 bytes vs 16 for usize pair).
//! Supports documents up to 4GB in size.

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
    pub start: u32,
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
    /// Panics in debug mode if values exceed u32::MAX.
    #[inline]
    pub fn from_usize(start: usize, end: usize) -> Self {
        debug_assert!(start <= u32::MAX as usize);
        debug_assert!(end <= u32::MAX as usize);
        Self {
            start: start as u32,
            end: end as u32,
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

    /// Get the slice as a str (assumes valid UTF-8).
    ///
    /// # Safety
    /// The caller must ensure the slice contains valid UTF-8.
    #[inline]
    pub fn slice_str<'a>(&self, input: &'a [u8]) -> &'a str {
        // SAFETY: Caller guarantees valid UTF-8
        unsafe { std::str::from_utf8_unchecked(self.slice(input)) }
    }

    /// Try to get the slice as a str.
    #[inline]
    pub fn try_slice_str<'a>(&self, input: &'a [u8]) -> Result<&'a str, std::str::Utf8Error> {
        std::str::from_utf8(self.slice(input))
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
    fn test_range_from_usize() {
        let r = Range::from_usize(100, 200);
        assert_eq!(r.start, 100);
        assert_eq!(r.end, 200);
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
