use ferromark::{Options, Renderer, to_html, to_html_with_options};

fn html_with_ids(input: &str) -> String {
    let options = ferromark::options!(Options::default();
        heading_ids: true,);
    to_html_with_options(input, &options)
}

fn html_without_ids(input: &str) -> String {
    let options = ferromark::options!(Options::default();
        heading_ids: false,);
    to_html_with_options(input, &options)
}

#[test]
fn test_basic_heading_id() {
    let html = html_with_ids("# Hello World");
    assert_eq!(html, "<h1 id=\"hello-world\">Hello World</h1>\n");
}

#[test]
fn test_heading_with_emphasis() {
    let html = html_with_ids("## Hello **World**");
    assert!(html.contains("id=\"hello-world\""), "Got: {html}");
}

#[test]
fn test_heading_with_code() {
    let html = html_with_ids("## The `code` function");
    assert!(html.contains("id=\"the-code-function\""), "Got: {html}");
}

#[test]
fn test_duplicate_headings() {
    let html = html_with_ids("# Hello\n\n# Hello\n\n# Hello");
    assert!(html.contains("id=\"hello\""), "First heading");
    assert!(html.contains("id=\"hello-1\""), "Second heading: {html}");
    assert!(html.contains("id=\"hello-2\""), "Third heading: {html}");
}

#[test]
fn natural_suffixes_cannot_collide_with_generated_ids() {
    assert_eq!(
        html_with_ids("# foo\n\n# foo\n\n# foo-1"),
        "<h1 id=\"foo\">foo</h1>\n<h1 id=\"foo-1\">foo</h1>\n<h1 id=\"foo-1-1\">foo-1</h1>\n"
    );
    assert_eq!(
        html_with_ids("# foo-1\n\n# foo\n\n# foo"),
        "<h1 id=\"foo-1\">foo-1</h1>\n<h1 id=\"foo\">foo</h1>\n<h1 id=\"foo-2\">foo</h1>\n"
    );
}

#[test]
fn suffix_cursor_skips_long_natural_suffix_chains() {
    let html = html_with_ids("# foo\n\n# foo\n\n# foo-1\n\n# foo\n\n# foo-2");
    assert!(html.contains("<h1 id=\"foo-1-1\">foo-1</h1>"), "{html}");
    assert!(html.contains("<h1 id=\"foo-2\">foo</h1>"), "{html}");
    assert!(html.contains("<h1 id=\"foo-2-1\">foo-2</h1>"), "{html}");
}

#[test]
fn natural_suffixes_and_empty_slug_fallbacks_are_unique_with_unicode() {
    let html = html_with_ids("# !!!\n\n# !!!\n\n# heading-1\n\n# Héllo\n\n# héllo\n\n# héllo-1");
    assert!(html.contains("<h1 id=\"heading\">!!!</h1>"), "{html}");
    assert!(html.contains("<h1 id=\"heading-1\">!!!</h1>"), "{html}");
    assert!(
        html.contains("<h1 id=\"heading-1-1\">heading-1</h1>"),
        "{html}"
    );
    assert!(html.contains("<h1 id=\"héllo\">Héllo</h1>"), "{html}");
    assert!(html.contains("<h1 id=\"héllo-1\">héllo</h1>"), "{html}");
    assert!(html.contains("<h1 id=\"héllo-1-1\">héllo-1</h1>"), "{html}");
}

#[test]
fn parse_headings_match_emitted_ids_and_renderer_reuse_resets_them() {
    let first = "# foo\n\n# foo\n\n# foo-1";
    let second = "# foo-1\n\n# foo\n\n# foo";
    let parsed = ferromark::parse(first);
    assert_eq!(
        parsed
            .headings
            .iter()
            .map(|heading| heading.id.as_deref())
            .collect::<Vec<_>>(),
        vec![Some("foo"), Some("foo-1"), Some("foo-1-1")]
    );
    assert!(parsed.html.contains("id=\"foo-1-1\""));

    let mut renderer = Renderer::new();
    assert_eq!(renderer.render(first), html_with_ids(first));
    assert_eq!(renderer.render(second), html_with_ids(second));
}

#[test]
fn headings_inside_footnotes_share_document_id_registry() {
    let options = ferromark::options!(Options::default();
        footnotes: true,);
    let html = to_html_with_options("# heading\n\nText[^note].\n\n[^note]: # heading", &options);

    assert!(html.contains("<h1 id=\"heading\">heading</h1>"), "{html}");
    assert!(html.contains("<h1 id=\"heading-1\">heading</h1>"), "{html}");
}

#[test]
fn test_heading_preserves_unicode() {
    let html = html_with_ids("# Héllo Wörld");
    assert!(html.contains("id=\"héllo-wörld\""), "Got: {html}");
}

#[test]
fn test_heading_underscore_hyphen() {
    let html = html_with_ids("# foo_bar-baz");
    assert!(html.contains("id=\"foo_bar-baz\""), "Got: {html}");
}

#[test]
fn test_heading_ids_disabled() {
    let html = html_without_ids("# Hello World");
    assert_eq!(html, "<h1>Hello World</h1>\n");
    assert!(!html.contains("id="));
}

#[test]
fn test_setext_heading_gets_id() {
    let html = html_with_ids("Hello World\n===========");
    assert!(html.contains("id=\"hello-world\""), "Got: {html}");
}

#[test]
fn test_heading_all_levels() {
    for level in 1..=6 {
        let input = format!("{} Test Heading", "#".repeat(level));
        let html = html_with_ids(&input);
        assert!(
            html.contains(&format!("<h{level} id=\"test-heading\">")),
            "Level {level}: {html}"
        );
    }
}

#[test]
fn test_heading_strips_special_chars() {
    let html = html_with_ids("# Hello, World! (2024)");
    // Commas, exclamation, parens stripped
    assert!(html.contains("id=\"hello-world-2024\""), "Got: {html}");
}

#[test]
fn test_heading_with_numbers() {
    let html = html_with_ids("# Section 42");
    assert!(html.contains("id=\"section-42\""), "Got: {html}");
}

#[test]
fn test_heading_default_options_have_ids() {
    // Default options should have heading_ids = true
    let html = to_html("# Hello");
    assert!(html.contains("id=\"hello\""), "Got: {html}");
}

#[test]
fn test_empty_heading() {
    let html = html_with_ids("# ");
    // Empty heading after stripping should get fallback
    assert!(html.contains("id=\"heading\""), "Got: {html}");
}

#[test]
fn test_heading_leading_trailing_spaces() {
    let html = html_with_ids("# Hello World");
    assert!(html.contains("id=\"hello-world\""), "Got: {html}");
}

#[test]
fn test_heading_mixed_case() {
    let html = html_with_ids("# Hello WORLD FoO");
    assert!(html.contains("id=\"hello-world-foo\""), "Got: {html}");
}
