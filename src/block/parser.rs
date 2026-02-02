//! Block parser implementation.

use crate::cursor::Cursor;
use crate::Range;

use super::event::BlockEvent;

/// Block parser state.
pub struct BlockParser<'a> {
    /// Input bytes.
    input: &'a [u8],
    /// Current cursor position.
    cursor: Cursor<'a>,
    /// Whether we're currently in a paragraph.
    in_paragraph: bool,
    /// Accumulated paragraph text ranges.
    paragraph_lines: Vec<Range>,
}

impl<'a> BlockParser<'a> {
    /// Create a new block parser.
    pub fn new(input: &'a [u8]) -> Self {
        Self {
            input,
            cursor: Cursor::new(input),
            in_paragraph: false,
            paragraph_lines: Vec::new(),
        }
    }

    /// Parse all blocks and collect events.
    pub fn parse(&mut self, events: &mut Vec<BlockEvent>) {
        while !self.cursor.is_eof() {
            self.parse_line(events);
        }

        // Close any open paragraph at end of input
        self.close_paragraph(events);
    }

    /// Parse a single line.
    fn parse_line(&mut self, events: &mut Vec<BlockEvent>) {
        let line_start = self.cursor.offset();

        // Skip leading spaces (up to 3 for most block elements)
        let indent = self.cursor.skip_spaces();

        // Check for blank line
        if self.cursor.is_eof() || self.cursor.at(b'\n') {
            if !self.cursor.is_eof() {
                self.cursor.bump(); // consume newline
            }
            self.close_paragraph(events);
            return;
        }

        // Try to parse block-level constructs (only if indent < 4)
        if indent < 4 {
            // Check for thematic break
            if self.try_thematic_break(events) {
                return;
            }

            // Check for ATX heading
            if self.try_atx_heading(events) {
                return;
            }
        }

        // Otherwise, it's paragraph content
        self.parse_paragraph_line(line_start, events);
    }

    /// Try to parse a thematic break.
    /// Returns true if successful.
    fn try_thematic_break(&mut self, events: &mut Vec<BlockEvent>) -> bool {
        let start_pos = self.cursor.offset();

        // Must start with -, *, or _
        let marker = match self.cursor.peek() {
            Some(b'-') | Some(b'*') | Some(b'_') => self.cursor.peek().unwrap(),
            _ => return false,
        };

        // Count markers and spaces
        let mut count = 0;
        let mut temp_cursor = self.cursor;

        while let Some(b) = temp_cursor.peek() {
            if b == marker {
                count += 1;
                temp_cursor.bump();
            } else if b == b' ' || b == b'\t' {
                temp_cursor.bump();
            } else if b == b'\n' {
                break;
            } else {
                // Invalid character
                return false;
            }
        }

        // Need at least 3 markers
        if count < 3 {
            return false;
        }

        // Consume the line
        self.cursor = temp_cursor;
        if !self.cursor.is_eof() && self.cursor.at(b'\n') {
            self.cursor.bump();
        }

        // Close any open paragraph
        self.close_paragraph(events);

        events.push(BlockEvent::ThematicBreak);
        true
    }

    /// Try to parse an ATX heading.
    /// Returns true if successful.
    fn try_atx_heading(&mut self, events: &mut Vec<BlockEvent>) -> bool {
        // Must start with #
        if !self.cursor.at(b'#') {
            return false;
        }

        let start_pos = self.cursor.offset();

        // Count # characters (1-6)
        let mut level = 0u8;
        let mut temp_cursor = self.cursor;

        while temp_cursor.at(b'#') && level < 7 {
            level += 1;
            temp_cursor.bump();
        }

        // Level must be 1-6
        if level == 0 || level > 6 {
            return false;
        }

        // Must be followed by space, tab, or end of line
        if !temp_cursor.is_eof()
            && !temp_cursor.at(b' ')
            && !temp_cursor.at(b'\t')
            && !temp_cursor.at(b'\n')
        {
            return false;
        }

        // Skip spaces after #
        temp_cursor.skip_whitespace();

        // Find content start and end
        let content_start = temp_cursor.offset();

        // Find end of line
        let line_end = match temp_cursor.find_newline() {
            Some(pos) => content_start + pos,
            None => content_start + temp_cursor.remaining(),
        };

        // Trim trailing # and spaces from content
        let content_end = self.trim_heading_end(content_start, line_end);

        // Update cursor to end of line
        self.cursor = Cursor::new_at(self.input, line_end);
        if !self.cursor.is_eof() && self.cursor.at(b'\n') {
            self.cursor.bump();
        }

        // Close any open paragraph
        self.close_paragraph(events);

        // Emit heading events
        events.push(BlockEvent::HeadingStart { level });

        if content_end > content_start {
            events.push(BlockEvent::Text(Range::from_usize(content_start, content_end)));
        }

        events.push(BlockEvent::HeadingEnd { level });

        true
    }

    /// Trim trailing # characters and spaces from heading content.
    fn trim_heading_end(&self, start: usize, end: usize) -> usize {
        if start >= end {
            return start;
        }

        let mut pos = end;

        // Trim trailing spaces
        while pos > start && (self.input[pos - 1] == b' ' || self.input[pos - 1] == b'\t') {
            pos -= 1;
        }

        // Check for closing # sequence
        let after_hashes = pos;
        while pos > start && self.input[pos - 1] == b'#' {
            pos -= 1;
        }

        // Closing hashes must be preceded by space (or be at start)
        if pos < after_hashes {
            if pos == start || self.input[pos - 1] == b' ' || self.input[pos - 1] == b'\t' {
                // Valid closing hashes, trim them and any preceding space
                while pos > start && (self.input[pos - 1] == b' ' || self.input[pos - 1] == b'\t') {
                    pos -= 1;
                }
            } else {
                // Hashes not preceded by space, keep them
                pos = after_hashes;
            }
        }

        pos
    }

    /// Parse a paragraph line.
    fn parse_paragraph_line(&mut self, line_start: usize, events: &mut Vec<BlockEvent>) {
        // Find end of line
        let content_start = self.cursor.offset();
        let line_end = match self.cursor.find_newline() {
            Some(pos) => content_start + pos,
            None => content_start + self.cursor.remaining(),
        };

        // Move cursor to next line
        self.cursor = Cursor::new_at(self.input, line_end);
        if !self.cursor.is_eof() && self.cursor.at(b'\n') {
            self.cursor.bump();
        }

        // If we weren't in a paragraph, we are now
        if !self.in_paragraph {
            self.in_paragraph = true;
        }

        // Add this line to paragraph content
        // We include from original line_start to capture any leading spaces we skipped
        // Actually, use content_start which is after indent
        if line_end > content_start {
            self.paragraph_lines.push(Range::from_usize(content_start, line_end));
        }
    }

    /// Close an open paragraph.
    fn close_paragraph(&mut self, events: &mut Vec<BlockEvent>) {
        if !self.in_paragraph {
            return;
        }

        self.in_paragraph = false;

        if self.paragraph_lines.is_empty() {
            return;
        }

        events.push(BlockEvent::ParagraphStart);

        // Emit text ranges for each line
        // For now, emit them separately; later we might merge
        for (i, range) in self.paragraph_lines.drain(..).enumerate() {
            if i > 0 {
                // Add soft break between lines (as a single space in text)
                // For now, just emit each range
            }
            events.push(BlockEvent::Text(range));
        }

        events.push(BlockEvent::ParagraphEnd);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(input: &str) -> Vec<BlockEvent> {
        let mut parser = BlockParser::new(input.as_bytes());
        let mut events = Vec::new();
        parser.parse(&mut events);
        events
    }

    fn get_text<'a>(input: &'a str, event: &BlockEvent) -> &'a str {
        match event {
            BlockEvent::Text(range) => {
                std::str::from_utf8(range.slice(input.as_bytes())).unwrap()
            }
            _ => panic!("Expected Text event"),
        }
    }

    #[test]
    fn test_empty_input() {
        let events = parse("");
        assert!(events.is_empty());
    }

    #[test]
    fn test_blank_lines() {
        let events = parse("\n\n\n");
        assert!(events.is_empty());
    }

    #[test]
    fn test_simple_paragraph() {
        let input = "Hello, world!";
        let events = parse(input);

        assert_eq!(events.len(), 3);
        assert_eq!(events[0], BlockEvent::ParagraphStart);
        assert_eq!(get_text(input, &events[1]), "Hello, world!");
        assert_eq!(events[2], BlockEvent::ParagraphEnd);
    }

    #[test]
    fn test_multiline_paragraph() {
        let input = "Line 1\nLine 2\nLine 3";
        let events = parse(input);

        assert_eq!(events.len(), 5);
        assert_eq!(events[0], BlockEvent::ParagraphStart);
        assert_eq!(get_text(input, &events[1]), "Line 1");
        assert_eq!(get_text(input, &events[2]), "Line 2");
        assert_eq!(get_text(input, &events[3]), "Line 3");
        assert_eq!(events[4], BlockEvent::ParagraphEnd);
    }

    #[test]
    fn test_paragraphs_separated_by_blank() {
        let input = "Para 1\n\nPara 2";
        let events = parse(input);

        assert_eq!(events.len(), 6);
        assert_eq!(events[0], BlockEvent::ParagraphStart);
        assert_eq!(get_text(input, &events[1]), "Para 1");
        assert_eq!(events[2], BlockEvent::ParagraphEnd);
        assert_eq!(events[3], BlockEvent::ParagraphStart);
        assert_eq!(get_text(input, &events[4]), "Para 2");
        assert_eq!(events[5], BlockEvent::ParagraphEnd);
    }

    #[test]
    fn test_thematic_break_dashes() {
        let events = parse("---");
        assert_eq!(events, vec![BlockEvent::ThematicBreak]);
    }

    #[test]
    fn test_thematic_break_asterisks() {
        let events = parse("***");
        assert_eq!(events, vec![BlockEvent::ThematicBreak]);
    }

    #[test]
    fn test_thematic_break_underscores() {
        let events = parse("___");
        assert_eq!(events, vec![BlockEvent::ThematicBreak]);
    }

    #[test]
    fn test_thematic_break_with_spaces() {
        let events = parse("- - -");
        assert_eq!(events, vec![BlockEvent::ThematicBreak]);
    }

    #[test]
    fn test_thematic_break_many() {
        let events = parse("----------");
        assert_eq!(events, vec![BlockEvent::ThematicBreak]);
    }

    #[test]
    fn test_thematic_break_too_few() {
        let input = "--";
        let events = parse(input);
        // Should be a paragraph, not a thematic break
        assert_eq!(events[0], BlockEvent::ParagraphStart);
    }

    #[test]
    fn test_thematic_break_mixed_invalid() {
        let input = "-*-";
        let events = parse(input);
        // Mixed markers = paragraph
        assert_eq!(events[0], BlockEvent::ParagraphStart);
    }

    #[test]
    fn test_atx_heading_h1() {
        let input = "# Heading";
        let events = parse(input);

        assert_eq!(events.len(), 3);
        assert_eq!(events[0], BlockEvent::HeadingStart { level: 1 });
        assert_eq!(get_text(input, &events[1]), "Heading");
        assert_eq!(events[2], BlockEvent::HeadingEnd { level: 1 });
    }

    #[test]
    fn test_atx_heading_h2() {
        let input = "## Heading";
        let events = parse(input);

        assert_eq!(events[0], BlockEvent::HeadingStart { level: 2 });
    }

    #[test]
    fn test_atx_heading_h6() {
        let input = "###### Heading";
        let events = parse(input);

        assert_eq!(events[0], BlockEvent::HeadingStart { level: 6 });
    }

    #[test]
    fn test_atx_heading_h7_invalid() {
        let input = "####### Heading";
        let events = parse(input);

        // 7 # is not a valid heading
        assert_eq!(events[0], BlockEvent::ParagraphStart);
    }

    #[test]
    fn test_atx_heading_empty() {
        let input = "#";
        let events = parse(input);

        assert_eq!(events.len(), 2);
        assert_eq!(events[0], BlockEvent::HeadingStart { level: 1 });
        assert_eq!(events[1], BlockEvent::HeadingEnd { level: 1 });
    }

    #[test]
    fn test_atx_heading_closing_hashes() {
        let input = "# Heading #";
        let events = parse(input);

        assert_eq!(events.len(), 3);
        assert_eq!(get_text(input, &events[1]), "Heading");
    }

    #[test]
    fn test_atx_heading_closing_hashes_multiple() {
        let input = "## Heading ##";
        let events = parse(input);

        assert_eq!(get_text(input, &events[1]), "Heading");
    }

    #[test]
    fn test_atx_heading_closing_hashes_with_space() {
        let input = "# Heading #  ";
        let events = parse(input);

        assert_eq!(get_text(input, &events[1]), "Heading");
    }

    #[test]
    fn test_atx_heading_no_space_after_hashes() {
        let input = "#Heading";
        let events = parse(input);

        // No space after # = not a heading
        assert_eq!(events[0], BlockEvent::ParagraphStart);
    }

    #[test]
    fn test_thematic_break_closes_paragraph() {
        let input = "Text\n---";
        let events = parse(input);

        assert_eq!(events.len(), 4);
        assert_eq!(events[0], BlockEvent::ParagraphStart);
        assert_eq!(events[2], BlockEvent::ParagraphEnd);
        assert_eq!(events[3], BlockEvent::ThematicBreak);
    }

    #[test]
    fn test_heading_closes_paragraph() {
        let input = "Text\n# Heading";
        let events = parse(input);

        assert_eq!(events[0], BlockEvent::ParagraphStart);
        assert_eq!(events[2], BlockEvent::ParagraphEnd);
        assert_eq!(events[3], BlockEvent::HeadingStart { level: 1 });
    }

    #[test]
    fn test_indented_content() {
        let input = "   Text with indent";
        let events = parse(input);

        // Up to 3 spaces is allowed for normal blocks
        assert_eq!(events[0], BlockEvent::ParagraphStart);
        assert_eq!(get_text(input, &events[1]), "Text with indent");
    }

    #[test]
    fn test_thematic_break_with_leading_spaces() {
        let events = parse("   ---");
        assert_eq!(events, vec![BlockEvent::ThematicBreak]);
    }
}
