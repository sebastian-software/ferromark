use crate::ast::{BlockQuote, Node, Span};

use super::Parser;
use super::cursor::LineIndent;
use super::lazy_paragraph::OpenParagraph;
use super::line_scan::{is_line_ending_byte, next_line_start as scan_next_line_start};
use super::spans::SourceMap;
use super::whitespace;
use crate::parser::error::ParseResult;

/// The walk as it read each line before the line facts, kept for the
/// container equivalence tests.
#[cfg(test)]
mod per_line;

impl<'a> Parser<'a> {
    /// Parses a block quote by stripping quote markers into arena storage.
    ///
    /// The nested parser needs a contiguous source string without the leading
    /// `>` markers. Building that string directly in the bump arena avoids the
    /// old two-step path of filling a system `String` and then copying it into
    /// arena storage before recursive parsing.
    pub(super) fn parse_block_quote(&mut self, start: usize) -> ParseResult<Option<Node<'a>>> {
        #[cfg(test)]
        if super::container_equivalence::per_line_walk() {
            return self.parse_block_quote_per_line(start);
        }

        // Collect lines belonging to this block quote and strip the `>` prefix.
        // Write straight into a bump-allocated `String` so we don't pay for
        // `String::new` (system allocator) followed by `alloc_str` (copy to
        // arena) on every block quote. Capacity is intentionally small —
        // bumpalo will grow it if needed, and oversizing wastes arena
        // bytes that can't be reclaimed until reset.
        let bytes = self.source.as_bytes();
        let mut inner = crate::allocator::String::with_capacity_in(128, self.allocator.bump());
        // Lazy continuation applies only while the quote's last block is
        // an open paragraph: the tracker follows the stripped content
        // closely enough to know when that is the case, and is asked only
        // when a line without the marker could continue the quote.
        let mut open_paragraph = OpenParagraph::default();
        let mut lazy_lines = rustc_hash::FxHashSet::default();
        let mut source_map = SourceMap::default();

        loop {
            if self.position >= bytes.len() {
                break;
            }

            let line_start = self.position;
            if self.is_line_comment_at(line_start) {
                let next = scan_next_line_start(bytes, line_start);
                source_map.push_line(
                    inner.len(),
                    next - line_start,
                    line_start,
                    next - line_start,
                );
                // The comment is copied verbatim and is not content the
                // tracker classifies: it catches up on what precedes it and
                // skips past it.
                open_paragraph.catch_up(&inner, &self.options);
                inner.push_str(&self.source[line_start..next]);
                open_paragraph.skip_to(inner.len());
                self.position = next;
                continue;
            }
            // The space/tab run stops at the line's terminator by itself, so
            // the byte after it tells a blank line apart. Testing that first
            // lets a blank line, which is what usually closes a quote, end
            // the walk before it pays for a terminator search.
            let indent = LineIndent::at(bytes, line_start);
            let run_end = line_start + indent.bytes;
            if run_end >= bytes.len() || is_line_ending_byte(bytes[run_end]) {
                break;
            }

            // Every other line needs its end. The search starts past the
            // measured run, and the run's columns are the ones the marker's
            // own arithmetic continues from.
            let facts = self.line_facts_with_indent(line_start, indent);
            let line = facts.line;
            let line_next = facts.next;
            let line_end = line_start + line.len();
            let trimmed = facts.after_indent();

            if let Some(after_gt) = trimmed.strip_prefix('>') {
                // The marker consumes `>` plus one column of following
                // whitespace. Expanding the rest of that whitespace run to
                // spaces (with original column arithmetic) keeps tab stops
                // aligned through the re-parse: `>\t\tfoo` becomes six
                // spaces + foo, i.e. indented code with two extra columns.
                let after_marker_column = indent.columns + 1;
                let ws_bytes = after_gt.as_bytes();
                let mut ws_len = 0usize;
                let mut ws_end_column = after_marker_column;
                while ws_len < ws_bytes.len() && matches!(ws_bytes[ws_len], b' ' | b'\t') {
                    ws_end_column = if ws_bytes[ws_len] == b'\t' {
                        (ws_end_column / 4 + 1) * 4
                    } else {
                        ws_end_column + 1
                    };
                    ws_len += 1;
                }
                let indent_columns = if ws_len > 0 {
                    ws_end_column.saturating_sub(after_marker_column + 1)
                } else {
                    0
                };
                let generated_start = inner.len();
                for _ in 0..indent_columns {
                    inner.push(' ');
                }
                let stripped_trimmed = &after_gt[ws_len..];
                inner.push_str(stripped_trimmed);
                inner.push('\n');
                let content_start = run_end + 1 + Self::quote_marker_space_bytes(after_gt);
                let source_len =
                    line_end.saturating_sub(content_start) + line_next.saturating_sub(line_end);
                source_map.push_line(
                    generated_start,
                    indent_columns + stripped_trimmed.len() + 1,
                    content_start,
                    source_len,
                );

                // Advance past this line (and the trailing newline if any).
                self.position = line_next;
            } else if !Self::quote_lazy_blocked(trimmed)
                // The line is not blank, so its first non-space, non-tab
                // byte is exactly where its space/tab run ended.
                && !self.line_starts_block(line_start, run_end)
                && open_paragraph.catch_up(&inner, &self.options)
            {
                // Lazy continuation: the line joins the quote's open
                // paragraph as if the `>` marker were present. Keeping the
                // original indentation means the re-parse still sees it as
                // paragraph continuation (an indented `- x` stays text),
                // and recording the offset stops setext reinterpretation.
                let generated_start = inner.len();
                lazy_lines.insert(inner.len() as u32);
                inner.push_str(line);
                inner.push('\n');
                source_map.push_line(
                    generated_start,
                    line.len() + 1,
                    line_start,
                    line.len() + line_next.saturating_sub(line_end),
                );
                self.position = line_next;
            } else {
                // Line doesn't start with `>`, block quote ends
                break;
            }
        }

        // Recursively parse the inner content from the same arena — no copy.
        let inner_str = inner.into_bump_str();
        let sub_parser = self.sub_parser_with_source_map(inner_str, lazy_lines, &source_map);
        let sub_doc = sub_parser
            .parse()
            .map_err(|error| error.remapped(&source_map))?;
        let mut children = sub_doc.children;
        for child in &mut children {
            source_map.remap_node_spans(child);
        }

        let span = Span::new(start as u32, self.position as u32);
        Ok(Some(Node::BlockQuote(
            self.allocator.boxed(BlockQuote { children, span }),
        )))
    }

    /// Lines that must not lazily continue a block quote paragraph even
    /// though they cannot interrupt one: a bare list marker opens an
    /// (empty) list block when the quote marker is imagined present.
    fn quote_lazy_blocked(trimmed: &str) -> bool {
        let line = whitespace::trim_end(trimmed);
        Self::try_parse_list_line(line) && {
            let after_digits = line.trim_start_matches(|ch: char| ch.is_ascii_digit());
            let after_marker = after_digits.trim_start_matches(['-', '*', '+', '.', ')']);
            whitespace::is_blank(after_marker)
        }
    }

    fn quote_marker_space_bytes(after_gt: &str) -> usize {
        usize::from(after_gt.as_bytes().first() == Some(&b' '))
    }
}
