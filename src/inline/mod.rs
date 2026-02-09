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
mod simd;
mod strikethrough;

pub use event::InlineEvent;
pub use links::AutolinkLiteralKind;

use crate::Range;
use code_span::{resolve_code_spans, extract_code_spans, CodeSpan};
use emphasis::{resolve_emphasis_with_stacks_into, EmphasisMatch, EmphasisStacks};
use strikethrough::{resolve_strikethrough_into, StrikethroughMatch};
use links::{find_autolinks_into, find_autolink_literals_into, resolve_links_into, resolve_reference_links_into, Autolink, AutolinkLiteral, Link, RefLink};
use crate::footnote::{FootnoteStore, normalize_footnote_label};
use crate::link_ref::LinkRefStore;
use marks::{collect_marks, flags, Mark, MarkBuffer};
use memchr::memchr;

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
    resolved_links: Vec<Link>,
    link_formed_opens: Vec<bool>,
    link_inactive_opens: Vec<bool>,
    link_used_closes: Vec<bool>,
    ref_links: Vec<RefLink>,
    ref_label_buf: String,
    ref_formed_opens: Vec<bool>,
    ref_used_closes: Vec<bool>,
    ref_occupied: Vec<(u32, u32)>,
    autolink_literals: Vec<AutolinkLiteral>,
    emphasis_stacks: EmphasisStacks,
    emphasis_matches: Vec<EmphasisMatch>,
    strikethrough_matches: Vec<StrikethroughMatch>,
    al_code_span_ranges: Vec<(u32, u32)>,
    al_link_ranges: Vec<(u32, u32)>,
    emit_points: Vec<EmitPoint>,
    emit_suppress_ranges: Vec<(u32, u32)>,
    html_code_ranges: Vec<(usize, usize)>,
    html_autolink_ranges: Vec<(usize, usize)>,
    footnote_refs: Vec<FootnoteRef>,
}

impl InlineParser {
    /// Create a new inline parser.
    pub fn new() -> Self {
        Self {
            mark_buffer: MarkBuffer::new(),
            open_brackets: Vec::with_capacity(32),
            close_brackets: Vec::with_capacity(32),
            autolinks: Vec::with_capacity(8),
            html_spans: Vec::with_capacity(8),
            html_ranges: Vec::with_capacity(8),
            link_dest_ranges: Vec::with_capacity(8),
            autolink_ranges: Vec::with_capacity(8),
            code_spans: Vec::with_capacity(8),
            link_boundaries: Vec::with_capacity(16),
            resolved_links: Vec::with_capacity(8),
            link_formed_opens: Vec::with_capacity(32),
            link_inactive_opens: Vec::with_capacity(32),
            link_used_closes: Vec::with_capacity(32),
            ref_links: Vec::with_capacity(8),
            ref_label_buf: String::with_capacity(64),
            ref_formed_opens: Vec::with_capacity(32),
            ref_used_closes: Vec::with_capacity(32),
            ref_occupied: Vec::with_capacity(8),
            autolink_literals: Vec::with_capacity(8),
            emphasis_stacks: EmphasisStacks::default(),
            emphasis_matches: Vec::with_capacity(16),
            strikethrough_matches: Vec::with_capacity(8),
            al_code_span_ranges: Vec::with_capacity(8),
            al_link_ranges: Vec::with_capacity(8),
            emit_points: Vec::with_capacity(64),
            emit_suppress_ranges: Vec::with_capacity(8),
            html_code_ranges: Vec::with_capacity(8),
            html_autolink_ranges: Vec::with_capacity(8),
            footnote_refs: Vec::with_capacity(4),
        }
    }

    /// Parse inline content and emit events.
    pub fn parse(
        &mut self,
        text: &[u8],
        link_refs: Option<&LinkRefStore>,
        allow_html: bool,
        events: &mut Vec<InlineEvent>,
    ) {
        self.parse_with_options(text, link_refs, allow_html, true, true, None, events);
    }

    /// Parse inline content with full GFM options.
    pub fn parse_with_options(
        &mut self,
        text: &[u8],
        link_refs: Option<&LinkRefStore>,
        allow_html: bool,
        strikethrough: bool,
        autolink_literals: bool,
        footnote_store: Option<&FootnoteStore>,
        events: &mut Vec<InlineEvent>,
    ) {
        let has_specials = has_inline_specials(text);

        // Check for potential autolink literal triggers when enabled
        let may_have_autolinks = autolink_literals && has_autolink_candidates(text);

        if !has_specials && !may_have_autolinks {
            if !text.is_empty() {
                events.push(InlineEvent::Text(Range::from_usize(0, text.len())));
            }
            return;
        }

        // Phase 1: Collect marks
        self.mark_buffer.reserve_for_text(text.len());
        if has_specials {
            collect_marks(text, &mut self.mark_buffer);
        } else {
            self.mark_buffer.clear();
        }

        if self.mark_buffer.is_empty() && !may_have_autolinks {
            // No special characters and no autolink candidates, emit as plain text
            if !text.is_empty() {
                events.push(InlineEvent::Text(Range::from_usize(0, text.len())));
            }
            return;
        }

        // Phase 2: Resolve marks by precedence
        // First: autolinks + raw HTML (skip if no '<' present)
        let has_lt = memchr(b'<', text).is_some();
        self.autolinks.clear();
        if has_lt {
            find_autolinks_into(text, &mut self.autolinks);
        }

        // Second: raw inline HTML (ignore code spans for now; we'll filter after resolving them)
        self.html_spans.clear();
        if has_lt && allow_html {
            find_html_spans_into(
                text,
                &[],
                &self.autolinks,
                &mut self.html_code_ranges,
                &mut self.html_autolink_ranges,
                &mut self.html_spans,
            );
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

        // Fourth: collect bracket positions and detect emphasis/strikethrough candidates in one pass
        let (has_emphasis_marks, has_strikethrough_marks) = Self::collect_brackets_and_scan_emphasis(
            self.mark_buffer.marks(),
            &mut self.open_brackets,
            &mut self.close_brackets,
        );
        let has_brackets = !self.open_brackets.is_empty() && !self.close_brackets.is_empty();
        self.open_brackets
            .retain(|&(pos, _)| !pos_in_spans(pos, &self.html_spans));
        // Filter out close brackets that are inside autolinks - they can't close links
        self.close_brackets
            .retain(|&pos| {
                !self.autolinks.iter().any(|al| pos > al.start && pos < al.end)
                    && !pos_in_spans(pos, &self.html_spans)
            });
        let has_inline_link_candidate = has_brackets && has_inline_link_opener(text);
        if has_inline_link_candidate {
            resolve_links_into(
                text,
                &self.open_brackets,
                &self.close_brackets,
                &mut self.resolved_links,
                &mut self.link_formed_opens,
                &mut self.link_inactive_opens,
                &mut self.link_used_closes,
            );
        } else {
            self.resolved_links.clear();
        }
        let resolved_links = &self.resolved_links;

        if has_brackets {
            if let Some(defs) = link_refs.filter(|defs| !defs.is_empty()) {
                resolve_reference_links_into(
                    text,
                    &self.open_brackets,
                    &self.close_brackets,
                    resolved_links,
                    defs,
                    &mut self.ref_links,
                    &mut self.ref_label_buf,
                    &mut self.ref_formed_opens,
                    &mut self.ref_used_closes,
                    &mut self.ref_occupied,
                );
            } else {
                self.ref_links.clear();
            }
            filter_html_spans_in_link_destinations(&mut self.html_spans, resolved_links);
        } else {
            self.ref_links.clear();
        }
        let resolved_ref_links = &self.ref_links;

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
        for link in resolved_ref_links {
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
        let emphasis_matches = if has_emphasis_marks {
            resolve_emphasis_with_stacks_into(
                self.mark_buffer.marks_mut(),
                &self.link_boundaries,
                &mut self.emphasis_stacks,
                &mut self.emphasis_matches,
            );
            self.emphasis_matches.as_slice()
        } else {
            self.emphasis_matches.clear();
            &[]
        };

        // Sixth: strikethrough (after emphasis, since they share the mark buffer)
        if has_strikethrough_marks && strikethrough {
            resolve_strikethrough_into(
                self.mark_buffer.marks_mut(),
                &self.link_boundaries,
                &mut self.strikethrough_matches,
            );
        } else {
            self.strikethrough_matches.clear();
        }
        let strikethrough_matches = self.strikethrough_matches.as_slice();

        // Seventh: autolink literals (bare URLs, www, emails)
        if may_have_autolinks {
            // Build code span ranges for overlap checking (reuse Vec)
            self.al_code_span_ranges.clear();
            self.al_code_span_ranges.extend(
                self.code_spans.iter().map(|cs| (cs.opener_pos, cs.closer_end))
            );
            // Build link ranges (inline links + ref links) (reuse Vec)
            self.al_link_ranges.clear();
            self.al_link_ranges.reserve(resolved_links.len() + resolved_ref_links.len());
            for link in resolved_links {
                self.al_link_ranges.push((link.start, link.end));
            }
            for link in resolved_ref_links {
                self.al_link_ranges.push((link.start, link.end));
            }
            find_autolink_literals_into(
                text,
                &self.al_code_span_ranges,
                &self.html_ranges,
                &self.autolink_ranges,
                &self.al_link_ranges,
                &mut self.autolink_literals,
            );
        } else {
            self.autolink_literals.clear();
        }

        // Eighth: footnote references (`[^label]`)
        self.footnote_refs.clear();
        if let Some(fn_store) = footnote_store {
            Self::resolve_footnote_refs(
                text,
                &self.open_brackets,
                &self.close_brackets,
                resolved_links,
                resolved_ref_links,
                &self.code_spans,
                fn_store,
                &mut self.footnote_refs,
            );
        }

        // Phase 3: Emit events
        let marks = self.mark_buffer.marks();
        Self::emit_events(
            marks,
            text,
            &self.code_spans,
            emphasis_matches,
            strikethrough_matches,
            &self.autolink_literals,
            resolved_links,
            resolved_ref_links,
            &self.autolinks,
            &self.html_spans,
            &self.link_dest_ranges,
            &self.autolink_ranges,
            &self.html_ranges,
            &self.footnote_refs,
            &mut self.emit_points,
            &mut self.emit_suppress_ranges,
            events,
        );
    }

    /// Resolve footnote references: `[^label]` patterns not consumed by links/images.
    fn resolve_footnote_refs(
        text: &[u8],
        open_brackets: &[(u32, bool)],
        close_brackets: &[u32],
        resolved_links: &[Link],
        resolved_ref_links: &[RefLink],
        code_spans: &[CodeSpan],
        footnote_store: &FootnoteStore,
        out: &mut Vec<FootnoteRef>,
    ) {
        out.clear();

        // For each open bracket, check if it's followed by `^label]`
        for &(open_pos, is_image) in open_brackets {
            if is_image {
                continue; // `![^` is an image, not a footnote
            }

            // Check if this bracket is inside a code span
            let in_code = code_spans.iter().any(|cs| {
                open_pos >= cs.opener_pos && open_pos < cs.closer_end
            });
            if in_code {
                continue;
            }

            // Check if this bracket is already consumed by a link/image
            let in_link = resolved_links.iter().any(|l| l.start == open_pos)
                || resolved_ref_links.iter().any(|l| l.start == open_pos);
            if in_link {
                continue;
            }

            // Check for `^` after `[`
            let caret_pos = (open_pos + 1) as usize;
            if caret_pos >= text.len() || text[caret_pos] != b'^' {
                continue;
            }

            // Read label: alphanumeric, dash, underscore
            let label_start = caret_pos + 1;
            let mut label_end = label_start;
            while label_end < text.len() {
                let b = text[label_end];
                if b.is_ascii_alphanumeric() || b == b'-' || b == b'_' {
                    label_end += 1;
                } else {
                    break;
                }
            }

            if label_end == label_start {
                continue; // Empty label
            }

            // Must be followed by `]`
            if label_end >= text.len() || text[label_end] != b']' {
                continue;
            }

            // Check that this `]` is in the close_brackets list (validates it's not escaped)
            let close_pos = label_end as u32;
            if !close_brackets.contains(&close_pos) {
                continue;
            }

            let label_bytes = &text[label_start..label_end];
            if let Some(normalized) = normalize_footnote_label(label_bytes) {
                if let Some(idx) = footnote_store.get_index(&normalized) {
                    out.push(FootnoteRef {
                        start: open_pos,
                        end: close_pos + 1,
                        def_index: idx as u32,
                    });
                }
            }
        }
    }

    /// Collect bracket positions for link parsing.
    /// Returns (has_emphasis_marks, has_strikethrough_marks).
    fn collect_brackets_and_scan_emphasis(
        marks: &[Mark],
        open_brackets: &mut Vec<(u32, bool)>,
        close_brackets: &mut Vec<u32>,
    ) -> (bool, bool) {
        let estimated = (marks.len() / 4).max(4);
        open_brackets.clear();
        close_brackets.clear();
        open_brackets.reserve(estimated);
        close_brackets.reserve(estimated);

        let mut has_emphasis_marks = false;
        let mut has_strikethrough_marks = false;
        for mark in marks {
            if mark.flags & flags::IN_CODE == 0 {
                if mark.ch == b'*' || mark.ch == b'_' {
                    has_emphasis_marks = true;
                } else if mark.ch == b'~' {
                    has_strikethrough_marks = true;
                }
            }
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
        (has_emphasis_marks, has_strikethrough_marks)
    }

    /// Emit events based on resolved marks.
    fn emit_events(
        marks: &[Mark],
        text: &[u8],
        code_spans: &[CodeSpan],
        emphasis_matches: &[EmphasisMatch],
        strikethrough_matches: &[StrikethroughMatch],
        autolink_literals: &[AutolinkLiteral],
        resolved_links: &[Link],
        resolved_ref_links: &[RefLink],
        autolinks: &[Autolink],
        html_spans: &[HtmlSpan],
        link_dest_ranges: &[(u32, u32)],
        autolink_ranges: &[(u32, u32)],
        html_ranges: &[(u32, u32)],
        footnote_refs: &[FootnoteRef],
        emit_points: &mut Vec<EmitPoint>,
        suppress_ranges: &mut Vec<(u32, u32)>,
        events: &mut Vec<InlineEvent>,
    ) {
        let mut pos = 0u32;
        let text_len = text.len() as u32;

        let mut link_dest_idx = 0usize;
        let mut autolink_idx = 0usize;
        let mut html_idx = 0usize;
        let mut autolink_span_idx = 0usize;

        // Build sorted list of events to emit
        let estimated_events = marks.len()
            + (code_spans.len() * 3)
            + (resolved_links.len() * 4)
            + (resolved_ref_links.len() * 4)
            + autolinks.len()
            + autolink_literals.len()
            + html_spans.len()
            + (emphasis_matches.len() * 2)
            + (strikethrough_matches.len() * 2);
        emit_points.clear();
        emit_points.reserve(estimated_events.max(8));
        events.reserve(estimated_events.max(8) + 4);

        // Add code span events (filter out spans whose opener is inside an autolink)
        for span in code_spans {
            // Skip code spans whose opener starts inside an autolink
            let inside_autolink = !autolink_ranges.is_empty()
                && pos_in_ranges_u32(span.opener_pos, autolink_ranges, &mut autolink_span_idx);
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

        // Add reference-style link events
        for link in resolved_ref_links {
            if link.is_image {
                emit_points.push(EmitPoint {
                    pos: link.start,
                    kind: EmitKind::ImageStartRef { def_index: link.def_index as u32 },
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
                    kind: EmitKind::LinkStartRef { def_index: link.def_index as u32 },
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

        // Add raw HTML events
        for span in html_spans {
            emit_points.push(EmitPoint {
                pos: span.start,
                kind: EmitKind::HtmlRaw { end: span.end },
                end: span.end,
            });
        }

        // Add autolink literal events
        for al in autolink_literals {
            emit_points.push(EmitPoint {
                pos: al.start,
                kind: EmitKind::AutolinkLiteral { end: al.end, kind: al.kind },
                end: al.end,
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

        // Add strikethrough events
        for m in strikethrough_matches {
            emit_points.push(EmitPoint {
                pos: m.opener_start,
                kind: EmitKind::StrikethroughStart,
                end: m.opener_end,
            });
            emit_points.push(EmitPoint {
                pos: m.closer_start,
                kind: EmitKind::StrikethroughEnd,
                end: m.closer_end,
            });
        }

        // Add footnote reference events
        for fref in footnote_refs {
            emit_points.push(EmitPoint {
                pos: fref.start,
                kind: EmitKind::FootnoteRef { def_index: fref.def_index },
                end: fref.end,
            });
        }

        // Add backslash escapes and hard breaks
        // Note: Hard breaks inside code spans should not be processed
        for mark in marks {
            // Skip marks inside code spans
            let in_code = mark.flags & flags::IN_CODE != 0;

            if mark.ch == b'\\' && mark.flags & flags::POTENTIAL_OPENER != 0 {
                // Check if mark is inside a link's destination area (the (...) part)
                // This includes URL, title, and any whitespace between them
                let in_link_dest = !link_dest_ranges.is_empty()
                    && pos_in_ranges_u32(mark.pos, link_dest_ranges, &mut link_dest_idx);
                // Check if mark is inside an autolink
                let in_autolink = !autolink_ranges.is_empty()
                    && pos_in_ranges_u32(mark.pos, autolink_ranges, &mut autolink_idx);
                let in_html = !html_ranges.is_empty()
                    && pos_in_ranges_u32(mark.pos, html_ranges, &mut html_idx);

                let escaped_char = text[(mark.pos + 1) as usize];
                if escaped_char == b'\n' && !in_code && !in_autolink && !in_html {
                    // Backslash before newline is a hard break (but not in code or autolinks)
                    emit_points.push(EmitPoint {
                        pos: mark.pos,
                        kind: EmitKind::HardBreak,
                        end: mark.end,
                    });
                } else if !in_code && !in_link_dest && !in_autolink && !in_html {
                    // Skip escapes inside link URLs/titles and autolinks (they're processed by renderer)
                    emit_points.push(EmitPoint {
                        pos: mark.pos,
                        kind: EmitKind::Escape(escaped_char),
                        end: mark.end,
                    });
                }
            } else if mark.ch == b'\n' && mark.flags & flags::POTENTIAL_OPENER != 0 {
                let in_link_dest = !link_dest_ranges.is_empty()
                    && pos_in_ranges_u32(mark.pos, link_dest_ranges, &mut link_dest_idx);
                let in_html = !html_ranges.is_empty()
                    && pos_in_ranges_u32(mark.pos, html_ranges, &mut html_idx);
                if in_code || in_link_dest || in_html {
                    continue;
                }
                // Two spaces before newline is a hard break (but not in code or link destinations)
                emit_points.push(EmitPoint {
                    pos: mark.pos,
                    kind: EmitKind::HardBreak,
                    end: mark.end,
                });
            } else if mark.ch == b'\n' && mark.flags & flags::POTENTIAL_CLOSER != 0 {
                let in_link_dest = !link_dest_ranges.is_empty()
                    && pos_in_ranges_u32(mark.pos, link_dest_ranges, &mut link_dest_idx);
                let in_html = !html_ranges.is_empty()
                    && pos_in_ranges_u32(mark.pos, html_ranges, &mut html_idx);
                if in_code || in_link_dest || in_html {
                    continue;
                }
                // Soft break (newline without 2+ spaces) - also not in code or link destinations
                emit_points.push(EmitPoint {
                    pos: mark.pos,
                    kind: EmitKind::SoftBreak,
                    end: mark.end,
                });
            }
        }

        // Sort by position (end events come after start events at same position)
        emit_points.sort_unstable_by_key(|p| (p.pos, matches!(p.kind,
            EmitKind::CodeSpanEnd | EmitKind::StrongEnd | EmitKind::EmphasisEnd |
            EmitKind::StrikethroughEnd | EmitKind::LinkEnd | EmitKind::ImageEnd
        )));

        // Build ranges to suppress (reference labels after link text)
        suppress_ranges.clear();
        suppress_ranges.reserve(resolved_ref_links.len());
        for link in resolved_ref_links {
            if link.end > link.text_end {
                suppress_ranges.push((link.text_end, link.end));
            }
        }

        // Emit events in order
        let mut skip_until = 0u32;
        let mut suppress_idx = 0usize;

        for point in emit_points.iter() {
            // Skip events inside suppressed ranges (e.g., reference labels)
            while suppress_idx < suppress_ranges.len()
                && point.pos >= suppress_ranges[suppress_idx].1
            {
                suppress_idx += 1;
            }
            if suppress_idx < suppress_ranges.len() {
                let (s, e) = suppress_ranges[suppress_idx];
                if point.pos > s && point.pos < e {
                    pos = pos.max(point.end);
                    continue;
                }
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
                EmitKind::StrikethroughStart => {
                    events.push(InlineEvent::StrikethroughStart);
                    pos = point.end;
                    skip_until = point.end;
                }
                EmitKind::StrikethroughEnd => {
                    events.push(InlineEvent::StrikethroughEnd);
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
                EmitKind::AutolinkLiteral { end, kind } => {
                    events.push(InlineEvent::AutolinkLiteral {
                        url: Range::from_usize(point.pos as usize, end as usize),
                        kind,
                    });
                    skip_until = end;
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
                EmitKind::FootnoteRef { def_index } => {
                    events.push(InlineEvent::FootnoteRef { def_index });
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

#[inline]
fn has_inline_specials(input: &[u8]) -> bool {
    #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
    {
        if let Some(result) = unsafe { simd::has_inline_specials_simd(input) } {
            return result;
        }
    }
    for &b in input {
        match b {
            b'*' | b'_' | b'`' | b'[' | b']' | b'<' | b'\\' | b'\n' | b'~' => {
                return true;
            }
            _ => {}
        }
    }
    false
}

/// Check if text might contain autolink literal triggers.
/// Uses memchr for SIMD-accelerated scanning of rare byte patterns
/// instead of matching common letters like 'h' and 'w'.
#[inline]
fn has_autolink_candidates(input: &[u8]) -> bool {
    // Check for @ (email autolinks) — rare in normal prose
    if memchr(b'@', input).is_some() {
        return true;
    }
    // Check for :// (URL autolinks: http://, https://, ftp://) — colon is rare
    let mut pos = 0;
    while let Some(offset) = memchr(b':', &input[pos..]) {
        let idx = pos + offset;
        if idx + 2 < input.len() && input[idx + 1] == b'/' && input[idx + 2] == b'/' {
            return true;
        }
        pos = idx + 1;
    }
    // Check for www. / WWW. (www autolinks)
    pos = 0;
    while pos + 3 < input.len() {
        if let Some(offset) = memchr(b'.', &input[pos + 1..]) {
            let dot = pos + 1 + offset;
            if dot >= 3 {
                let s = dot - 3;
                if (input[s] | 0x20) == b'w'
                    && (input[s + 1] | 0x20) == b'w'
                    && (input[s + 2] | 0x20) == b'w'
                {
                    return true;
                }
            }
            pos = dot + 1;
        } else {
            break;
        }
    }
    false
}

#[inline]
fn has_inline_link_opener(input: &[u8]) -> bool {
    let mut pos = 0usize;
    while let Some(offset) = memchr(b']', &input[pos..]) {
        let idx = pos + offset;
        if idx + 1 < input.len() && input[idx + 1] == b'(' {
            return true;
        }
        pos = idx + 1;
    }
    false
}

impl Default for InlineParser {
    fn default() -> Self {
        Self::new()
    }
}

/// A resolved footnote reference.
#[derive(Debug, Clone, Copy)]
struct FootnoteRef {
    /// Start position (the `[`).
    start: u32,
    /// End position (after `]`).
    end: u32,
    /// Index into the footnote store.
    def_index: u32,
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
    StrikethroughStart,
    StrikethroughEnd,
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
    AutolinkLiteral { end: u32, kind: links::AutolinkLiteralKind },
    HtmlRaw { end: u32 },
    FootnoteRef { def_index: u32 },
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
    code_ranges: &mut Vec<(usize, usize)>,
    autolink_ranges: &mut Vec<(usize, usize)>,
    spans: &mut Vec<HtmlSpan>,
) {
    spans.clear();
    let len = text.len();

    code_ranges.clear();
    code_ranges.reserve(code_spans.len().saturating_sub(code_ranges.capacity()));
    code_ranges.extend(
        code_spans
            .iter()
            .map(|cs| (cs.opener_pos as usize, cs.closer_end as usize)),
    );
    code_ranges.sort_by_key(|(s, _)| *s);

    autolink_ranges.clear();
    autolink_ranges.reserve(autolinks.len().saturating_sub(autolink_ranges.capacity()));
    autolink_ranges.extend(autolinks.iter().map(|al| (al.start as usize, al.end as usize)));
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

        if pos_in_ranges(pos, code_ranges, &mut code_idx)
            || pos_in_ranges(pos, autolink_ranges, &mut autolink_idx)
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
        parser.parse(text.as_bytes(), None, true, &mut events);
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
