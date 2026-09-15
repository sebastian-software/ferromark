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
