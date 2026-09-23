use crate::allocator::Vec;
use crate::ast::{AlignKind, Node, Span, Table, TableCell, TableRow};
use memchr::memchr;

use super::Parser;
use super::line_scan::{line_end, line_terminator_end};
use super::table_cell_source::{remap_table_cell_inline_spans, unescape_table_pipes};
use super::table_pipes::{EscapedPipes, Pipe, PipeCursor, is_escaped_table_pipe};
use super::whitespace;
use crate::parser::error::ParseResult;

/// One cell of a table row, as the row splitter found it.
///
/// `escapes` bounds the `\|` occurrences the splitter passed over on its
/// way to this cell's terminator, in the coordinates of `content`, so the
/// cell decoder never looks for them outside that stretch, and not at all
/// when there is none.
struct RowCell<'a> {
    content: &'a str,
    start: usize,
    end: usize,
    escapes: EscapedPipes,
    colspan: usize,
}

impl<'a> Parser<'a> {
    /// Returns true when the next two lines look like a GFM table header.
    ///
    /// Table detection sits behind a cheap `|` guard in block dispatch, but it
    /// still runs on many prose lines containing pipes. Peek the first two
    /// lines directly with `memchr` and inspect slices in place instead of
    /// collecting `lines().take(2)` into a temporary `Vec`.
    pub(super) fn try_parse_table(&self) -> bool {
        let bytes = self.source.as_bytes();
        let p0 = self.position;
        let nl0 = line_end(bytes, p0);
        if nl0 == bytes.len() {
            return false;
        }
        // `nl0` already located the header line's terminator, so the
        // delimiter line starts one two-byte step past it.
        let p1 = self.skip_line_comments_from(line_terminator_end(bytes, nl0));
        if p1 >= bytes.len() {
            return false;
        }
        let nl1 = line_end(bytes, p1);

        let first_line = whitespace::trim(&self.source[p0..nl0]);
        if memchr(b'|', first_line.as_bytes()).is_none() {
            return false;
        }

        // Second line must be the delimiter row (contains | and -)
        let second_line = whitespace::trim(&self.source[p1..nl1]);
        if memchr(b'|', second_line.as_bytes()).is_none()
            || memchr(b'-', second_line.as_bytes()).is_none()
        {
            return false;
        }

        let header_cells = Self::table_header_cells(self.options.merged_table_cells, first_line);
        header_cells > 0 && Self::table_delimiter_cells(second_line) == Some(header_cells)
    }

    /// The cells a trimmed header line contributes, counting horizontal
    /// spans when merged cells are on.
    pub(super) fn table_header_cells(merged_table_cells: bool, line: &'a str) -> usize {
        if merged_table_cells {
            Self::table_row_cells_with_spans(line)
                .map(|cell| cell.colspan)
                .sum()
        } else {
            Self::table_row_cells(line).count()
        }
    }

    /// The cell count of a trimmed delimiter row, or `None` when a cell is
    /// not a delimiter cell.
    pub(super) fn table_delimiter_cells(line: &'a str) -> Option<usize> {
        let mut cells = 0;
        for cell in Self::table_row_cells(line) {
            delimiter_alignment(cell)?;
            cells += 1;
        }
        Some(cells)
    }

    pub(super) fn parse_table(&mut self, start: usize) -> ParseResult<Option<Node<'a>>> {
        let mut align: Vec<'a, AlignKind> = self.allocator.new_vec();

        // Parse header row
        let header_start = self.position;
        let header_line = self.consume_line();
        self.position = self.skip_line_comments_from(self.position);

        // Parse delimiter row to get alignment
        let delimiter_line = self.consume_line();
        for cell in Self::table_row_cells(delimiter_line) {
            if let Some(alignment) = delimiter_alignment(cell) {
                align.push(alignment);
            }
        }
        let column_count = align.len();

        // Build the table AST directly instead of first collecting row slices
        // into short-lived heap Vecs. Each consumed source line is parsed into
        // arena-backed cells immediately, which keeps the table path linear in
        // the input and avoids throwaway row containers.
        let mut children: Vec<'a, TableRow<'a>> = self.allocator.new_vec();
        children.push(self.parse_table_row(
            header_line,
            header_start,
            column_count,
            self.options.merged_table_cells,
        )?);

        // Parse body rows
        loop {
            self.position = self.skip_line_comments_from(self.position);
            if self.is_at_end() {
                break;
            }

            if self.options.table_attributes && self.is_table_attributes_line(self.position) {
                break;
            }

            // A blank line or another block-level construct terminates the
            // table. Ordinary lines remain data rows even without a pipe.
            let Some(trimmed_start) = self.first_non_whitespace_in_line(self.position) else {
                break;
            };
            if self
                .probe_line_without_table(self.position, trimmed_start)
                .starts_block
            {
                break;
            }

            let row_start = self.position;
            let row_line = self.consume_line();
            children.push(self.parse_table_row(
                row_line,
                row_start,
                column_count,
                self.options.merged_table_cells,
            )?);
        }

        let attributes = self.parse_table_attributes()?;
        let span = Span::new(start as u32, self.position as u32);
        Ok(Some(Node::Table(self.allocator.boxed(Table {
            align,
            children,
            attributes,
            span,
        }))))
    }

    /// Parses a table row into arena-backed AST cells without temporary heap
    /// collection.
    ///
    /// The row iterator yields borrowed cell slices from the original line.
    /// Inline parsing then writes cell children into the parser arena, so no
    /// intermediate `Vec<&str>` or owned cell text is needed.
    pub(super) fn parse_table_row(
        &self,
        line: &'a str,
        line_start: usize,
        column_count: usize,
        merged: bool,
    ) -> ParseResult<TableRow<'a>> {
        // Every row is truncated or padded to the delimiter's exact width.
        // Reserve it before parsing inline children: those arena allocations
        // otherwise prevent the cell vector from growing in place, leaving
        // each discarded backing buffer live until the arena is dropped.
        let mut cells: Vec<'a, TableCell<'a>> = self.allocator.new_vec_with_capacity(column_count);
        let line_end = line_start + line.len();
        let mut logical_columns = 0;
        if merged {
            for cell in Self::table_row_cells_with_spans(line) {
                if logical_columns >= column_count {
                    break;
                }
                let colspan = cell.colspan.min(column_count - logical_columns);
                cells.push(self.parse_table_cell(&cell, line_start, colspan)?);
                logical_columns += colspan;
            }
        } else {
            for cell in Self::table_row_cells_with_offsets(line).take(column_count) {
                cells.push(self.parse_table_cell(&cell, line_start, 1)?);
                logical_columns += 1;
            }
        }
        while logical_columns < column_count {
            cells.push(TableCell {
                children: self.allocator.new_vec(),
                span: Span::new(line_end as u32, line_end as u32),
                colspan: 1,
            });
            logical_columns += 1;
        }
        Ok(TableRow {
            children: cells,
            span: Span::new(line_start as u32, line_end as u32),
        })
    }

    fn parse_table_cell(
        &self,
        cell: &RowCell<'a>,
        line_start: usize,
        colspan: usize,
    ) -> ParseResult<TableCell<'a>> {
        let cell_content = unescape_table_pipes(self.allocator, cell.content, cell.escapes);
        let cell_children = if let Some(source_map) = &cell_content.source_map {
            let mut children = self.parse_inline_block(cell_content.content, 0)?;
            let source_offset = (line_start + cell.start) as u32;
            for child in &mut children {
                remap_table_cell_inline_spans(child, source_offset, source_map);
            }
            children
        } else {
            self.parse_inline_block(cell_content.content, line_start + cell.start)?
        };
        Ok(TableCell {
            children: cell_children,
            span: Span::new(
                (line_start + cell.start) as u32,
                (line_start + cell.end) as u32,
            ),
            colspan,
        })
    }

    /// Iterates table row cells from a line.
    ///
    /// Leading/trailing pipes are syntax delimiters, not empty cells in this
    /// parser's table model, so they are stripped once before splitting.
    pub(super) fn table_row_cells(line: &'a str) -> impl Iterator<Item = &'a str> {
        Self::table_row_cells_with_offsets(line).map(|cell| cell.content)
    }

    fn table_row_cells_with_offsets(line: &'a str) -> impl Iterator<Item = RowCell<'a>> {
        let (trimmed, trimmed_start) = whitespace::trim_with_leading(line);
        let mut content_start = trimmed_start;
        let mut content_end = trimmed_start + trimmed.len();
        if line[content_start..content_end].starts_with('|') {
            content_start += 1;
        }
        if content_start < content_end
            && line[content_start..content_end].ends_with('|')
            && !is_escaped_table_pipe(
                &line.as_bytes()[content_start..content_end],
                content_end - content_start - 1,
            )
        {
            content_end -= 1;
        }
        let content = &line[content_start..content_end];
        let bytes = content.as_bytes();
        let mut cell_start = 0;
        // One cursor walks the whole row. Every pipe it reports already
        // carries its escape state, and the escaped ones it passes over are
        // handed to the cell decoder instead of being looked for again.
        let mut pipes = PipeCursor::new(bytes, 0);

        std::iter::from_fn(move || {
            if cell_start > bytes.len() {
                return None;
            }

            let mut escapes = EscapedPipes::default();
            while let Some(pipe) = pipes.next_pipe() {
                if pipe.escaped {
                    escapes.record(pipe.offset);
                    continue;
                }
                let raw = &content[cell_start..pipe.offset];
                let (cell, start, end) = trim_cell(raw, content_start + cell_start);
                cell_start = pipe.offset + 1;
                return Some(RowCell {
                    content: cell,
                    start,
                    end,
                    escapes: escapes.shifted(start - content_start),
                    colspan: 1,
                });
            }

            let raw = &content[cell_start..];
            let (cell, start, end) = trim_cell(raw, content_start + cell_start);
            cell_start = bytes.len() + 1;
            Some(RowCell {
                content: cell,
                start,
                end,
                escapes: escapes.shifted(start - content_start),
                colspan: 1,
            })
        })
    }

    /// Iterates row cells while preserving adjacent unescaped pipe runs as
    /// horizontal spans. A terminal run is kept intact so `value ||` can
    /// express a span even though the final pipe is also the usual row
    /// boundary marker.
    fn table_row_cells_with_spans(line: &'a str) -> impl Iterator<Item = RowCell<'a>> {
        let (trimmed, trimmed_start) = whitespace::trim_with_leading(line);
        let mut content_start = trimmed_start;
        let mut content_end = trimmed_start + trimmed.len();
        if line[content_start..content_end].starts_with('|') {
            content_start += 1;
        }
        if content_start < content_end
            && line[content_start..content_end].ends_with('|')
            && !is_escaped_table_pipe(
                &line.as_bytes()[content_start..content_end],
                content_end - content_start - 1,
            )
        {
            let final_pipe = content_end - 1;
            let mut run_start = final_pipe;
            while run_start > content_start
                && line.as_bytes()[run_start - 1] == b'|'
                && !is_escaped_table_pipe(line.as_bytes(), run_start - 1)
            {
                run_start -= 1;
            }
            if run_start == final_pipe {
                content_end -= 1;
            }
        }
        let content = &line[content_start..content_end];
        let bytes = content.as_bytes();
        let mut cell_start = 0;
        let mut pipes = PipeCursor::new(bytes, 0);

        std::iter::from_fn(move || {
            if cell_start > bytes.len() {
                return None;
            }

            let mut escapes = EscapedPipes::default();
            while let Some(pipe) = pipes.next_pipe() {
                if pipe.escaped {
                    escapes.record(pipe.offset);
                    continue;
                }
                let raw = &content[cell_start..pipe.offset];
                let (cell, start, end) = trim_cell(raw, content_start + cell_start);
                let mut pipe_count = 1;
                // Adjacent pipes widen the cell. A `|` right after a pipe
                // has no backslash in front of it, so it is never escaped,
                // and it is the next pipe the cursor reports: taking it
                // from the cursor keeps the walk in step with the run.
                while bytes.get(pipe.offset + pipe_count) == Some(&b'|') {
                    let adjacent = pipes.next_pipe();
                    debug_assert_eq!(
                        adjacent,
                        Some(Pipe {
                            offset: pipe.offset + pipe_count,
                            escaped: false,
                        })
                    );
                    pipe_count += 1;
                }
                // A preserved terminal run is the complete remainder of
                // the row. Mark the iterator finished after yielding it;
                // otherwise the next call fabricates an empty cell.
                cell_start = if pipe.offset + pipe_count == bytes.len() {
                    bytes.len() + 1
                } else {
                    pipe.offset + pipe_count
                };
                return Some(RowCell {
                    content: cell,
                    start,
                    end,
                    escapes: escapes.shifted(start - content_start),
                    colspan: pipe_count,
                });
            }

            let raw = &content[cell_start..];
            let (cell, start, end) = trim_cell(raw, content_start + cell_start);
            cell_start = bytes.len() + 1;
            Some(RowCell {
                content: cell,
                start,
                end,
                escapes: escapes.shifted(start - content_start),
                colspan: 1,
            })
        })
    }
}

fn trim_cell(cell: &str, offset: usize) -> (&str, usize, usize) {
    let (trimmed, leading) = whitespace::trim_with_leading(cell);
    let start = offset + leading;
    (trimmed, start, start + trimmed.len())
}

fn delimiter_alignment(cell: &str) -> Option<AlignKind> {
    let trimmed = whitespace::trim(cell);
    let left = trimmed.starts_with(':');
    let right = trimmed.ends_with(':');
    let hyphens = trimmed.strip_prefix(':').unwrap_or(trimmed);
    let hyphens = hyphens.strip_suffix(':').unwrap_or(hyphens);

    if hyphens.is_empty() || !hyphens.bytes().all(|byte| byte == b'-') {
        return None;
    }

    Some(match (left, right) {
        (true, true) => AlignKind::Center,
        (true, false) => AlignKind::Left,
        (false, true) => AlignKind::Right,
        (false, false) => AlignKind::None,
    })
}

#[cfg(test)]
mod reference;
#[cfg(test)]
mod tests;
