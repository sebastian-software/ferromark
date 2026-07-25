//! Tests for the document-level parse API (headings, front matter) and the
//! link base path option.

use ferromark::{Options, to_html_with_options};

#[test]
fn parse_collects_headings_with_ids_and_plain_text() {
    let result = ferromark::parse("# Top\n\n## Sub *emph* `code`\n\n### [Linked](https://e.com)\n");

    assert_eq!(result.headings.len(), 3);
    assert_eq!(result.headings[0].level, 1);
    assert_eq!(result.headings[0].id.as_deref(), Some("top"));
    assert_eq!(result.headings[0].text, "Top");
    assert_eq!(result.headings[1].level, 2);
    assert_eq!(result.headings[1].id.as_deref(), Some("sub-emph-code"));
    assert_eq!(result.headings[1].text, "Sub emph code");
    assert_eq!(result.headings[2].level, 3);
    assert_eq!(result.headings[2].text, "Linked");
}

#[test]
fn parse_headings_decode_entities_in_text() {
    let result = ferromark::parse("## Ben &amp; Jerry\n");

    assert_eq!(result.headings[0].text, "Ben & Jerry");
    // The id reflects the existing slug behavior on raw source text.
    assert_eq!(result.headings[0].id.as_deref(), Some("ben-amp-jerry"));
}

#[test]
fn parse_heading_ids_absent_when_disabled() {
    let result = ferromark::parse_with_options("# Top\n", &Options::commonmark());

    assert_eq!(result.headings.len(), 1);
    assert_eq!(result.headings[0].id, None);
    assert_eq!(result.headings[0].text, "Top");
}

#[test]
fn parse_duplicate_headings_get_deduplicated_ids() {
    let result = ferromark::parse("# Same\n\n# Same\n");

    assert_eq!(result.headings[0].id.as_deref(), Some("same"));
    assert_eq!(result.headings[1].id.as_deref(), Some("same-1"));
}

#[test]
fn parse_returns_front_matter_and_headings_together() {
    let result = ferromark::parse("---\ntitle: X\n---\n# Body\n");

    assert_eq!(result.front_matter, Some("title: X\n"));
    assert_eq!(result.headings[0].text, "Body");
    assert!(result.html.contains("<h1 id=\"body\">Body</h1>"));
}

#[test]
fn parse_setext_headings_are_collected() {
    let result = ferromark::parse("Title\n=====\n\nSub\n---\n");

    assert_eq!(result.headings.len(), 2);
    assert_eq!(result.headings[0].level, 1);
    assert_eq!(result.headings[1].level, 2);
}

fn base_options(base: &str) -> Options {
    Options {
        link_base_path: Some(base.into()),
        ..Options::default()
    }
}

#[test]
fn link_base_path_prefixes_internal_links() {
    let html = to_html_with_options("[docs](/guide)", &base_options("/docs"));
    assert!(html.contains("<a href=\"/docs/guide\">"), "{html}");
}

#[test]
fn link_base_path_ignores_external_anchor_and_protocol_relative() {
    let options = base_options("/docs");
    let html = to_html_with_options(
        "[a](https://example.com/x) [b](#frag) [c](//cdn.example.com/x) [d](relative)",
        &options,
    );
    assert!(
        html.contains("<a href=\"https://example.com/x\">"),
        "{html}"
    );
    assert!(html.contains("<a href=\"#frag\">"), "{html}");
    assert!(html.contains("<a href=\"//cdn.example.com/x\">"), "{html}");
    assert!(html.contains("<a href=\"relative\">"), "{html}");
}

#[test]
fn link_base_path_skips_already_prefixed_links() {
    let html = to_html_with_options("[x](/docs/page)", &base_options("/docs"));
    assert!(html.contains("<a href=\"/docs/page\">"), "{html}");
}

#[test]
fn link_base_path_normalizes_trailing_slash_and_root() {
    let html = to_html_with_options("[x](/page)", &base_options("/docs/"));
    assert!(html.contains("<a href=\"/docs/page\">"), "{html}");

    let html = to_html_with_options("[x](/page)", &base_options("/"));
    assert!(html.contains("<a href=\"/page\">"), "{html}");
}

#[test]
fn link_base_path_applies_to_reference_links() {
    let html = to_html_with_options("[x][ref]\n\n[ref]: /guide\n", &base_options("/docs"));
    assert!(html.contains("<a href=\"/docs/guide\">"), "{html}");
}

#[test]
fn link_base_path_does_not_rewrite_images() {
    let html = to_html_with_options("![alt](/img.png)", &base_options("/docs"));
    assert!(html.contains("<img src=\"/img.png\""), "{html}");
}
