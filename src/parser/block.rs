use crate::ast::{Heading, Node, Paragraph, Span};
use memchr::{memchr, memchr3};

use super::Parser;
use super::line_scan::{
    is_line_ending_byte, line_end as scan_line_end, line_terminator_end,
    next_line_start as scan_next_line_start,
};
use super::whitespace;
use crate::parser::error::{ParseErrorKind, ParseResult};

impl<'a> Parser<'a> {
    pub(super) fn parse_block(&mut self) -> ParseResult<Option<Node<'a>>> {
        // The blank-line walk stops on the first non-space, non-tab byte of
        // the line it settles on, so it already knows what the dispatcher
        // needs to discriminate on.
        let first_non_whitespace = self.skip_blank_lines();

        if self.is_at_end() {
            return Ok(None);
        }

        let start = self.position;
        let bytes = self.source.as_bytes();
        debug_assert_eq!(
            first_non_whitespace,
            self.first_non_whitespace_in_line(start),
            "the blank-line walk must report what a fresh indentation scan finds"
        );
        let Some(trimmed_start) = first_non_whitespace else {
            // Nothing but whitespace remains on this line. `skip_blank_lines`
            // normally consumes it, so reaching here means the line ends at
            // EOF; advance past it regardless so the caller's
            // `while !is_at_end` loop always makes progress.
            self.position = self.source.len();
            return Ok(None);
        };

        // Dedented comment lines retain their slashes. Reuse the prefix
        // already needed by block dispatch before consulting physical origins.
        if bytes[trimmed_start] == b'/' && self.is_line_comment_at(start) {
            self.position = self.skip_line_comments_from(start);
            return Ok(None);
        }

        // Comments are skipped before enforcing the nesting bound, as before.
        if self.options.max_nesting_depth > 0 && self.nesting_depth > self.options.max_nesting_depth
        {
            return Err(ParseErrorKind::NestingTooDeep {
                span: Span::new(self.position as u32, self.position as u32),
                max_depth: self.options.max_nesting_depth,
            }
            .into());
        }

        // Four columns of indentation start an indented code block; no
        // other block construct can begin on such a line. (This runs at
        // block level only — an indented line after an open paragraph is
        // lazy continuation, handled by `parse_paragraph`.)
        let line_indent = self.line_indent_width(start, trimmed_start);
        if line_indent >= 4 {
            return self.parse_indented_code(start);
        }

        // Fast block dispatch.
        //
        // Most documentation lines are plain paragraph text. The old shape
        // built `line` and `trimmed` up front, then tried each block parser in
        // sequence; that meant every paragraph paid for newline search,
        // trimming, and several failed recognizers. Here the first
        // non-whitespace byte is used as a cheap discriminator. Only marker
        // families that can actually begin with that byte materialize the
        // full line slice and run their more expensive syntax checks.
        //
        // Keep this table in sync with `line_starts_block`: paragraph parsing
        // uses that helper to decide when a following line terminates the
        // paragraph, so the two dispatchers must agree on block starts.
        //
        // Arms that materialize the line record where it ends, so the table
        // guard and the paragraph loop below reuse that offset instead of
        // searching the same bytes for the terminator again.
        let mut line_end_hint = None;
        match bytes[trimmed_start] {
            b'#' if self.try_parse_heading_start(start, trimmed_start) => {
                return self.parse_heading(start);
            }
            b'-' | b'*' => {
                let line = self.line_at(start);
                line_end_hint = Some(start + line.len());
                let trimmed = &line[trimmed_start - start..];
                if Self::try_parse_thematic_break_line(line) {
                    return self.parse_thematic_break(start, start + line.len());
                }
                if let Some(first_item) =
                    self.parse_list_item_line_from_trimmed(start, line, trimmed)
                {
                    return self.parse_list(start, line_indent, first_item);
                }
            }
            b'_' => {
                let line = self.line_at(start);
                line_end_hint = Some(start + line.len());
                if Self::try_parse_thematic_break_line(line) {
                    return self.parse_thematic_break(start, start + line.len());
                }
            }
            b'>' => return self.parse_block_quote(start),
            b'`' | b'~' => {
                let line = self.line_at(start);
                line_end_hint = Some(start + line.len());
                let trimmed = &line[trimmed_start - start..];
                if Self::try_parse_fenced_code_at(line, trimmed) {
                    return self.parse_fenced_code(start, line_indent);
                }
            }
            b'$' if self.options.math => {
                let line = self.line_at(start);
                line_end_hint = Some(start + line.len());
                let trimmed = &line[trimmed_start - start..];
                if self.try_parse_math_block_at(start, line, trimmed) {
                    return self.parse_math_block(start);
                }
            }
            b'{' => {
                if self.options.mdx
                    && let Some(node) = self.try_parse_mdx_flow_expression(start, trimmed_start)?
                {
                    return Ok(Some(node));
                }
            }
            b'<' => {
                if self.options.mdx
                    && let Some(node) = self.try_parse_mdx_jsx_flow(start, trimmed_start)?
                {
                    return Ok(Some(node));
                }
                let line = self.line_at(start);
                line_end_hint = Some(start + line.len());
                let trimmed = &line[trimmed_start - start..];
                if let Some(html_start) = Self::parse_html_block_start(trimmed) {
                    return self.parse_html_block(start, html_start);
                }
                // Type-7 blocks (a lone complete tag) start blocks but can
                // never interrupt a paragraph, so only this dispatcher —
                // not line_starts_block — recognizes them.
                if Self::is_html_block_type7_line(trimmed) {
                    return self.parse_html_block(start, super::html::HtmlBlockStart::Other);
                }
            }
            b'+' | b'0'..=b'9' => {
                let line = self.line_at(start);
                line_end_hint = Some(start + line.len());
                let trimmed = &line[trimmed_start - start..];
                if let Some(first_item) =
                    self.parse_list_item_line_from_trimmed(start, line, trimmed)
                {
                    return self.parse_list(start, line_indent, first_item);
                }
            }
            b'i' | b'e' => {
                if self.options.mdx
                    && let Some(node) = self.try_parse_mdxjs_esm(start, trimmed_start)
                {
                    return Ok(Some(node));
                }
            }
            _ => {}
        }

        // Table recognition is the one feature that cannot be decided from
        // the first byte because table headers usually look like ordinary
        // paragraph text. Guard the expensive two-line delimiter check with
        // a same-line `|` probe so non-table prose does one marker scan and
        // then falls through to paragraph parsing — carrying the line end
        // that scan reached, which is the first thing `parse_paragraph`
        // needs.
        let first_line_end = if !self.options.tables {
            line_end_hint.map(|end| line_terminator_end(bytes, end))
        } else if let Some(end) = line_end_hint {
            // The line is already bounded, so the guard only has to ask
            // whether a pipe lives inside it.
            if memchr(b'|', &bytes[start..end]).is_some() && self.try_parse_table() {
                return self.parse_table(start);
            }
            Some(line_terminator_end(bytes, end))
        } else {
            match memchr3(b'|', b'\n', b'\r', &bytes[start..]) {
                Some(off) if bytes[start + off] == b'|' => {
                    if self.try_parse_table() {
                        return self.parse_table(start);
                    }
                    // The pipe stopped the scan short of the terminator;
                    // finishing the line from there keeps this to one pass
                    // and spares `parse_paragraph` a fresh one.
                    Some(line_terminator_end(
                        bytes,
                        scan_line_end(bytes, start + off + 1),
                    ))
                }
                Some(off) => Some(line_terminator_end(bytes, start + off)),
                None => Some(self.source.len()),
            }
        };
        debug_assert!(
            first_line_end.is_none_or(|end| end == scan_next_line_start(bytes, start)),
            "a reported first line end must match a fresh line scan"
        );

        // Footnote definitions share the `[label]:` shape with link
        // reference definitions, so they get first refusal when the
        // extension is on; otherwise `[^1]: text` would be swallowed as a
        // link reference with label `^1`.
        if bytes[trimmed_start] == b'['
            && self.at_footnote_definition(start)
            && let Some(node) = self.try_parse_footnote_definition_node()?
        {
            return Ok(Some(node));
        }

        // Link reference definitions look like paragraphs but are
        // consumed as their own (non-rendered) nodes.
        if self.options.allow_link_refs
            && bytes[trimmed_start] == b'['
            && let Some(node) = self.try_parse_definition_node()
        {
            return Ok(Some(node));
        }

        if let Some(node) = self.parse_definition_list(start)? {
            return Ok(Some(node));
        }

        // Default: parse as paragraph.
        self.parse_paragraph(start, first_line_end)
    }

    /// Dispatch once so the disabled paragraph loop carries no comment
    /// discovery or comment-map state. Also used by the math fallback.
    #[inline]
    pub(super) fn parse_paragraph(
        &mut self,
        start: usize,
        first_line_end: Option<usize>,
    ) -> ParseResult<Option<Node<'a>>> {
        if self.options.line_comments {
            self.parse_paragraph_impl::<true>(start, first_line_end)
        } else {
            self.parse_paragraph_impl::<false>(start, first_line_end)
        }
    }

    fn parse_paragraph_impl<const LINE_COMMENTS: bool>(
        &mut self,
        start: usize,
        first_line_end: Option<usize>,
    ) -> ParseResult<Option<Node<'a>>> {
        let bytes = self.source.as_bytes();

        // Block dispatch and the math fallback reach here after
        // `skip_blank_lines` + its block dispatch — the very checks
        // `line_starts_block` re-runs — have already classified the current
        // line as a non-blank, non-block paragraph line. So consume the first
        // line unconditionally instead of re-deriving that verdict with
        // another `current_line` memchr + `trim_start` + dispatch (+ table
        // `memchr`). This also removes the infinite-loop hazard the two
        // dispatchers guard against: by always advancing past line one we can
        // never return `Ok(None)` without progress on a non-blank line.
        let mut content_end = match first_line_end {
            Some(end) => end,
            None => scan_next_line_start(bytes, start),
        };
        self.position = content_end;
        let mut first_line_comment = None;

        loop {
            if self.is_at_end() {
                break;
            }

            // Check for blank line (paragraph end): scan whitespace and
            // peek the next byte. Cheaper than the prior
            // `skip_whitespace` + `peek` + reset dance.
            let line_start = self.position;
            let mut cursor = line_start;
            while cursor < bytes.len() && matches!(bytes[cursor], b' ' | b'\t') {
                cursor += 1;
            }
            if cursor >= bytes.len() || is_line_ending_byte(bytes[cursor]) {
                break;
            }

            if LINE_COMMENTS && bytes[cursor] == b'/' && self.is_line_comment_at(line_start) {
                first_line_comment.get_or_insert(line_start);
                self.position = self.skip_line_comments_from(line_start);
                continue;
            }

            // Setext heading underline: while a paragraph is open this
            // takes precedence over every block start (`Foo\n---` is an
            // h2, not a paragraph followed by a thematic break), so it
            // must be checked before `line_starts_block`.
            if let Some(depth) = self.setext_underline_depth(line_start, cursor) {
                let heading_end = scan_next_line_start(bytes, line_start);
                self.position = heading_end;
                if LINE_COMMENTS
                    && let Some(first_comment) =
                        first_line_comment.filter(|&line| line < content_end)
                {
                    return self.parse_commented_paragraph(
                        start,
                        content_end,
                        first_comment,
                        Some((depth, heading_end)),
                    );
                }
                let (content, leading) =
                    whitespace::trim_with_leading(&self.source[start..content_end]);
                let (content, id, classes) = self.split_heading_attributes(content);
                let children = self.parse_inline_block(content, start + leading)?;
                return Ok(Some(Node::Heading(self.allocator.boxed(Heading {
                    depth,
                    id,
                    classes,
                    children,
                    span: Span::new(start as u32, heading_end as u32),
                }))));
            }

            // Check for block-level element that would end paragraph. The
            // probe's table scan usually reaches the newline, in which case
            // consuming the line costs no further scanning.
            let probe = self.probe_line(line_start, cursor);
            if probe.starts_block {
                break;
            }

            content_end = match probe.line_end {
                Some(line_end) if line_end < bytes.len() => line_terminator_end(bytes, line_end),
                Some(_) => self.source.len(),
                None => scan_next_line_start(bytes, line_start),
            };
            self.position = content_end;
        }

        // A paragraph has no block children to contribute to definition
        // collection. Its boundary must still be parsed by the real grammar.
        if self.phase == super::ParsePhase::Definitions {
            return Ok(None);
        }

        if LINE_COMMENTS
            && let Some(first_comment) = first_line_comment.filter(|&line| line < content_end)
        {
            return self.parse_commented_paragraph(start, content_end, first_comment, None);
        }
        // Inline spans start at the content, past the indentation that
        // `start` still includes.
        let (content, leading) = whitespace::trim_with_leading(&self.source[start..content_end]);
        if content.is_empty() {
            return Ok(None);
        }
        let span = Span::new(start as u32, content_end as u32);
        let children = self.parse_inline_block(content, start + leading)?;
        Ok(Some(Node::Paragraph(
            self.allocator.boxed(Paragraph { children, span }),
        )))
    }

    /// Only paragraphs with an observed, included comment need filtering and
    /// span remapping. Keep that state out of the ordinary borrowed-source
    /// path, including when comments are enabled but absent. A trailing
    /// comment at or beyond `content_end` is already outside the text slice.
    #[inline(never)]
    fn parse_commented_paragraph(
        &self,
        start: usize,
        content_end: usize,
        first_comment: usize,
        heading: Option<(u8, usize)>,
    ) -> ParseResult<Option<Node<'a>>> {
        let (raw_content, source_map) =
            self.without_line_comments_with_first(start, content_end, Some(first_comment));
        let (content, leading) = whitespace::trim_with_leading(raw_content);
        if heading.is_none() && content.is_empty() {
            return Ok(None);
        }
        let (content, id, classes) = if heading.is_some() {
            self.split_heading_attributes(content)
        } else {
            (content, None, self.allocator.new_vec())
        };
        // A remapped copy starts its own coordinates at zero; a borrowed
        // slice starts at `start`.
        let offset = if source_map.is_some() {
            leading
        } else {
            start + leading
        };
        let mut children =
            self.parse_inline_block(content, offset)
                .map_err(|error| match &source_map {
                    Some(map) => error.remapped(map),
                    None => error,
                })?;
        if let Some(map) = source_map {
            for child in &mut children {
                map.remap_node_spans(child);
            }
        }
        let node = if let Some((depth, heading_end)) = heading {
            Node::Heading(self.allocator.boxed(Heading {
                depth,
                id,
                classes,
                children,
                span: Span::new(start as u32, heading_end as u32),
            }))
        } else {
            Node::Paragraph(self.allocator.boxed(Paragraph {
                children,
                span: Span::new(start as u32, content_end as u32),
            }))
        };
        Ok(Some(node))
    }

    /// Returns the setext heading depth (1 for `=`, 2 for `-`) when the
    /// line starting at `line_start` is a setext underline: at most three
    /// leading spaces, a run of a single marker character, and nothing but
    /// trailing whitespace. `first_non_ws` is the position of the line's
    /// first non-space/tab byte (already computed by the paragraph loop).
    fn setext_underline_depth(&self, line_start: usize, first_non_ws: usize) -> Option<u8> {
        let bytes = self.source.as_bytes();
        // The marker byte rejects on the first load for essentially every
        // paragraph line, so it goes ahead of the indent walk and the lazy
        // set lookup rather than after them.
        let marker = bytes[first_non_ws];
        let depth = match marker {
            b'=' => 1,
            b'-' => 2,
            _ => return None,
        };
        // A tab in the indent always reaches column 4+, so spaces only.
        if first_non_ws - line_start > 3
            || bytes[line_start..first_non_ws]
                .iter()
                .any(|&byte| byte != b' ')
        {
            return None;
        }
        // A lazily-continued line is paragraph text by construction and
        // can never underline the paragraph it continues.
        if self
            .lazy_lines
            .as_ref()
            .is_some_and(|lazy_lines| lazy_lines.contains(&(line_start as u32)))
        {
            return None;
        }
        let mut i = first_non_ws;
        while i < bytes.len() && bytes[i] == marker {
            i += 1;
        }
        while i < bytes.len() && matches!(bytes[i], b' ' | b'\t') {
            i += 1;
        }
        if i < bytes.len() && !is_line_ending_byte(bytes[i]) {
            return None;
        }
        Some(depth)
    }
}
