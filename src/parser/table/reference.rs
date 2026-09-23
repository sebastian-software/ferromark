//! The row splitters as they were before [`PipeCursor`] walked a row once,
//! kept verbatim as the oracle for differential tests: one `memchr` call
//! per pipe and a backwards walk over the backslashes in front of each.
//!
//! [`PipeCursor`]: crate::parser::table_pipes::PipeCursor

use memchr::memchr;

use super::trim_cell;
use crate::parser::table_cell_source::reference::is_escaped_table_pipe;
use crate::parser::whitespace;

pub(super) fn table_row_cells_with_offsets(
    line: &str,
) -> impl Iterator<Item = (&str, usize, usize)> {
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

    std::iter::from_fn(move || {
        if cell_start > bytes.len() {
            return None;
        }

        let mut search_start = cell_start;
        while let Some(relative) = memchr(b'|', &bytes[search_start..]) {
            let pipe = search_start + relative;
            if !is_escaped_table_pipe(bytes, pipe) {
                let raw = &content[cell_start..pipe];
                let (cell, start, end) = trim_cell(raw, content_start + cell_start);
                cell_start = pipe + 1;
                return Some((cell, start, end));
            }
            search_start = pipe + 1;
        }

        let raw = &content[cell_start..];
        let (cell, start, end) = trim_cell(raw, content_start + cell_start);
        cell_start = bytes.len() + 1;
        Some((cell, start, end))
    })
}

pub(super) fn table_row_cells_with_spans(
    line: &str,
) -> impl Iterator<Item = (&str, usize, usize, usize)> {
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

    std::iter::from_fn(move || {
        if cell_start > bytes.len() {
            return None;
        }

        let mut search_start = cell_start;
        while let Some(relative) = memchr(b'|', &bytes[search_start..]) {
            let pipe = search_start + relative;
            if !is_escaped_table_pipe(bytes, pipe) {
                let raw = &content[cell_start..pipe];
                let (cell, start, end) = trim_cell(raw, content_start + cell_start);
                let mut pipe_count = 1;
                while pipe + pipe_count < bytes.len()
                    && bytes[pipe + pipe_count] == b'|'
                    && !is_escaped_table_pipe(bytes, pipe + pipe_count)
                {
                    pipe_count += 1;
                }
                cell_start = if pipe + pipe_count == bytes.len() {
                    bytes.len() + 1
                } else {
                    pipe + pipe_count
                };
                return Some((cell, start, end, pipe_count));
            }
            search_start = pipe + 1;
        }

        let raw = &content[cell_start..];
        let (cell, start, end) = trim_cell(raw, content_start + cell_start);
        cell_start = bytes.len() + 1;
        Some((cell, start, end, 1))
    })
}
