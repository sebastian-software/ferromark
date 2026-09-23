//! The list item walk as it read each line before the line facts: a
//! terminator search, a blank test, an indentation count and a dedent walk
//! per line, with a trim and a first-content walk again once a line turned
//! out to hold a marker or to continue a paragraph lazily. Tests only; the
//! container equivalence tests parse every input through this and through
//! the production walk and compare the results.

use super::super::Parser;
use super::super::lazy_paragraph::OpenParagraph;
use super::super::list_item::ParsedListItem;
use super::super::whitespace;
use super::ListItemSource;

impl<'a> Parser<'a> {
    pub(super) fn consume_item_continuation_per_line(
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
        let item_is_empty = whitespace::is_blank(item.content);
        let mut item_source = None;
        let mut item_end = self.position;
        let mut gap_spread = false;
        let mut next_item = None;
        // Lazy paragraph continuation is only valid while the item's last
        // consumed line kept a paragraph open (not right after blanks, and
        // not after a fence, an HTML block, a heading or a table). The
        // tracker that knows is asked only when such a line turns up.
        let mut after_blank = false;
        let mut open_paragraph = OpenParagraph::default();

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
                // The comment is not content the tracker classifies: it
                // catches up on what precedes it and skips past it.
                open_paragraph.catch_up(&source.text, &self.options);
                // Preserve normal list dedenting even when a nested code or
                // HTML parser will keep this eligible line as literal content.
                let stripped = Self::push_line_without_indent(
                    &mut source.text,
                    continuation_line,
                    content_indent,
                );
                source.text.push('\n');
                open_paragraph.skip_to(source.text.len());
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

            if whitespace::is_blank(continuation_line) {
                let mut lookahead = continuation_next;
                // The line that stops the walk is the one a sibling marker
                // would be read from, so carry it out instead of scanning it
                // again below.
                let mut lookahead_line = "";
                while lookahead < self.source.len() {
                    let (line, next) = self.line_and_next(lookahead);
                    if !whitespace::is_blank(line) && !self.is_line_comment_at(lookahead) {
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
            if item_is_empty || after_blank || self.line_starts_block_per_line() {
                break;
            }
            let paragraph_open = match &item_source {
                Some(source) => open_paragraph.catch_up(&source.text, &self.options),
                None => open_paragraph.catch_up_first_line(item.content, &self.options),
            };
            if !paragraph_open {
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
