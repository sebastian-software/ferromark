// Test fixtures deliberately use owned formatting outside parser hot paths.
#![allow(clippy::disallowed_macros)]

use super::{escaped_pipe_scan_start, is_escaped_table_pipe, unescape_table_pipes};
use ferromark_allocator::Allocator;

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
                let scan_start = escaped_pipe_scan_start(bytes);
                assert_eq!(scan_start.is_some(), expected.is_some(), "{source:?}");
                if let (Some(start), Some(first)) = (scan_start, expected) {
                    assert!(
                        start <= first,
                        "must not skip the first escaped pipe: {source:?}"
                    );
                }
            }
        }
        assert_eq!(
            escaped_pipe_scan_start("x".repeat(prefix_len).as_bytes()),
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
        let result = unescape_table_pipes(&allocator, &source);
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
    let result = unescape_table_pipes(&allocator, r"a\|\|b");
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
        let result = unescape_table_pipes(&allocator, source);
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
