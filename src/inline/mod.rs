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
use code_span::{resolve_code_spans, extract_code_spans, CodeSpan};
use emphasis::{resolve_emphasis, EmphasisMatch};
use links::{find_autolinks_into, resolve_links, resolve_reference_links, Autolink, Link, RefLink};
use crate::link_ref::LinkRefStore;
use marks::{collect_marks, flags, Mark, MarkBuffer};

/// Inline parser state.
pub struct InlineParser {
    /// Reusable mark buffer.
    mark_buffer: MarkBuffer,
    open_brackets: Vec<(u32, bool)>,
    close_brackets: Vec<u32>,
    autolinks: Vec<Autolink>,
    html_spans: Vec<HtmlSpan>,
    html_ranges: Vec<(u32, u32)>,
    link_dest_ranges: Vec<(u32, u32)>,
    autolink_ranges: Vec<(u32, u32)>,
    code_spans: Vec<CodeSpan>,
    link_boundaries: Vec<(u32, u32)>,
    link_events: Vec<EmitPoint>,
    emphasis_events: Vec<EmitPoint>,
}

impl InlineParser {
    /// Create a new inline parser.
    pub fn new() -> Self {
        Self {
            mark_buffer: MarkBuffer::new(),
            open_brackets: Vec::new(),
            close_brackets: Vec::new(),
            autolinks: Vec::new(),
            html_spans: Vec::new(),
            html_ranges: Vec::new(),
            link_dest_ranges: Vec::new(),
            autolink_ranges: Vec::new(),
            code_spans: Vec::new(),
            link_boundaries: Vec::new(),
            link_events: Vec::new(),
            emphasis_events: Vec::new(),
        }
    }

    /// Parse inline content and emit events.
    pub fn parse(&mut self, text: &[u8], link_refs: Option<&LinkRefStore>, events: &mut Vec<InlineEvent>) {
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
        // First: autolinks + raw HTML (skip if no '<' present)
        let has_lt = text.iter().any(|&b| b == b'<');
        self.autolinks.clear();
        if has_lt {
            find_autolinks_into(text, &mut self.autolinks);
        }

        // Second: raw inline HTML (ignore code spans for now; we'll filter after resolving them)
        self.html_spans.clear();
        if has_lt {
            find_html_spans_into(text, &[], &self.autolinks, &mut self.html_spans);
        }
        self.html_ranges.clear();
        self.html_ranges.reserve(self.html_spans.len());
        self.html_ranges.extend(self.html_spans.iter().map(|s| (s.start, s.end)));

        // Third: code spans (highest precedence, but should not start inside HTML tags)
        resolve_code_spans(self.mark_buffer.marks_mut(), text, &self.html_ranges);

        // Get code span ranges for filtering
        self.code_spans.clear();
        self.code_spans.extend(extract_code_spans(self.mark_buffer.marks()));
        filter_html_spans_in_code_spans(&mut self.html_spans, &self.code_spans);

        // Filter autolinks that start inside code spans
        self.autolinks.retain(|al| {
            !self.code_spans.iter().any(|cs| {
                al.start >= cs.opener_pos && al.start < cs.closer_end
            })
        });

        // Fourth: links and images (skip if no brackets)
        Self::collect_brackets(self.mark_buffer.marks(), &mut self.open_brackets, &mut self.close_brackets);
        let has_brackets = !self.open_brackets.is_empty() && !self.close_brackets.is_empty();
        self.open_brackets
            .retain(|&(pos, _)| !pos_in_spans(pos, &self.html_spans));
        // Filter out close brackets that are inside autolinks - they can't close links
        self.close_brackets
            .retain(|&pos| {
                !self.autolinks.iter().any(|al| pos > al.start && pos < al.end)
                    && !pos_in_spans(pos, &self.html_spans)
            });
        let (resolved_links, resolved_ref_links) = if has_brackets {
            let resolved_links = resolve_links(text, &self.open_brackets, &self.close_brackets);
            let resolved_ref_links = link_refs
                .filter(|defs| !defs.is_empty())
                .map(|defs| resolve_reference_links(text, &self.open_brackets, &self.close_brackets, &resolved_links, defs))
                .unwrap_or_default();
            filter_html_spans_in_link_destinations(&mut self.html_spans, &resolved_links);
            (resolved_links, resolved_ref_links)
        } else {
            (Vec::new(), Vec::new())
        };

        self.link_dest_ranges.clear();
        self.link_dest_ranges.extend(resolved_links.iter().filter_map(|link| {
            let start = link.text_end + 1;
            let end = link.end;
            (start < end).then_some((start, end))
        }));

        self.autolink_ranges.clear();
        self.autolink_ranges.extend(self.autolinks.iter().map(|al| (al.start, al.end)));

        // Fifth: emphasis (lowest precedence)
        // Pass link and autolink boundaries so emphasis can't cross them
        self.link_boundaries.clear();
        self.link_boundaries.reserve(
            resolved_links.len()
                + resolved_ref_links.len()
                + self.autolinks.len()
                + self.html_spans.len(),
        );
        self.link_boundaries.extend(resolved_links.iter().map(|l| (l.start, l.text_end)));
        for link in &resolved_ref_links {
            self.link_boundaries.push((link.start, link.text_end));
        }
        // Also include autolinks - delimiters inside <url> should not form emphasis
        for autolink in &self.autolinks {
            self.link_boundaries.push((autolink.start, autolink.end));
        }
        // Also include raw HTML spans
        for span in &self.html_spans {
            self.link_boundaries.push((span.start, span.end));
        }
        let emphasis_matches = resolve_emphasis(self.mark_buffer.marks_mut(), &self.link_boundaries);

        // Phase 3: Emit events
        let marks = self.mark_buffer.marks();
        let (link_events, emphasis_events) = (&mut self.link_events, &mut self.emphasis_events);
        Self::emit_events(
            marks,
            text,
            &self.code_spans,
            &emphasis_matches,
            &resolved_links,
            &resolved_ref_links,
            &self.autolinks,
            &self.html_spans,
            &self.link_dest_ranges,
            &self.autolink_ranges,
            &self.html_ranges,
            link_events,
            emphasis_events,
            events,
        );
    }

    /// Collect bracket positions for link parsing.
    fn collect_brackets(marks: &[Mark], open_brackets: &mut Vec<(u32, bool)>, close_brackets: &mut Vec<u32>) {
        let estimated = (marks.len() / 4).max(4);
        open_brackets.clear();
        close_brackets.clear();
        open_brackets.reserve(estimated);
        close_brackets.reserve(estimated);

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
    }

    /// Emit events based on resolved marks.
    fn emit_events(
        marks: &[Mark],
        text: &[u8],
        code_spans: &[CodeSpan],
        emphasis_matches: &[EmphasisMatch],
        resolved_links: &[Link],
        resolved_ref_links: &[RefLink],
        autolinks: &[Autolink],
        html_spans: &[HtmlSpan],
        link_dest_ranges: &[(u32, u32)],
        autolink_ranges: &[(u32, u32)],
        html_ranges: &[(u32, u32)],
        link_events: &mut Vec<EmitPoint>,
        emphasis_events: &mut Vec<EmitPoint>,
        events: &mut Vec<InlineEvent>,
    ) {
        let mut pos = 0u32;
        let text_len = text.len() as u32;

        let mut code_cursor = CodeCursor::new(code_spans, autolink_ranges);
        link_events.clear();
        link_events.reserve(
            (resolved_links.len() + resolved_ref_links.len()) * 2,
        );
        for link in resolved_links {
            if link.is_image {
                link_events.push(EmitPoint {
                    pos: link.start,
                    kind: EmitKind::ImageStart {
                        url_start: link.url_start,
                        url_end: link.url_end,
                        title_start: link.title_start,
                        title_end: link.title_end,
                    },
                    end: link.start + 2,
                });
                link_events.push(EmitPoint {
                    pos: link.text_end,
                    kind: EmitKind::ImageEnd,
                    end: link.end,
                });
            } else {
                link_events.push(EmitPoint {
                    pos: link.start,
                    kind: EmitKind::LinkStart {
                        url_start: link.url_start,
                        url_end: link.url_end,
                        title_start: link.title_start,
                        title_end: link.title_end,
                    },
                    end: link.start + 1,
                });
                link_events.push(EmitPoint {
                    pos: link.text_end,
                    kind: EmitKind::LinkEnd,
                    end: link.end,
                });
            }
        }
        for link in resolved_ref_links {
            if link.is_image {
                link_events.push(EmitPoint {
                    pos: link.start,
                    kind: EmitKind::ImageStartRef {
                        def_index: link.def_index as u32,
                    },
                    end: link.start + 2,
                });
                link_events.push(EmitPoint {
                    pos: link.text_end,
                    kind: EmitKind::ImageEnd,
                    end: link.end,
                });
            } else {
                link_events.push(EmitPoint {
                    pos: link.start,
                    kind: EmitKind::LinkStartRef {
                        def_index: link.def_index as u32,
                    },
                    end: link.start + 1,
                });
                link_events.push(EmitPoint {
                    pos: link.text_end,
                    kind: EmitKind::LinkEnd,
                    end: link.end,
                });
            }
        }
        link_events.sort_by_key(|p| (p.pos, is_end_kind(&p.kind)));

        let mut link_cursor = EventCursor::new(link_events);
        let mut autolink_cursor = AutolinkCursor::new(autolinks);
        let mut html_cursor = HtmlCursor::new(html_spans);
        emphasis_events.clear();
        emphasis_events.reserve(emphasis_matches.len() * 2);
        for m in emphasis_matches {
            let is_strong = m.count == 2;
            if is_strong {
                emphasis_events.push(EmitPoint {
                    pos: m.opener_start,
                    kind: EmitKind::StrongStart,
                    end: m.opener_end,
                });
                emphasis_events.push(EmitPoint {
                    pos: m.closer_start,
                    kind: EmitKind::StrongEnd,
                    end: m.closer_end,
                });
            } else {
                emphasis_events.push(EmitPoint {
                    pos: m.opener_start,
                    kind: EmitKind::EmphasisStart,
                    end: m.opener_end,
                });
                emphasis_events.push(EmitPoint {
                    pos: m.closer_start,
                    kind: EmitKind::EmphasisEnd,
                    end: m.closer_end,
                });
            }
        }
        emphasis_events.sort_by_key(|p| (p.pos, is_end_kind(&p.kind)));

        let mut emphasis_cursor = EventCursor::new(emphasis_events);
        let mut mark_cursor = MarkCursor::new(marks, text, link_dest_ranges, autolink_ranges, html_ranges);

        // Build ranges to suppress (reference labels after link text)
        let mut suppress_ranges: Vec<(u32, u32)> = Vec::new();
        for link in resolved_ref_links {
            if link.end > link.text_end {
                suppress_ranges.push((link.text_end, link.end));
            }
        }

        // Emit events in order
        let mut skip_until = 0u32;

        while let Some(point) = next_event(
            &mut code_cursor,
            &mut link_cursor,
            &mut autolink_cursor,
            &mut html_cursor,
            &mut emphasis_cursor,
            &mut mark_cursor,
        ) {
            // Skip events inside suppressed ranges (e.g., reference labels)
            if suppress_ranges.iter().any(|&(s, e)| point.pos > s && point.pos < e) {
                pos = pos.max(point.end);
                continue;
            }
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
                EmitKind::LinkStartRef { def_index } => {
                    events.push(InlineEvent::LinkStartRef { def_index });
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
                EmitKind::ImageStartRef { def_index } => {
                    events.push(InlineEvent::ImageStartRef { def_index });
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
                EmitKind::HtmlRaw { end } => {
                    events.push(InlineEvent::Html(Range::from_usize(
                        point.pos as usize,
                        end as usize,
                    )));
                    skip_until = end;
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
    LinkStartRef { def_index: u32 },
    LinkEnd,
    ImageStart { url_start: u32, url_end: u32, title_start: Option<u32>, title_end: Option<u32> },
    ImageStartRef { def_index: u32 },
    ImageEnd,
    AutolinkUrl { content_start: u32, content_end: u32 },
    AutolinkEmail { content_start: u32, content_end: u32 },
    HtmlRaw { end: u32 },
}

#[inline]
fn is_end_kind(kind: &EmitKind) -> bool {
    matches!(
        kind,
        EmitKind::CodeSpanEnd
            | EmitKind::StrongEnd
            | EmitKind::EmphasisEnd
            | EmitKind::LinkEnd
            | EmitKind::ImageEnd
    )
}

struct CodeCursor<'a> {
    spans: &'a [CodeSpan],
    autolink_ranges: &'a [(u32, u32)],
    idx: usize,
    stage: u8,
    autolink_idx: usize,
    current: Option<EmitPoint>,
}

impl<'a> CodeCursor<'a> {
    fn new(spans: &'a [CodeSpan], autolink_ranges: &'a [(u32, u32)]) -> Self {
        let mut cursor = Self {
            spans,
            autolink_ranges,
            idx: 0,
            stage: 0,
            autolink_idx: 0,
            current: None,
        };
        cursor.current = cursor.next_event();
        cursor
    }

    fn advance(&mut self) {
        self.current = self.next_event();
    }

    fn next_event(&mut self) -> Option<EmitPoint> {
        while self.idx < self.spans.len() {
            let span = self.spans[self.idx];
            if !self.autolink_ranges.is_empty()
                && pos_in_ranges_u32(span.opener_pos, self.autolink_ranges, &mut self.autolink_idx)
            {
                self.idx += 1;
                self.stage = 0;
                continue;
            }

            let event = match self.stage {
                0 => EmitPoint {
                    pos: span.opener_pos,
                    kind: EmitKind::CodeSpanStart,
                    end: span.opener_end,
                },
                1 => {
                    let (content_start, content_end) = span.content_range();
                    EmitPoint {
                        pos: content_start,
                        kind: EmitKind::CodeContent(content_end),
                        end: content_end,
                    }
                }
                _ => EmitPoint {
                    pos: span.closer_pos,
                    kind: EmitKind::CodeSpanEnd,
                    end: span.closer_end,
                },
            };

            if self.stage >= 2 {
                self.idx += 1;
                self.stage = 0;
            } else {
                self.stage += 1;
            }

            return Some(event);
        }
        None
    }
}

struct EventCursor<'a> {
    events: &'a [EmitPoint],
    idx: usize,
    current: Option<EmitPoint>,
}

impl<'a> EventCursor<'a> {
    fn new(events: &'a [EmitPoint]) -> Self {
        let mut cursor = Self {
            events,
            idx: 0,
            current: None,
        };
        cursor.current = cursor.next_event();
        cursor
    }

    fn advance(&mut self) {
        self.current = self.next_event();
    }

    fn next_event(&mut self) -> Option<EmitPoint> {
        if self.idx >= self.events.len() {
            return None;
        }
        let event = self.events[self.idx];
        self.idx += 1;
        Some(event)
    }
}

struct AutolinkCursor<'a> {
    autolinks: &'a [Autolink],
    idx: usize,
    current: Option<EmitPoint>,
}

impl<'a> AutolinkCursor<'a> {
    fn new(autolinks: &'a [Autolink]) -> Self {
        let mut cursor = Self {
            autolinks,
            idx: 0,
            current: None,
        };
        cursor.current = cursor.next_event();
        cursor
    }

    fn advance(&mut self) {
        self.current = self.next_event();
    }

    fn next_event(&mut self) -> Option<EmitPoint> {
        if self.idx >= self.autolinks.len() {
            return None;
        }
        let al = &self.autolinks[self.idx];
        self.idx += 1;
        Some(if al.is_email {
            EmitPoint {
                pos: al.start,
                kind: EmitKind::AutolinkEmail {
                    content_start: al.content_start,
                    content_end: al.content_end,
                },
                end: al.end,
            }
        } else {
            EmitPoint {
                pos: al.start,
                kind: EmitKind::AutolinkUrl {
                    content_start: al.content_start,
                    content_end: al.content_end,
                },
                end: al.end,
            }
        })
    }
}

struct HtmlCursor<'a> {
    spans: &'a [HtmlSpan],
    idx: usize,
    current: Option<EmitPoint>,
}

impl<'a> HtmlCursor<'a> {
    fn new(spans: &'a [HtmlSpan]) -> Self {
        let mut cursor = Self {
            spans,
            idx: 0,
            current: None,
        };
        cursor.current = cursor.next_event();
        cursor
    }

    fn advance(&mut self) {
        self.current = self.next_event();
    }

    fn next_event(&mut self) -> Option<EmitPoint> {
        if self.idx >= self.spans.len() {
            return None;
        }
        let span = self.spans[self.idx];
        self.idx += 1;
        Some(EmitPoint {
            pos: span.start,
            kind: EmitKind::HtmlRaw { end: span.end },
            end: span.end,
        })
    }
}

struct MarkCursor<'a> {
    marks: &'a [Mark],
    text: &'a [u8],
    link_dest_ranges: &'a [(u32, u32)],
    autolink_ranges: &'a [(u32, u32)],
    html_ranges: &'a [(u32, u32)],
    idx: usize,
    link_dest_idx: usize,
    autolink_idx: usize,
    html_idx: usize,
    current: Option<EmitPoint>,
}

impl<'a> MarkCursor<'a> {
    fn new(
        marks: &'a [Mark],
        text: &'a [u8],
        link_dest_ranges: &'a [(u32, u32)],
        autolink_ranges: &'a [(u32, u32)],
        html_ranges: &'a [(u32, u32)],
    ) -> Self {
        let mut cursor = Self {
            marks,
            text,
            link_dest_ranges,
            autolink_ranges,
            html_ranges,
            idx: 0,
            link_dest_idx: 0,
            autolink_idx: 0,
            html_idx: 0,
            current: None,
        };
        cursor.current = cursor.next_event();
        cursor
    }

    fn advance(&mut self) {
        self.current = self.next_event();
    }

    fn next_event(&mut self) -> Option<EmitPoint> {
        while self.idx < self.marks.len() {
            let mark = self.marks[self.idx];
            self.idx += 1;

            let in_code = mark.flags & flags::IN_CODE != 0;
            let in_link_dest = !self.link_dest_ranges.is_empty()
                && pos_in_ranges_u32(mark.pos, self.link_dest_ranges, &mut self.link_dest_idx);
            let in_autolink = !self.autolink_ranges.is_empty()
                && pos_in_ranges_u32(mark.pos, self.autolink_ranges, &mut self.autolink_idx);
            let in_html = !self.html_ranges.is_empty()
                && pos_in_ranges_u32(mark.pos, self.html_ranges, &mut self.html_idx);

            if mark.ch == b'\\' && mark.flags & flags::POTENTIAL_OPENER != 0 {
                let escaped_char = self.text[(mark.pos + 1) as usize];
                if escaped_char == b'\n' && !in_code && !in_autolink && !in_html {
                    return Some(EmitPoint {
                        pos: mark.pos,
                        kind: EmitKind::HardBreak,
                        end: mark.end,
                    });
                } else if !in_code && !in_link_dest && !in_autolink && !in_html {
                    return Some(EmitPoint {
                        pos: mark.pos,
                        kind: EmitKind::Escape(escaped_char),
                        end: mark.end,
                    });
                }
            } else if mark.ch == b'\n'
                && mark.flags & flags::POTENTIAL_OPENER != 0
                && !in_code
                && !in_link_dest
                && !in_html
            {
                return Some(EmitPoint {
                    pos: mark.pos,
                    kind: EmitKind::HardBreak,
                    end: mark.end,
                });
            } else if mark.ch == b'\n'
                && mark.flags & flags::POTENTIAL_CLOSER != 0
                && !in_code
                && !in_link_dest
                && !in_html
            {
                return Some(EmitPoint {
                    pos: mark.pos,
                    kind: EmitKind::SoftBreak,
                    end: mark.end,
                });
            }
        }
        None
    }
}

fn next_event(
    code: &mut CodeCursor<'_>,
    links: &mut EventCursor<'_>,
    autolinks: &mut AutolinkCursor<'_>,
    html: &mut HtmlCursor<'_>,
    emphasis: &mut EventCursor<'_>,
    marks: &mut MarkCursor<'_>,
) -> Option<EmitPoint> {
    let mut best: Option<(u8, EmitPoint)> = None;

    let mut consider = |id: u8, current: Option<EmitPoint>| {
        let Some(ev) = current else { return; };
        let key = (ev.pos, is_end_kind(&ev.kind));
        if let Some((_, best_ev)) = best {
            let best_key = (best_ev.pos, is_end_kind(&best_ev.kind));
            if key < best_key {
                best = Some((id, ev));
            }
        } else {
            best = Some((id, ev));
        }
    };

    consider(0, code.current);
    consider(1, links.current);
    consider(2, autolinks.current);
    consider(3, html.current);
    consider(4, emphasis.current);
    consider(5, marks.current);

    let Some((id, ev)) = best else { return None; };
    match id {
        0 => code.advance(),
        1 => links.advance(),
        2 => autolinks.advance(),
        3 => html.advance(),
        4 => emphasis.advance(),
        _ => marks.advance(),
    }
    Some(ev)
}

#[derive(Debug, Clone, Copy)]
struct HtmlSpan {
    start: u32,
    end: u32,
}

fn pos_in_spans(pos: u32, spans: &[HtmlSpan]) -> bool {
    let mut lo = 0usize;
    let mut hi = spans.len();
    while lo < hi {
        let mid = (lo + hi) / 2;
        let span = spans[mid];
        if pos < span.start {
            hi = mid;
        } else if pos >= span.end {
            lo = mid + 1;
        } else {
            return true;
        }
    }
    false
}

fn find_html_spans_into(
    text: &[u8],
    code_spans: &[CodeSpan],
    autolinks: &[Autolink],
    spans: &mut Vec<HtmlSpan>,
) {
    spans.clear();
    let len = text.len();

    let mut code_ranges: Vec<(usize, usize)> = code_spans
        .iter()
        .map(|cs| (cs.opener_pos as usize, cs.closer_end as usize))
        .collect();
    code_ranges.sort_by_key(|(s, _)| *s);

    let mut autolink_ranges: Vec<(usize, usize)> = autolinks
        .iter()
        .map(|al| (al.start as usize, al.end as usize))
        .collect();
    autolink_ranges.sort_by_key(|(s, _)| *s);

    let mut code_idx = 0usize;
    let mut autolink_idx = 0usize;

    let mut pos = 0usize;
    while pos < len {
        if text[pos] != b'<' {
            pos += 1;
            continue;
        }
        if is_escaped(text, pos) {
            pos += 1;
            continue;
        }

        if pos_in_ranges(pos, &code_ranges, &mut code_idx)
            || pos_in_ranges(pos, &autolink_ranges, &mut autolink_idx)
        {
            pos += 1;
            continue;
        }

        if let Some(end) = parse_inline_html(text, pos) {
            spans.push(HtmlSpan {
                start: pos as u32,
                end: end as u32,
            });
            pos = end;
        } else {
            pos += 1;
        }
    }
}

fn filter_html_spans_in_link_destinations(spans: &mut Vec<HtmlSpan>, links: &[Link]) {
    if spans.is_empty() || links.is_empty() {
        return;
    }
    spans.retain(|span| {
        !links.iter().any(|link| {
            let dest_start = (link.text_end + 1) as u32;
            span.start >= dest_start && span.start < link.end
        })
    });
}

fn filter_html_spans_in_code_spans(spans: &mut Vec<HtmlSpan>, code_spans: &[CodeSpan]) {
    if spans.is_empty() || code_spans.is_empty() {
        return;
    }
    spans.retain(|span| {
        !code_spans.iter().any(|cs| {
            span.start >= cs.opener_pos && span.start < cs.closer_end
        })
    });
}

fn pos_in_ranges(pos: usize, ranges: &[(usize, usize)], idx: &mut usize) -> bool {
    while *idx < ranges.len() && pos >= ranges[*idx].1 {
        *idx += 1;
    }
    *idx < ranges.len() && pos >= ranges[*idx].0
}

fn pos_in_ranges_u32(pos: u32, ranges: &[(u32, u32)], idx: &mut usize) -> bool {
    while *idx < ranges.len() && pos >= ranges[*idx].1 {
        *idx += 1;
    }
    *idx < ranges.len() && pos >= ranges[*idx].0
}

#[inline]
fn is_escaped(text: &[u8], pos: usize) -> bool {
    if pos == 0 {
        return false;
    }
    let mut backslashes = 0usize;
    let mut i = pos;
    while i > 0 && text[i - 1] == b'\\' {
        backslashes += 1;
        i -= 1;
    }
    backslashes % 2 == 1
}

fn parse_inline_html(text: &[u8], start: usize) -> Option<usize> {
    if text.get(start) != Some(&b'<') {
        return None;
    }
    if text.get(start + 1) == Some(&b'!') {
        if text[start..].starts_with(b"<!--") {
            return parse_html_comment(text, start);
        }
        if text[start..].starts_with(b"<![CDATA[") {
            return find_subsequence(text, start + 9, b"]]>").map(|end| end + 3);
        }
        return parse_html_declaration(text, start);
    }
    if text.get(start + 1) == Some(&b'?') {
        return find_subsequence(text, start + 2, b"?>").map(|end| end + 2);
    }
    parse_html_tag(text, start)
}

fn parse_html_comment(text: &[u8], start: usize) -> Option<usize> {
    let i = start + 4;
    if i >= text.len() {
        return None;
    }
    if text[i] == b'>' {
        return Some(i + 1);
    }
    if text[i] == b'-' && text.get(i + 1) == Some(&b'>') {
        return Some(i + 2);
    }
    find_subsequence(text, i, b"-->").map(|end| end + 3)
}

fn parse_html_declaration(text: &[u8], start: usize) -> Option<usize> {
    if text.get(start + 2).map_or(true, |b| !b.is_ascii_alphabetic()) {
        return None;
    }
    let mut i = start + 2;
    while i < text.len() && (text[i].is_ascii_alphanumeric() || text[i] == b'-') {
        i += 1;
    }
    while i < text.len() && is_html_whitespace(text[i]) {
        i += 1;
    }
    find_subsequence_byte(text, i, b'>').map(|end| end + 1)
}

fn parse_html_tag(text: &[u8], start: usize) -> Option<usize> {
    let len = text.len();
    let mut i = start + 1;
    if i >= len {
        return None;
    }

    let mut is_closing = false;
    if text[i] == b'/' {
        is_closing = true;
        i += 1;
    }

    if i >= len || !text[i].is_ascii_alphabetic() {
        return None;
    }
    i += 1;
    while i < len && (text[i].is_ascii_alphanumeric() || text[i] == b'-') {
        i += 1;
    }

    if is_closing {
        while i < len && is_html_whitespace(text[i]) {
            i += 1;
        }
        if i < len && text[i] == b'>' {
            return Some(i + 1);
        }
        return None;
    }

    loop {
        if i >= len {
            return None;
        }
        if text[i] == b'>' {
            return Some(i + 1);
        }
        if text[i] == b'/' {
            i += 1;
            return if i < len && text[i] == b'>' { Some(i + 1) } else { None };
        }
        if !is_html_whitespace(text[i]) {
            return None;
        }
        while i < len && is_html_whitespace(text[i]) {
            i += 1;
        }
        if i >= len {
            return None;
        }
        if text[i] == b'>' {
            return Some(i + 1);
        }
        if text[i] == b'/' {
            i += 1;
            return if i < len && text[i] == b'>' { Some(i + 1) } else { None };
        }

        if !is_attr_name_start(text[i]) {
            return None;
        }
        i += 1;
        while i < len && is_attr_name_char(text[i]) {
            i += 1;
        }

        let ws_start = i;
        while i < len && is_html_whitespace(text[i]) {
            i += 1;
        }

        if i < len && text[i] == b'=' {
            i += 1;
            while i < len && is_html_whitespace(text[i]) {
                i += 1;
            }
            if i >= len {
                return None;
            }
            let quote = text[i];
            if quote == b'"' || quote == b'\'' {
                i += 1;
                let value_start = i;
                while i < len && text[i] != quote {
                    i += 1;
                }
                if i >= len {
                    return None;
                }
                if i > value_start && text[i - 1] == b'\\' {
                    return None;
                }
                i += 1;
            } else {
                let mut had = false;
                while i < len && !is_html_whitespace(text[i]) {
                    let b = text[i];
                    if b == b'"' || b == b'\'' || b == b'=' || b == b'<' || b == b'>' || b == b'`' {
                        break;
                    }
                    had = true;
                    i += 1;
                }
                if !had {
                    return None;
                }
            }
        } else {
            i = ws_start;
        }
    }
}

#[inline]
fn is_html_whitespace(b: u8) -> bool {
    matches!(b, b' ' | b'\t' | b'\n' | b'\r' | b'\x0c')
}

#[inline]
fn is_attr_name_start(b: u8) -> bool {
    b.is_ascii_alphabetic() || b == b'_' || b == b':'
}

#[inline]
fn is_attr_name_char(b: u8) -> bool {
    is_attr_name_start(b) || b.is_ascii_digit() || b == b'.' || b == b'-'
}

fn find_subsequence(text: &[u8], start: usize, needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || start >= text.len() || text.len() < needle.len() {
        return None;
    }
    text[start..]
        .windows(needle.len())
        .position(|w| w == needle)
        .map(|idx| start + idx)
}

fn find_subsequence_byte(text: &[u8], start: usize, needle: u8) -> Option<usize> {
    if start >= text.len() {
        return None;
    }
    text[start..]
        .iter()
        .position(|&b| b == needle)
        .map(|idx| start + idx)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_inline(text: &str) -> Vec<InlineEvent> {
        let mut parser = InlineParser::new();
        let mut events = Vec::new();
        parser.parse(text.as_bytes(), None, &mut events);
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

    #[test]
    fn test_html_tag_with_newline_in_attributes() {
        let input = "<a foo=\"bar\" bam = 'baz <em>\"</em>'\n_boolean zoop:33=zoop:33 />";
        assert!(
            parse_inline_html(b"<a foo=\"bar\"\n_boolean />", 0).is_some(),
            "Expected basic multiline tag to parse"
        );
        assert!(
            parse_inline_html(b"<a foo=\"bar\" bam = 'baz <em>\"</em>'\n_boolean />", 0).is_some(),
            "Expected quoted attribute with inline tags to parse"
        );
        let end = parse_inline_html(input.as_bytes(), 0);
        assert!(end.is_some(), "Expected inline HTML parser to match the tag");
        let events = parse_inline(input);
        let mut found = false;
        for event in events {
            if let InlineEvent::Html(range) = event {
                if range.start == 0 {
                    let slice = range.slice(input.as_bytes());
                    assert!(slice.starts_with(b"<a "), "Expected HTML span to start at <a>");
                    found = true;
                    break;
                }
            }
        }
        assert!(found, "Expected raw HTML tag to be parsed at start");
    }
}
