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
        resolve_code_spans(self.mark_buffer.marks_mut(), text);

        // Get code span ranges for filtering
        let code_spans: Vec<_> = extract_code_spans(self.mark_buffer.marks()).collect();

        // Second: autolinks (filter out those inside code spans)
        let autolinks: Vec<_> = find_autolinks(text)
            .into_iter()
            .filter(|al| {
                // Autolink should not start inside a code span
                !code_spans.iter().any(|cs| {
                    al.start >= cs.opener_pos && al.start < cs.closer_end
                })
            })
            .collect();

        // Third: links and images
        let (open_brackets, close_brackets) = self.collect_brackets();
        let resolved_links = resolve_links(text, &open_brackets, &close_brackets);

        // Fourth: emphasis (lowest precedence)
        // Pass link boundaries so emphasis can't cross them
        let link_boundaries: Vec<(u32, u32)> = resolved_links
            .iter()
            .map(|l| (l.start, l.text_end))
            .collect();
        let emphasis_matches = resolve_emphasis(self.mark_buffer.marks_mut(), &link_boundaries);

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

        // Add code span events (filter out spans whose opener is inside an autolink)
        for span in extract_code_spans(marks) {
            // Skip code spans whose opener starts inside an autolink
            let inside_autolink = autolinks.iter().any(|al| {
                span.opener_pos >= al.start && span.opener_pos < al.end
            });
            if inside_autolink {
                continue;
            }

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
        // Note: Hard breaks inside code spans should not be processed
        for mark in marks {
            // Skip marks inside code spans
            let in_code = mark.flags & flags::IN_CODE != 0;

            // Check if mark is inside a link's URL or title (not the link text)
            let in_link_dest = resolved_links.iter().any(|link| {
                // URL range: url_start to url_end
                // Title range: title_start to title_end (if present)
                let in_url = mark.pos >= link.url_start && mark.pos < link.url_end;
                let in_title = link.title_start.map_or(false, |ts| {
                    let te = link.title_end.unwrap();
                    mark.pos >= ts && mark.pos < te
                });
                in_url || in_title
            });

            // Check if mark is inside an autolink
            let in_autolink = autolinks.iter().any(|al| {
                mark.pos >= al.start && mark.pos < al.end
            });

            if mark.ch == b'\\' && mark.flags & flags::POTENTIAL_OPENER != 0 {
                let escaped_char = text[(mark.pos + 1) as usize];
                if escaped_char == b'\n' && !in_code && !in_autolink {
                    // Backslash before newline is a hard break (but not in code or autolinks)
                    emit_points.push(EmitPoint {
                        pos: mark.pos,
                        kind: EmitKind::HardBreak,
                        end: mark.end,
                    });
                } else if !in_code && !in_link_dest && !in_autolink {
                    // Skip escapes inside link URLs/titles and autolinks (they're processed by renderer)
                    emit_points.push(EmitPoint {
                        pos: mark.pos,
                        kind: EmitKind::Escape(escaped_char),
                        end: mark.end,
                    });
                }
            } else if mark.ch == b'\n' && mark.flags & flags::POTENTIAL_OPENER != 0 && !in_code {
                // Two spaces before newline is a hard break (but not in code)
                emit_points.push(EmitPoint {
                    pos: mark.pos,
                    kind: EmitKind::HardBreak,
                    end: mark.end,
                });
            } else if mark.ch == b'\n' && mark.flags & flags::POTENTIAL_CLOSER != 0 && !in_code {
                // Soft break (newline without 2+ spaces) - also not in code
                emit_points.push(EmitPoint {
                    pos: mark.pos,
                    kind: EmitKind::SoftBreak,
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

                    // CommonMark: line endings are converted to spaces first,
                    // then if the string both begins AND ends with a space,
                    // and doesn't consist entirely of spaces, strip one space from each end.
                    // Note: we treat \n as equivalent to space for stripping purposes.
                    if end > start + 1 {
                        let content = &text[start..end];
                        let first_is_space = content[0] == b' ' || content[0] == b'\n';
                        let last_is_space = content[content.len() - 1] == b' ' || content[content.len() - 1] == b'\n';
                        let not_all_space = content.iter().any(|&b| b != b' ' && b != b'\n');

                        if first_is_space && last_is_space && not_all_space {
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
                EmitKind::SoftBreak => {
                    events.push(InlineEvent::SoftBreak);
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
    SoftBreak,
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

    #[test]
    fn test_image() {
        let input = "![alt](image.png)";
        let events = parse_inline(input);

        // Should have ImageStart and ImageEnd
        assert!(events.iter().any(|e| matches!(e, InlineEvent::ImageStart { .. })), "No ImageStart found");
        assert!(events.iter().any(|e| matches!(e, InlineEvent::ImageEnd)), "No ImageEnd found");

        // Should NOT have a text event with just "!"
        for event in &events {
            if let InlineEvent::Text(range) = event {
                let text = range.slice(input.as_bytes());
                assert!(text != b"!", "Found standalone ! as text");
            }
        }
    }
}
