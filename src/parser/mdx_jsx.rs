//! MDX JSX parse: PascalCase / member names, fragments, spreads, expressions.

use crate::ast::{MdxJsxFlowElement, MdxJsxTextElement, Node, Span};

use super::whitespace;
use super::{JsxCloserGap, Parser};
use crate::parser::error::ParseResult;

mod braces;
mod children;
mod expression;
mod scan;

pub(super) fn looks_like_jsx_open(bytes: &[u8], at: usize, compatible: bool) -> bool {
    if compatible {
        scan::looks_like_compatible_jsx_open(bytes, at)
    } else {
        scan::looks_like_jsx_open(bytes, at)
    }
}

impl<'a> Parser<'a> {
    /// Parses a flow JSX element starting at the current line.
    ///
    /// On failure the cursor is left unchanged so HTML / paragraph dispatch
    /// can run. Unclosed tags do not panic or emit a half-parsed node.
    pub(super) fn try_parse_mdx_jsx_flow(
        &mut self,
        start: usize,
        trimmed_start: usize,
    ) -> ParseResult<Option<Node<'a>>> {
        if !self.options.mdx
            || !looks_like_jsx_open(
                self.source.as_bytes(),
                trimmed_start,
                self.options.mdx_compatible,
            )
        {
            return Ok(None);
        }
        if self.options.mdx_compatible
            && super::inline::autolink_end(self.source, trimmed_start).is_some()
        {
            return Ok(None);
        }

        let source = self.source;
        let mut attributes = self.allocator.new_vec();
        let Some(open) =
            self.scan_mdx_jsx_open(source, trimmed_start, 0, &mut attributes, &mut |at| {
                self.matching_brace_end(source, at)
            })?
        else {
            return Ok(None);
        };

        let (self_closing, children, element_end) = if open.self_closing {
            if !scan::only_ws_until_eol(self.source.as_bytes(), open.end) {
                return Ok(None);
            }
            (true, self.allocator.new_vec(), open.end)
        } else {
            let Some((close_start, close_end)) =
                self.mdx_jsx_close(self.source, open.end, open.name)?
            else {
                return Ok(None);
            };
            // A flow element owns its line: anything but whitespace after
            // the matching closer makes the tag text JSX inside a
            // paragraph, the same way a tag that does not start the line
            // is handled. Checked before the children are parsed so the
            // rejected line costs nothing.
            if !scan::only_ws_until_eol(self.source.as_bytes(), close_end) {
                return Ok(None);
            }
            let children = self.parse_jsx_flow_children(open.end, close_start)?;
            (false, children, close_end)
        };

        self.position = scan::after_trailing_line_ws(self.source.as_bytes(), element_end);
        Ok(Some(Node::MdxJsxFlowElement(self.allocator.boxed(
            MdxJsxFlowElement {
                name: open.name,
                attributes,
                children,
                self_closing,
                span: Span::new(start as u32, self.position as u32),
            },
        ))))
    }

    /// Parses a text JSX element at `pos` inside inline `content`.
    pub(super) fn try_parse_mdx_jsx_text(
        &self,
        content: &'a str,
        pos: usize,
        offset: usize,
    ) -> ParseResult<Option<(Node<'a>, usize)>> {
        if !self.options.mdx
            || !looks_like_jsx_open(content.as_bytes(), pos, self.options.mdx_compatible)
        {
            return Ok(None);
        }

        let mut attributes = self.allocator.new_vec();
        let Some(open) =
            self.scan_mdx_jsx_open(content, pos, offset, &mut attributes, &mut |at| {
                self.matching_brace_end(content, at)
            })?
        else {
            return Ok(None);
        };

        let (self_closing, children, end) = if open.self_closing {
            (true, self.allocator.new_vec(), open.end)
        } else {
            let Some((close_start, close_end)) =
                self.mdx_jsx_close(content, open.end, open.name)
                    .map_err(|error| error.remapped(&super::spans::OffsetMap(offset as u32)))?
            else {
                return Ok(None);
            };
            let children =
                self.parse_jsx_phrasing(&content[open.end..close_start], offset + open.end)?;
            (false, children, close_end)
        };

        Ok(Some((
            Node::MdxJsxTextElement(self.allocator.boxed(MdxJsxTextElement {
                name: open.name,
                attributes,
                children,
                self_closing,
                span: Span::new((offset + pos) as u32, (offset + end) as u32),
            })),
            end,
        )))
    }

    /// The closing tag that matches the opening tag ending at `from`, which
    /// is what an opening tag needs before it can be a node at all.
    ///
    /// The answer is the one `scan::find_matching_close` reaches. One walk
    /// settles it for every opener it passes, so a run of them — `<A>`
    /// repeated nests one level per tag — costs one walk instead of one
    /// each, and the walk that answers doubles as the search.
    fn mdx_jsx_close(
        &self,
        content: &'a str,
        from: usize,
        name: Option<&'a str>,
    ) -> ParseResult<Option<(usize, usize)>> {
        #[cfg(feature = "jsx")]
        if self.options.mdx_compatible {
            return match scan::compatible_matching_close(content, from, name, &mut |at| {
                self.matching_brace_end(content, at)
            }) {
                Ok(Some(close)) => Ok(Some(close)),
                Ok(None) => Err(super::mdx_compatible::invalid(
                    Span::new(from as u32, content.len() as u32),
                    "unclosed JSX element; expected its matching closing tag",
                )),
                Err(at) => Err(super::mdx_compatible::invalid(
                    Span::new(at as u32, (at + 1) as u32),
                    "invalid JSX nesting or JavaScript expression; closing tags must match the most recent opening tag",
                )),
            };
        }
        let slice = (content.as_ptr() as usize, content.len());
        let cached = self
            .extension_memos()
            .mdx_jsx_closer_presence
            .borrow()
            .get(&(slice.0, slice.1, from, name))
            .copied();
        if let Some(close) = cached {
            return Ok(close);
        }
        if let Some(gap) = self.extension_memos().mdx_jsx_closer_gap.get()
            && gap.slice == slice
            && gap.name == name
            && gap.from <= from
            && from < gap.until
        {
            return Ok(None);
        }
        let walk = scan::record_matching_closes(
            content,
            from,
            name,
            &mut |at| self.matching_brace_end(content, at),
            &mut |opener, close| {
                self.extension_memos()
                    .mdx_jsx_closer_presence
                    .borrow_mut()
                    .insert((slice.0, slice.1, opener, name), close);
            },
        );
        if walk.close.is_none() {
            let (from, until) = walk.read;
            self.extension_memos()
                .mdx_jsx_closer_gap
                .set(Some(JsxCloserGap {
                    slice,
                    name,
                    from,
                    until,
                }));
        }
        Ok(walk.close)
    }

    fn scan_mdx_jsx_open(
        &self,
        content: &'a str,
        pos: usize,
        offset: usize,
        attributes: &mut crate::allocator::Vec<'a, crate::ast::MdxJsxAttributeEntry<'a>>,
        skip_brace: &mut impl FnMut(usize) -> Option<usize>,
    ) -> ParseResult<Option<scan::JsxOpen<'a>>> {
        #[cfg(feature = "jsx")]
        if self.options.mdx_compatible {
            let open = scan::scan_compatible_jsx_open(content, pos, offset, attributes, skip_brace)
                .ok_or_else(|| super::mdx_compatible::invalid(
                    Span::new((offset + pos) as u32, (offset + content.len()) as u32),
                    "invalid or unclosed JSX opening tag; quote attribute literals and close JavaScript expressions",
                ))?;
            super::mdx_compatible::validate_open(
                &content[pos..open.end],
                open.self_closing,
                Span::new((offset + pos) as u32, (offset + open.end) as u32),
            )?;
            return Ok(Some(open));
        }
        Ok(scan::scan_jsx_open(
            content, pos, offset, attributes, skip_brace,
        ))
    }

    /// The byte after the `}` that closes the `{` at `start`, or `None`
    /// when nothing in `content` closes it.
    ///
    /// One walk decides every brace it passes, so a run of `{` costs one
    /// walk in total. See `braces::record_brace_matches`.
    ///
    /// The record is kept as a distance, not as an offset: the same bytes
    /// are asked about both as a slice of the source and as a slice of a
    /// paragraph's content, which name one range under the same addresses
    /// but count from different zeros. The window is keyed by the end of
    /// the slice as well, because a slice that stops earlier is a slice a
    /// closer can fall outside of.
    pub(super) fn matching_brace_end(&self, content: &'a str, start: usize) -> Option<usize> {
        let base = content.as_ptr() as usize;
        let end = base + content.len();
        let brace = base + start;
        let cached = self
            .extension_memos()
            .brace_matches
            .borrow()
            .get(&(brace, end))
            .copied();
        if let Some(distance) = cached {
            return distance.map(|distance| start + distance);
        }
        #[cfg(feature = "jsx")]
        if self.options.mdx_compatible {
            let close = super::mdx_compatible::expression_end(content, start);
            self.extension_memos()
                .brace_matches
                .borrow_mut()
                .insert((brace, end), close.map(|close| close - start));
            return close;
        }
        if let Some((slice_end, from, until)) = self.extension_memos().brace_gap.get()
            && slice_end == end
            && from <= brace
            && brace < until
        {
            return None;
        }
        let walk = braces::record_brace_matches(content.as_bytes(), start, &mut |brace, close| {
            self.extension_memos()
                .brace_matches
                .borrow_mut()
                .insert((base + brace, end), close.map(|close| close - brace));
        });
        if walk.close.is_none() {
            let (from, until) = walk.read;
            self.extension_memos()
                .brace_gap
                .set(Some((end, base + from, base + until)));
        }
        walk.close
    }

    fn parse_jsx_phrasing(
        &self,
        content: &'a str,
        offset: usize,
    ) -> ParseResult<crate::allocator::Vec<'a, Node<'a>>> {
        self.parse_inline(content, offset)
    }

    fn parse_jsx_flow_children(
        &self,
        inner_start: usize,
        inner_end: usize,
    ) -> ParseResult<crate::allocator::Vec<'a, Node<'a>>> {
        if inner_start >= inner_end || whitespace::is_blank(&self.source[inner_start..inner_end]) {
            return Ok(self.allocator.new_vec());
        }
        let inner = &self.source[inner_start..inner_end];
        #[cfg(feature = "jsx")]
        let child_source = if self.options.mdx_compatible {
            let protected = self.mdx_javascript_ranges(inner);
            children::normalize_indentation_with_protected(self.allocator, inner, &protected)
        } else {
            children::normalize_indentation(self.allocator, inner)
        };
        #[cfg(not(feature = "jsx"))]
        let child_source = children::normalize_indentation(self.allocator, inner);
        let sub =
            self.sub_parser_with_lazy_lines(child_source.source, rustc_hash::FxHashSet::default());
        let mut children = sub
            .parse()
            .map_err(|error| match &child_source.offsets {
                Some(offsets) => children::remap_error(error, inner_start as u32, offsets),
                None => error.remapped(&super::spans::OffsetMap(inner_start as u32)),
            })?
            .children;
        for child in &mut children {
            if let Some(offsets) = &child_source.offsets {
                children::remap_node_spans(child, inner_start as u32, offsets);
            } else {
                Self::offset_node_spans(child, inner_start as u32);
            }
        }
        Ok(children)
    }

    /// Protects JS continuations from JSX child indentation normalization.
    /// The first line still loses the Markdown container indent; subsequent
    /// bytes inside a template/string/expression remain exactly authored.
    #[cfg(feature = "jsx")]
    fn mdx_javascript_ranges(&self, content: &'a str) -> smallvec::SmallVec<[(usize, usize); 8]> {
        let mut ranges = smallvec::SmallVec::new();
        let bytes = content.as_bytes();
        let mut cursor = 0;
        let mut line_start = 0;
        while cursor < bytes.len() {
            let previous = cursor;
            if matches!(bytes[cursor], b'i' | b'e')
                && bytes[line_start..cursor]
                    .iter()
                    .all(|byte| matches!(byte, b' ' | b'\t'))
                && super::mdx_esm::looks_like_esm(bytes, cursor)
                && let Ok(end) = super::mdx_compatible::esm_end(content, cursor)
            {
                ranges.push((cursor, end));
                if let Some(relative) = bytes[cursor..end]
                    .iter()
                    .rposition(|byte| matches!(byte, b'\n' | b'\r'))
                {
                    line_start = cursor + relative + 1;
                }
                cursor = end;
                continue;
            }
            match bytes[cursor] {
                b'\\' => cursor = (cursor + 2).min(bytes.len()),
                b'`' | b'~' => {
                    cursor = scan::compatible_fence_end(content, cursor)
                        .or_else(|| {
                            (bytes[cursor] == b'`')
                                .then(|| braces::skip_backticks(bytes, cursor))
                                .flatten()
                        })
                        .unwrap_or(cursor + 1);
                }
                b'{' => {
                    if let Some(end) = self.matching_brace_end(content, cursor) {
                        ranges.push((cursor, end));
                        cursor = end;
                    } else {
                        cursor += 1;
                    }
                }
                b'<' if looks_like_jsx_open(bytes, cursor, true) => {
                    if let Some(end) = super::inline::autolink_end(content, cursor) {
                        cursor = end;
                        continue;
                    }
                    let mut attributes = self.allocator.new_vec();
                    if let Some(open) = scan::scan_compatible_jsx_open(
                        content,
                        cursor,
                        0,
                        &mut attributes,
                        &mut |at| self.matching_brace_end(content, at),
                    ) {
                        ranges.push((cursor, open.end));
                        cursor = open.end;
                    } else {
                        cursor += 1;
                    }
                }
                _ => cursor += 1,
            }
            if let Some(relative) = bytes[previous..cursor]
                .iter()
                .rposition(|byte| matches!(byte, b'\n' | b'\r'))
            {
                line_start = previous + relative + 1;
            }
        }
        ranges
    }

    /// A physical paragraph line cannot end inside a JSX element or JS
    /// expression, even when its continuation includes a blank line or a
    /// character that would normally start another Markdown block.
    #[cfg(feature = "jsx")]
    pub(super) fn mdx_paragraph_end(&self, start: usize, mut end: usize) -> ParseResult<usize> {
        if !self.options.mdx_compatible {
            return Ok(end);
        }
        let bytes = self.source.as_bytes();
        let mut cursor = start;
        while cursor < end {
            match bytes[cursor] {
                b'\\' => cursor = (cursor + 2).min(end),
                b'`' => cursor = braces::skip_backticks(bytes, cursor).unwrap_or(cursor + 1),
                b'{' => {
                    if let Some(close) = self.matching_brace_end(self.source, cursor) {
                        cursor = close;
                    } else {
                        return Err(super::mdx_compatible::invalid(
                            Span::new(cursor as u32, end as u32),
                            "invalid or unclosed JavaScript expression in Markdown",
                        ));
                    }
                }
                b'<' if looks_like_jsx_open(bytes, cursor, true) => {
                    if let Some(close) = super::inline::autolink_end(self.source, cursor) {
                        cursor = close;
                        continue;
                    }
                    let mut attributes = self.allocator.new_vec();
                    let open = self.scan_mdx_jsx_open(
                        self.source,
                        cursor,
                        0,
                        &mut attributes,
                        &mut |at| self.matching_brace_end(self.source, at),
                    )?;
                    if let Some(open) = open {
                        cursor = if open.self_closing {
                            open.end
                        } else {
                            self.mdx_jsx_close(self.source, open.end, open.name)?
                                .map_or(open.end, |(_, close)| close)
                        };
                    }
                }
                _ => cursor += 1,
            }
            if cursor > end {
                end = super::line_scan::next_line_start(bytes, cursor);
            }
        }
        Ok(end)
    }
}
