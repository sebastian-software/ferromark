//! The cell decoder as it was before the row splitter handed it the escapes
//! it had seen, kept verbatim as the oracle for differential tests.
//!
//! It finds the escapes of a standalone cell on its own: every byte of a
//! short cell, `memchr` plus a backwards walk per pipe of a long one, and
//! then every pipe again from the first escape on.

use memchr::memchr;

use super::{TableCellContent, TableCellSourceMap, push_unescaped_table_cell_slice};
use crate::allocator::Allocator;

pub(in crate::parser) fn unescape_table_pipes<'a>(
    allocator: &'a Allocator,
    content: &'a str,
) -> TableCellContent<'a> {
    let bytes = content.as_bytes();
    let Some(first_pipe) = escaped_pipe_scan_start(bytes) else {
        return TableCellContent {
            content,
            source_map: None,
        };
    };

    let mut unescaped = crate::allocator::String::with_capacity_in(content.len(), allocator.bump());
    let mut source_map = TableCellSourceMap::new(allocator);
    let mut copied_through = 0;
    let mut search_start = first_pipe;
    while let Some(relative) = memchr(b'|', &bytes[search_start..]) {
        let pipe = search_start + relative;
        if is_escaped_table_pipe(bytes, pipe) {
            push_unescaped_table_cell_slice(content, copied_through, pipe - 1, &mut unescaped);
            source_map.record_removed(unescaped.len());
            unescaped.push('|');
            copied_through = pipe + 1;
        }
        search_start = pipe + 1;
    }
    push_unescaped_table_cell_slice(content, copied_through, content.len(), &mut unescaped);
    source_map.finish(unescaped.len());
    TableCellContent {
        content: unescaped.into_bump_str(),
        source_map: Some(source_map),
    }
}

pub(in crate::parser) fn escaped_pipe_scan_start(bytes: &[u8]) -> Option<usize> {
    if bytes.len() < 64 {
        return bytes
            .iter()
            .enumerate()
            .any(|(index, &byte)| byte == b'|' && is_escaped_table_pipe(bytes, index))
            .then_some(0);
    }
    memchr::memchr_iter(b'|', bytes).find(|&index| is_escaped_table_pipe(bytes, index))
}

pub(in crate::parser) fn is_escaped_table_pipe(bytes: &[u8], pipe: usize) -> bool {
    bytes[..pipe]
        .iter()
        .rev()
        .take_while(|&&byte| byte == b'\\')
        .count()
        % 2
        == 1
}
