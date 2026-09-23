// Test fixtures deliberately use owned formatting outside parser hot paths.
#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    clippy::disallowed_types
)]

//! Differential tests: the row splitters and the cell decoder against the
//! verbatim previous implementations in `reference` modules, cell by cell,
//! over generated rows and every line of a frozen measurement corpus.

use std::path::Path;

use super::{Parser, RowCell, reference};
use crate::allocator::Allocator;
use crate::parser::table_cell_source::{self, unescape_table_pipes};

/// Checks one row in both splitting modes: the same cells with the same
/// offsets and spans, and every cell decoding to the same text and source
/// map as the previous decoder produced from the cell alone.
fn check_row(allocator: &Allocator, line: &str) {
    let cells: Vec<RowCell<'_>> = Parser::table_row_cells_with_offsets(line).collect();
    let expected: Vec<_> = reference::table_row_cells_with_offsets(line).collect();
    assert_eq!(cells.len(), expected.len(), "cell count of {line:?}");
    for (cell, &(content, start, end)) in cells.iter().zip(&expected) {
        assert_eq!(
            (cell.content, cell.start, cell.end, cell.colspan),
            (content, start, end, 1),
            "{line:?}"
        );
        check_decoded(allocator, cell, line);
    }

    let cells: Vec<RowCell<'_>> = Parser::table_row_cells_with_spans(line).collect();
    let expected: Vec<_> = reference::table_row_cells_with_spans(line).collect();
    assert_eq!(cells.len(), expected.len(), "merged cell count of {line:?}");
    for (cell, &(content, start, end, colspan)) in cells.iter().zip(&expected) {
        assert_eq!(
            (cell.content, cell.start, cell.end, cell.colspan),
            (content, start, end, colspan),
            "merged {line:?}"
        );
        check_decoded(allocator, cell, line);
    }
}

fn check_decoded(allocator: &Allocator, cell: &RowCell<'_>, line: &str) {
    let decoded = unescape_table_pipes(allocator, cell.content, cell.escapes);
    let expected = table_cell_source::reference::unescape_table_pipes(allocator, cell.content);
    assert_eq!(decoded.content, expected.content, "cell of {line:?}");
    match (&decoded.source_map, &expected.source_map) {
        (None, None) => assert!(
            std::ptr::eq(decoded.content, cell.content),
            "a cell without escapes stays borrowed: {line:?}"
        ),
        (Some(map), Some(expected_map)) => {
            for boundary in 0..=decoded.content.len() + 2 {
                assert_eq!(
                    map.boundary_offset(boundary),
                    expected_map.boundary_offset(boundary),
                    "cell of {line:?}, boundary {boundary}"
                );
            }
        }
        _ => panic!("source maps differ in presence for a cell of {line:?}"),
    }
}

/// A small deterministic generator, so a failure names a reproducible row.
struct XorShift(u64);

impl XorShift {
    fn below(&mut self, bound: usize) -> usize {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 % bound as u64) as usize
    }
}

#[test]
fn rows_split_and_decode_like_the_previous_implementation() {
    let allocator = Allocator::new();
    let edges = [
        ("", ""),
        ("|", "|"),
        ("| ", " |"),
        ("  |", "|  "),
        ("|", r"\|"),
        ("|", "\\"),
        ("", "||"),
        ("\t| ", " ||| "),
    ];
    let suffixes = [
        "",
        "|",
        r" \| end",
        r" \\| end",
        " `a|b` ",
        r" `\|` ",
        " 日本語 \\| 🙂",
        "|||",
    ];
    // A pipe at every offset across the 16-byte vector windows and 64-byte
    // source-map blocks, behind backslash runs of every parity up to longer
    // than a window, inside every kind of row edge.
    for prefix in 0..=70_usize {
        for backslashes in 0..=20_usize {
            for (lead, tail) in edges {
                for suffix in suffixes {
                    let mut row = String::from(lead);
                    row.push_str(&"x".repeat(prefix));
                    row.push_str(&"\\".repeat(backslashes));
                    row.push('|');
                    row.push_str(suffix);
                    row.push_str(tail);
                    check_row(&allocator, &row);
                }
            }
        }
    }
}

#[test]
fn random_rows_split_and_decode_like_the_previous_implementation() {
    let allocator = Allocator::new();
    let fragments = [
        "|",
        "||",
        "|||",
        r"\|",
        r"\\|",
        r"\\\|",
        "\\",
        r"\\",
        r"\\\\\\\\\\\\\\\\\\",
        "`",
        "`a|b`",
        r"`\|`",
        "**b\\|p**",
        " ",
        "\t",
        "x",
        "word ",
        "日本語",
        "🙂",
        "-",
        ":",
    ];
    let mut random = XorShift(0x2545_F491_4F6C_DD1D);
    for _ in 0..30_000 {
        let mut row = String::new();
        let pieces = if random.below(8) == 0 {
            random.below(160)
        } else {
            random.below(24)
        };
        for _ in 0..pieces {
            row.push_str(fragments[random.below(fragments.len())]);
        }
        check_row(&allocator, &row);
    }
}

#[test]
fn corpus_lines_split_and_decode_like_the_previous_implementation() {
    let relative = "docs/reports/2026-09-14-simd-round/corpus.json.gz";
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(relative);
    let Ok(raw) = std::fs::read(&path) else {
        // Published crate archives carry `tests/` but not `docs/`.
        return;
    };
    let json = crate::parser::prepass::gzip::gunzip(&raw)
        .unwrap_or_else(|error| panic!("{relative}: {error}"));
    let corpus: serde_json::Value =
        serde_json::from_slice(&json).unwrap_or_else(|error| panic!("{relative}: {error}"));
    let cases = corpus["cases"].as_array().expect("cases array");
    assert_eq!(cases.len(), 72, "{relative}: case count");
    let allocator = Allocator::new();
    let mut lines = 0usize;
    for case in cases {
        let input = case["input"].as_str().expect("case input");
        // Every line is split as a row, table or not, with and without its
        // line ending, so prose, code and markup all pass through.
        for line in input.split_inclusive('\n') {
            check_row(&allocator, line);
            check_row(&allocator, line.trim_end_matches(['\n', '\r']));
            lines += 1;
        }
    }
    assert!(
        lines > 12_000,
        "expected the whole corpus, got {lines} lines"
    );
}
