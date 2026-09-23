// Test fixtures deliberately use owned formatting outside parser hot paths.
#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    clippy::disallowed_types
)]

use super::unescape_table_pipes;
use crate::allocator::Allocator;
use crate::parser::table_pipes::{EscapedPipes, is_escaped_table_pipe};

/// Decodes a standalone cell the way the row splitter would: with the record
/// of the escaped pipes it passed over inside that cell.
fn decode<'a>(allocator: &'a Allocator, content: &'a str) -> super::TableCellContent<'a> {
    unescape_table_pipes(allocator, content, EscapedPipes::scan(content.as_bytes()))
}

#[test]
fn pipe_candidates_match_scalar_scan_across_length_and_escape_boundaries() {
    for prefix_len in 0..=128 {
        for backslashes in 0..=5 {
            for suffix in ["", "|", " \\| end", " 日本語 \\| 🙂"] {
                let source = format!(
                    "{}{}|{suffix}",
                    "x".repeat(prefix_len),
                    "\\".repeat(backslashes)
                );
                let bytes = source.as_bytes();
                let expected = bytes.iter().enumerate().find_map(|(index, &byte)| {
                    (byte == b'|' && is_escaped_table_pipe(bytes, index)).then_some(index)
                });
                let scan_start = EscapedPipes::scan(bytes).bounds().map(|(first, _)| first);
                assert_eq!(scan_start, expected, "{source:?}");
                assert_eq!(
                    super::reference::escaped_pipe_scan_start(bytes).is_some(),
                    expected.is_some(),
                    "{source:?}"
                );
            }
        }
        assert_eq!(
            EscapedPipes::scan("x".repeat(prefix_len).as_bytes()).bounds(),
            None
        );
    }
}

#[test]
fn skipped_unescaped_pipes_keep_content_and_sparse_source_map() {
    for prefix_len in [0, 8, 31, 63, 64, 65, 128] {
        let prefix = "x".repeat(prefix_len);
        let source = format!("{prefix}|日本語\\|end");
        let allocator = Allocator::new();
        let result = decode(&allocator, &source);
        assert_eq!(result.content, format!("{prefix}|日本語|end"));
        let source_map = result.source_map.unwrap();
        let removed_at = prefix_len + "|日本語".len();
        assert_eq!(source_map.removed_total, 1);
        assert_eq!(
            source_map.blocks[removed_at / 64].bits & (1 << (removed_at % 64)),
            1 << (removed_at % 64)
        );
        for boundary in 0..=result.content.len() {
            let expected = boundary + usize::from(boundary > removed_at);
            assert_eq!(source_map.boundary_offset(boundary) as usize, expected);
        }
        assert_eq!(
            source_map.boundary_offset(result.content.len() + 100),
            source.len() as u32
        );
    }
}

#[test]
fn adjacent_escaped_pipes_map_each_generated_boundary() {
    let allocator = Allocator::new();
    let result = decode(&allocator, r"a\|\|b");
    assert_eq!(result.content, "a||b");
    let source_map = result.source_map.unwrap();
    assert_eq!(source_map.removed_total, 2);
    let mapped = std::array::from_fn(|boundary| source_map.boundary_offset(boundary));
    assert_eq!(mapped, [0, 1, 3, 5, 6]);
}

#[test]
fn sparse_map_matches_dense_scalar_oracle() {
    let allocator = Allocator::new();
    for source in [
        r"plain text",
        r"a\|b|c",
        r"a\\|b\|c",
        r"\|\|\|",
        r"slashes \\| and escaped \| pipes",
    ] {
        let result = decode(&allocator, source);
        let bytes = source.as_bytes();
        let mut expected_content = allocator.new_string();
        let mut expected_offsets = [0usize; 128];
        let mut expected_len = 1;
        let mut source_index = 0;
        while source_index < bytes.len() {
            if bytes[source_index] == b'|'
                && is_escaped_table_pipe(bytes, source_index)
                && source_index > 0
            {
                // The slash was already emitted by the previous iteration;
                // remove it and repair the boundary immediately before the
                // generated pipe.
                expected_content.pop();
                expected_len -= 1;
                expected_offsets[expected_len - 1] = source_index - 1;
                expected_content.push('|');
                expected_offsets[expected_len] = source_index + 1;
                expected_len += 1;
                source_index += 1;
            } else {
                expected_content.push(bytes[source_index] as char);
                source_index += 1;
                expected_offsets[expected_len] = source_index;
                expected_len += 1;
            }
        }
        assert_eq!(result.content, expected_content.as_str(), "{source:?}");
        let source_map = result.source_map.as_ref();
        for (boundary, expected) in expected_offsets[..expected_len].iter().copied().enumerate() {
            assert_eq!(
                source_map.map_or(boundary as u32, |map| map.boundary_offset(boundary)) as usize,
                expected,
                "{source:?} boundary {boundary}"
            );
        }
        if let Some(map) = source_map {
            assert_eq!(
                map.boundary_offset(result.content.len() + 100),
                source.len() as u32
            );
        }
    }
}

#[test]
fn decoding_with_the_splitter_record_matches_the_standalone_decoder() {
    // Cells as the previous decoder saw them on its own, including stray
    // unescaped pipes the splitter never leaves inside a cell.
    let fragments = [
        "x",
        " ",
        "|",
        r"\|",
        r"\\|",
        r"\\\|",
        "\\",
        "`",
        "`a|b`",
        "日本語",
        "🙂",
    ];
    let allocator = Allocator::new();
    let mut state = 0x9E37_79B9_7F4A_7C15_u64;
    let mut next = move |bound: usize| {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        (state % bound as u64) as usize
    };
    let mut cells = std::vec::Vec::new();
    for _ in 0..4_000 {
        let mut cell = std::string::String::new();
        for _ in 0..next(48) {
            cell.push_str(fragments[next(fragments.len())]);
        }
        cells.push(cell);
    }
    for prefix_len in [0, 1, 15, 16, 17, 63, 64, 65] {
        for suffix in [r"\|", r"a\|b", r"\\\|\|", r"\\\\\\\\\\\\\\\\\|"] {
            cells.push(format!(
                "{}{suffix}{}",
                "x".repeat(prefix_len),
                "y".repeat(prefix_len)
            ));
        }
    }
    for cell in &cells {
        let decoded = decode(&allocator, cell);
        let expected = super::reference::unescape_table_pipes(&allocator, cell);
        assert_eq!(decoded.content, expected.content, "{cell:?}");
        match (&decoded.source_map, &expected.source_map) {
            (None, None) => assert!(std::ptr::eq(decoded.content, cell.as_str()), "{cell:?}"),
            (Some(map), Some(expected_map)) => {
                for boundary in 0..=decoded.content.len() + 2 {
                    assert_eq!(
                        map.boundary_offset(boundary),
                        expected_map.boundary_offset(boundary),
                        "{cell:?} boundary {boundary}"
                    );
                }
            }
            _ => panic!("source maps differ in presence: {cell:?}"),
        }
    }
}
