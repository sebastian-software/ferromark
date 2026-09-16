use ferromark::{
    HtmlRendererOptions, ParseErrorKind, ParserOptions, to_html, to_html_into,
    to_html_into_with_options, to_html_with_options,
};

#[test]
fn owned_output_uses_rust_defaults() {
    assert_eq!(
        to_html("**Hello**").unwrap(),
        "<p><strong>Hello</strong></p>\n"
    );
    assert_eq!(to_html("<b>Hello</b>").unwrap(), "<p><b>Hello</b></p>\n");
    assert_eq!(to_html("").unwrap(), "");
}

/// The default entry points build the renderer's static default options
/// instead of the owned `HtmlRendererOptions` struct. Every default the
/// renderer reads — hard breaks, base URL, source path, code annotation key,
/// and the autolink pattern list — must still produce identical bytes.
#[test]
fn default_entry_points_match_explicit_default_options() {
    for source in [
        "",
        "plain text",
        "# Title\n\n## Title\n\n## Title\n",
        "Visit https://example.com and http://example.org/a?b=c#d now.\n",
        "[link](/relative.md) and [abs](https://example.com)\n",
        "Line one  \nline two\n",
        "> [!NOTE]\n> Callout body with https://example.com\n",
        "[[toc]]\n\n# One\n\n## Two\n\n### Three\n\n#### Four\n",
        "Ref[^a] and again[^a]\n\n[^a]: Note body\n",
        "```rust annotate=\"add:1\"\nfn main() {}\n```\n",
        "| a | b |\n| --- | --- |\n| 1 | 2 |\n",
        "<div class=\"raw\"><script>ok()</script></div>\n",
        "日本語の見出し\n===\n",
    ] {
        let explicit = to_html_with_options(
            source,
            ParserOptions::default(),
            HtmlRendererOptions::default(),
        )
        .unwrap();
        assert_eq!(to_html(source).unwrap(), explicit, "{source:?}");

        let mut appended = String::from("prefix\n");
        to_html_into(source, &mut appended).unwrap();
        let mut appended_explicit = String::from("prefix\n");
        to_html_into_with_options(
            source,
            &mut appended_explicit,
            ParserOptions::default(),
            HtmlRendererOptions::default(),
        )
        .unwrap();
        assert_eq!(appended, appended_explicit, "{source:?}");
        assert_eq!(appended, format!("prefix\n{explicit}"), "{source:?}");
    }
}

#[test]
fn explicit_options_control_syntax_and_html_policy() {
    let html = to_html_with_options(
        "==Important== <b>text</b> [bad](javascript:alert)",
        ParserOptions {
            highlight: true,
            ..ParserOptions::default()
        },
        HtmlRendererOptions {
            sanitize: true,
            ..HtmlRendererOptions::default()
        },
    )
    .unwrap();
    assert!(html.contains("<mark>Important</mark>"));
    assert!(html.contains("&lt;b&gt;text&lt;/b&gt;"));
    assert!(!html.contains("javascript:"));
}

#[test]
fn append_preserves_prefix_and_document_isolation() {
    let mut output = String::from("prefix\n");
    to_html_into("[target]\n\n- [target]: /url", &mut output).unwrap();
    let first_end = output.len();
    to_html_into("[target]", &mut output).unwrap();
    assert!(output.starts_with("prefix\n<p><a href=\"/url\">target</a></p>\n"));
    assert_eq!(&output[first_end..], "<p>[target]</p>\n");
    let before_empty = output.clone();
    to_html_into("", &mut output).unwrap();
    assert_eq!(output, before_empty);
}

#[test]
fn parse_failure_leaves_existing_output_unchanged() {
    let mut output = String::from("existing");
    let error = to_html_into_with_options(
        "> > > too deep",
        &mut output,
        ParserOptions {
            max_nesting_depth: 1,
            ..ParserOptions::default()
        },
        HtmlRendererOptions::default(),
    )
    .unwrap_err();
    assert!(matches!(
        error.kind(),
        ParseErrorKind::NestingTooDeep { .. }
    ));
    assert_eq!(output, "existing");
    assert!(
        to_html_with_options(
            "> > > too deep",
            ParserOptions {
                max_nesting_depth: 1,
                ..ParserOptions::default()
            },
            HtmlRendererOptions::default(),
        )
        .is_err()
    );
}
