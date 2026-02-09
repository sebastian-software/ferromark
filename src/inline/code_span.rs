//! Code span resolution.
//!
//! Code spans have highest precedence among inline elements.
//! Backtick runs must match exactly.

use super::marks::{Mark, flags};

/// Resolve code spans in mark buffer.
/// Marks matching backtick openers with their closers.
///
/// Handles backslash escape rules:
/// - A backtick preceded by backslash OUTSIDE the code span is escaped (not a delimiter)
/// - A backslash INSIDE the code span is literal (doesn't escape the closer)
pub fn resolve_code_spans(marks: &mut [Mark], text: &[u8], html_spans: &[(u32, u32)]) {
    let len = marks.len();

    for i in 0..len {
        if marks[i].ch != b'`' || marks[i].is_resolved() {
            continue;
        }

        // Check if this opener is preceded by a backslash (would be escaped)
        let opener_pos = marks[i].pos as usize;
        if pos_in_spans(opener_pos as u32, html_spans) {
            continue;
        }
        if opener_pos > 0 && text[opener_pos - 1] == b'\\' {
            // Check if that backslash is itself escaped (\\`)
            let backslash_escaped = opener_pos > 1 && text[opener_pos - 2] == b'\\';
            if !backslash_escaped {
                // This backtick is escaped, not a valid opener
                continue;
            }
        }

        let opener_len = marks[i].len();
        let opener_end = marks[i].end as usize;

        // Look for matching closer with same backtick count
        for j in (i + 1)..len {
            if marks[j].ch != b'`' || marks[j].is_resolved() {
                continue;
            }

            if marks[j].len() == opener_len {
                let closer_pos = marks[j].pos as usize;
                // Check if closer is preceded by backslash
                if closer_pos > 0 && text[closer_pos - 1] == b'\\' {
                    // Is this backslash inside the code span content?
                    // Content is from opener_end to closer_pos
                    let backslash_pos = closer_pos - 1;
                    if backslash_pos >= opener_end {
                        // Backslash is inside the code span, so it's literal
                        // This is a valid closer
                    } else {
                        // Backslash is outside (before content), so it escapes the closer
                        // Check if the backslash itself is escaped
                        let backslash_escaped =
                            backslash_pos > 0 && text[backslash_pos - 1] == b'\\';
                        if !backslash_escaped {
                            // This closer is escaped, skip it
                            continue;
                        }
                    }
                }

                // Found matching closer
                marks[i].resolve();
                marks[j].resolve();

                // Mark everything in between as being inside code
                for mark in &mut marks[(i + 1)..j] {
                    mark.flags |= flags::IN_CODE;
                }

                break;
            }
        }
    }
}

#[inline]
fn pos_in_spans(pos: u32, spans: &[(u32, u32)]) -> bool {
    spans.iter().any(|&(start, end)| pos >= start && pos < end)
}

/// Extract code span content ranges from resolved marks.
/// Returns iterator of (opener_pos, closer_end, content_start, content_end).
pub fn extract_code_spans(marks: &[Mark]) -> impl Iterator<Item = CodeSpan> + '_ {
    let mut i = 0;
    std::iter::from_fn(move || {
        while i < marks.len() {
            let mark = &marks[i];
            if mark.ch == b'`' && mark.is_resolved() {
                // This is an opener, find the closer
                let opener_end = mark.end;
                for (j, mark_j) in marks.iter().enumerate().skip(i + 1) {
                    if mark_j.ch == b'`' && mark_j.is_resolved() && mark_j.len() == mark.len()
                    {
                        let closer_pos = mark_j.pos;
                        let result = CodeSpan {
                            opener_pos: mark.pos,
                            opener_end,
                            closer_pos,
                            closer_end: mark_j.end,
                        };
                        i = j + 1;
                        return Some(result);
                    }
                }
            }
            i += 1;
        }
        None
    })
}

/// A resolved code span.
#[derive(Debug, Clone, Copy)]
pub struct CodeSpan {
    /// Start of opening backticks.
    pub opener_pos: u32,
    /// End of opening backticks.
    pub opener_end: u32,
    /// Start of closing backticks.
    pub closer_pos: u32,
    /// End of closing backticks.
    pub closer_end: u32,
}

impl CodeSpan {
    /// Get the content range (between the backticks).
    pub fn content_range(&self) -> (u32, u32) {
        (self.opener_end, self.closer_pos)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inline::marks::{MarkBuffer, collect_marks};

    #[test]
    fn test_simple_code_span() {
        let text = b"hello `code` world";
        let mut buffer = MarkBuffer::new();
        collect_marks(text, &mut buffer);
        resolve_code_spans(buffer.marks_mut(), text, &[]);

        let spans: Vec<_> = extract_code_spans(buffer.marks()).collect();
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].content_range(), (7, 11)); // "code"
    }

    #[test]
    fn test_double_backtick() {
        let text = b"``code with ` backtick``";
        let mut buffer = MarkBuffer::new();
        collect_marks(text, &mut buffer);
        resolve_code_spans(buffer.marks_mut(), text, &[]);

        let spans: Vec<_> = extract_code_spans(buffer.marks()).collect();
        assert_eq!(spans.len(), 1);
    }

    #[test]
    fn test_unmatched_backticks() {
        let text = b"hello `code`` world";
        let mut buffer = MarkBuffer::new();
        collect_marks(text, &mut buffer);
        resolve_code_spans(buffer.marks_mut(), text, &[]);

        // Single backtick can't match double backtick
        let spans: Vec<_> = extract_code_spans(buffer.marks()).collect();
        assert_eq!(spans.len(), 0);
    }

    #[test]
    fn test_multiple_code_spans() {
        let text = b"`a` and `b`";
        let mut buffer = MarkBuffer::new();
        collect_marks(text, &mut buffer);
        resolve_code_spans(buffer.marks_mut(), text, &[]);

        let spans: Vec<_> = extract_code_spans(buffer.marks()).collect();
        assert_eq!(spans.len(), 2);
    }

    #[test]
    fn test_emphasis_inside_code() {
        let text = b"`*not emphasis*`";
        let mut buffer = MarkBuffer::new();
        collect_marks(text, &mut buffer);
        resolve_code_spans(buffer.marks_mut(), text, &[]);

        // Asterisks inside code should be marked as IN_CODE
        for mark in buffer.marks() {
            if mark.ch == b'*' {
                assert!(mark.flags & flags::IN_CODE != 0);
            }
        }
    }

    #[test]
    fn test_backslash_inside_code_span() {
        // `foo\`bar` - backslash is inside code span, so backtick at pos 5 is valid closer
        let text = b"`foo\\`bar`";
        let mut buffer = MarkBuffer::new();
        collect_marks(text, &mut buffer);
        resolve_code_spans(buffer.marks_mut(), text, &[]);

        let spans: Vec<_> = extract_code_spans(buffer.marks()).collect();
        assert_eq!(spans.len(), 1);
        // Code span should be `foo\` (positions 0-5)
        assert_eq!(spans[0].opener_pos, 0);
        assert_eq!(spans[0].closer_pos, 5);
    }

    #[test]
    fn test_escaped_backtick_not_opener() {
        // \`not code` - backtick at pos 1 is escaped, not a valid opener
        let text = b"\\`not code`";
        let mut buffer = MarkBuffer::new();
        collect_marks(text, &mut buffer);
        resolve_code_spans(buffer.marks_mut(), text, &[]);

        let spans: Vec<_> = extract_code_spans(buffer.marks()).collect();
        // No code spans should be found
        assert_eq!(spans.len(), 0);
    }
}
