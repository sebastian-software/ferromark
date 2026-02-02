//! Mark collection and buffer for inline parsing.
//!
//! Marks represent potential delimiter positions (backticks, asterisks, etc.)
//! collected in a single pass before resolution.

use crate::limits;

/// Flags for mark state.
pub mod flags {
    pub const POTENTIAL_OPENER: u8 = 0b0001;
    pub const POTENTIAL_CLOSER: u8 = 0b0010;
    pub const RESOLVED: u8 = 0b0100;
    /// Mark is part of a code span (skip in emphasis processing).
    pub const IN_CODE: u8 = 0b1000;
}

/// A potential delimiter mark.
#[derive(Debug, Clone, Copy)]
pub struct Mark {
    /// Start position in text.
    pub pos: u32,
    /// End position (pos + run_length).
    pub end: u32,
    /// The delimiter character.
    pub ch: u8,
    /// State flags.
    pub flags: u8,
}

impl Mark {
    /// Create a new mark.
    #[inline]
    pub fn new(pos: u32, end: u32, ch: u8, flags: u8) -> Self {
        Self { pos, end, ch, flags }
    }

    /// Length of the delimiter run.
    #[inline]
    pub fn len(&self) -> u32 {
        self.end - self.pos
    }

    /// Check if this mark can open emphasis.
    #[inline]
    pub fn can_open(&self) -> bool {
        self.flags & flags::POTENTIAL_OPENER != 0 && self.flags & flags::RESOLVED == 0
    }

    /// Check if this mark can close emphasis.
    #[inline]
    pub fn can_close(&self) -> bool {
        self.flags & flags::POTENTIAL_CLOSER != 0 && self.flags & flags::RESOLVED == 0
    }

    /// Check if this mark has been resolved.
    #[inline]
    pub fn is_resolved(&self) -> bool {
        self.flags & flags::RESOLVED != 0
    }

    /// Mark as resolved.
    #[inline]
    pub fn resolve(&mut self) {
        self.flags |= flags::RESOLVED;
    }
}

/// Buffer for collecting marks during inline scanning.
#[derive(Debug)]
pub struct MarkBuffer {
    marks: Vec<Mark>,
}

impl MarkBuffer {
    /// Create a new mark buffer.
    pub fn new() -> Self {
        Self {
            marks: Vec::with_capacity(64),
        }
    }

    /// Clear the buffer for reuse.
    #[inline]
    pub fn clear(&mut self) {
        self.marks.clear();
    }

    /// Add a mark if we haven't exceeded the limit.
    #[inline]
    pub fn push(&mut self, mark: Mark) {
        if self.marks.len() < limits::MAX_INLINE_MARKS {
            self.marks.push(mark);
        }
    }

    /// Get marks slice.
    #[inline]
    pub fn marks(&self) -> &[Mark] {
        &self.marks
    }

    /// Get mutable marks slice.
    #[inline]
    pub fn marks_mut(&mut self) -> &mut [Mark] {
        &mut self.marks
    }

    /// Number of marks.
    #[inline]
    pub fn len(&self) -> usize {
        self.marks.len()
    }

    /// Check if empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.marks.is_empty()
    }
}

impl Default for MarkBuffer {
    fn default() -> Self {
        Self::new()
    }
}

/// Lookup table for special inline characters.
/// Returns true if the character might be a delimiter.
pub static SPECIAL_CHARS: [bool; 256] = {
    let mut table = [false; 256];
    table[b'`' as usize] = true;  // Code span
    table[b'*' as usize] = true;  // Emphasis
    table[b'_' as usize] = true;  // Emphasis
    table[b'\\' as usize] = true; // Escape
    table[b'\n' as usize] = true; // Line break
    table[b'[' as usize] = true;  // Link (future)
    table[b']' as usize] = true;  // Link (future)
    table[b'<' as usize] = true;  // Autolink/HTML (future)
    table[b'&' as usize] = true;  // Entity (future)
    table
};

/// Scan text and collect marks.
/// Returns the collected marks.
pub fn collect_marks(text: &[u8], buffer: &mut MarkBuffer) {
    buffer.clear();

    let mut pos = 0;
    let len = text.len();

    while pos < len {
        let b = text[pos];

        if !SPECIAL_CHARS[b as usize] {
            pos += 1;
            continue;
        }

        match b {
            b'`' => {
                // Count consecutive backticks
                let start = pos;
                while pos < len && text[pos] == b'`' {
                    pos += 1;
                }
                let run_len = pos - start;

                // Code span backticks can be both opener and closer
                if run_len <= limits::MAX_CODE_SPAN_BACKTICKS {
                    buffer.push(Mark::new(
                        start as u32,
                        pos as u32,
                        b'`',
                        flags::POTENTIAL_OPENER | flags::POTENTIAL_CLOSER,
                    ));
                }
            }

            b'*' | b'_' => {
                // Count consecutive asterisks/underscores
                let start = pos;
                let ch = b;
                while pos < len && text[pos] == ch {
                    pos += 1;
                }

                // Determine opener/closer status based on surrounding chars
                let before = if start > 0 { text[start - 1] } else { b' ' };
                let after = if pos < len { text[pos] } else { b' ' };

                let flags = compute_emphasis_flags(ch, before, after);

                if flags != 0 {
                    buffer.push(Mark::new(start as u32, pos as u32, ch, flags));
                }
            }

            b'\\' => {
                // Backslash escape - mark position and skip escaped char
                if pos + 1 < len && is_escapable(text[pos + 1]) {
                    buffer.push(Mark::new(
                        pos as u32,
                        (pos + 2) as u32,
                        b'\\',
                        flags::POTENTIAL_OPENER, // Mark for processing
                    ));
                    pos += 2;
                } else {
                    pos += 1;
                }
            }

            b'\n' => {
                // Check for hard break (two spaces before newline)
                let has_hard_break = pos >= 2
                    && text[pos - 1] == b' '
                    && text[pos - 2] == b' ';

                if has_hard_break {
                    buffer.push(Mark::new(
                        (pos - 2) as u32,
                        (pos + 1) as u32,
                        b'\n',
                        flags::POTENTIAL_OPENER, // Hard break marker
                    ));
                }
                pos += 1;
            }

            b'[' => {
                // Check for image: ![
                let is_image = pos > 0 && text[pos - 1] == b'!';
                buffer.push(Mark::new(
                    pos as u32,
                    (pos + 1) as u32,
                    b'[',
                    if is_image { flags::POTENTIAL_OPENER | flags::IN_CODE } else { flags::POTENTIAL_OPENER },
                ));
                pos += 1;
            }

            b']' => {
                buffer.push(Mark::new(
                    pos as u32,
                    (pos + 1) as u32,
                    b']',
                    flags::POTENTIAL_CLOSER,
                ));
                pos += 1;
            }

            b'<' => {
                // Potential autolink
                buffer.push(Mark::new(
                    pos as u32,
                    (pos + 1) as u32,
                    b'<',
                    flags::POTENTIAL_OPENER,
                ));
                pos += 1;
            }

            _ => {
                // Other special chars (entities, etc.)
                pos += 1;
            }
        }
    }
}

/// Compute opener/closer flags for emphasis delimiters.
/// Based on CommonMark "left-flanking" and "right-flanking" rules.
fn compute_emphasis_flags(ch: u8, before: u8, after: u8) -> u8 {
    let before_space = is_whitespace_or_start(before);
    let after_space = is_whitespace_or_end(after);
    let before_punct = is_punctuation(before);
    let after_punct = is_punctuation(after);

    // Left-flanking: not followed by whitespace, and either
    // not followed by punctuation or preceded by whitespace/punctuation
    let left_flanking = !after_space
        && (!after_punct || before_space || before_punct);

    // Right-flanking: not preceded by whitespace, and either
    // not preceded by punctuation or followed by whitespace/punctuation
    let right_flanking = !before_space
        && (!before_punct || after_space || after_punct);

    let mut flags = 0;

    if ch == b'*' {
        // Asterisk: can open if left-flanking, close if right-flanking
        if left_flanking {
            flags |= flags::POTENTIAL_OPENER;
        }
        if right_flanking {
            flags |= flags::POTENTIAL_CLOSER;
        }
    } else {
        // Underscore: more restrictive rules
        // Can open if left-flanking and (not right-flanking or preceded by punctuation)
        if left_flanking && (!right_flanking || before_punct) {
            flags |= flags::POTENTIAL_OPENER;
        }
        // Can close if right-flanking and (not left-flanking or followed by punctuation)
        if right_flanking && (!left_flanking || after_punct) {
            flags |= flags::POTENTIAL_CLOSER;
        }
    }

    flags
}

#[inline]
fn is_whitespace_or_start(b: u8) -> bool {
    b == b' ' || b == b'\t' || b == b'\n' || b == b'\r' || b == 0
}

#[inline]
fn is_whitespace_or_end(b: u8) -> bool {
    b == b' ' || b == b'\t' || b == b'\n' || b == b'\r' || b == 0
}

#[inline]
fn is_punctuation(b: u8) -> bool {
    matches!(b,
        b'!' | b'"' | b'#' | b'$' | b'%' | b'&' | b'\'' | b'(' | b')' |
        b'*' | b'+' | b',' | b'-' | b'.' | b'/' | b':' | b';' | b'<' |
        b'=' | b'>' | b'?' | b'@' | b'[' | b'\\' | b']' | b'^' | b'_' |
        b'`' | b'{' | b'|' | b'}' | b'~'
    )
}

/// Characters that can be escaped with backslash.
#[inline]
fn is_escapable(b: u8) -> bool {
    matches!(b,
        b'!' | b'"' | b'#' | b'$' | b'%' | b'&' | b'\'' | b'(' | b')' |
        b'*' | b'+' | b',' | b'-' | b'.' | b'/' | b':' | b';' | b'<' |
        b'=' | b'>' | b'?' | b'@' | b'[' | b'\\' | b']' | b'^' | b'_' |
        b'`' | b'{' | b'|' | b'}' | b'~'
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mark_size() {
        assert!(std::mem::size_of::<Mark>() <= 16);
    }

    #[test]
    fn test_collect_backticks() {
        let mut buffer = MarkBuffer::new();
        collect_marks(b"hello `code` world", &mut buffer);

        assert_eq!(buffer.len(), 2);
        assert_eq!(buffer.marks()[0].ch, b'`');
        assert_eq!(buffer.marks()[0].len(), 1);
        assert_eq!(buffer.marks()[1].ch, b'`');
    }

    #[test]
    fn test_collect_emphasis() {
        let mut buffer = MarkBuffer::new();
        collect_marks(b"hello *world*", &mut buffer);

        assert_eq!(buffer.len(), 2);
        assert!(buffer.marks()[0].can_open());
        assert!(buffer.marks()[1].can_close());
    }

    #[test]
    fn test_collect_escape() {
        let mut buffer = MarkBuffer::new();
        collect_marks(b"hello \\* world", &mut buffer);

        assert_eq!(buffer.len(), 1);
        assert_eq!(buffer.marks()[0].ch, b'\\');
    }

    #[test]
    fn test_underscore_intraword() {
        let mut buffer = MarkBuffer::new();
        // Underscores within words should not be openers/closers
        collect_marks(b"foo_bar_baz", &mut buffer);

        // Both underscores are intraword - they shouldn't work
        for mark in buffer.marks() {
            if mark.ch == b'_' {
                // Intraword underscores: right-flanking AND left-flanking
                // So they can't open or close
                assert!(!mark.can_open() || !mark.can_close(),
                    "Intraword underscore should not be both opener and closer");
            }
        }
    }
}
