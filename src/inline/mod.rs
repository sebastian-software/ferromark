//! Inline parser for Markdown.
//!
//! Uses a three-phase approach for efficiency:
//! 1. Mark Collection: Single pass collecting delimiter positions
//! 2. Mark Resolution: Process by precedence (code spans → emphasis)
//! 3. Event Emission: Walk resolved marks and emit events

mod code_span;
mod emphasis;
pub mod event;
pub mod marks;

pub use event::InlineEvent;

use crate::Range;
use code_span::{resolve_code_spans, extract_code_spans};
use emphasis::{resolve_emphasis, EmphasisMatch};
use marks::{collect_marks, flags, MarkBuffer};

/// Inline parser state.
pub struct InlineParser {
    /// Reusable mark buffer.
    mark_buffer: MarkBuffer,
}

impl InlineParser {
    /// Create a new inline parser.
    pub fn new() -> Self {
        Self {
            mark_buffer: MarkBuffer::new(),
        }
    }

    /// Parse inline content and emit events.
    pub fn parse(&mut self, text: &[u8], events: &mut Vec<InlineEvent>) {
        // Phase 1: Collect marks
        collect_marks(text, &mut self.mark_buffer);

        if self.mark_buffer.is_empty() {
            // No special characters, emit as plain text
            if !text.is_empty() {
                events.push(InlineEvent::Text(Range::from_usize(0, text.len())));
            }
            return;
        }

        // Phase 2: Resolve marks by precedence
        // First: code spans (highest precedence)
        resolve_code_spans(self.mark_buffer.marks_mut());

        // Second: emphasis
        let emphasis_matches = resolve_emphasis(self.mark_buffer.marks_mut());

        // Phase 3: Emit events
        self.emit_events(text, &emphasis_matches, events);
    }

    /// Emit events based on resolved marks.
    fn emit_events(&self, text: &[u8], emphasis_matches: &[EmphasisMatch], events: &mut Vec<InlineEvent>) {
        let marks = self.mark_buffer.marks();
        let mut pos = 0u32;
        let text_len = text.len() as u32;

        // Build sorted list of events to emit
        let mut emit_points: Vec<EmitPoint> = Vec::new();

        // Add code span events
        for span in extract_code_spans(marks) {
            emit_points.push(EmitPoint {
                pos: span.opener_pos,
                kind: EmitKind::CodeSpanStart,
                end: span.opener_end,
            });
            emit_points.push(EmitPoint {
                pos: span.closer_pos,
                kind: EmitKind::CodeSpanEnd,
                end: span.closer_end,
            });
            // Code content
            let (content_start, content_end) = span.content_range();
            emit_points.push(EmitPoint {
                pos: content_start,
                kind: EmitKind::CodeContent(content_end),
                end: content_end,
            });
        }

        // Add emphasis events
        for m in emphasis_matches {
            let is_strong = m.count == 2;

            if is_strong {
                emit_points.push(EmitPoint {
                    pos: m.opener_start,
                    kind: EmitKind::StrongStart,
                    end: m.opener_end,
                });
                emit_points.push(EmitPoint {
                    pos: m.closer_start,
                    kind: EmitKind::StrongEnd,
                    end: m.closer_end,
                });
            } else {
                emit_points.push(EmitPoint {
                    pos: m.opener_start,
                    kind: EmitKind::EmphasisStart,
                    end: m.opener_end,
                });
                emit_points.push(EmitPoint {
                    pos: m.closer_start,
                    kind: EmitKind::EmphasisEnd,
                    end: m.closer_end,
                });
            }
        }

        // Add backslash escapes
        for mark in marks {
            if mark.ch == b'\\' && mark.flags & flags::POTENTIAL_OPENER != 0 {
                emit_points.push(EmitPoint {
                    pos: mark.pos,
                    kind: EmitKind::Escape(text[(mark.pos + 1) as usize]),
                    end: mark.end,
                });
            }
        }

        // Sort by position
        emit_points.sort_by_key(|p| (p.pos, matches!(p.kind, EmitKind::CodeSpanEnd | EmitKind::StrongEnd | EmitKind::EmphasisEnd)));

        // Emit events in order
        let mut skip_until = 0u32;

        for point in &emit_points {
            // Emit text before this point
            if point.pos > pos && point.pos > skip_until {
                let text_start = pos.max(skip_until);
                if point.pos > text_start {
                    events.push(InlineEvent::Text(Range::from_usize(
                        text_start as usize,
                        point.pos as usize,
                    )));
                }
            }

            match point.kind {
                EmitKind::CodeSpanStart => {
                    // Skip the opening backticks
                    pos = point.end;
                }
                EmitKind::CodeSpanEnd => {
                    // Skip the closing backticks
                    skip_until = point.end;
                }
                EmitKind::CodeContent(end) => {
                    // Emit code content (strip leading/trailing space if single space)
                    let mut start = point.pos as usize;
                    let mut end = end as usize;

                    // CommonMark: strip one leading and one trailing space if present
                    // and content doesn't consist entirely of spaces
                    if end > start + 1 {
                        let content = &text[start..end];
                        if content.starts_with(b" ") && content.ends_with(b" ")
                            && content.iter().any(|&b| b != b' ')
                        {
                            start += 1;
                            end -= 1;
                        }
                    }

                    // Normalize line endings to spaces
                    events.push(InlineEvent::Code(Range::from_usize(start, end)));
                    skip_until = end as u32;
                }
                EmitKind::EmphasisStart => {
                    events.push(InlineEvent::EmphasisStart);
                    pos = point.end;
                    skip_until = point.end;
                }
                EmitKind::EmphasisEnd => {
                    events.push(InlineEvent::EmphasisEnd);
                    skip_until = point.end;
                }
                EmitKind::StrongStart => {
                    events.push(InlineEvent::StrongStart);
                    pos = point.end;
                    skip_until = point.end;
                }
                EmitKind::StrongEnd => {
                    events.push(InlineEvent::StrongEnd);
                    skip_until = point.end;
                }
                EmitKind::Escape(ch) => {
                    events.push(InlineEvent::EscapedChar(ch));
                    skip_until = point.end;
                }
            }

            pos = pos.max(point.end);
        }

        // Emit remaining text
        if pos < text_len {
            let start = pos.max(skip_until);
            if start < text_len {
                events.push(InlineEvent::Text(Range::from_usize(
                    start as usize,
                    text_len as usize,
                )));
            }
        }
    }
}

impl Default for InlineParser {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy)]
struct EmitPoint {
    pos: u32,
    kind: EmitKind,
    end: u32,
}

#[derive(Debug, Clone, Copy)]
enum EmitKind {
    CodeSpanStart,
    CodeSpanEnd,
    CodeContent(u32), // end position
    EmphasisStart,
    EmphasisEnd,
    StrongStart,
    StrongEnd,
    Escape(u8),
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_inline(text: &str) -> Vec<InlineEvent> {
        let mut parser = InlineParser::new();
        let mut events = Vec::new();
        parser.parse(text.as_bytes(), &mut events);
        events
    }

    fn get_text<'a>(input: &'a str, event: &InlineEvent) -> &'a str {
        match event {
            InlineEvent::Text(range) | InlineEvent::Code(range) => {
                std::str::from_utf8(range.slice(input.as_bytes())).unwrap()
            }
            _ => panic!("Expected text event"),
        }
    }

    #[test]
    fn test_plain_text() {
        let events = parse_inline("hello world");
        assert_eq!(events.len(), 1);
        assert!(matches!(&events[0], InlineEvent::Text(_)));
    }

    #[test]
    fn test_code_span() {
        let input = "hello `code` world";
        let events = parse_inline(input);

        // Should have: Text("hello "), Code("code"), Text(" world")
        assert!(events.iter().any(|e| matches!(e, InlineEvent::Code(_))));

        for event in &events {
            if let InlineEvent::Code(range) = event {
                assert_eq!(range.slice(input.as_bytes()), b"code");
            }
        }
    }

    #[test]
    fn test_emphasis() {
        let events = parse_inline("hello *world*");

        assert!(events.iter().any(|e| matches!(e, InlineEvent::EmphasisStart)));
        assert!(events.iter().any(|e| matches!(e, InlineEvent::EmphasisEnd)));
    }

    #[test]
    fn test_strong() {
        let events = parse_inline("hello **world**");

        assert!(events.iter().any(|e| matches!(e, InlineEvent::StrongStart)));
        assert!(events.iter().any(|e| matches!(e, InlineEvent::StrongEnd)));
    }

    #[test]
    fn test_backslash_escape() {
        let events = parse_inline("hello \\*world\\*");

        // Should have escaped characters instead of emphasis
        let escaped_count = events.iter().filter(|e| matches!(e, InlineEvent::EscapedChar(_))).count();
        assert_eq!(escaped_count, 2);
    }

    #[test]
    fn test_code_span_with_backticks() {
        let input = "`` `code` ``";
        let events = parse_inline(input);

        // Double backticks should contain single backticks
        for event in &events {
            if let InlineEvent::Code(range) = event {
                let content = std::str::from_utf8(range.slice(input.as_bytes())).unwrap();
                assert!(content.contains('`'));
            }
        }
    }

    #[test]
    fn test_emphasis_not_in_code() {
        let events = parse_inline("`*not emphasis*`");

        // Should not have emphasis events
        assert!(!events.iter().any(|e| matches!(e, InlineEvent::EmphasisStart)));
    }
}
