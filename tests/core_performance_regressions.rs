use ferromark::{HtmlWriter, Options, Renderer, to_html_with_options};

#[test]
fn reused_renderer_passes_the_complete_trusted_commonmark_corpus() {
    let options = ferromark::options!(Options::commonmark();
        render_policy: ferromark::RenderPolicy::Trusted,
    );
    let mut renderer = Renderer::with_options(options);
    let examples: Vec<serde_json::Value> = serde_json::from_str(include_str!("spec.json")).unwrap();
    // Forward and reverse order exercise reuse after different predecessor
    // documents, including unterminated HTML, fences and reference definitions.
    for example in examples.iter().chain(examples.iter().rev()) {
        assert_eq!(
            renderer.render(example["markdown"].as_str().unwrap()),
            example["html"].as_str().unwrap(),
            "spec example {}",
            example["example"],
        );
    }
}

#[test]
fn text_writer_preserves_entity_and_escape_semantics_across_scan_boundaries() {
    // Compare to the original whole-segment entity decoding path. Include the
    // multi-codepoint fixup's interaction with text before the first ampersand.
    for prefix_len in [0, 1, 15, 16, 17, 63, 64, 127, 128, 129, 1024] {
        let prefix = "x".repeat(prefix_len);
        for (source, expected) in [
            ("plain ü text", "plain ü text"),
            ("< > \" '", "&lt; &gt; &quot; '"),
            ("&amp;lt;", "&amp;lt;"),
            ("&#0; &ngE;", "� ≧̸"),
            ("&unknown; &amp", "&amp;unknown; &amp;amp"),
            ("≧ &ngE;", "≧̸ ≧̸"),
            ("< &quot; &lt; &#38;", "&lt; &quot; &lt; &amp;"),
        ] {
            let mut writer = HtmlWriter::new();
            writer.write_text_with_entities(format!("{prefix}{source}").as_bytes());
            assert_eq!(writer.as_bytes(), format!("{prefix}{expected}").as_bytes());
        }
    }
    let mut writer = HtmlWriter::new();
    writer.write_text_with_entities(b"prefix\xff&amp;");
    assert!(
        writer.is_empty(),
        "preserve the existing invalid UTF-8 fallback"
    );
}

#[test]
fn reused_parser_does_not_leak_scratch_between_different_document_shapes() {
    let options = ferromark::options!(Options::default();
        footnotes: true, inline_footnotes: true, line_comments: true,
        definition_lists: true, highlight: true, superscript: true, subscript: true,
    );
    let mut renderer = Renderer::with_options(options.clone());
    let long_paragraph = "paragraph continuation\n".repeat(512);
    let documents = [
        long_paragraph.as_str(),
        "[a]: /first\n  \"multiline title\"\n\n[a]",
        "[a] and *emphasis* and `code`",
        "    indented code\n\n\n    continuation\n\n",
        "plain paragraph\n<!-- comment -->\ncontinued\n",
        "> - nested\n>   continuation\n>\n>   loose paragraph\n",
        "[^note]: footnote\n\nnote[^note]",
        "^[inline note] and [missing][a]",
        "```rust\nunclosed fence",
        "<script>\nunclosed HTML",
        "| a | b |\n| - | - |\n| x | y |",
        "term\n: definition\n",
        "==highlight== ^super^ ~sub~ **strong**",
        "",
        "fresh",
    ];
    for _ in 0..3 {
        for input in documents {
            assert_eq!(
                renderer.render(input),
                to_html_with_options(input, &options)
            );
        }
    }
}

#[test]
fn multiline_html_keeps_policy_filter_and_container_boundaries() {
    use ferromark::RenderPolicy;
    for newline in ["\n", "\r\n"] {
        let input = "<div>\nfirst & raw\n<script>unsafe</script>\n</div>\n".replace('\n', newline);
        let options = ferromark::options!(Options::commonmark();
            render_policy: RenderPolicy::Trusted,
        );
        assert_eq!(
            to_html_with_options(&input, &options),
            "<div>\nfirst & raw\n<script>unsafe</script>\n</div>\n".replace('\n', newline)
        );
        let filtered = ferromark::options!(options.clone(); disallowed_raw_html: true);
        assert_eq!(
            to_html_with_options(&input, &filtered),
            "<div>\nfirst & raw\n&lt;script>unsafe&lt;/script>\n</div>\n".replace('\n', newline)
        );
        let escaped = ferromark::options!(options; render_policy: RenderPolicy::Untrusted);
        assert_eq!(
            to_html_with_options(&input, &escaped),
            "&lt;div&gt;\nfirst &amp; raw\n&lt;script&gt;unsafe&lt;/script&gt;\n&lt;/div&gt;\n"
                .replace('\n', newline)
        );
    }
    let options = ferromark::options!(Options::commonmark();
        render_policy: RenderPolicy::Trusted,
    );
    assert_eq!(
        to_html_with_options("> <div>\n> first\n> second\n> </div>\n\nafter", &options),
        "<blockquote>\n<div>\nfirst\nsecond\n</div>\n</blockquote>\n<p>after</p>\n"
    );
}
