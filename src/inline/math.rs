//! Math span resolution.
//!
//! Follows code span pattern: `$` for inline math, `$$` for display math.
//! Content inside is not parsed for inline markup.

use super::marks::{flags, Mark};

/// A resolved math span.
#[derive(Debug, Clone, Copy)]
pub struct MathSpan {
    /// Start of opening delimiter.
    pub opener_pos: u32,
    /// End of opening delimiter.
    pub opener_end: u32,
    /// Start of closing delimiter.
    pub closer_pos: u32,
    /// End of closing delimiter.
    pub closer_end: u32,
    /// Whether this is display math ($$) vs inline ($).
    pub is_display: bool,
}

impl MathSpan {
    /// Get the content range (between the delimiters).
    pub fn content_range(&self) -> (u32, u32) {
        (self.opener_end, self.closer_pos)
    }
}

/// Resolve math spans in mark buffer.
/// Similar to code span resolution: `$` matches `$`, `$$` matches `$$`.
/// Marks everything between as IN_CODE to prevent further inline parsing.
pub fn resolve_math_spans(marks: &mut [Mark], text: &[u8]) -> Vec<MathSpan> {
    let mut spans = Vec::new();
    let len = marks.len();

    for i in 0..len {
        if marks[i].ch != b'$' || marks[i].is_resolved() || marks[i].flags & flags::IN_CODE != 0 {
            continue;
        }

        // Check if preceded by backslash escape (not a valid opener)
        let opener_pos = marks[i].pos as usize;
        if opener_pos > 0 && text[opener_pos - 1] == b'\\' {
            let backslash_escaped = opener_pos > 1 && text[opener_pos - 2] == b'\\';
            if !backslash_escaped {
                continue;
            }
        }

        let opener_len = marks[i].len();

        // Look for matching closer with same dollar sign count
        for j in (i + 1)..len {
            if marks[j].ch != b'$' || marks[j].is_resolved() {
                continue;
            }

            if marks[j].len() == opener_len {
                let closer_pos = marks[j].pos as usize;
                // Check if closer is preceded by backslash (inside content = literal)
                if closer_pos > 0 && text[closer_pos - 1] == b'\\' {
                    let opener_end = marks[i].end as usize;
                    let backslash_pos = closer_pos - 1;
                    if backslash_pos >= opener_end {
                        // Backslash is inside content, it's literal — valid closer
                    } else {
                        let backslash_escaped =
                            backslash_pos > 0 && text[backslash_pos - 1] == b'\\';
                        if !backslash_escaped {
                            continue;
                        }
                    }
                }

                // Found matching closer
                marks[i].resolve();
                marks[j].resolve();

                // Mark everything in between as IN_CODE
                for k in (i + 1)..j {
                    marks[k].flags |= flags::IN_CODE;
                }

                spans.push(MathSpan {
                    opener_pos: marks[i].pos,
                    opener_end: marks[i].end,
                    closer_pos: marks[j].pos,
                    closer_end: marks[j].end,
                    is_display: opener_len == 2,
                });

                break;
            }
        }
    }

    spans
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inline::marks::{collect_marks, MarkBuffer};

    #[test]
    fn test_simple_inline_math() {
        let text = b"hello $x^2$ world";
        let mut buffer = MarkBuffer::new();
        collect_marks(text, &mut buffer);
        let spans = resolve_math_spans(buffer.marks_mut(), text);
        assert_eq!(spans.len(), 1);
        assert!(!spans[0].is_display);
        let (start, end) = spans[0].content_range();
        assert_eq!(&text[start as usize..end as usize], b"x^2");
    }

    #[test]
    fn test_display_math() {
        let text = b"hello $$E=mc^2$$ world";
        let mut buffer = MarkBuffer::new();
        collect_marks(text, &mut buffer);
        let spans = resolve_math_spans(buffer.marks_mut(), text);
        assert_eq!(spans.len(), 1);
        assert!(spans[0].is_display);
        let (start, end) = spans[0].content_range();
        assert_eq!(&text[start as usize..end as usize], b"E=mc^2");
    }

    #[test]
    fn test_unmatched_dollar() {
        let text = b"hello $ world";
        let mut buffer = MarkBuffer::new();
        collect_marks(text, &mut buffer);
        let spans = resolve_math_spans(buffer.marks_mut(), text);
        assert_eq!(spans.len(), 0);
    }

    #[test]
    fn test_escaped_dollar() {
        let text = b"hello \\$x\\$ world";
        let mut buffer = MarkBuffer::new();
        collect_marks(text, &mut buffer);
        let spans = resolve_math_spans(buffer.marks_mut(), text);
        assert_eq!(spans.len(), 0);
    }

    #[test]
    fn test_multiple_math_spans() {
        let text = b"$a$ and $b$";
        let mut buffer = MarkBuffer::new();
        collect_marks(text, &mut buffer);
        let spans = resolve_math_spans(buffer.marks_mut(), text);
        assert_eq!(spans.len(), 2);
    }
}
