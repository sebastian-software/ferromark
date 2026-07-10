use ferromark::{Options, limits, to_html, to_html_with_options};

#[test]
fn block_container_nesting_is_bounded() {
    let markdown = format!("{}content", "> ".repeat(limits::MAX_BLOCK_NESTING * 2));
    let html = to_html(&markdown);

    assert_eq!(
        html.matches("<blockquote>").count(),
        limits::MAX_BLOCK_NESTING
    );
}

#[test]
fn inline_mark_collection_is_bounded() {
    let markdown = "*x* ".repeat(limits::MAX_INLINE_MARKS);
    let html = to_html(&markdown);

    assert!(html.matches("<em>").count() <= limits::MAX_INLINE_MARKS / 2);
    assert!(html.ends_with("</p>\n"));
}

#[test]
fn oversized_backtick_runs_stay_literal() {
    let fence = "`".repeat(limits::MAX_CODE_SPAN_BACKTICKS + 1);
    let html = to_html(&format!("{fence}code{fence}"));

    assert!(!html.contains("<code>"));
    assert!(html.contains(&fence));
}

#[test]
fn link_destination_parentheses_are_bounded() {
    let at_limit = format!(
        "[ok](url{}{})",
        "(".repeat(limits::MAX_LINK_PAREN_DEPTH),
        ")".repeat(limits::MAX_LINK_PAREN_DEPTH)
    );
    let over_limit = format!(
        "[no](url{}{})",
        "(".repeat(limits::MAX_LINK_PAREN_DEPTH + 1),
        ")".repeat(limits::MAX_LINK_PAREN_DEPTH + 1)
    );

    assert!(to_html(&at_limit).contains("<a href="));
    assert!(!to_html(&over_limit).contains("<a href="));
}

#[test]
fn ordered_list_marker_digits_are_bounded() {
    let at_limit = format!("{}. item", "1".repeat(limits::MAX_LIST_MARKER_DIGITS));
    let over_limit = format!("{}. item", "1".repeat(limits::MAX_LIST_MARKER_DIGITS + 1));

    assert!(to_html(&at_limit).starts_with("<ol"));
    assert!(to_html(&over_limit).starts_with("<p>"));
}

#[test]
fn table_columns_are_bounded() {
    let columns = limits::MAX_TABLE_COLUMNS + 16;
    let header = std::iter::repeat_n("cell", columns)
        .collect::<Vec<_>>()
        .join(" | ");
    let delimiter = std::iter::repeat_n("---", columns)
        .collect::<Vec<_>>()
        .join(" | ");
    let markdown = format!("| {header} |\n| {delimiter} |\n");
    let html = to_html_with_options(
        &markdown,
        &Options {
            tables: true,
            ..Options::default()
        },
    );

    assert_eq!(html.matches("<th>").count(), limits::MAX_TABLE_COLUMNS);
}
