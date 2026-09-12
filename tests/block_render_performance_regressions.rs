use ferromark::{
    BlockEvent, BlockParser, FencedCodeBlock, FencedCodeRenderer, Options, Range, RenderPolicy,
    TrustedHtml, to_html_with_options,
};

fn blocks(input: &str, options: Options) -> Vec<BlockEvent> {
    let mut parser = BlockParser::new_with_options(input.as_bytes(), options);
    let mut events = Vec::new();
    parser.parse(&mut events);
    events
}

#[test]
fn root_fences_keep_physical_ranges_and_resume_after_closing_delimiters() {
    for fence in ["```", "~~~~"] {
        for newline in ["\n", "\r\n"] {
            for indent in ["", " ", "  ", "   "] {
                for tail in ["", "tail ü", "tail\r", "\t "] {
                    let opening = format!("{indent}{fence}rust{newline}");
                    let body = format!(
                        "{indent}a & ü{newline}{indent}// retained{newline}{indent}{newline}"
                    );
                    let input = format!("{opening}{body}{tail}");
                    let events = blocks(&input, Options::commonmark());
                    let ranges: Vec<_> = events
                        .iter()
                        .filter_map(|event| match event {
                            BlockEvent::Code(range) => Some(*range),
                            _ => None,
                        })
                        .collect();
                    let mut offset = opening.len();
                    let expected: Vec<_> = input[offset..]
                        .split_inclusive('\n')
                        .map(|line| {
                            let skipped = if !indent.is_empty() && line.starts_with('\t') {
                                1
                            } else {
                                line.bytes()
                                    .take(indent.len())
                                    .take_while(|b| *b == b' ')
                                    .count()
                            };
                            let range = Range::from_usize(offset + skipped, offset + line.len());
                            offset += line.len();
                            range
                        })
                        .collect();
                    assert_eq!(ranges, expected, "{input:?}");
                    assert_eq!(events.last(), Some(&BlockEvent::CodeBlockEnd));
                    let closed = format!("{opening}{body}{indent}{fence} \t{newline}# After\n");
                    let options = ferromark::options!(Options::commonmark(); line_comments: true, definition_lists: true);
                    assert_eq!(
                        to_html_with_options(&closed, &options),
                        format!(
                            "<pre><code class=\"language-rust\">a &amp; ü{newline}// retained{newline}{newline}</code></pre>\n<h1>After</h1>\n"
                        )
                    );
                }
            }
        }
    }
}

#[test]
fn list_fixup_keeps_raw_parser_events_and_handles_unbalanced_public_input() {
    let input = "- first\n\n- second\n  - nested\n\n    loose\n";
    let mut events = blocks(input, Options::commonmark());
    assert!(
        events
            .iter()
            .filter_map(|event| match event {
                BlockEvent::ListStart { tight, .. } => Some(*tight),
                _ => None,
            })
            .all(|tight| tight)
    );
    ferromark::fixup_list_tight(&mut events);
    assert!(
        events
            .iter()
            .filter_map(|event| match event {
                BlockEvent::ListStart { tight, .. } => Some(*tight),
                _ => None,
            })
            .all(|tight| !tight)
    );
    let fixed = events.clone();
    ferromark::fixup_list_tight(&mut events);
    assert_eq!(events, fixed);

    let kind = ferromark::ListKind::Unordered;
    let mut events = vec![
        BlockEvent::ListEnd { kind, tight: false },
        BlockEvent::ListStart { kind, tight: true },
    ];
    let original = events.clone();
    ferromark::fixup_list_tight(&mut events);
    assert_eq!(events, original);
}

#[test]
fn fenced_callbacks_receive_complete_unicode_code_and_metadata() {
    #[derive(Default)]
    struct Capture(Vec<(String, String, String)>);
    impl FencedCodeRenderer for Capture {
        fn render(&mut self, block: FencedCodeBlock<'_>) -> Option<TrustedHtml> {
            self.0.push((
                block.language.unwrap_or_default().to_owned(),
                block.meta.unwrap_or_default().to_owned(),
                block.code.to_owned(),
            ));
            None
        }
    }
    for input in [
        "```rust title=ü\nα\n\tβ &\n```\n",
        "> ```rust title=ü\n> α\n> \tβ &\n> ```\n",
    ] {
        let mut capture = Capture::default();
        let options =
            ferromark::options!(Options::commonmark(); render_policy: RenderPolicy::Trusted);
        let actual = ferromark::to_html_with_renderer(input, &options, &mut capture);
        assert_eq!(actual, to_html_with_options(input, &options));
        assert_eq!(
            capture.0,
            vec![(
                "rust".to_owned(),
                "title=ü".to_owned(),
                "α\n\tβ &\n".to_owned()
            )]
        );
    }
}

#[test]
fn writer_string_conversion_preserves_utf8_errors_at_scan_boundaries() {
    for size in [
        0, 1, 15, 16, 17, 127, 128, 129, 1023, 1024, 1025, 4095, 4096, 4097, 65536,
    ] {
        for position in [0, size / 2, size] {
            for inserted in [
                b"plain".as_slice(),
                "ü中🦀".as_bytes(),
                &[0xff],
                &[0x80],
                &[0xc0, 0x80],
                &[0xed, 0xa0, 0x80],
                &[0xf4, 0x90, 0x80, 0x80],
                &[0xe2, 0x82],
            ] {
                let mut bytes = vec![b'x'; position];
                bytes.extend_from_slice(inserted);
                bytes.resize(bytes.len() + size - position, b'x');
                let mut writer = ferromark::HtmlWriter::new();
                writer.write_bytes(&bytes);
                assert_eq!(writer.as_str(), std::str::from_utf8(&bytes));
                assert_eq!(writer.into_string(), String::from_utf8(bytes));
            }
        }
    }
}

#[test]
fn renderer_reuse_handles_transitions_between_tight_loose_and_absent_lists() {
    let options = ferromark::options!(Options::commonmark();
        footnotes: true, definition_lists: true,
    );
    let documents = [
        (
            "- one\n- two\n",
            "<ul>\n<li>one</li>\n<li>two</li>\n</ul>\n",
        ),
        (
            "- one\n\n- two\n",
            "<ul>\n<li>\n<p>one</p>\n</li>\n<li>\n<p>two</p>\n</li>\n</ul>\n",
        ),
        ("plain\n", "<p>plain</p>\n"),
        ("", ""),
    ];
    let mut renderer = ferromark::Renderer::with_options(options);
    for &(input, expected) in documents.iter().chain(documents.iter().rev()) {
        assert_eq!(renderer.render(input), expected);
    }
}

#[test]
fn list_fixup_pairs_deeply_nested_starts_and_ends() {
    let kind = ferromark::ListKind::Unordered;
    for depth in [0, 1, 8, 9, 64] {
        let mut events = vec![BlockEvent::ListStart { kind, tight: true }; depth];
        for i in (0..depth).rev() {
            events.push(BlockEvent::ListEnd {
                kind,
                tight: i % 2 == 0,
            });
        }
        ferromark::fixup_list_tight(&mut events);
        for (i, event) in events[..depth].iter().enumerate() {
            assert_eq!(
                event,
                &BlockEvent::ListStart {
                    kind,
                    tight: i % 2 == 0
                }
            );
        }
    }
}
