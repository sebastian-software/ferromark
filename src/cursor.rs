//! Pointer-based cursor for high-performance byte scanning.
//!
//! Uses raw pointers internally for maximum scanning speed,
//! wrapped in a safe API with bounds checking at block entry.

use crate::Range;

/// A cursor for efficient byte-by-byte scanning.
///
/// Internally uses raw pointers to avoid bounds checks in tight loops.
/// The cursor is bounds-checked at creation and when advancing past known-safe regions.
///
/// # Example
/// ```
/// use md_fast::cursor::Cursor;
///
/// let input = b"Hello, World!";
/// let mut cursor = Cursor::new(input);
///
/// assert_eq!(cursor.peek(), Some(b'H'));
/// cursor.advance(7);
/// assert_eq!(cursor.peek(), Some(b'W'));
/// ```
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
    #[inline]
    pub fn new_at(input: &'a [u8], offset: usize) -> Self {
        debug_assert!(offset <= input.len());
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

    /// Peek the current byte without bounds check.
    ///
    /// # Safety
    /// Caller must ensure cursor is not at EOF.
    #[inline]
    pub unsafe fn peek_unchecked(&self) -> u8 {
        debug_assert!(!self.is_eof());
        // SAFETY: Caller guarantees not at EOF
        unsafe { *self.ptr }
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

    /// Advance by n bytes.
    #[inline]
    pub fn advance(&mut self, n: usize) {
        debug_assert!(n <= self.remaining());
        // SAFETY: n <= remaining checked in debug
        self.ptr = unsafe { self.ptr.add(n) };
    }

    /// Advance by 1 byte.
    #[inline]
    pub fn bump(&mut self) {
        debug_assert!(!self.is_eof());
        self.ptr = unsafe { self.ptr.add(1) };
    }

    /// Consume and return current byte.
    #[inline]
    pub fn next(&mut self) -> Option<u8> {
        if self.is_eof() {
            None
        } else {
            // SAFETY: not at EOF
            let b = unsafe { *self.ptr };
            self.ptr = unsafe { self.ptr.add(1) };
            Some(b)
        }
    }

    /// Check if current position matches a byte.
    #[inline]
    pub fn at(&self, b: u8) -> bool {
        self.peek() == Some(b)
    }

    /// Check if current position matches any of the given bytes.
    #[inline]
    pub fn at_any(&self, bytes: &[u8]) -> bool {
        match self.peek() {
            Some(b) => bytes.contains(&b),
            None => false,
        }
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
            self.bump();
        }
        self.offset() - start
    }

    /// Skip whitespace (space and tab).
    #[inline]
    pub fn skip_whitespace(&mut self) -> usize {
        self.skip_while(|b| b == b' ' || b == b'\t')
    }

    /// Skip spaces only.
    #[inline]
    pub fn skip_spaces(&mut self) -> usize {
        self.skip_while(|b| b == b' ')
    }

    /// Consume a specific byte if present.
    #[inline]
    pub fn eat(&mut self, b: u8) -> bool {
        if self.at(b) {
            self.bump();
            true
        } else {
            false
        }
    }

    /// Consume a specific byte sequence if present.
    #[inline]
    pub fn eat_bytes(&mut self, bytes: &[u8]) -> bool {
        if self.remaining() < bytes.len() {
            return false;
        }
        // SAFETY: remaining >= bytes.len()
        let slice = unsafe { std::slice::from_raw_parts(self.ptr, bytes.len()) };
        if slice == bytes {
            self.advance(bytes.len());
            true
        } else {
            false
        }
    }

    /// Get a range from a start offset to current position.
    #[inline]
    pub fn range_from(&self, start: usize) -> Range {
        Range::from_usize(start, self.offset())
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

    /// Advance to the next newline, returning the range of the line (excluding newline).
    #[inline]
    pub fn consume_line(&mut self) -> Range {
        let start = self.offset();
        match self.find_newline() {
            Some(pos) => {
                let end = start + pos;
                self.advance(pos + 1); // Skip past newline
                Range::from_usize(start, end)
            }
            None => {
                // No newline found, consume rest of input
                let end = start + self.remaining();
                self.advance(self.remaining());
                Range::from_usize(start, end)
            }
        }
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
    fn test_cursor_advance() {
        let mut cursor = Cursor::new(b"Hello");
        assert_eq!(cursor.peek(), Some(b'H'));

        cursor.advance(2);
        assert_eq!(cursor.offset(), 2);
        assert_eq!(cursor.peek(), Some(b'l'));

        cursor.bump();
        assert_eq!(cursor.offset(), 3);
        assert_eq!(cursor.peek(), Some(b'l'));
    }

    #[test]
    fn test_cursor_next() {
        let mut cursor = Cursor::new(b"abc");
        assert_eq!(cursor.next(), Some(b'a'));
        assert_eq!(cursor.next(), Some(b'b'));
        assert_eq!(cursor.next(), Some(b'c'));
        assert_eq!(cursor.next(), None);
    }

    #[test]
    fn test_cursor_at() {
        let cursor = Cursor::new(b"abc");
        assert!(cursor.at(b'a'));
        assert!(!cursor.at(b'b'));
        assert!(cursor.at_any(b"axy"));
        assert!(!cursor.at_any(b"xyz"));
    }

    #[test]
    fn test_cursor_skip_while() {
        let mut cursor = Cursor::new(b"   abc");
        let skipped = cursor.skip_spaces();
        assert_eq!(skipped, 3);
        assert_eq!(cursor.peek(), Some(b'a'));
    }

    #[test]
    fn test_cursor_skip_whitespace() {
        let mut cursor = Cursor::new(b" \t abc");
        let skipped = cursor.skip_whitespace();
        assert_eq!(skipped, 3);
        assert_eq!(cursor.peek(), Some(b'a'));
    }

    #[test]
    fn test_cursor_eat() {
        let mut cursor = Cursor::new(b"abc");
        assert!(cursor.eat(b'a'));
        assert!(!cursor.eat(b'a'));
        assert!(cursor.eat(b'b'));
    }

    #[test]
    fn test_cursor_eat_bytes() {
        let mut cursor = Cursor::new(b"hello world");
        assert!(cursor.eat_bytes(b"hello"));
        assert_eq!(cursor.peek(), Some(b' '));
        assert!(!cursor.eat_bytes(b"hello"));
        assert!(cursor.eat_bytes(b" world"));
        assert!(cursor.is_eof());
    }

    #[test]
    fn test_cursor_find() {
        let cursor = Cursor::new(b"hello\nworld");
        assert_eq!(cursor.find(b'\n'), Some(5));
        assert_eq!(cursor.find(b'x'), None);
    }

    #[test]
    fn test_cursor_consume_line() {
        let mut cursor = Cursor::new(b"line1\nline2\nline3");

        let line1 = cursor.consume_line();
        assert_eq!(line1.slice(b"line1\nline2\nline3"), b"line1");

        let line2 = cursor.consume_line();
        assert_eq!(line2.slice(b"line1\nline2\nline3"), b"line2");

        let line3 = cursor.consume_line();
        assert_eq!(line3.slice(b"line1\nline2\nline3"), b"line3");

        assert!(cursor.is_eof());
    }

    #[test]
    fn test_cursor_consume_line_no_trailing_newline() {
        let mut cursor = Cursor::new(b"hello");
        let line = cursor.consume_line();
        assert_eq!(line.slice(b"hello"), b"hello");
        assert!(cursor.is_eof());
    }

    #[test]
    fn test_cursor_range_from() {
        let mut cursor = Cursor::new(b"hello world");
        cursor.advance(6);
        let range = cursor.range_from(0);
        assert_eq!(range.start, 0);
        assert_eq!(range.end, 6);
    }

    #[test]
    fn test_cursor_new_at() {
        let input = b"hello world";
        let cursor = Cursor::new_at(input, 6);
        assert_eq!(cursor.offset(), 6);
        assert_eq!(cursor.peek(), Some(b'w'));
    }
}
