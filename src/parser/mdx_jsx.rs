//! MDX JSX parse: PascalCase / member names, fragments, spreads, expressions.

use crate::ast::{MdxJsxFlowElement, MdxJsxTextElement, Node, Span};

use super::{JsxCloserGap, Parser};
use crate::parser::error::ParseResult;

mod braces;
mod children;
mod expression;
mod scan;

pub(super) fn looks_like_jsx_open(bytes: &[u8], at: usize) -> bool {
    scan::looks_like_jsx_open(bytes, at)
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
        if !self.options.mdx || !scan::looks_like_jsx_open(self.source.as_bytes(), trimmed_start) {
            return Ok(None);
        }

        let source = self.source;
        let mut attributes = self.allocator.new_vec();
        let Some(open) =
            scan::scan_jsx_open(source, trimmed_start, 0, &mut attributes, &mut |at| {
                self.matching_brace_end(source, at)
            })
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
                self.mdx_jsx_close(self.source, open.end, open.name)
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
        if !self.options.mdx || !scan::looks_like_jsx_open(content.as_bytes(), pos) {
            return Ok(None);
        }

        let mut attributes = self.allocator.new_vec();
        let Some(open) = scan::scan_jsx_open(content, pos, offset, &mut attributes, &mut |at| {
            self.matching_brace_end(content, at)
        }) else {
            return Ok(None);
        };

        let (self_closing, children, end) = if open.self_closing {
            (true, self.allocator.new_vec(), open.end)
        } else {
            let Some((close_start, close_end)) = self.mdx_jsx_close(content, open.end, open.name)
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
    ) -> Option<(usize, usize)> {
        let slice = (content.as_ptr() as usize, content.len());
        let cached = self
            .extension_memos()
            .mdx_jsx_closer_presence
            .borrow()
            .get(&(slice.0, slice.1, from, name))
            .copied();
        if let Some(close) = cached {
            return close;
        }
        if let Some(gap) = self.extension_memos().mdx_jsx_closer_gap.get()
            && gap.slice == slice
            && gap.name == name
            && gap.from <= from
            && from < gap.until
        {
            return None;
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
        walk.close
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
        if inner_start >= inner_end || self.source[inner_start..inner_end].trim().is_empty() {
            return Ok(self.allocator.new_vec());
        }
        let inner = &self.source[inner_start..inner_end];
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
}
