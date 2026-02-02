//! Inline parser for Markdown.
//!
//! Uses a three-phase approach for efficiency:
//! 1. Mark Collection: Single pass collecting delimiter positions
//! 2. Mark Resolution: Process by precedence (code spans → links → emphasis)
//! 3. Event Emission: Walk resolved marks and emit events

mod code_span;
mod emphasis;
pub mod event;
mod links;
pub mod marks;

pub use event::InlineEvent;

use crate::Range;
use code_span::{resolve_code_spans, extract_code_spans};
use emphasis::{resolve_emphasis, EmphasisMatch};
use links::{find_autolinks, resolve_links, Autolink, Link};
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

        // Second: autolinks
        let autolinks = find_autolinks(text);

        // Third: links and images
        let (open_brackets, close_brackets) = self.collect_brackets();
        let resolved_links = resolve_links(text, &open_brackets, &close_brackets);

        // Fourth: emphasis (lowest precedence)
        let emphasis_matches = resolve_emphasis(self.mark_buffer.marks_mut());

        // Phase 3: Emit events
        self.emit_events(text, &emphasis_matches, &resolved_links, &autolinks, events);
    }

    /// Collect bracket positions for link parsing.
    fn collect_brackets(&self) -> (Vec<(u32, bool)>, Vec<u32>) {
        let marks = self.mark_buffer.marks();
        let mut open_brackets = Vec::new();
        let mut close_brackets = Vec::new();

        for mark in marks {
            // Skip brackets inside code spans
            if mark.flags & flags::IN_CODE != 0 && mark.ch != b'[' {
                continue;
            }

            match mark.ch {
                b'[' => {
                    // IN_CODE flag repurposed for "is_image" for brackets
                    let is_image = mark.flags & flags::IN_CODE != 0;
                    open_brackets.push((mark.pos, is_image));
                }
                b']' => {
                    close_brackets.push(mark.pos);
                }
                _ => {}
            }
        }

        (open_brackets, close_brackets)
    }

    /// Emit events based on resolved marks.
    fn emit_events(
        &self,
        text: &[u8],
        emphasis_matches: &[EmphasisMatch],
        resolved_links: &[Link],
        autolinks: &[Autolink],
        events: &mut Vec<InlineEvent>,
    ) {
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

        // Add link events
        for link in resolved_links {
            if link.is_image {
                emit_points.push(EmitPoint {
                    pos: link.start,
                    kind: EmitKind::ImageStart {
                        url_start: link.url_start,
                        url_end: link.url_end,
                        title_start: link.title_start,
                        title_end: link.title_end,
                    },
                    end: link.start + 2, // ![
                });
                emit_points.push(EmitPoint {
                    pos: link.text_end,
                    kind: EmitKind::ImageEnd,
                    end: link.end,
                });
            } else {
                emit_points.push(EmitPoint {
                    pos: link.start,
                    kind: EmitKind::LinkStart {
                        url_start: link.url_start,
                        url_end: link.url_end,
                        title_start: link.title_start,
                        title_end: link.title_end,
                    },
                    end: link.start + 1, // [
                });
                emit_points.push(EmitPoint {
                    pos: link.text_end,
                    kind: EmitKind::LinkEnd,
                    end: link.end,
                });
            }
        }

        // Add autolink events
        for autolink in autolinks {
            if autolink.is_email {
                emit_points.push(EmitPoint {
                    pos: autolink.start,
                    kind: EmitKind::AutolinkEmail {
                        content_start: autolink.content_start,
                        content_end: autolink.content_end,
                    },
                    end: autolink.end,
                });
            } else {
                emit_points.push(EmitPoint {
                    pos: autolink.start,
                    kind: EmitKind::AutolinkUrl {
                        content_start: autolink.content_start,
                        content_end: autolink.content_end,
                    },
                    end: autolink.end,
                });
            }
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

        // Add backslash escapes and hard breaks
        for mark in marks {
            if mark.ch == b'\\' && mark.flags & flags::POTENTIAL_OPENER != 0 {
                let escaped_char = text[(mark.pos + 1) as usize];
                if escaped_char == b'\n' {
                    // Backslash before newline is a hard break
                    emit_points.push(EmitPoint {
                        pos: mark.pos,
                        kind: EmitKind::HardBreak,
                        end: mark.end,
                    });
                } else {
                    emit_points.push(EmitPoint {
                        pos: mark.pos,
                        kind: EmitKind::Escape(escaped_char),
                        end: mark.end,
                    });
                }
            } else if mark.ch == b'\n' && mark.flags & flags::POTENTIAL_OPENER != 0 {
                // Two spaces before newline is a hard break
                emit_points.push(EmitPoint {
                    pos: mark.pos,
                    kind: EmitKind::HardBreak,
                    end: mark.end,
                });
            }
        }

        // Sort by position (end events come after start events at same position)
        emit_points.sort_by_key(|p| (p.pos, matches!(p.kind,
            EmitKind::CodeSpanEnd | EmitKind::StrongEnd | EmitKind::EmphasisEnd |
            EmitKind::LinkEnd | EmitKind::ImageEnd
        )));

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
                EmitKind::HardBreak => {
                    events.push(InlineEvent::HardBreak);
                    skip_until = point.end;
                }
                EmitKind::LinkStart { url_start, url_end, title_start, title_end } => {
                    events.push(InlineEvent::LinkStart {
                        url: Range::from_usize(url_start as usize, url_end as usize),
                        title: title_start.map(|s| Range::from_usize(s as usize, title_end.unwrap() as usize)),
                    });
                    pos = point.end;
                    skip_until = point.end;
                }
                EmitKind::LinkEnd => {
                    events.push(InlineEvent::LinkEnd);
                    skip_until = point.end;
                }
                EmitKind::ImageStart { url_start, url_end, title_start, title_end } => {
                    events.push(InlineEvent::ImageStart {
                        url: Range::from_usize(url_start as usize, url_end as usize),
                        title: title_start.map(|s| Range::from_usize(s as usize, title_end.unwrap() as usize)),
                    });
                    pos = point.end;
                    skip_until = point.end;
                }
                EmitKind::ImageEnd => {
                    events.push(InlineEvent::ImageEnd);
                    skip_until = point.end;
                }
                EmitKind::AutolinkUrl { content_start, content_end } => {
                    events.push(InlineEvent::Autolink {
                        url: Range::from_usize(content_start as usize, content_end as usize),
                        is_email: false,
                    });
                    skip_until = point.end;
                }
                EmitKind::AutolinkEmail { content_start, content_end } => {
                    events.push(InlineEvent::Autolink {
                        url: Range::from_usize(content_start as usize, content_end as usize),
                        is_email: true,
                    });
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
    HardBreak,
    LinkStart { url_start: u32, url_end: u32, title_start: Option<u32>, title_end: Option<u32> },
    LinkEnd,
    ImageStart { url_start: u32, url_end: u32, title_start: Option<u32>, title_end: Option<u32> },
    ImageEnd,
    AutolinkUrl { content_start: u32, content_end: u32 },
    AutolinkEmail { content_start: u32, content_end: u32 },
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
