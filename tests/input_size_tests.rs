use ferromark::{
    BlockParser, InlineParser, MAX_INPUT_BYTES, Options, try_parse, try_parse_with_options,
    try_to_html, try_to_html_into, try_to_html_with_options, validate_input_size,
};

#[test]
fn checked_public_apis_preserve_normal_rendering() {
    let options = Options::default();
    assert!(try_to_html("# Heading").unwrap().contains("Heading</h1>"));
    assert!(
        try_to_html_with_options("*text*", &options)
            .unwrap()
            .contains("<em>text</em>")
    );

    let mut output = Vec::new();
    try_to_html_into("paragraph", &mut output).unwrap();
    assert_eq!(output, b"<p>paragraph</p>\n");

    assert_eq!(try_parse("# Heading").unwrap().headings.len(), 1);
    assert_eq!(
        try_parse_with_options("# Heading", &options).unwrap().html,
        "<h1 id=\"heading\">Heading</h1>\n"
    );

    let mut parser = BlockParser::try_new(b"paragraph").unwrap();
    let mut events = Vec::new();
    parser.parse(&mut events);
    assert!(!events.is_empty());

    let mut inline = InlineParser::new();
    let mut inline_events = Vec::new();
    inline
        .try_parse(b"paragraph", None, false, &mut inline_events)
        .unwrap();
    assert!(!inline_events.is_empty());
}

#[test]
fn public_size_check_exposes_a_non_panicking_preflight() {
    assert!(validate_input_size(MAX_INPUT_BYTES).is_ok());
}

#[cfg(all(feature = "mdx", target_pointer_width = "64"))]
#[test]
fn mdx_checked_apis_compile_and_preserve_normal_results() {
    use ferromark::mdx::{try_parse_events, try_render, try_segment, try_segment_spanned};

    assert_eq!(try_segment("{value}").unwrap().len(), 1);
    assert_eq!(try_segment_spanned("{value}").unwrap().len(), 1);
    assert!(try_render("{value}").unwrap().body.contains("{value}"));
    assert!(!try_parse_events("{value}").unwrap().events.is_empty());
}
