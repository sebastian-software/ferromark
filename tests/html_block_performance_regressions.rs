use ferromark::{BlockEvent, BlockParser, Options, Range, RenderPolicy, to_html_with_options};

fn blocks(input: &str, options: Options) -> Vec<BlockEvent> {
    let mut parser = BlockParser::new_with_options(input.as_bytes(), options);
    let mut events = Vec::new();
    parser.parse(&mut events);
    events
}

#[test]
fn root_html_retains_each_physical_line_and_resumes_markdown() {
    for (opening, closing, blank_terminated) in [
        ("<script>", "</ScRiPt>", false),
        ("<pre>", "</pre>", false),
        ("<style>", "</style>", false),
        ("<textarea>", "</textarea>", false),
        ("<!--", "-->", false),
        ("<?pi", "?>", false),
        ("<![CDATA[", "]]>", false),
        ("<!DOCTYPE", ">", false),
        ("<div>", "</div>", true),
        ("<custom>", "</custom>", true),
    ] {
        for newline in ["\n", "\r\n"] {
            let html = format!("  {opening}{newline}\t body & ü{newline}{closing}{newline}");
            let input = format!(
                "{html}{}# After\n",
                if blank_terminated { newline } else { "" }
            );
            let mut expected = vec![BlockEvent::HtmlBlockStart];
            let mut offset = 0;
            for line in html.split_inclusive('\n') {
                expected.push(BlockEvent::HtmlBlockText(Range::from_usize(
                    offset,
                    offset + line.len(),
                )));
                offset += line.len();
            }
            expected.push(BlockEvent::HtmlBlockEnd);
            let options =
                ferromark::options!(Options::commonmark(); render_policy: RenderPolicy::Trusted);
            let events = blocks(&input, options.clone());
            assert_eq!(&events[..expected.len()], expected, "{input:?}");
            assert!(matches!(
                events[expected.len()],
                BlockEvent::HeadingStart { .. }
            ));
            assert_eq!(
                to_html_with_options(&input, &options),
                format!("{html}<h1>After</h1>\n")
            );
        }
    }
}

#[test]
fn html_continuations_keep_comments_and_container_markers_as_source() {
    let options = ferromark::options!(Options::commonmark();
        render_policy: RenderPolicy::Trusted, line_comments: true, definition_lists: true,
    );
    let input = "<div>\n// retained\n> retained\n- retained\n\n// removed\n# After\n";
    assert_eq!(
        to_html_with_options(input, &options),
        "<div>\n// retained\n> retained\n- retained\n<h1>After</h1>\n"
    );
    let input = "> <div>\n> first\n> second\noutside\n\n- <pre>\n  inside\noutside again\n";
    let ranges: Vec<_> = blocks(input, options)
        .into_iter()
        .filter_map(|event| match event {
            BlockEvent::HtmlBlockText(range) => Some(range.slice(input.as_bytes()).to_vec()),
            _ => None,
        })
        .collect();
    assert_eq!(
        ranges,
        ["<div>\n", "first\n", "second\n", "<pre>\n", "inside\n"].map(|s| s.as_bytes().to_vec())
    );
}

#[test]
fn unterminated_html_retains_trailing_whitespace_and_eof_ranges() {
    for opening in [
        "<script>",
        "<!--",
        "<?pi",
        "<![CDATA[",
        "<!DOCTYPE",
        "<div>",
        "<custom>",
    ] {
        for ending in ["text", "text\r", "\t ", ""] {
            let input = format!("{opening}\n{ending}");
            let options =
                ferromark::options!(Options::commonmark(); render_policy: RenderPolicy::Trusted);
            let events = blocks(&input, options.clone());
            assert_eq!(events.last(), Some(&BlockEvent::HtmlBlockEnd));
            let blank_terminated = matches!(opening, "<div>" | "<custom>");
            let expected = if blank_terminated && ending == "\t " {
                format!("{opening}\n")
            } else {
                input.clone()
            };
            assert_eq!(
                to_html_with_options(&input, &options),
                expected,
                "{input:?}"
            );
        }
    }
}

#[test]
fn html_end_markers_remain_confined_to_a_physical_line() {
    for (opening, split_marker, closing) in [
        ("<script>", "</scr\nipt>", "</ScRiPt>"),
        ("<!--", "--\n>", "-->"),
        ("<?pi", "?\n>", "?>"),
        ("<![CDATA[", "]]\n>", "]]>"),
    ] {
        let html = format!("{opening}\n{split_marker}\n\n# Still HTML\n{closing}\n");
        let input = format!("{html}# After\n");
        let options =
            ferromark::options!(Options::commonmark(); render_policy: RenderPolicy::Trusted);
        assert_eq!(
            to_html_with_options(&input, &options),
            format!("{html}<h1>After</h1>\n")
        );
        assert_eq!(
            blocks(&input, options)
                .iter()
                .filter(|e| matches!(e, BlockEvent::HtmlBlockEnd))
                .count(),
            1
        );
    }
}

#[test]
fn reused_html_parser_preserves_policies_and_definition_state() {
    let documents = [
        "term\n: definition\n\n<div>\nraw\n\nnext\n: description\n",
        "- item\n\n<div>\nraw\n\n- next\n",
        "<script>\n// retained\n</script>\n// removed\n# After\n",
        "<!--\nunterminated",
        "<div>\n\t\rafter\n",
        "[^a]: <div>\n    raw\n\nuse[^a]\n",
        "[a]: /path\n\n<div>\n[a]\n\n[a]\n",
        "",
    ];
    for policy in [RenderPolicy::Trusted, RenderPolicy::Untrusted] {
        for filter in [false, true] {
            let options = ferromark::options!(Options::commonmark();
                render_policy: policy, disallowed_raw_html: filter,
                line_comments: true, definition_lists: true, footnotes: true,
            );
            let mut renderer = ferromark::Renderer::with_options(options.clone());
            for input in documents.iter().chain(documents.iter().rev()) {
                assert_eq!(
                    renderer.render(input),
                    to_html_with_options(input, &options)
                );
            }
        }
    }
}

#[test]
fn all_commonmark_block_tags_interrupt_paragraphs_case_insensitively() {
    // Type 7 tags cannot interrupt a paragraph, so this also catches a known
    // block tag accidentally falling through to the generic HTML recognizer.
    let tags = "address article aside base basefont blockquote body caption center col colgroup dd details dialog dir div dl dt fieldset figcaption figure footer form frame frameset head header hr html iframe legend li link main menu menuitem nav noframes ol optgroup option p param section source summary table tbody td tfoot th thead title tr track ul h1 h2 h3 h4 h5 h6";
    for tag in tags.split_whitespace() {
        for name in [
            tag.to_owned(),
            tag.to_ascii_uppercase(),
            format!("{}{}", tag[..1].to_ascii_uppercase(), &tag[1..]),
        ] {
            for slash in ["", "/"] {
                let input = format!("before\n<{slash}{name}>\nafter\n");
                assert!(
                    blocks(&input, Options::commonmark()).contains(&BlockEvent::HtmlBlockStart),
                    "{input}"
                );
            }
        }
    }
    for tag in [
        "divx",
        "xdiv",
        "tdx",
        "h0",
        "h7",
        "h10",
        "custom",
        "",
        "article-x",
        "menuitems",
    ] {
        let input = format!("before\n<{tag}>\nafter\n");
        assert!(
            !blocks(&input, Options::commonmark()).contains(&BlockEvent::HtmlBlockStart),
            "{input}"
        );
    }
}

#[test]
fn deep_html_indentation_preserves_raw_lines_and_following_code_columns() {
    let indent = " \t".repeat(256);
    for (opening, ending) in [("<div>", ""), ("<custom>", ""), ("<pre>", "</pre>")] {
        let first = format!("{opening}\n");
        let content = format!("{indent}<p>raw</p>\r\n");
        let last = format!("{indent}{ending}\r\n");
        let html = if ending.is_empty() {
            format!("{first}{content}")
        } else {
            format!("{first}{content}{last}")
        };
        let input = format!("{first}{content}{last}    code\n\n# After\n");
        let options =
            ferromark::options!(Options::commonmark(); render_policy: RenderPolicy::Trusted);
        assert_eq!(
            to_html_with_options(&input, &options),
            format!("{html}<pre><code>code\n</code></pre>\n<h1>After</h1>\n")
        );
        let ranges: Vec<_> = blocks(&input, options)
            .into_iter()
            .filter_map(|event| match event {
                BlockEvent::HtmlBlockText(range) => Some(range),
                _ => None,
            })
            .collect();
        let mut offset = 0;
        let expected: Vec<_> = html
            .split_inclusive('\n')
            .map(|line| {
                let range = Range::from_usize(offset, offset + line.len());
                offset += line.len();
                range
            })
            .collect();
        assert_eq!(ranges, expected);
    }
}

#[test]
fn root_html_line_scans_preserve_boundaries_and_unterminated_tails() {
    let options = ferromark::options!(Options::commonmark(); render_policy: RenderPolicy::Trusted);
    for width in (0..=80).chain([127, 128, 129, 255, 256, 257, 1024, 65536]) {
        for newline in ["\n", "\r\n"] {
            let first = "<pre>\n";
            let body = format!("{}é\0\r{newline}", "x".repeat(width));
            for tail in ["", "x", "é", "\r", "\t "] {
                let input = format!("{first}{body}{body}{tail}");
                let mut expected = vec![BlockEvent::HtmlBlockStart];
                let mut offset = 0;
                for line in input.split_inclusive('\n') {
                    expected.push(BlockEvent::HtmlBlockText(Range::from_usize(
                        offset,
                        offset + line.len(),
                    )));
                    offset += line.len();
                }
                expected.push(BlockEvent::HtmlBlockEnd);
                assert_eq!(blocks(&input, options.clone()), expected);
                assert_eq!(to_html_with_options(&input, &options), input);
            }
        }
    }
}
