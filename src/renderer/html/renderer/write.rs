//! Low-level output helpers for the HTML renderer.
//!
//! These methods sit between visitor code and raw string buffers. They centralize
//! escaping, autolinking, heading-id emission, and raw HTML handling so the visitor
//! methods can describe document structure without duplicating output mechanics.

use std::fmt::{Display, Write as _};

use crate::ast::{Heading, Node, Span};
use smallvec::SmallVec;

use super::super::abbreviation::write_abbreviations_into_with_boundaries;
use super::super::autolink::find_autolink_match;
use super::super::escape::{
    write_attribute_escaped_into, write_escaped_into, write_url_escaped_into,
};
use super::super::heading::{
    HEADING_PERMALINK_CLASS, collect_heading_text_into, heading_has_permalink_marker,
    single_text_child,
};
use super::{HtmlRenderer, reserve_heading_scratch};

pub(super) fn adjacent_text_boundary_before(children: &[Node<'_>], index: usize) -> Option<char> {
    for child in children[..index].iter().rev() {
        match child {
            Node::Text(text) => {
                if let Some(character) = text.value.chars().next_back() {
                    return Some(character);
                }
            }
            _ => break,
        }
    }
    None
}

pub(super) fn adjacent_text_boundary_after(children: &[Node<'_>], index: usize) -> Option<char> {
    for child in children.iter().skip(index + 1) {
        match child {
            Node::Text(text) => {
                if let Some(character) = text.value.chars().next() {
                    return Some(character);
                }
            }
            _ => break,
        }
    }
    None
}

impl HtmlRenderer {
    pub(in crate::renderer::html::renderer) fn write(&mut self, s: &str) {
        self.output.push_str(s);
    }

    pub(in crate::renderer::html::renderer) fn write_display(&mut self, value: impl Display) {
        let _ = write!(self.output, "{value}");
    }

    pub(in crate::renderer::html::renderer) fn write_escaped(&mut self, s: &str) {
        // Escape timing lives in `write_escaped_into` (detail span
        // `renderer::escape_text`) so the fast paths that bypass this
        // wrapper are measured too.
        write_escaped_into(&mut self.output, s);
    }

    pub(in crate::renderer::html::renderer) fn write_attribute_escaped(&mut self, s: &str) {
        write_attribute_escaped_into(&mut self.output, s);
    }

    /// Emits the optional `data-source-span` attribute.
    ///
    /// Every block visitor calls this, and the option is off by default, so
    /// the gate stays inline while the formatting body is kept out of line:
    /// the common case is one field load and a branch, not a call.
    #[inline]
    pub(in crate::renderer::html::renderer) fn write_source_span_attr(&mut self, span: Span) {
        if !self.options.source_spans || span.start == span.end {
            return;
        }
        self.write_source_span_attr_value(span);
    }

    #[inline(never)]
    fn write_source_span_attr_value(&mut self, span: Span) {
        self.write(" data-source-span=\"");
        self.write_display(span.start);
        self.write("-");
        self.write_display(span.end);
        self.write("\"");
    }

    /// Walks `s` and emits an `<a>` tag for each registered URL pattern match.
    ///
    /// The caller has already gated on the autolink option and link nesting
    /// state, so this routine can focus on the hot loop: use the per-render
    /// first-byte index to jump to possible URL starts, escape the intervening
    /// non-URL text in chunks, then write the matched URL once for `href` and
    /// once for visible text. If the index is absent, we fail open by emitting
    /// escaped text rather than rebuilding the index here.
    pub(in crate::renderer::html::renderer) fn write_text_with_autolinks(&mut self, s: &str) {
        let bytes = s.as_bytes();
        // Reuse the per-render first-byte index (see `autolink_index`). If it's
        // absent the caller's gating slipped — fall back to emitting the text
        // verbatim rather than rebuilding the index here.
        let Some(index) = self.autolink_index.as_ref() else {
            write_escaped_into(&mut self.output, s);
            return;
        };
        // A match must contain the index's gate needle (`://` for the
        // default `http://`/`https://` patterns), so one memchr plus a short
        // compare proves "nothing can match here" for almost every prose
        // node — skipping the candidate walk with its per-first-byte
        // boundary checks.
        if !index.may_match(bytes) {
            write_escaped_into(&mut self.output, s);
            return;
        }
        // Borrow the relevant fields disjointly so the URL scan (which only
        // reads `options`/`autolink_index`) and the output writes can coexist.
        // The pattern list is a slice of `Cow<str>`. Resolving each entry to
        // a plain `&str` here, once per text node that can hold a match, keeps
        // the per-candidate prefix loop in `find_autolink_match` free of the
        // `Cow` discriminant test; the common no-match path never gets here.
        let patterns: SmallVec<[&str; 4]> = self
            .options
            .autolink_patterns()
            .iter()
            .map(AsRef::as_ref)
            .collect();
        let patterns: &[&str] = &patterns;
        let target_blank = self.options.autolink_target_blank;
        let out = &mut self.output;
        let mut cursor = 0usize;
        while cursor < bytes.len() {
            let Some((match_start, url_end)) = find_autolink_match(s, cursor, patterns, index)
            else {
                break;
            };
            // Emit the literal text preceding the URL.
            if match_start > cursor {
                write_escaped_into(out, &s[cursor..match_start]);
            }
            let url = &s[match_start..url_end];
            out.push_str("<a href=\"");
            write_url_escaped_into(out, url);
            out.push('"');
            if target_blank {
                out.push_str(" target=\"_blank\" rel=\"noopener noreferrer\"");
            }
            out.push('>');
            // The visible text is the URL itself; escape it like any text.
            write_escaped_into(out, url);
            out.push_str("</a>");
            cursor = url_end;
        }
        if cursor < bytes.len() {
            write_escaped_into(out, &s[cursor..]);
        }
    }

    pub(in crate::renderer::html::renderer) fn write_url_escaped(&mut self, s: &str) {
        write_url_escaped_into(&mut self.output, s);
    }

    pub(in crate::renderer::html::renderer) fn sanitized_url<'a>(
        &self,
        url: &'a str,
        fallback: &'static str,
    ) -> &'a str {
        if !self.options.sanitize {
            return url;
        }

        let trimmed =
            url.trim_matches(|ch: char| ch.is_ascii_control() || ch.is_ascii_whitespace());

        if Self::is_safe_url(trimmed) {
            trimmed
        } else {
            fallback
        }
    }

    pub(in crate::renderer::html::renderer) fn is_safe_url(url: &str) -> bool {
        if url.bytes().any(|byte| byte.is_ascii_control()) {
            return false;
        }

        let Some(colon_index) = url.find(':') else {
            return true;
        };

        let first_path_marker = url.find(&['/', '?', '#'][..]).unwrap_or(usize::MAX);
        if first_path_marker < colon_index {
            return true;
        }

        // Every allowed scheme is ASCII and at most six bytes. Normalize
        // into bounded stack storage instead of allocating for each link.
        let mut scheme = [0; 6];
        let mut len = 0;
        for byte in url[..colon_index]
            .bytes()
            .filter(|byte| !byte.is_ascii_whitespace())
        {
            let Some(slot) = scheme.get_mut(len) else {
                return false;
            };
            *slot = byte.to_ascii_lowercase();
            len += 1;
        }

        matches!(&scheme[..len], b"http" | b"https" | b"mailto" | b"tel")
    }

    pub(in crate::renderer::html::renderer) fn write_html_value(&mut self, value: &str) {
        if self.options.sanitize {
            self.write_escaped(value);
            return;
        }

        // URL rewriting produces an owned string; the tag filter then runs
        // over whichever form we ended up with so both paths get filtered.
        let rewritten = if self.options.convert_md_links {
            Some(self.rewrite_html_root_urls(value))
        } else {
            None
        };
        let value = rewritten.as_deref().unwrap_or(value);

        if (self.options.disallow_raw_html || self.in_mdx_island_children)
            && crate::renderer::html::tagfilter::needs_filtering(value)
        {
            crate::renderer::html::tagfilter::write_filtered_into(&mut self.output, value);
        } else {
            self.write(value);
        }
    }

    /// Writes inline HTML and tracks only its surrounding inline container
    /// when abbreviation matching is enabled. Block HTML is deliberately
    /// emitted by `write_html_value` without changing this state.
    pub(in crate::renderer::html::renderer) fn write_inline_html_value(&mut self, value: &str) {
        // Keep text inside authored tags untouched even when those tags are
        // escaped by sanitization or disallowed-HTML filtering. The scope is
        // reset at the end of this inline container, so an unclosed tag cannot
        // affect later paragraphs or headings.
        self.update_raw_html_depth(value);
        self.write_html_value(value);
    }

    /// Writes an inline text value, replacing its soft breaks.
    ///
    /// A soft break is a line ending inside inline content: the newline that
    /// joins two lines of the same paragraph, heading, or table cell. The
    /// inline parser keeps it as a `\n` in the text value — folded into the
    /// surrounding run for an LF source, or as its own one-character node
    /// where the run had to stop — and turns a hard break into a
    /// [`crate::ast::Break`] node instead, so every line ending that reaches
    /// here is a soft break.
    ///
    /// The cached flag keeps the whole question off the hot path while the
    /// default `"\n"` is configured, where writing the line ending verbatim
    /// and writing the configured value are the same thing.
    #[inline]
    pub(in crate::renderer::html::renderer) fn write_inline_text(&mut self, value: &str) {
        self.write_inline_text_with_boundaries(value, None, None);
    }

    #[inline]
    pub(in crate::renderer::html::renderer) fn write_inline_text_with_boundaries(
        &mut self,
        value: &str,
        before: Option<char>,
        after: Option<char>,
    ) {
        let abbreviations_enabled = self
            .abbreviation_state
            .as_ref()
            .is_some_and(|state| state.raw_html_depth == 0 && state.mdx_depth == 0);
        if abbreviations_enabled {
            if self.options.custom_soft_break {
                self.write_inline_text_with_abbreviations_and_soft_breaks(value, before, after);
            } else {
                self.write_inline_text_run_with_abbreviations(value, before, after);
            }
        } else if self.options.custom_soft_break {
            self.write_inline_text_with_soft_breaks(value);
        } else {
            self.write_inline_text_run(value);
        }
    }

    #[inline(never)]
    fn write_inline_text_with_abbreviations_and_soft_breaks(
        &mut self,
        value: &str,
        before: Option<char>,
        after: Option<char>,
    ) {
        let mut start = 0;
        for (index, line) in value.split('\n').enumerate() {
            if index > 0 {
                self.output.push_str(self.options.soft_break());
            }
            let line_before = (start == 0).then_some(before).flatten();
            let end = start + line.len();
            let line_after = (end == value.len()).then_some(after).flatten();
            self.write_inline_text_run_with_abbreviations(line, line_before, line_after);
            start = end + 1;
        }
    }

    /// Applies abbreviation recognition only to prose around renderer URLs.
    ///
    /// A URL match is written as one link, so no generated abbreviation can
    /// enter its visible URL text or its destination.
    fn write_inline_text_run_with_abbreviations(
        &mut self,
        value: &str,
        before: Option<char>,
        after: Option<char>,
    ) {
        let Some(state) = self.abbreviation_state.as_ref() else {
            self.write_inline_text_run(value);
            return;
        };
        let matcher = &state.matcher;
        let bytes = value.as_bytes();
        let index = &state.url_index;
        if !index.may_match(bytes) {
            write_abbreviations_into_with_boundaries(
                &mut self.output,
                value,
                matcher,
                before,
                after,
            );
            return;
        }

        let patterns = state
            .url_patterns
            .iter()
            .map(String::as_str)
            .collect::<SmallVec<[&str; 4]>>();
        let out = &mut self.output;
        let mut cursor = 0usize;
        let mut before_segment = before;
        while cursor < bytes.len() {
            let Some((match_start, url_end)) = find_autolink_match(value, cursor, &patterns, index)
            else {
                break;
            };
            if match_start > cursor {
                write_abbreviations_into_with_boundaries(
                    out,
                    &value[cursor..match_start],
                    matcher,
                    before_segment,
                    value[match_start..].chars().next(),
                );
            }
            let url = &value[match_start..url_end];
            let should_link = !self.in_link
                && self.autolink_index.is_some()
                && self.options.autolink_patterns().iter().any(|pattern| {
                    url.get(..pattern.len())
                        .is_some_and(|prefix| prefix.eq_ignore_ascii_case(pattern))
                });
            if should_link {
                out.push_str("<a href=\"");
                write_url_escaped_into(out, url);
                out.push('"');
                if self.options.autolink_target_blank {
                    out.push_str(" target=\"_blank\" rel=\"noopener noreferrer\"");
                }
                out.push('>');
                write_escaped_into(out, url);
                out.push_str("</a>");
            } else {
                write_escaped_into(out, url);
            }
            before_segment = value[..url_end].chars().next_back();
            cursor = url_end;
        }
        if cursor < bytes.len() {
            write_abbreviations_into_with_boundaries(
                out,
                &value[cursor..],
                matcher,
                before_segment,
                after,
            );
        }
    }

    /// Tracks trusted raw HTML elements so their contents remain authored.
    fn update_raw_html_depth(&mut self, html: &str) {
        let Some(state) = self.abbreviation_state.as_mut() else {
            return;
        };
        let bytes = html.as_bytes();
        let mut cursor = 0usize;
        while cursor < bytes.len() {
            if bytes[cursor..].starts_with(b"<!--") {
                if let Some(end) = html[cursor + 4..].find("-->") {
                    cursor += 4 + end + 3;
                    continue;
                }
                break;
            }
            if bytes[cursor] != b'<' {
                cursor += 1;
                continue;
            }

            let mut name_start = cursor + 1;
            let closing = bytes.get(name_start) == Some(&b'/');
            if closing {
                name_start += 1;
            }
            let mut name_end = name_start;
            while bytes.get(name_end).is_some_and(|byte| {
                byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b':' | b'_')
            }) {
                name_end += 1;
            }
            let Some(name) = bytes.get(name_start..name_end).filter(|name| {
                !name.is_empty()
                    && matches!(
                        bytes.get(name_end),
                        Some(b'>')
                            | Some(b'/')
                            | Some(b' ')
                            | Some(b'\t')
                            | Some(b'\n')
                            | Some(b'\r')
                    )
            }) else {
                cursor += 1;
                continue;
            };

            let mut end = name_end;
            let mut quote = None;
            while let Some(byte) = bytes.get(end).copied() {
                match (quote, byte) {
                    (Some(current), value) if current == value => quote = None,
                    (None, b'\'' | b'"') => quote = Some(byte),
                    (None, b'>') => break,
                    _ => {}
                }
                end += 1;
            }
            if end >= bytes.len() {
                break;
            }

            if closing {
                state.raw_html_depth = state.raw_html_depth.saturating_sub(1);
            } else if !is_void_html_tag(name)
                && bytes[name_end..end]
                    .iter()
                    .rev()
                    .find(|byte| !byte.is_ascii_whitespace())
                    .is_none_or(|byte| *byte != b'/')
            {
                state.raw_html_depth = state.raw_html_depth.saturating_add(1);
            }
            cursor = end + 1;
        }
    }

    #[inline(never)]
    fn write_inline_text_with_soft_breaks(&mut self, value: &str) {
        let mut lines = value.split('\n');
        if let Some(first) = lines.next() {
            self.write_inline_text_run(first);
        }
        for line in lines {
            self.output.push_str(self.options.soft_break());
            self.write_inline_text_run(line);
        }
    }

    /// Writes one run of inline text with no line ending in it.
    #[inline]
    fn write_inline_text_run(&mut self, value: &str) {
        // See the matching gate in `visit_inline_node`: `autolink_index` is
        // `Some` exactly when `autolink_urls` holds and the pattern list is
        // not empty, so this one `Option` check replaces three field reads.
        if self.autolink_index.is_some() && !self.in_link {
            self.write_text_with_autolinks(value);
        } else {
            write_escaped_into(&mut self.output, value);
        }
    }

    pub(in crate::renderer::html::renderer) fn visit_inline_node(&mut self, node: &Node<'_>) {
        // Text is the overwhelmingly common child of paragraphs / headings
        // / links / emphasis / strong, etc. — on the bundled corpora it
        // accounts for roughly 60-70% of inline visits. Inlining the
        // write here skips the trait's 20-arm `walk_node` match and the
        // `visit_text` wrapper, both of which are the only thing
        // `visit_text` would do anyway (escape into `self.output`).
        match node {
            // The autolink builtin lives on this hot path too: when the flag
            // is on (and we're not already inside an `<a>`) we have to scan
            // the text for URLs before escaping. The common case — flag off,
            // default soft break — collapses back to a single
            // `write_escaped_into` call thanks to the early boolean checks in
            // `write_inline_text`.
            Node::Text(text) => self.write_inline_text(text.value),
            Node::Html(html) => self.write_inline_html_value(html.value),
            _ => self.render_node(node),
        }
    }

    /// Visits inline children while treating adjacent plain text as one token
    /// stream for abbreviation boundary checks.
    pub(in crate::renderer::html::renderer) fn render_inline_children(
        &mut self,
        children: &[Node<'_>],
    ) {
        if self.abbreviation_state.is_none() {
            for child in children {
                self.visit_inline_node(child);
            }
            return;
        }

        self.begin_inline_abbreviation_scope();
        for (index, child) in children.iter().enumerate() {
            if let Node::Text(text) = child {
                self.write_inline_text_with_boundaries(
                    text.value,
                    adjacent_text_boundary_before(children, index),
                    adjacent_text_boundary_after(children, index),
                );
            } else {
                self.visit_inline_node(child);
            }
        }
        self.end_inline_abbreviation_scope();
    }

    pub(in crate::renderer::html::renderer) fn begin_inline_abbreviation_scope(&mut self) {
        if let Some(state) = self.abbreviation_state.as_mut() {
            if state.inline_depth == 0 {
                state.raw_html_depth = 0;
            }
            state.inline_depth = state.inline_depth.saturating_add(1);
        }
    }

    pub(in crate::renderer::html::renderer) fn end_inline_abbreviation_scope(&mut self) {
        if let Some(state) = self.abbreviation_state.as_mut() {
            state.inline_depth = state.inline_depth.saturating_sub(1);
            if state.inline_depth == 0 {
                state.raw_html_depth = 0;
            }
        }
    }

    /// Plans the heading's unique id and writes it into `self.output`.
    ///
    /// Permalinks read the same planned id back from the planner so the
    /// `href` matches the `id` attribute, including duplicate `-N` suffixes.
    pub(in crate::renderer::html::renderer) fn write_heading_id(&mut self, heading: &Heading<'_>) {
        self.prepare_heading_id(heading);
        self.write_prepared_heading_id();
    }

    /// Emits the planned ID, including any configured prefix.
    ///
    /// Only an author-supplied `{#id}` can contain a byte that attribute
    /// escaping replaces. A generated slug is lowercase alphanumerics, `-`,
    /// and an optional `-N` suffix, so running the CR/LF `memchr2` pass and
    /// the escape scanner over it can only ever copy it back unchanged — the
    /// `heading_id_is_explicit` flag lets that whole pass be skipped.
    ///
    /// The prefix is validated to ASCII letters, digits, `_`, and `-`, so it
    /// needs no escaping. Explicit IDs still go through the attribute escaper.
    fn write_prepared_heading_id(&mut self) {
        let id = self.heading_id_planner.id(self.heading_id);
        if self.heading_id_is_explicit {
            write_attribute_escaped_into(&mut self.output, id);
        } else {
            self.output.push_str(id);
        }
    }

    pub(in crate::renderer::html::renderer) fn write_heading_permalink_if_needed(
        &mut self,
        heading: &Heading<'_>,
    ) {
        if !self.options.heading_ids || !self.options.heading_permalinks {
            return;
        }
        if heading_has_permalink_marker(
            &heading.children,
            "",
            self.heading_id_planner.id(self.heading_id),
        ) {
            return;
        }
        self.output.push_str("<a class=\"");
        self.output.push_str(HEADING_PERMALINK_CLASS);
        self.output.push_str("\" href=\"#");
        // The `href` fragment is the same id the `id` attribute just emitted,
        // so it takes the same verbatim/escaped decision.
        self.write_prepared_heading_id();
        if self.heading_text_scratch.is_empty() {
            self.output
                .push_str("\" aria-label=\"Permalink to this section\">#</a>");
            return;
        }
        self.output.push_str("\" aria-label=\"Permalink to &quot;");
        write_escaped_into(&mut self.output, &self.heading_text_scratch);
        self.output.push_str("&quot;\">#</a>");
    }

    fn prepare_heading_id(&mut self, heading: &Heading<'_>) {
        // `heading_text_scratch` has exactly two readers: the slugifier and
        // the permalink's `aria-label`. Most headings are a single `Text`
        // child, which the slugifier can read straight out of the source, and
        // an explicit `{#id}` skips the slugifier altogether — so the
        // concatenation only has to run when a reader will actually see its
        // result. The buffer is still cleared on every heading so that a
        // skipped fill can never leave the previous heading's text where the
        // permalink would read it.
        let single_text = single_text_child(&heading.children);
        let permalink_reads_text = self.options.heading_permalinks;
        self.heading_text_scratch.clear();
        if permalink_reads_text || (heading.id.is_none() && single_text.is_none()) {
            reserve_heading_scratch(&mut self.heading_text_scratch);
            collect_heading_text_into(&heading.children, &mut self.heading_text_scratch);
        }

        // The planner keeps the full emitted ID in its own storage, so neither
        // path copies the ID into a renderer buffer.
        if let Some(id) = heading.id {
            self.heading_id_is_explicit = true;
            let prefix = self.options.heading_id_prefix();
            self.heading_id = if prefix.is_empty() {
                self.heading_id_planner.claim(id)
            } else {
                self.heading_id_planner.claim_with_prefix(prefix, id)
            };
            return;
        }
        self.heading_id_is_explicit = false;
        // A single `Text` child is slugified straight from the source; the
        // slug itself is written into the planner's storage, where a
        // not-yet-taken slug is claimed in place.
        let text = single_text.unwrap_or(&self.heading_text_scratch);
        let prefix = self.options.heading_id_prefix();
        self.heading_id = if prefix.is_empty() {
            self.heading_id_planner.claim_slug(text)
        } else {
            self.heading_id_planner.claim_slug_with_prefix(prefix, text)
        };
    }
}

fn is_void_html_tag(name: &[u8]) -> bool {
    [
        b"area".as_slice(),
        b"base",
        b"br",
        b"col",
        b"embed",
        b"hr",
        b"img",
        b"input",
        b"link",
        b"meta",
        b"param",
        b"source",
        b"track",
        b"wbr",
    ]
    .iter()
    .any(|void| name.eq_ignore_ascii_case(void))
}

#[cfg(test)]
mod heading_equivalence;
#[cfg(test)]
mod tests;
