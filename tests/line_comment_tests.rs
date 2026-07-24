use ferromark::{BlockEvent, BlockParser, Options, Range, RenderPolicy, to_html_with_options};

fn with_line_comments(input: &str) -> String {
    to_html_with_options(
        input,
        &Options {
            line_comments: true,
            ..Options::default()
        },
    )
}

#[test]
fn line_comments_are_disabled_by_default() {
    assert_eq!(
        to_html_with_options("// private note", &Options::default()),
        "<p>// private note</p>\n"
    );
}

#[test]
fn enabled_line_comments_emit_no_html() {
    assert_eq!(with_line_comments("// first\n   // second\n//\n"), "");
}

#[test]
fn comments_between_paragraph_lines_behave_like_removed_source_lines() {
    assert_eq!(
        with_line_comments("first\n// private\nsecond\n"),
        "<p>first\nsecond</p>\n"
    );
}

#[test]
fn explicit_blank_lines_still_separate_paragraphs() {
    assert_eq!(
        with_line_comments("first\n\n// private\n\nsecond\n"),
        "<p>first</p>\n<p>second</p>\n"
    );
}

#[test]
fn urls_trailing_slashes_and_escaped_markers_remain_text() {
    assert_eq!(
        with_line_comments("https://example.com\nText // ordinary\n\\// escaped\n"),
        "<p>https://example.com\nText // ordinary\n// escaped</p>\n"
    );
}

#[test]
fn four_space_indentation_remains_code() {
    assert_eq!(
        with_line_comments("   // hidden\n    // code\n"),
        "<pre><code>// code\n</code></pre>\n"
    );
}

#[test]
fn fenced_code_and_raw_html_blocks_remain_opaque() {
    assert_eq!(
        with_line_comments("```\n// code\n```\n"),
        "<pre><code>// code\n</code></pre>\n"
    );

    let html = to_html_with_options(
        "<div>\n// html content\n</div>\n",
        &Options {
            line_comments: true,
            render_policy: RenderPolicy::Trusted,
            ..Options::default()
        },
    );
    assert_eq!(html, "<div>\n// html content\n</div>\n");
}

#[test]
fn comments_do_not_break_existing_container_structure() {
    assert_eq!(
        with_line_comments("> first\n// private\n> second\n"),
        "<blockquote>\n<p>first\nsecond</p>\n</blockquote>\n"
    );
    assert_eq!(
        with_line_comments("- first\n// private\n- second\n"),
        "<ul>\n<li>first</li>\n<li>second</li>\n</ul>\n"
    );
}

#[test]
fn explicit_container_prefixes_are_not_comment_markers() {
    assert_eq!(
        with_line_comments("> // visible\n"),
        "<blockquote>\n<p>// visible</p>\n</blockquote>\n"
    );
}

#[test]
fn comments_do_not_prevent_setext_headings_or_tables() {
    assert_eq!(
        with_line_comments("Heading\n// private\n---\n"),
        "<h2 id=\"heading\">Heading</h2>\n"
    );

    let table = with_line_comments("A | B\n// private\n- | -\n1 | 2\n");
    assert!(table.starts_with("<table>\n<thead>\n<tr>\n<th>A</th>\n<th>B</th>"));
    assert!(!table.contains("private"));
}

#[test]
fn semantic_comment_events_keep_exact_source_order_and_range() {
    let input = "first\n// private\nsecond\n";
    let mut parser = BlockParser::new_with_options(
        input.as_bytes(),
        Options {
            line_comments: true,
            ..Options::default()
        },
    );
    let mut events = Vec::new();
    parser.parse(&mut events);

    assert_eq!(
        events,
        vec![
            BlockEvent::ParagraphStart,
            BlockEvent::Text(Range::new(0, 5)),
            BlockEvent::Comment(Range::new(6, 16)),
            BlockEvent::SoftBreak,
            BlockEvent::Text(Range::new(17, 23)),
            BlockEvent::ParagraphEnd,
        ]
    );
    assert_eq!(
        Range::new(6, 16).slice_str(input.as_bytes()).unwrap(),
        "// private"
    );
}

#[test]
fn crlf_comment_ranges_exclude_the_line_ending() {
    let input = "// private\r\nvisible\r\n";
    let mut parser = BlockParser::new_with_options(
        input.as_bytes(),
        Options {
            line_comments: true,
            ..Options::default()
        },
    );
    let mut events = Vec::new();
    parser.parse(&mut events);

    assert!(events.contains(&BlockEvent::Comment(Range::new(0, 10))));
    assert_eq!(with_line_comments(input), "<p>visible\r</p>\n");
}
