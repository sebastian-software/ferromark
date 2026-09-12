//! Pointer-based cursor for high-performance byte scanning.
//!
//! Uses raw pointers internally for maximum scanning speed,
//! wrapped in a safe API with bounds checking at block entry.

/// A cursor for efficient byte-by-byte scanning.
///
/// Internally uses raw pointers to avoid bounds checks in tight loops.
/// The cursor is bounds-checked at creation and when advancing past known-safe regions.
#[derive(Clone, Copy)]
pub struct Cursor<'a> {
    ptr: *const u8,
    end: *const u8,
    base: *const u8,
    _marker: std::marker::PhantomData<&'a [u8]>,
}

impl<'a> Cursor<'a> {
    /// Create a new cursor over a byte slice.
    #[inline]
    pub fn new(input: &'a [u8]) -> Self {
        let ptr = input.as_ptr();
        let end = unsafe { ptr.add(input.len()) };
        Self {
            ptr,
            end,
            base: ptr,
            _marker: std::marker::PhantomData,
        }
    }

    /// Create a cursor starting at an offset.
    ///
    /// # Panics
    ///
    /// Panics if `offset` is past the end of `input`.
    #[inline]
    pub fn new_at(input: &'a [u8], offset: usize) -> Self {
        assert!(
            offset <= input.len(),
            "cursor offset {offset} exceeds input length {}",
            input.len()
        );
        let base = input.as_ptr();
        let ptr = unsafe { base.add(offset) };
        let end = unsafe { base.add(input.len()) };
        Self {
            ptr,
            end,
            base,
            _marker: std::marker::PhantomData,
        }
    }

    /// Current offset from the start of input.
    #[inline]
    pub fn offset(&self) -> usize {
        // SAFETY: ptr >= base by construction
        unsafe { self.ptr.offset_from(self.base) as usize }
    }

    /// Number of bytes remaining.
    #[inline]
    pub fn remaining(&self) -> usize {
        // SAFETY: end >= ptr by construction
        unsafe { self.end.offset_from(self.ptr) as usize }
    }

    /// Check if cursor is at end of input.
    #[inline]
    pub fn is_eof(&self) -> bool {
        self.ptr >= self.end
    }

    /// Peek the current byte without advancing.
    #[inline]
    pub fn peek(&self) -> Option<u8> {
        if self.is_eof() {
            None
        } else {
            // SAFETY: not at EOF
            Some(unsafe { *self.ptr })
        }
    }

    /// Peek the current byte, returning 0 at EOF.
    ///
    /// Useful for lookup tables where 0 is a sentinel.
    #[inline]
    pub fn peek_or_zero(&self) -> u8 {
        if self.is_eof() {
            0
        } else {
            unsafe { *self.ptr }
        }
    }

    /// Peek at byte n positions ahead.
    #[inline]
    pub fn peek_ahead(&self, n: usize) -> Option<u8> {
        if n >= self.remaining() {
            None
        } else {
            // SAFETY: n < remaining
            Some(unsafe { *self.ptr.add(n) })
        }
    }

    /// Advance without checking the remaining length.
    ///
    /// This is crate-private so parser hot paths can avoid duplicate checks
    /// after their grammar logic has already established the bound.
    ///
    /// # Safety
    ///
    /// `n` must not exceed [`Self::remaining`].
    #[inline]
    pub(crate) unsafe fn advance_unchecked(&mut self, n: usize) {
        debug_assert!(n <= self.remaining());
        // SAFETY: The caller guarantees the resulting pointer stays within
        // the allocation or exactly one byte past its end.
        self.ptr = unsafe { self.ptr.add(n) };
    }

    /// Advance by one byte without checking for EOF.
    ///
    /// # Safety
    ///
    /// The cursor must not be at EOF.
    #[inline]
    pub(crate) unsafe fn bump_unchecked(&mut self) {
        debug_assert!(!self.is_eof());
        // SAFETY: The caller guarantees one byte remains.
        self.ptr = unsafe { self.ptr.add(1) };
    }

    /// Check if current position matches a byte.
    #[inline]
    pub fn at(&self, b: u8) -> bool {
        self.peek() == Some(b)
    }

    /// Skip while predicate is true.
    #[inline]
    pub fn skip_while<F>(&mut self, mut predicate: F) -> usize
    where
        F: FnMut(u8) -> bool,
    {
        let start = self.offset();
        while let Some(b) = self.peek() {
            if !predicate(b) {
                break;
            }
            // SAFETY: `peek` returned a byte, so the cursor is not at EOF.
            unsafe { self.bump_unchecked() };
        }
        self.offset() - start
    }

    /// Skip whitespace (space and tab).
    #[inline]
    pub fn skip_whitespace(&mut self) -> usize {
        self.skip_while(|b| b == b' ' || b == b'\t')
    }

    /// Get the remaining bytes as a slice.
    #[inline]
    pub fn remaining_slice(&self) -> &'a [u8] {
        // SAFETY: ptr and end are valid pointers from the same allocation
        unsafe { std::slice::from_raw_parts(self.ptr, self.remaining()) }
    }

    /// Find the next occurrence of a byte using memchr.
    #[inline]
    pub fn find(&self, needle: u8) -> Option<usize> {
        memchr::memchr(needle, self.remaining_slice())
    }

    /// Find the next newline.
    #[inline]
    pub fn find_newline(&self) -> Option<usize> {
        self.find(b'\n')
    }
}

impl std::fmt::Debug for Cursor<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Cursor")
            .field("offset", &self.offset())
            .field("remaining", &self.remaining())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cursor_new() {
        let input = b"Hello";
        let cursor = Cursor::new(input);
        assert_eq!(cursor.offset(), 0);
        assert_eq!(cursor.remaining(), 5);
        assert!(!cursor.is_eof());
    }

    #[test]
    fn test_cursor_empty() {
        let cursor = Cursor::new(b"");
        assert_eq!(cursor.offset(), 0);
        assert_eq!(cursor.remaining(), 0);
        assert!(cursor.is_eof());
        assert_eq!(cursor.peek(), None);
    }

    #[test]
    fn test_cursor_peek() {
        let cursor = Cursor::new(b"abc");
        assert_eq!(cursor.peek(), Some(b'a'));
        assert_eq!(cursor.peek_ahead(0), Some(b'a'));
        assert_eq!(cursor.peek_ahead(1), Some(b'b'));
        assert_eq!(cursor.peek_ahead(2), Some(b'c'));
        assert_eq!(cursor.peek_ahead(3), None);
    }

    #[test]
    fn test_cursor_at() {
        let cursor = Cursor::new(b"abc");
        assert!(cursor.at(b'a'));
        assert!(!cursor.at(b'b'));
    }

    #[test]
    fn test_cursor_skip_whitespace() {
        let mut cursor = Cursor::new(b" \t abc");
        let skipped = cursor.skip_whitespace();
        assert_eq!(skipped, 3);
        assert_eq!(cursor.peek(), Some(b'a'));
    }

    #[test]
    fn test_cursor_find() {
        let cursor = Cursor::new(b"hello\nworld");
        assert_eq!(cursor.find(b'\n'), Some(5));
        assert_eq!(cursor.find(b'x'), None);
    }

    #[test]
    fn test_cursor_new_at() {
        let input = b"hello world";
        let cursor = Cursor::new_at(input, 6);
        assert_eq!(cursor.offset(), 6);
        assert_eq!(cursor.peek(), Some(b'w'));
    }

    #[test]
    #[should_panic(expected = "cursor offset 2 exceeds input length 1")]
    fn new_at_panics_when_offset_exceeds_input() {
        let _ = Cursor::new_at(b"x", 2);
    }
}
