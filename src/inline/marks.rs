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

                // Determine opener/closer status based on surrounding Unicode chars
                let flags = compute_emphasis_flags_with_context(ch, text, start, pos);

                if flags != 0 {
                    buffer.push(Mark::new(start as u32, pos as u32, ch, flags));
                }
            }

            b'\\' => {
                // Backslash escape - mark position and skip escaped char
                if pos + 1 < len {
                    let next = text[pos + 1];
                    if is_escapable(next) || next == b'\n' {
                        // Regular escape or hard line break (backslash before newline)
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
                } else {
                    pos += 1;
                }
            }

            b'\n' => {
                // Check for hard break (two or more spaces before newline)
                let has_hard_break = pos >= 2
                    && text[pos - 1] == b' '
                    && text[pos - 2] == b' ';

                if has_hard_break {
                    // Find the start of ALL trailing spaces, not just 2
                    let mut space_start = pos - 2;
                    while space_start > 0 && text[space_start - 1] == b' ' {
                        space_start -= 1;
                    }
                    buffer.push(Mark::new(
                        space_start as u32,
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
///
/// This version takes the full text and positions to properly handle Unicode.
fn compute_emphasis_flags_with_context(ch: u8, text: &[u8], start: usize, end: usize) -> u8 {
    let before_space = is_preceded_by_whitespace(text, start);
    let after_space = is_followed_by_whitespace(text, end);
    let before_punct = is_preceded_by_punctuation(text, start);
    let after_punct = is_followed_by_punctuation(text, end);

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

/// Check if position is preceded by Unicode whitespace (or start of text).
#[inline]
fn is_preceded_by_whitespace(text: &[u8], pos: usize) -> bool {
    if pos == 0 {
        return true; // Start of text counts as whitespace
    }

    // Check for ASCII whitespace first (most common case)
    let prev = text[pos - 1];
    if prev == b' ' || prev == b'\t' || prev == b'\n' || prev == b'\r' {
        return true;
    }

    // Check for multi-byte UTF-8 whitespace
    // Non-breaking space U+00A0: 0xC2 0xA0
    if pos >= 2 && text[pos - 2] == 0xC2 && text[pos - 1] == 0xA0 {
        return true;
    }

    // Other common Unicode whitespace (3-byte sequences starting with 0xE2)
    // U+2000-U+200A (various spaces), U+202F (narrow no-break space),
    // U+205F (medium mathematical space), U+3000 (ideographic space)
    if pos >= 3 && text[pos - 3] == 0xE2 {
        let b2 = text[pos - 2];
        let b3 = text[pos - 1];
        // U+2000-U+200A: E2 80 80-8A
        if b2 == 0x80 && (0x80..=0x8A).contains(&b3) {
            return true;
        }
        // U+202F: E2 80 AF
        if b2 == 0x80 && b3 == 0xAF {
            return true;
        }
        // U+205F: E2 81 9F
        if b2 == 0x81 && b3 == 0x9F {
            return true;
        }
    }
    // U+3000 (ideographic space): E3 80 80
    if pos >= 3 && text[pos - 3] == 0xE3 && text[pos - 2] == 0x80 && text[pos - 1] == 0x80 {
        return true;
    }

    false
}

/// Check if position is followed by Unicode whitespace (or end of text).
#[inline]
fn is_followed_by_whitespace(text: &[u8], pos: usize) -> bool {
    if pos >= text.len() {
        return true; // End of text counts as whitespace
    }

    let next = text[pos];

    // ASCII whitespace (most common case)
    if next == b' ' || next == b'\t' || next == b'\n' || next == b'\r' {
        return true;
    }

    // Non-breaking space U+00A0: 0xC2 0xA0
    if next == 0xC2 && pos + 1 < text.len() && text[pos + 1] == 0xA0 {
        return true;
    }

    // Other Unicode whitespace (3-byte sequences)
    if next == 0xE2 && pos + 2 < text.len() {
        let b2 = text[pos + 1];
        let b3 = text[pos + 2];
        // U+2000-U+200A
        if b2 == 0x80 && (0x80..=0x8A).contains(&b3) {
            return true;
        }
        // U+202F
        if b2 == 0x80 && b3 == 0xAF {
            return true;
        }
        // U+205F
        if b2 == 0x81 && b3 == 0x9F {
            return true;
        }
    }
    // U+3000
    if next == 0xE3 && pos + 2 < text.len() && text[pos + 1] == 0x80 && text[pos + 2] == 0x80 {
        return true;
    }

    false
}

/// Check if position is preceded by Unicode punctuation.
/// For CommonMark purposes, this includes currency and other symbols.
#[inline]
fn is_preceded_by_punctuation(text: &[u8], pos: usize) -> bool {
    if pos == 0 {
        return false;
    }

    let prev = text[pos - 1];

    // ASCII punctuation (most common)
    if is_ascii_punctuation(prev) {
        return true;
    }

    // Check for multi-byte UTF-8 punctuation/symbols
    // We need to look backwards to find the start of the character
    if prev >= 0x80 {
        // It's a continuation byte or start of multi-byte char

        // 2-byte sequences (Latin-1 Supplement: U+00A1-U+00BF)
        // Includes: ¡¢£¤¥¦§¨©ª«¬­®¯°±²³´µ¶·¸¹º»¼½¾¿
        // These are treated as punctuation/symbols for flanking rules
        if pos >= 2 && text[pos - 2] == 0xC2 {
            let cp_low = text[pos - 1];
            if (0xA1..=0xBF).contains(&cp_low) {
                return true;
            }
        }
        if pos >= 2 && text[pos - 2] == 0xC3 {
            // U+00C0-U+00FF - mostly letters, but check for ×(D7) and ÷(F7)
            let cp_low = text[pos - 1];
            if cp_low == 0x97 || cp_low == 0xB7 { // × and ÷
                return true;
            }
        }

        // 3-byte sequences
        if pos >= 3 && text[pos - 3] == 0xE2 {
            let b2 = text[pos - 2];
            let b3 = text[pos - 1];
            // U+2010-U+2027 (dashes, quotes, etc.)
            if b2 == 0x80 && (0x90..=0xA7).contains(&b3) {
                return true;
            }
            // U+2030-U+205E (per mille, prime, etc.)
            if b2 == 0x80 && (0xB0..=0xBF).contains(&b3) {
                return true;
            }
            if b2 == 0x81 && (0x80..=0x9E).contains(&b3) {
                return true;
            }
            // U+20A0-U+20CF (Currency Symbols) - includes €
            // € (U+20AC) = E2 82 AC
            if b2 == 0x82 && (0xA0..=0xCF).contains(&b3) {
                return true;
            }
        }
    }

    false
}

/// Check if position is followed by Unicode punctuation.
/// For CommonMark purposes, this includes currency and other symbols.
#[inline]
fn is_followed_by_punctuation(text: &[u8], pos: usize) -> bool {
    if pos >= text.len() {
        return false;
    }

    let next = text[pos];

    // ASCII punctuation (most common)
    if is_ascii_punctuation(next) {
        return true;
    }

    // Check for multi-byte UTF-8 punctuation/symbols
    if next >= 0xC0 {
        // Start of multi-byte sequence

        // 2-byte sequences (Latin-1 Supplement: U+00A1-U+00BF)
        if next == 0xC2 && pos + 1 < text.len() {
            let cp_low = text[pos + 1];
            // Includes currency symbols like £ (A3), ¥ (A5), etc.
            if (0xA1..=0xBF).contains(&cp_low) {
                return true;
            }
        }
        if next == 0xC3 && pos + 1 < text.len() {
            let cp_low = text[pos + 1];
            if cp_low == 0x97 || cp_low == 0xB7 { // × and ÷
                return true;
            }
        }

        // 3-byte sequences
        if next == 0xE2 && pos + 2 < text.len() {
            let b2 = text[pos + 1];
            let b3 = text[pos + 2];
            // U+2010-U+2027 (dashes, quotes, etc.)
            if b2 == 0x80 && (0x90..=0xA7).contains(&b3) {
                return true;
            }
            // U+2030-U+205E (per mille, prime, etc.)
            if b2 == 0x80 && (0xB0..=0xBF).contains(&b3) {
                return true;
            }
            if b2 == 0x81 && (0x80..=0x9E).contains(&b3) {
                return true;
            }
            // U+20A0-U+20CF (Currency Symbols) - includes €
            // € (U+20AC) = E2 82 AC
            if b2 == 0x82 && (0xA0..=0xCF).contains(&b3) {
                return true;
            }
        }
    }

    false
}

/// Check if a byte is ASCII punctuation.
#[inline]
fn is_ascii_punctuation(b: u8) -> bool {
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

    #[test]
    fn test_unicode_whitespace_nbsp() {
        // Non-breaking space U+00A0 should count as whitespace
        // "*\u{a0}a\u{a0}*" should NOT be emphasis because it's surrounded by whitespace
        let text = "*\u{a0}a\u{a0}*".as_bytes();
        let mut buffer = MarkBuffer::new();
        collect_marks(text, &mut buffer);

        // Both asterisks should be considered adjacent to whitespace
        // First * is followed by NBSP (whitespace) - not left-flanking, so can't open
        // Last * is preceded by NBSP (whitespace) - not right-flanking, so can't close
        // Since neither can open nor close, they won't be added as marks at all
        // This is correct - they shouldn't form emphasis
        assert_eq!(buffer.len(), 0, "No marks should be collected when asterisks are surrounded by whitespace");
    }

    #[test]
    fn test_unicode_punctuation_precedes() {
        // Example 352: a*"foo"*
        // The first * is preceded by 'a' (letter) and followed by '"' (punctuation)
        // Left-flanking: not followed by space AND (not followed by punct OR preceded by space/punct)
        // Since it's followed by punct and NOT preceded by space/punct, it's NOT left-flanking
        let text = b"a*\"foo\"*";
        let mut buffer = MarkBuffer::new();
        collect_marks(text, &mut buffer);

        // Should have 2 marks for the asterisks
        assert_eq!(buffer.len(), 2);
        let first = &buffer.marks()[0];
        let last = &buffer.marks()[1];

        // First *: preceded by 'a' (not punct, not space), followed by '"' (punct)
        // left_flanking = !after_space && (!after_punct || before_space || before_punct)
        //              = true && (!true || false || false)
        //              = true && false = false
        // So first * is NOT left-flanking, can't open
        assert!(!first.can_open(), "First * should not open: preceded by letter, followed by punct");

        // Last *: preceded by '"' (punct), followed by end (space)
        // right_flanking = !before_space && (!before_punct || after_space || after_punct)
        //               = true && (!true || true || false)
        //               = true && true = true
        // But wait, it's also:
        // left_flanking = !after_space && (!after_punct || before_space || before_punct)
        //              = false (followed by end which is space)
        // So it IS right-flanking but NOT left-flanking
        assert!(last.can_close(), "Last * should close: preceded by punct, followed by end");
    }
}
