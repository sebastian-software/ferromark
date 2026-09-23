// Test fixtures deliberately use owned formatting outside parser hot paths.
#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    clippy::disallowed_types
)]

use super::{EscapedPipes, Pipe, PipeCursor, is_escaped_table_pipe};
use memchr::memchr_iter;

/// The scan the table parser used before [`PipeCursor`] existed: find every
/// pipe with `memchr`, then walk backwards over the backslashes in front of
/// it. It is the reference the vector path has to reproduce exactly.
fn scalar_pipes(bytes: &[u8], from: usize) -> Vec<Pipe> {
    memchr_iter(b'|', &bytes[from..])
        .map(|relative| Pipe {
            offset: from + relative,
            escaped: is_escaped_table_pipe(bytes, from + relative),
        })
        .collect()
}

fn cursor_pipes(bytes: &[u8], from: usize) -> Vec<Pipe> {
    let mut cursor = PipeCursor::new(bytes, from);
    let mut pipes = Vec::new();
    while let Some(pipe) = cursor.next_pipe() {
        pipes.push(pipe);
    }
    // A spent cursor stays spent.
    assert_eq!(cursor.next_pipe(), None);
    pipes
}

/// Row shapes that exercise window boundaries, backslash runs that straddle
/// them, adjacent pipes, leading and trailing pipes, and multibyte cells.
fn row_shapes() -> Vec<String> {
    let mut rows = vec![
        String::new(),
        String::from("|"),
        String::from("||"),
        String::from("no pipes at all"),
        String::from(r"\|"),
        String::from(r"\\|"),
        String::from(r"\\\|"),
        String::from(r"a\|b|c"),
        String::from(r"| a \| b | c \\| d |"),
        String::from(r"| 日本語 \| 🙂 | ok |"),
        String::from(r"|||| trailing run ||||"),
        String::from(r"\\\\\\\\\\\\\\\\|"),
        String::from("row ending in a backslash \\"),
    ];
    // A pipe placed at every offset around and across the window and
    // 64-byte boundaries, behind backslash runs of every parity, with and
    // without a run that reaches back into the window before it.
    for prefix in 0..70_usize {
        for backslashes in 0..=6_usize {
            for suffix in ["", "|", r" \| tail", r" \\| tail", " 日本語", "\\"] {
                let mut row = "x".repeat(prefix);
                row.push_str(&"\\".repeat(backslashes));
                row.push('|');
                row.push_str(suffix);
                rows.push(row);
            }
        }
    }
    // Long backslash runs that fill and overrun a whole window.
    for backslashes in 12..=36_usize {
        let mut row = String::from("cell ");
        row.push_str(&"\\".repeat(backslashes));
        row.push_str("|rest");
        rows.push(row);
    }
    // Pipes packed into consecutive windows, escaped and not, so the
    // cursor moves from window to window without `memchr` in between.
    for token in [r"\|", "|", r"\\|", r"a\|", "`|`", r"**b\|p** "] {
        for repeats in [1, 7, 8, 9, 15, 16, 17, 31, 33, 64, 65] {
            rows.push(token.repeat(repeats));
            rows.push(format!(" {} | end ", token.repeat(repeats)));
        }
    }
    rows
}

#[test]
fn cursor_reports_the_same_pipes_and_escapes_as_the_scalar_scan() {
    for row in row_shapes() {
        let bytes = row.as_bytes();
        for from in 0..=bytes.len() {
            assert_eq!(
                cursor_pipes(bytes, from),
                scalar_pipes(bytes, from),
                "row {row:?} from {from}"
            );
        }
    }
}

#[test]
fn cutting_the_slice_after_a_pipe_changes_nothing_before_the_cut() {
    // The cell decoder walks its cell cut right after the last escape.
    for row in row_shapes() {
        let bytes = row.as_bytes();
        for end in 0..=bytes.len() {
            let cut = &bytes[..end];
            for from in [0, end / 3, end / 2, end.saturating_sub(1), end] {
                assert_eq!(
                    cursor_pipes(cut, from),
                    scalar_pipes(cut, from),
                    "row {row:?} cut at {end} from {from}"
                );
            }
        }
    }
}

#[test]
fn escaped_pipe_records_bound_the_escapes_of_a_cell() {
    for row in row_shapes() {
        let bytes = row.as_bytes();
        let escaped: Vec<usize> = scalar_pipes(bytes, 0)
            .into_iter()
            .filter(|pipe| pipe.escaped)
            .map(|pipe| pipe.offset)
            .collect();
        let bounds = EscapedPipes::scan(bytes).bounds();
        assert_eq!(
            bounds,
            escaped
                .first()
                .map(|&first| (first, *escaped.last().expect("a first implies a last"))),
            "{row:?}"
        );
    }
}

#[test]
fn shifting_a_record_moves_it_into_trimmed_cell_coordinates() {
    let mut escapes = EscapedPipes::default();
    assert_eq!(escapes.bounds(), None);
    assert_eq!(escapes.shifted(7).bounds(), None);
    escapes.record(9);
    assert_eq!(escapes.bounds(), Some((9, 9)));
    escapes.record(21);
    assert_eq!(escapes.bounds(), Some((9, 21)));
    assert_eq!(escapes.shifted(4).bounds(), Some((5, 17)));
    assert_eq!(escapes.shifted(9).bounds(), Some((0, 12)));
    assert_eq!(escapes.shifted(0).bounds(), Some((9, 21)));

    let mut at_zero = EscapedPipes::default();
    at_zero.record(0);
    assert_eq!(at_zero.bounds(), Some((0, 0)));
    assert_eq!(at_zero.shifted(0).bounds(), Some((0, 0)));
}
