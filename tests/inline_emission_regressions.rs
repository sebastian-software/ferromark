use ferromark::{InlineEvent, InlineParser, Options, Range, RenderPolicy, to_html_with_options};

fn trusted_options() -> Options {
    ferromark::options!(Options::commonmark(); render_policy: RenderPolicy::Trusted)
}

#[test]
fn code_content_keeps_its_range_and_closing_boundary() {
    let mut parser = InlineParser::new();
    let mut events = Vec::new();
    for (input, content, html) in [
        ("` a `", "a", "<p><code>a</code></p>\n"),
        ("``b`c``", "b`c", "<p><code>b`c</code></p>\n"),
        ("`d\n e`", "d\n e", "<p><code>d e</code></p>\n"),
    ] {
        events.clear();
        parser.parse(input.as_bytes(), None, true, &mut events);
        let code: Vec<_> = events
            .iter()
            .filter_map(|event| match event {
                InlineEvent::Code(range) => Some(range.slice_str(input.as_bytes()).unwrap()),
                _ => None,
            })
            .collect();
        assert_eq!(code, [content]);
        assert_eq!(to_html_with_options(input, &trusted_options()), html);
    }

    events.clear();
    parser.parse(b"*`x`*`y`", None, true, &mut events);
    assert_eq!(
        events,
        [
            InlineEvent::EmphasisStart,
            InlineEvent::Code(Range::new(2, 3)),
            InlineEvent::EmphasisEnd,
            InlineEvent::Code(Range::new(6, 7)),
        ]
    );
}

#[test]
fn nested_links_and_images_keep_their_own_destinations_and_titles() {
    let input = "[outer ![image](/second \"two\")](/first \"one\") and [last](/third)";
    let expected = "<p><a href=\"/first\" title=\"one\">outer <img src=\"/second\" alt=\"image\" title=\"two\" /></a> and <a href=\"/third\">last</a></p>\n";
    let mut renderer = ferromark::Renderer::with_options(trusted_options());
    for _ in 0..3 {
        assert_eq!(renderer.render(input), expected);
        assert_eq!(
            renderer.render("[fresh](/other \"fresh title\")"),
            "<p><a href=\"/other\" title=\"fresh title\">fresh</a></p>\n"
        );
    }
}

#[test]
fn inline_html_search_preserves_escapes_code_and_autolinks() {
    for prefix_len in [0, 15, 16, 17, 63, 64, 65, 1024] {
        let prefix = "é".repeat(prefix_len);
        let input = format!(
            "start{prefix} text \\<b> `<i>` <span title=\">\">ok</span> <https://example.com>"
        );
        let expected = format!(
            "<p>start{prefix} text &lt;b&gt; <code>&lt;i&gt;</code> <span title=\">\">ok</span> <a href=\"https://example.com\">https://example.com</a></p>\n"
        );
        assert_eq!(to_html_with_options(&input, &trusted_options()), expected);
    }
}

#[test]
fn math_and_inline_notes_preserve_trimmed_source_ranges() {
    let input = "$ a $ and $$ b $$ and ^[note *body*]";
    let mut parser = InlineParser::new();
    let mut events = Vec::new();
    parser.parse_with_options(
        input.as_bytes(),
        None,
        true,
        true,
        true,
        true,
        true,
        true,
        true,
        true,
        None,
        &mut events,
    );
    let content: Vec<_> = events
        .iter()
        .filter_map(|event| match event {
            InlineEvent::MathInline(range) => Some(("inline", range)),
            InlineEvent::MathDisplay(range) => Some(("display", range)),
            InlineEvent::InlineFootnote(range) => Some(("note", range)),
            _ => None,
        })
        .map(|(kind, range)| (kind, range.slice_str(input.as_bytes()).unwrap()))
        .collect();
    assert_eq!(
        content,
        [("inline", "a"), ("display", "b"), ("note", "note *body*")]
    );
}

#[test]
fn code_point_compaction_preserves_existing_overlap_events() {
    // These snapshots preserve existing resolution behavior, not normative
    // Markdown expectations. Correcting overlapping math/link/code constructs
    // is separate from changing their internal storage and emission cost.
    for (input, expected) in [
        (
            "$`a`$",
            vec![
                InlineEvent::MathInline(Range::new(1, 4)),
                InlineEvent::Code(Range::new(2, 3)),
                InlineEvent::Text(Range::new(4, 5)),
            ],
        ),
        (
            "[](`a`)",
            vec![
                InlineEvent::LinkStart {
                    url: Range::new(3, 6),
                    title: None,
                },
                InlineEvent::LinkEnd,
                InlineEvent::Code(Range::new(4, 5)),
                InlineEvent::Text(Range::new(6, 7)),
            ],
        ),
    ] {
        let mut parser = InlineParser::new();
        let mut events = Vec::new();
        parser.parse_with_options(
            input.as_bytes(),
            None,
            true,
            true,
            false,
            false,
            false,
            false,
            true,
            false,
            None,
            &mut events,
        );
        assert_eq!(events, expected, "{input}");
    }
}
