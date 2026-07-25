use ferromark::{BlockEvent, BlockParser, Options, to_html_with_options};

fn options() -> Options {
    Options {
        tables: true,
        merged_table_cells: true,
        ..Options::default()
    }
}

fn render(input: &str) -> String {
    to_html_with_options(input, &options())
}

#[test]
fn consecutive_pipes_remain_empty_gfm_cells_when_disabled() {
    let input = "\
| Name | Price | Tax |
| --- | --- | --- |
| Gift | 0$ ||";
    let html = to_html_with_options(input, &Options::default());

    assert!(!html.contains("colspan="), "{html}");
    assert!(html.contains("<td>0$</td>\n<td></td>"), "{html}");
}

#[test]
fn trailing_pipe_count_becomes_colspan() {
    let input = "\
| Name | Price | Tax |
| --- | --- | --- |
| Gift | 0$ ||";
    let html = render(input);

    assert!(html.contains("<td colspan=\"2\">0$</td>"), "{html}");
    assert_eq!(html.matches("<td").count(), 2, "{html}");
}

#[test]
fn three_pipes_span_three_columns() {
    let html = render(
        "\
| A | B | C | D |
| --- | --- | --- | --- |
| First | Remaining |||",
    );

    assert!(html.contains("<td colspan=\"3\">Remaining</td>"), "{html}");
    assert_eq!(html.matches("<td").count(), 2, "{html}");
}

#[test]
fn one_row_can_contain_multiple_merged_cells() {
    let html = render(
        "\
| 1 | 2 | 3 | 4 | 5 |
| --- | --- | --- | --- | --- |
| A || B || C |",
    );

    assert_eq!(html.matches("colspan=\"2\"").count(), 2, "{html}");
    assert!(html.contains("<td colspan=\"2\">A</td>"), "{html}");
    assert!(html.contains("<td colspan=\"2\">B</td>"), "{html}");
    assert!(html.contains("<td>C</td>"), "{html}");
}

#[test]
fn merged_header_cells_participate_in_table_recognition() {
    let html = render(
        "\
| Group || Other |
| :--- | ---: | :---: |
| A | B | C |",
    );

    assert!(
        html.contains("<th align=\"left\" colspan=\"2\">Group</th>"),
        "{html}"
    );
    assert!(html.contains("<th align=\"center\">Other</th>"), "{html}");
}

#[test]
fn merged_cells_compose_with_underlying_column_width_hints() {
    let options = Options {
        tables: true,
        merged_table_cells: true,
        table_column_widths: true,
        ..Options::default()
    };
    let html = to_html_with_options(
        "\
| Group ||
| -- | ------ |
| all ||",
        &options,
    );

    assert!(html.contains("<col style=\"width: 25%\">"), "{html}");
    assert!(html.contains("<col style=\"width: 75%\">"), "{html}");
    assert!(html.contains("<th colspan=\"2\">Group</th>"), "{html}");
    assert!(html.contains("<td colspan=\"2\">all</td>"), "{html}");
}

#[test]
fn whitespace_between_pipes_preserves_an_explicit_empty_cell() {
    let html = render(
        "\
| A | B | C |
| --- | --- | --- |
| first | | third |",
    );

    assert!(!html.contains("colspan="), "{html}");
    assert!(html.contains("<td></td>"), "{html}");
    assert_eq!(html.matches("<td").count(), 3, "{html}");
}

#[test]
fn ragged_rows_are_padded_after_the_last_span() {
    let html = render(
        "\
| A | B | C | D |
| --- | --- | --- | --- |
| merged || third |",
    );

    assert!(html.contains("<td colspan=\"2\">merged</td>"), "{html}");
    assert!(html.contains("<td>third</td>\n<td></td>"), "{html}");
    assert_eq!(html.matches("<td").count(), 3, "{html}");
}

#[test]
fn spans_are_clamped_to_the_table_width() {
    let html = render(
        "\
| A | B | C | D |
| --- | --- | --- | --- |
| all |||||",
    );

    assert!(html.contains("<td colspan=\"4\">all</td>"), "{html}");
    assert_eq!(html.matches("<td").count(), 1, "{html}");
}

#[test]
fn merged_cells_keep_inline_markdown_and_starting_column_alignment() {
    let html = render(
        "\
| A | B | C |
| :--- | ---: | :---: |
| first | **two columns** ||",
    );

    assert!(
        html.contains("<td align=\"right\" colspan=\"2\"><strong>two columns</strong></td>"),
        "{html}"
    );
}

#[test]
fn parser_emits_semantic_colspans() {
    let input = b"\
| A || B |
| --- | --- | --- |
| C || D |";
    let mut parser = BlockParser::new_with_options(input, options());
    let mut events = Vec::new();
    parser.parse(&mut events);

    let spans: Vec<u16> = events
        .iter()
        .filter_map(|event| match event {
            BlockEvent::TableCellStart { colspan, .. } => Some(*colspan),
            _ => None,
        })
        .collect();
    assert_eq!(spans, vec![2, 1, 2, 1]);
}

#[test]
fn escaped_and_code_span_pipes_do_not_change_colspan() {
    let html = render(
        "\
| A | B | C |
| --- | --- | --- |
| `a|b` | escaped \\| pipe ||",
    );

    assert!(html.contains("<td><code>a|b</code></td>"), "{html}");
    assert!(
        html.contains("<td colspan=\"2\">escaped | pipe</td>"),
        "{html}"
    );
}
