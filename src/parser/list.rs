use crate::allocator::Vec;
use crate::ast::{List, ListItem, Node, Span};

use self::item_source::ListItemSource;
use super::Parser;
use super::lazy_paragraph::OpenParagraph;
use super::line_scan::{is_line_ending_byte, line_terminator_end};
use super::list_item::ParsedListItem;
use crate::parser::error::ParseResult;

mod item_source;

impl<'a> Parser<'a> {
    pub(super) fn parse_list(
        &mut self,
        start: usize,
        baseline_indent: usize,
        first_item: ParsedListItem<'a>,
    ) -> ParseResult<Option<Node<'a>>> {
        // Block dispatch has already parsed the first marker to distinguish a
        // real list from marker-shaped paragraph text. Reuse that result.
        let ordered = first_item.ordered;
        let marker = first_item.marker;
        let list_start = first_item.start;
        let mut item = first_item;

        let mut children: Vec<'a, ListItem<'a>> = self.allocator.new_vec();
        let mut list_spread = false;

        loop {
            let line_start = self.position;
            // Every marker line reaching this loop was already scanned to its
            // end by the recognizer that produced `item`, which recorded that
            // offset as `content_source_end`. Re-deriving the length from it
            // keeps the marker line to one terminator search per item.
            let line_len = item.content_source_end - line_start;
            debug_assert_eq!(
                line_len,
                self.line_at(line_start).len(),
                "list marker line length must match a fresh line scan"
            );

            // Consume the marker line.
            self.position += line_len;
            let bytes = self.source.as_bytes();
            let consumed_newline =
                self.position < bytes.len() && is_line_ending_byte(bytes[self.position]);
            if consumed_newline {
                self.position = line_terminator_end(bytes, self.position);
            }

            let mut lazy_lines = rustc_hash::FxHashSet::default();
            let (gap_spread, item_end, item_source, next_item) = if self.is_at_end() {
                (false, self.position, None, None)
            } else {
                self.consume_item_continuation(
                    &item,
                    baseline_indent,
                    consumed_newline,
                    &mut lazy_lines,
                )
            };

            let mut content_spread = false;
            let item_children = if item_source.is_none()
                && Self::can_inline_parse_list_item(item.content)
            {
                self.parse_inline_list_item_children(item.content, item.content_offset, item_end)?
            } else {
                let item_source = item_source
                    .unwrap_or_else(|| self.init_list_item_source(&item, consumed_newline));
                let source_map = item_source.source_map;
                let item_source = item_source.text.into_bump_str();
                let sub_parser =
                    self.sub_parser_with_source_map(item_source, lazy_lines, &source_map);
                let sub_doc = sub_parser
                    .parse()
                    .map_err(|error| error.remapped(&source_map))?;
                // The item directly contains blank-separated blocks iff a
                // gap between consecutive top-level children spans a line
                // break (spans are still in item-source coordinates).
                content_spread = item_content_has_blank_gap(item_source, &sub_doc.children);
                let mut item_children = sub_doc.children;
                for child in &mut item_children {
                    source_map.remap_node_spans(child);
                }
                item_children
            };
            list_spread |= gap_spread || content_spread;

            let list_item = ListItem {
                checked: item.checked,
                spread: content_spread,
                children: item_children,
                span: Span::new(line_start as u32, item_end as u32),
            };
            // The first push keeps one-item lists at bumpalo's exact minimum.
            // Once a second sibling is known to exist, skip the otherwise
            // inevitable two-slot allocation and its copy; four slots cover
            // the common short list without penalizing the single-item case.
            if children.len() == 1 {
                children.reserve(3);
            }
            children.push(list_item);

            let Some(next_item) = next_item else {
                break;
            };
            if next_item.ordered != ordered || next_item.marker != marker {
                // A different marker starts a new list at the block level.
                break;
            }
            item = next_item;
        }

        let span = Span::new(start as u32, self.position as u32);
        Ok(Some(Node::List(self.allocator.boxed(List {
            ordered,
            start: list_start,
            spread: list_spread,
            children,
            span,
        }))))
    }

    /// Consumes one item's continuation lines: indented content
    /// (paragraphs, nested blocks — the item sub-parser sorts them out),
    /// interior blank lines, and lazy paragraph continuation. Returns
    /// whether the item/list turned loose, the item end position, and the
    /// dedented source when block re-parsing is needed. A parsed sibling is
    /// carried back to the caller so its marker is not scanned twice.
    fn consume_item_continuation(
        &mut self,
        item: &ParsedListItem<'a>,
        baseline_indent: usize,
        consumed_newline: bool,
        lazy_lines: &mut rustc_hash::FxHashSet<u32>,
    ) -> (
        bool,
        usize,
        Option<ListItemSource<'a>>,
        Option<ParsedListItem<'a>>,
    ) {
        let content_indent = item.content_indent;
        let item_is_empty = item.content.trim().is_empty();
        let mut item_source = None;
        let mut item_end = self.position;
        let mut gap_spread = false;
        let mut next_item = None;
        // Lazy paragraph continuation is only valid while the item's last
        // consumed line kept a paragraph open (not right after blanks, and
        // not after a fence, an HTML block, a heading or a table).
        let mut after_blank = false;
        let mut open_paragraph = OpenParagraph::default();
        open_paragraph.observe(item.content, &self.options);

        loop {
            if self.is_at_end() {
                break;
            }

            let continuation_start = self.position;
            let (continuation_line, continuation_next) = self.line_and_next(continuation_start);

            if self.is_line_comment_at(continuation_start) {
                let source = item_source
                    .get_or_insert_with(|| self.init_list_item_source(item, consumed_newline));
                let generated_start = source.text.len();
                // Preserve normal list dedenting even when a nested code or
                // HTML parser will keep this eligible line as literal content.
                let stripped = Self::push_line_without_indent(
                    &mut source.text,
                    continuation_line,
                    content_indent,
                );
                source.text.push('\n');
                source.source_map.push_line_with_block_start(
                    generated_start,
                    source.text.len() - generated_start,
                    continuation_start,
                    continuation_start + stripped,
                    continuation_next - (continuation_start + stripped),
                );
                self.position = continuation_next;
                item_end = self.position;
                continue;
            }

            if continuation_line.trim().is_empty() {
                let mut lookahead = continuation_next;
                // The line that stops the walk is the one a sibling marker
                // would be read from, so carry it out instead of scanning it
                // again below.
                let mut lookahead_line = "";
                while lookahead < self.source.len() {
                    let (line, next) = self.line_and_next(lookahead);
                    if !line.trim().is_empty() && !self.is_line_comment_at(lookahead) {
                        lookahead_line = line;
                        break;
                    }
                    lookahead = next;
                }

                if lookahead >= self.source.len() {
                    break;
                }

                let next_indent = self.calc_indentation(lookahead);
                // An item with no content yet cannot continue past a
                // blank line, but its list may (`* a\n*\n\n* c`).
                if next_indent >= content_indent && !(item_is_empty && item_source.is_none()) {
                    // Interior blank line(s): the item continues below.
                    let item_source = item_source
                        .get_or_insert_with(|| self.init_list_item_source(item, consumed_newline));
                    let mut blank_start = continuation_start;
                    // The first replayed line is the blank line that opened
                    // this branch; its successor is already known.
                    let mut first_blank_next = Some(continuation_next);
                    while blank_start < lookahead {
                        let blank_next = first_blank_next
                            .take()
                            .unwrap_or_else(|| self.next_line_start(blank_start));
                        if self.is_line_comment_at(blank_start) {
                            blank_start = blank_next;
                            continue;
                        }
                        let generated_start = item_source.text.len();
                        item_source.text.push('\n');
                        item_source.source_map.push_blank_line(
                            generated_start,
                            blank_start,
                            blank_next.saturating_sub(blank_start),
                        );
                        blank_start = blank_next;
                    }
                    self.position = lookahead;
                    item_end = self.position;
                    after_blank = true;
                    open_paragraph.observe_blank();
                    continue;
                }

                if next_indent >= baseline_indent
                    && next_indent <= baseline_indent + 3
                    && let Some(sibling) = self
                        .parse_list_item_line_from_line(lookahead, lookahead_line)
                        .filter(|next| next.ordered == item.ordered && next.marker == item.marker)
                {
                    // Blank line between siblings: the list is loose.
                    self.position = lookahead;
                    gap_spread = true;
                    next_item = Some(sibling);
                    break;
                }

                break;
            }

            let current_indent = self.calc_indentation(continuation_start);
            if current_indent >= content_indent {
                // Indented continuation content.
                let item_source = item_source
                    .get_or_insert_with(|| self.init_list_item_source(item, consumed_newline));
                let generated_start = item_source.text.len();
                let source_offset_in_line = Self::push_line_without_indent(
                    &mut item_source.text,
                    continuation_line,
                    content_indent,
                );
                open_paragraph.observe(&item_source.text[generated_start..], &self.options);
                item_source.text.push('\n');
                let source_start = continuation_start + source_offset_in_line;
                item_source.source_map.push_line_with_block_start(
                    generated_start,
                    item_source.text.len() - generated_start,
                    continuation_start,
                    source_start,
                    continuation_next.saturating_sub(source_start),
                );
                self.position = continuation_next;
                item_end = self.position;
                after_blank = false;
                continue;
            }

            // A list marker (indented at most three columns past the
            // baseline — deeper "markers" are just text) ends this item.
            if current_indent >= baseline_indent
                && current_indent <= baseline_indent + 3
                && let Some(sibling) =
                    self.parse_list_item_line_from_line(continuation_start, continuation_line)
            {
                // A thematic break can overlap list syntax only when an
                // unordered item's content starts with the same `-` or
                // `*` marker. All ordinary item text skips the full-line
                // marker scan that previously ran before every sibling.
                let could_be_thematic = !sibling.ordered
                    && matches!(sibling.marker, b'-' | b'*')
                    && sibling.content.as_bytes().first() == Some(&sibling.marker);
                if !could_be_thematic || !Self::try_parse_thematic_break_line(continuation_line) {
                    next_item = Some(sibling);
                }
                break;
            }

            // A block start interrupts the item; anything else lazily
            // continues the item's trailing paragraph regardless of its
            // indentation (CommonMark laziness) — and only a paragraph: a
            // line after a closed fence, an HTML block, a heading or a
            // table belongs to the enclosing block instead.
            if item_is_empty
                || after_blank
                || !open_paragraph.paragraph_open()
                || self.line_starts_block()
            {
                break;
            }
            let source = item_source
                .get_or_insert_with(|| self.init_list_item_source(item, consumed_newline));
            // Keep the lazy line's own indentation: the sub-parse then
            // treats it as paragraph continuation even when it looks like
            // an (over-indented) marker, e.g. `- e` five columns deep.
            // Recording the offset stops setext reinterpretation.
            let generated_start = source.text.len();
            lazy_lines.insert(source.text.len() as u32);
            source.text.push_str(continuation_line);
            open_paragraph.observe(continuation_line, &self.options);
            source.text.push('\n');
            source.source_map.push_line(
                generated_start,
                source.text.len() - generated_start,
                continuation_start,
                continuation_next.saturating_sub(continuation_start),
            );
            self.position = continuation_next;
            item_end = self.position;
        }

        (gap_spread, item_end, item_source, next_item)
    }
}

/// Whether a gap between consecutive top-level children of an item's
/// sub-parsed source spans a line break — the spec's "directly contain
/// two block-level elements with a blank line between them".
fn item_content_has_blank_gap(source: &str, children: &[Node<'_>]) -> bool {
    children.windows(2).any(|pair| {
        let gap_start = block_span(&pair[0]).end as usize;
        let gap_end = block_span(&pair[1]).start as usize;
        source
            .get(gap_start..gap_end)
            .is_some_and(|gap| gap.contains('\n'))
    })
}

fn block_span(node: &Node<'_>) -> Span {
    match node {
        Node::Paragraph(n) => n.span,
        Node::Heading(n) => n.span,
        Node::ThematicBreak(n) => n.span,
        Node::BlockQuote(n) => n.span,
        Node::List(n) => n.span,
        Node::CodeBlock(n) => n.span,
        Node::MathBlock(n) => n.span,
        Node::Html(n) => n.span,
        Node::Table(n) => n.span,
        Node::DefinitionList(n) => n.span,
        Node::Definition(n) => n.span,
        Node::FootnoteDefinition(n) => n.span,
        _ => Span::new(0, 0),
    }
}
