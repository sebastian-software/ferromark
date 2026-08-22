use ferromark::{
    InlineEvent, InlineParser, LinkRefDef, LinkRefStore, Options, limits, to_html,
    to_html_with_options,
};

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
fn reference_link_resolution_budget_is_shared_across_paragraphs() {
    let paragraphs = limits::MAX_REFERENCE_RESOLUTION_WORK / 3 + 2;
    let markdown = format!("[x]: /safe\n\n{}", "[x]\n\n".repeat(paragraphs));
    let html = to_html(&markdown);

    // A `[x]` reference spends two bracket records plus one label byte. The
    // final paragraphs are deliberately left literal once the document budget
    // is exhausted, proving that the allowance does not reset per paragraph.
    assert_eq!(html.matches("<a href=\"/safe\">").count(), paragraphs - 2);
    assert!(html.ends_with("<p>[x]</p>\n"));
}

#[cfg(feature = "mdx")]
#[test]
fn mdx_reference_budget_is_shared_across_markdown_segments() {
    use ferromark::mdx::{MdxEvent, parse_events};

    let references_per_segment = limits::MAX_REFERENCE_RESOLUTION_WORK / 3;
    let segment = "[x]\n\n".repeat(references_per_segment);
    let input = format!("[x]: /safe\n\n{segment}<Component />\n\n{segment}");
    let stream = parse_events(&input);

    // A flow JSX separator creates two independent Markdown segments, but
    // they are still one MDX document and therefore share one work allowance.
    assert_eq!(
        stream
            .events
            .iter()
            .filter(|event| matches!(event, MdxEvent::Inline(InlineEvent::LinkStartRef { .. })))
            .count(),
        references_per_segment
    );
    assert!(
        stream
            .events
            .iter()
            .any(|event| matches!(event, MdxEvent::FlowJsxSelfClose(_)))
    );
}

#[test]
fn reusable_inline_parser_resets_reference_budget_per_document() {
    let mut refs = LinkRefStore::new();
    refs.insert(
        "x".to_owned(),
        LinkRefDef {
            url: b"/safe".to_vec(),
            title: None,
        },
    );
    let mut parser = InlineParser::new();
    let mut events = Vec::new();

    // One document depletes its allowance. Reusing the public parser for a
    // separate document must start a fresh document budget.
    let exhausting_document = "[x] ".repeat(limits::MAX_REFERENCE_RESOLUTION_WORK / 3 + 1);
    parser.parse(
        exhausting_document.as_bytes(),
        Some(&refs),
        true,
        &mut events,
    );

    events.clear();
    parser.parse(b"[x]", Some(&refs), true, &mut events);
    assert!(
        matches!(
            events.first(),
            Some(InlineEvent::LinkStartRef { def_index: 0 })
        ),
        "a reusable parser must not retain the prior document's exhausted budget: {events:?}"
    );
}

#[test]
fn exhausted_reference_budget_keeps_completed_candidates_in_the_current_paragraph() {
    let completed_paragraphs = limits::MAX_REFERENCE_RESOLUTION_WORK / 3 - 3;
    let markdown = format!(
        "[x]: /safe\n\n{}[x] [x] [x] [x]",
        "[x]\n\n".repeat(completed_paragraphs)
    );
    let html = to_html(&markdown);

    // The final paragraph has budget for its bracket structure and exactly
    // three labels. Its completed candidates still resolve; only the final
    // candidate falls back to literal text.
    assert_eq!(
        html.matches("<a href=\"/safe\">").count(),
        completed_paragraphs + 3
    );
    assert!(html.ends_with("<a href=\"/safe\">x</a> [x]</p>\n"));
}

#[test]
fn exhausted_reference_budget_keeps_completed_outer_full_reference_links() {
    // Leave enough work for the final paragraph's bracket structure and its
    // outer full-reference candidate, but not its unresolved inner openers.
    let completed_paragraphs = (limits::MAX_REFERENCE_RESOLUTION_WORK - 24) / 3;
    let markdown = format!(
        "[x]: /safe\n[label]: /target\n\n{}[x [a [b] ] ][label]",
        "[x]\n\n".repeat(completed_paragraphs)
    );
    let html = to_html(&markdown);

    // The uninspected `[a ...]` and `[b]` syntax degrades to literal text,
    // but it must not discard the already completed outer full reference.
    assert_eq!(
        html.matches("<a href=\"/safe\">").count(),
        completed_paragraphs
    );
    assert!(
        html.ends_with("<p><a href=\"/target\">x [a [b] ] </a></p>\n"),
        "{html}"
    );
}

#[test]
fn nested_reference_brackets_remain_bounded_and_literal_after_exhaustion() {
    let depth = limits::MAX_INLINE_MARKS / 2 - 1;
    let nested = format!("{}x{}", "[".repeat(depth), "]".repeat(depth));
    let markdown = format!("[x]: /safe\n\n{nested}\n\n{nested}");
    let html = to_html(&markdown);

    // The first deeply nested paragraph consumes the shared label-inspection
    // allowance. The second cannot re-run quadratic nesting work.
    assert_eq!(html.matches("<a href=\"/safe\">").count(), 0);
    assert!(html.contains(&nested));
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
        &ferromark::options!(Options::default();
            tables: true,),
    );

    assert_eq!(html.matches("<th>").count(), limits::MAX_TABLE_COLUMNS);
}
