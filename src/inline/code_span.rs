//! Code span resolution.
//!
//! Code spans have highest precedence among inline elements.
//! Backtick runs must match exactly.

use super::marks::{flags, Mark};

/// Resolve code spans in mark buffer.
/// Marks matching backtick openers with their closers.
pub fn resolve_code_spans(marks: &mut [Mark]) {
    let len = marks.len();

    for i in 0..len {
        if marks[i].ch != b'`' || marks[i].is_resolved() {
            continue;
        }

        let opener_len = marks[i].len();

        // Look for matching closer with same backtick count
        for j in (i + 1)..len {
            if marks[j].ch != b'`' || marks[j].is_resolved() {
                continue;
            }

            if marks[j].len() == opener_len {
                // Found matching closer
                marks[i].resolve();
                marks[j].resolve();

                // Mark everything in between as being inside code
                for k in (i + 1)..j {
                    marks[k].flags |= flags::IN_CODE;
                }

                break;
            }
        }
    }
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
                for j in (i + 1)..marks.len() {
                    if marks[j].ch == b'`' && marks[j].is_resolved() && marks[j].len() == mark.len() {
                        let closer_pos = marks[j].pos;
                        let result = CodeSpan {
                            opener_pos: mark.pos,
                            opener_end,
                            closer_pos,
                            closer_end: marks[j].end,
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
    use crate::inline::marks::{collect_marks, MarkBuffer};

    #[test]
    fn test_simple_code_span() {
        let mut buffer = MarkBuffer::new();
        collect_marks(b"hello `code` world", &mut buffer);
        resolve_code_spans(buffer.marks_mut());

        let spans: Vec<_> = extract_code_spans(buffer.marks()).collect();
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].content_range(), (7, 11)); // "code"
    }

    #[test]
    fn test_double_backtick() {
        let mut buffer = MarkBuffer::new();
        collect_marks(b"``code with ` backtick``", &mut buffer);
        resolve_code_spans(buffer.marks_mut());

        let spans: Vec<_> = extract_code_spans(buffer.marks()).collect();
        assert_eq!(spans.len(), 1);
    }

    #[test]
    fn test_unmatched_backticks() {
        let mut buffer = MarkBuffer::new();
        collect_marks(b"hello `code`` world", &mut buffer);
        resolve_code_spans(buffer.marks_mut());

        // Single backtick can't match double backtick
        let spans: Vec<_> = extract_code_spans(buffer.marks()).collect();
        assert_eq!(spans.len(), 0);
    }

    #[test]
    fn test_multiple_code_spans() {
        let mut buffer = MarkBuffer::new();
        collect_marks(b"`a` and `b`", &mut buffer);
        resolve_code_spans(buffer.marks_mut());

        let spans: Vec<_> = extract_code_spans(buffer.marks()).collect();
        assert_eq!(spans.len(), 2);
    }

    #[test]
    fn test_emphasis_inside_code() {
        let mut buffer = MarkBuffer::new();
        collect_marks(b"`*not emphasis*`", &mut buffer);
        resolve_code_spans(buffer.marks_mut());

        // Asterisks inside code should be marked as IN_CODE
        for mark in buffer.marks() {
            if mark.ch == b'*' {
                assert!(mark.flags & flags::IN_CODE != 0);
            }
        }
    }
}
