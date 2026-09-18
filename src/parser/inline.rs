use crate::allocator::Vec;
use crate::ast::{Node, Span, Text};

use super::Parser;
use crate::parser::error::{ParseErrorKind, ParseResult};

mod autolink;
mod code_span;
mod emphasis;
mod entity;
mod gfm_autolink;
mod image;
mod line_break;
mod link_target;
mod marker_scan;
mod scan;
mod script_span;

pub(in crate::parser) use self::marker_scan::InlineMarkerScan;
use self::script_span::same_marker_neighbor;
use super::line_scan::{is_line_ending_byte, line_terminator_end};

pub(in crate::parser) use self::autolink::autolink_end;
pub(in crate::parser) use self::link_target::{
    parse_destination as parse_link_destination, parse_title as parse_link_title,
};

impl<'a> Parser<'a> {
    /// Child slots to reserve for `content_len` bytes of inline content.
    ///
    /// A bump-allocated `Vec` cannot extend the block it owns — bumpalo
    /// hands back a fresh region and memcpies — so growing copies every
    /// node so far at each doubling step and abandons the old block in the
    /// arena. Measured over the bundled corpora the node count tracks the
    /// content length closely (p90 ≈ one node per 20 bytes in every length
    /// bucket), so reserving that covers most blocks in one allocation and
    /// still uses ~3% *less* arena than growing did. The floor keeps short
    /// spans at bumpalo's own minimum; the ceiling stops a long paragraph
    /// from reserving a kilobyte it will not fill.
    fn inline_children_capacity(content_len: usize) -> usize {
        const BYTES_PER_NODE: usize = 20;
        (content_len / BYTES_PER_NODE).clamp(4, 12)
    }

    pub(super) fn push_text(
        children: &mut Vec<'a, Node<'a>>,
        value: &'a str,
        start: usize,
        end: usize,
    ) {
        children.push(Node::Text(Text {
            value,
            span: Span::new(start as u32, end as u32),
        }));
    }

    /// Parses the inline content of a block-level construct (paragraph,
    /// heading, table cell, list item paragraph) and runs the block-scoped
    /// post-passes on the result — today, the GFM autolink rewrite.
    ///
    /// Nested inline contexts (link text and image alt) call [`Self::parse_inline`] directly instead: the autolink
    /// pass itself recurses through emphasis-like containers, so running it
    /// per nested sequence both re-scanned the same nodes and — for link
    /// text, which GFM excludes from autolinking — made nested `<a>`s.
    pub(super) fn parse_inline_block(
        &self,
        content: &'a str,
        offset: usize,
    ) -> ParseResult<Vec<'a, Node<'a>>> {
        if self.phase == super::ParsePhase::Definitions {
            return Ok(self.allocator.new_vec());
        }
        let mut children = self.parse_inline(content, offset)?;
        let scan = self
            .options
            .autolinks
            .then(|| gfm_autolink::may_contain_autolink(content))
            .flatten();
        if let Some(scan) = scan {
            self.apply_gfm_autolinks(&mut children, scan);
        }
        Ok(children)
    }

    fn allows_mdx_text_expression(&self) -> bool {
        self.options.mdx
    }

    /// Opens one inline context and bounds how many may nest.
    ///
    /// Link text, image alt text, wiki-link labels, script spans and inline
    /// JSX phrasing re-enter [`Self::parse_inline`] once per bracket level,
    /// so `[[[[...` recurses as deeply as the input is nested. A stack
    /// overflow aborts the process instead of unwinding, so the depth has to
    /// be refused before the recursion happens rather than recovered from
    /// afterwards.
    ///
    /// Counted like the block bound in `parse_block`: the outermost context
    /// is depth zero, so `max_nesting_depth` levels of nesting are allowed
    /// and the next one fails. Inline and block depth are separate counts
    /// against the same limit — a document can be that deep in blocks *and*
    /// that deep in inline brackets — because block containers cannot occur
    /// inside inline content, so the two only ever add up along a path once.
    ///
    /// Within inline content the count is shared, because emphasis and
    /// brackets *do* add up along a path: the guard also carries the depth
    /// each level reports back to the one above it, which is what
    /// [`Parser::process_emphasis`] needs to bound the tree pairing builds.
    pub(super) fn enter_inline(&self, offset: usize) -> ParseResult<InlineDepthGuard<'_>> {
        let depth = self.inline_depth.get();
        if self.options.max_nesting_depth > 0 && depth > self.options.max_nesting_depth {
            return Err(ParseErrorKind::NestingTooDeep {
                span: Span::new(offset as u32, offset as u32),
                max_depth: self.options.max_nesting_depth,
            }
            .into());
        }
        self.inline_depth.set(depth + 1);
        let nested = self.nested_depth_cell();
        Ok(InlineDepthGuard {
            depth: &self.inline_depth,
            nested,
            outer_nested: nested.replace(0),
        })
    }

    /// The nested-inline-depth counter as it stands, without allocating it.
    ///
    /// An inline context that is opened and then thrown away whole — the
    /// in-place bracket walk that finds a construct reaching past its
    /// closing bracket — has to leave the counter as it found it, or the
    /// probe that settles the same text afterwards would be counted twice.
    pub(super) fn nested_inline_depth(&self) -> usize {
        self.nested_inline_depth
            .get()
            .map_or(0, std::cell::Cell::get)
    }

    /// Puts back a value from [`Self::nested_inline_depth`].
    pub(super) fn restore_nested_inline_depth(&self, depth: usize) {
        if let Some(cell) = self.nested_inline_depth.get() {
            cell.set(depth);
        }
    }

    /// The shared depth cell, allocated on the first inline context of a
    /// parse. A parser that never reaches inline content — the definition
    /// pre-pass, and every rejected block probe — leaves the arena
    /// untouched, which its own tests hold it to.
    fn nested_depth_cell(&self) -> &'a std::cell::Cell<usize> {
        if let Some(cell) = self.nested_inline_depth.get() {
            return cell;
        }
        let cell: &'a std::cell::Cell<usize> = &*self.allocator.alloc(std::cell::Cell::new(0));
        self.nested_inline_depth.set(Some(cell));
        cell
    }

    pub(super) fn parse_inline(
        &self,
        content: &'a str,
        offset: usize,
    ) -> ParseResult<Vec<'a, Node<'a>>> {
        let _depth = self.enter_inline(offset)?;
        let bytes = content.as_bytes();
        let mut markers = InlineMarkerScan::new(&self.options);
        let first_special = markers.next(bytes, 0);

        // Plain text is both the most common inline shape and exactly one AST
        // node. Reserving the general four-node floor here wasted three
        // full Node slots for every prose block and plain table cell. The
        // scan is required by the normal loop anyway, so use its no-marker
        // result to build the exact one-slot representation and return.
        if first_special == content.len() {
            if content.is_empty() {
                return Ok(self.allocator.new_vec());
            }
            let mut children = self.allocator.new_vec_with_capacity(1);
            Self::push_text(&mut children, content, offset, offset + content.len());
            return Ok(children);
        }

        let mut children = self
            .allocator
            .new_vec_with_capacity(Self::inline_children_capacity(content.len()));
        let mut delimiters = self.allocator.new_vec();
        let mut pos = 0;
        let mut first_scan = Some(first_special);

        while pos < content.len() {
            let start = pos;
            // Plain text is the common inline case. Jump over bytes that
            // cannot start any inline construct, then push that entire run as
            // one Text node. This keeps the parser on bulk byte scans for
            // prose and only enters the slower match when a real marker byte
            // has been reached.
            pos = first_scan
                .take()
                .unwrap_or_else(|| markers.next(bytes, pos));

            // Fold soft line breaks into the running text node. A newline
            // with non-whitespace on both sides is a soft break with nothing
            // to strip, so its rendered form is the literal `\n` already
            // inside the source run — emitting `"line"`, `"\n"`, `"next"` as
            // three nodes only slowed every later pass. This is also the
            // shape remark produces (mdast has no softbreak node; line
            // endings live inside `text` values). Prose is dominated by this
            // case; a newline touching spaces or tabs still takes
            // `parse_line_break` below for hard-break detection and
            // whitespace stripping.
            while pos > start
                && pos + 1 < content.len()
                && bytes[pos] == b'\n'
                && !matches!(bytes[pos - 1], b' ' | b'\t')
                && !matches!(bytes[pos + 1], b' ' | b'\t' | b'\n')
            {
                pos = markers.next(bytes, pos + 1);
            }

            if pos > start {
                Self::push_text(
                    &mut children,
                    &content[start..pos],
                    offset + start,
                    offset + pos,
                );
            }
            if pos >= content.len() {
                break;
            }

            self.parse_inline_special(
                content,
                offset,
                &mut children,
                &mut delimiters,
                &mut markers,
                &mut pos,
            )?;
        }

        if !delimiters.is_empty() {
            self.process_emphasis(&mut children, &mut delimiters)?;
        }
        Ok(children)
    }

    fn parse_inline_special(
        &self,
        content: &'a str,
        offset: usize,
        children: &mut Vec<'a, Node<'a>>,
        delimiters: &mut Vec<'a, emphasis::Delimiter>,
        markers: &mut InlineMarkerScan,
        pos: &mut usize,
    ) -> ParseResult<()> {
        let bytes = content.as_bytes();
        match bytes[*pos] {
            b'\\' if *pos + 1 < content.len() && is_line_ending_byte(bytes[*pos + 1]) => {
                let end = line_terminator_end(bytes, *pos + 1);
                let span = Span::new((offset + *pos) as u32, (offset + end) as u32);
                children.push(Node::Break(crate::ast::Break { span }));
                *pos = end;
                // Leading whitespace of the next line is not content.
                while *pos < content.len() && matches!(bytes[*pos], b' ' | b'\t') {
                    *pos += 1;
                }
            }
            byte if is_line_ending_byte(byte) => {
                Self::parse_line_break(content, offset, children, pos);
            }
            b'&' => {
                // Entity / numeric character references decode to literal
                // text (the result can never open or close markup).
                if let Some((value, len)) = entity::scan_entity(&content[*pos..]) {
                    let end = *pos + len;
                    let text: &'a str = match value {
                        entity::EntityValue::Named(expansion) => expansion,
                        entity::EntityValue::Char(ch) => {
                            let mut buf = [0u8; 4];
                            self.allocator.alloc_str(ch.encode_utf8(&mut buf))
                        }
                    };
                    Self::push_text(children, text, offset + *pos, offset + end);
                    *pos = end;
                } else {
                    Self::push_text(children, "&", offset + *pos, offset + *pos + 1);
                    *pos += 1;
                }
            }
            b'{' if self.allows_mdx_text_expression() => {
                if let Some((node, end)) = self.try_parse_mdx_text_expression(content, *pos, offset)
                {
                    children.push(node);
                    *pos = end;
                } else {
                    Self::push_text(children, "{", offset + *pos, offset + *pos + 1);
                    *pos += 1;
                }
            }
            b'<' => self.parse_inline_html_or_text(content, offset, children, pos)?,
            b'\\' if *pos + 1 < content.len() && bytes[*pos + 1].is_ascii_punctuation() => {
                // A backslash escapes only ASCII punctuation (CommonMark
                // "Backslash escapes"). The escaped character is emitted as
                // literal text so it can't open any inline construct.
                *pos += 1;
                let span_start = offset + *pos - 1;
                Self::push_text(
                    children,
                    &content[*pos..*pos + 1],
                    span_start,
                    offset + *pos + 1,
                );
                *pos += 1;
            }
            b'\\' => {
                // Backslash before anything else (letters, digits, spaces,
                // multibyte characters, or end of input) is a literal
                // backslash; the following character is parsed normally.
                Self::push_text(children, "\\", offset + *pos, offset + *pos + 1);
                *pos += 1;
            }
            b'=' if self.options.highlight => {
                let run = Self::marker_run_len(bytes, *pos, b'=');
                if run == 2 {
                    self.push_delimiter_run(content, offset, children, delimiters, pos);
                } else {
                    Self::push_text(
                        children,
                        &content[*pos..*pos + run],
                        offset + *pos,
                        offset + *pos + run,
                    );
                    *pos += run;
                }
            }
            b'^' if self.options.inline_footnotes && bytes.get(*pos + 1) == Some(&b'[') => {
                self.parse_inline_footnote(content, offset, children, pos)?;
            }
            b'~' if self.options.strikethrough => {
                let run_len = Self::marker_run_len(bytes, *pos, b'~');
                // A caller that explicitly enables both extensions keeps the
                // existing single-tilde subscript grammar. Formal GFM leaves
                // subscript off, so its single tilde is strikethrough.
                if run_len == 1 && self.options.subscript {
                    self.parse_subscript_span(content, offset, children, pos)?;
                } else if run_len <= 2 {
                    self.push_delimiter_run(content, offset, children, delimiters, pos);
                } else {
                    Self::push_text(
                        children,
                        &content[*pos..*pos + run_len],
                        offset + *pos,
                        offset + *pos + run_len,
                    );
                    *pos += run_len;
                }
            }
            b'~' if self.options.subscript && !same_marker_neighbor(bytes, *pos, b'~') => {
                self.parse_subscript_span(content, offset, children, pos)?;
            }
            b'^' if self.options.superscript && !same_marker_neighbor(bytes, *pos, b'^') => {
                self.parse_superscript_span(content, offset, children, pos)?;
            }
            b'$' if self.options.math => {
                Self::parse_inline_math(content, offset, children, pos);
            }
            b'*' | b'_' => {
                self.push_delimiter_run(content, offset, children, delimiters, pos);
            }
            b'`' => self.parse_inline_code(content, offset, children, pos),
            b'[' => {
                // Whether the bracket became a link only matters to a
                // bracket around it (`Parser::parse_bracket_text`).
                self.parse_link(content, offset, children, markers, pos)?;
            }
            b'!' => self.parse_image(content, offset, children, pos)?,
            _ => {
                Self::push_text(
                    children,
                    &content[*pos..*pos + 1],
                    offset + *pos,
                    offset + *pos + 1,
                );
                *pos += 1;
            }
        }
        Ok(())
    }

    fn parse_inline_html_or_text(
        &self,
        content: &'a str,
        offset: usize,
        children: &mut Vec<'a, Node<'a>>,
        pos: &mut usize,
    ) -> ParseResult<()> {
        // An autolink, a JSX tag and an inline HTML tag all have to close
        // with `>`. Each of the three parsers below only reports that there
        // is none by scanning to the end of the content, so a line holding
        // `<` with no `>` after it paid three walks per `<` — quadratic over
        // a run of them, and `a < b` is ordinary prose.
        if !self.has_closer_from(content, *pos + 1, b'>') {
            Self::push_text(children, "<", offset + *pos, offset + *pos + 1);
            *pos += 1;
            return Ok(());
        }

        if let Some((link, end)) = self.parse_autolink(content, *pos, offset) {
            children.push(link);
            *pos = end;
        } else if let Some((node, end)) = self.try_parse_mdx_jsx_text(content, *pos, offset)? {
            children.push(node);
            *pos = end;
        } else if let Some((mut html, end)) = Self::parse_inline_html(content, *pos, offset) {
            self.normalize_inline_html(&mut html);
            children.push(Node::Html(html));
            *pos = end;
        } else {
            Self::push_text(children, "<", offset + *pos, offset + *pos + 1);
            *pos += 1;
        }
        Ok(())
    }
}

/// Closes the inline context opened by `Parser::enter_inline`, on every exit
/// from `parse_inline` — including the `?` returns inside it.
pub(super) struct InlineDepthGuard<'p> {
    depth: &'p std::cell::Cell<usize>,
    nested: &'p std::cell::Cell<usize>,
    /// What the enclosing level had accumulated before this one started.
    outer_nested: usize,
}

impl Drop for InlineDepthGuard<'_> {
    fn drop(&mut self) {
        self.depth.set(self.depth.get().saturating_sub(1));
        // This level's tree becomes one child subtree of the level above:
        // the node built around it is one level deeper than the deepest
        // thing in it, and the level above keeps the deepest of its
        // children. Abandoned sub-parses (a speculative link probe that
        // ends up literal) are folded in the same way, which over- rather
        // than under-counts and keeps the bound safe.
        let produced = self.nested.get() + 1;
        self.nested.set(self.outer_nested.max(produced));
    }
}
