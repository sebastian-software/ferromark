//! The block quote walk as it read each line before the line facts: a
//! space/tab walk for the blank test, a terminator search, and a second
//! column walk for the marker. Tests only; the container equivalence tests
//! parse every input through this and through the production walk and
//! compare the results.

use crate::ast::{BlockQuote, Node, Span};

use super::super::Parser;
use super::super::lazy_paragraph::OpenParagraph;
use super::super::line_scan::{
    line_end as scan_line_end, line_terminator_end, next_line_start as scan_next_line_start,
};
use super::super::spans::SourceMap;
use crate::parser::error::ParseResult;

impl<'a> Parser<'a> {
    pub(super) fn parse_block_quote_per_line(
        &mut self,
        start: usize,
    ) -> ParseResult<Option<Node<'a>>> {
        let bytes = self.source.as_bytes();
        let mut inner = crate::allocator::String::with_capacity_in(128, self.allocator.bump());
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
                open_paragraph.catch_up(&inner, &self.options);
                inner.push_str(&self.source[line_start..next]);
                open_paragraph.skip_to(inner.len());
                self.position = next;
                continue;
            }
            let mut ws_cursor = line_start;
            while ws_cursor < bytes.len() && matches!(bytes[ws_cursor], b' ' | b'\t') {
                ws_cursor += 1;
            }

            // Blank line ends the block quote
            if ws_cursor >= bytes.len() || matches!(bytes[ws_cursor], b'\n' | b'\r') {
                break;
            }

            let line_end = scan_line_end(bytes, line_start);
            let line = &self.source[line_start..line_end];
            let trimmed_offset = ws_cursor - line_start;
            let trimmed = &line[trimmed_offset..];

            if let Some(after_gt) = trimmed.strip_prefix('>') {
                let mut column = 0usize;
                for &byte in &line.as_bytes()[..trimmed_offset] {
                    column = if byte == b'\t' {
                        (column / 4 + 1) * 4
                    } else {
                        column + 1
                    };
                }
                let after_marker_column = column + 1;
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
                let content_start =
                    line_start + trimmed_offset + 1 + Self::quote_marker_space_bytes(after_gt);
                let line_next = line_terminator_end(bytes, line_end);
                let source_len =
                    line_end.saturating_sub(content_start) + line_next.saturating_sub(line_end);
                source_map.push_line(
                    generated_start,
                    indent_columns + stripped_trimmed.len() + 1,
                    content_start,
                    source_len,
                );

                self.position = line_next;
            } else if !Self::quote_lazy_blocked(trimmed)
                && !self.line_starts_block_per_line()
                && open_paragraph.catch_up(&inner, &self.options)
            {
                let generated_start = inner.len();
                lazy_lines.insert(inner.len() as u32);
                inner.push_str(line);
                inner.push('\n');
                let line_next = line_terminator_end(bytes, line_end);
                source_map.push_line(
                    generated_start,
                    line.len() + 1,
                    line_start,
                    line.len() + line_next.saturating_sub(line_end),
                );
                self.position = line_next;
            } else {
                break;
            }
        }

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
}
