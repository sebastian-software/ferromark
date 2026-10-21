#[path = "support/edge.rs"]
mod edge_support;

use edge_support::render;
use ferromark::allocator::Allocator;
use ferromark::parser::{Parser, ParserOptions};
use ferromark::renderer::{AbbreviationOptions, HtmlRenderer, HtmlRendererOptions};

#[test]
fn html_blocks_are_escaped_when_sanitize_is_enabled() {
    let allocator = Allocator::new();
    let mut children = allocator.new_vec();
    children.push(ferromark::ast::Node::Html(ferromark::ast::Html {
        value: "<script>alert(1)</script>",
        span: ferromark::ast::Span::new(0, 25),
    }));
    let doc = ferromark::ast::Document {
        front_matter: None,
        children,
        span: ferromark::ast::Span::new(0, 25),
    };

    let mut renderer = HtmlRenderer::with_options(HtmlRendererOptions {
        sanitize: true,
        ..Default::default()
    });
    let html = renderer.render(&doc);

    assert_eq!(html, "&lt;script&gt;alert(1)&lt;/script&gt;\n");
}

#[test]
fn unsafe_link_urls_are_neutralized_when_sanitize_is_enabled() {
    let html = render(
        "[run](javascript:alert(1))",
        ParserOptions::default(),
        HtmlRendererOptions {
            sanitize: true,
            ..Default::default()
        },
    );

    assert_eq!(html, "<p><a href=\"#\">run</a></p>\n");
}

#[test]
fn obfuscated_unsafe_link_schemes_are_neutralized() {
    let html = render(
        "[run](JaVaScRiPt:alert(1))",
        ParserOptions::default(),
        HtmlRendererOptions {
            sanitize: true,
            ..Default::default()
        },
    );

    assert_eq!(html, "<p><a href=\"#\">run</a></p>\n");
}

#[test]
fn custom_autolinks_obey_url_sanitization() {
    let source = "ssh://danger.example [manual](ssh://danger.example) http://example.com https://example.org mailto:help@example.com";
    let allocator = Allocator::new();
    let document = Parser::with_options(&allocator, source, ParserOptions::default())
        .parse()
        .unwrap();
    let renderer_options = HtmlRendererOptions {
        sanitize: true,
        autolink_patterns: vec![
            "ssh://".into(),
            "http://".into(),
            "https://".into(),
            "mailto:".into(),
        ]
        .into(),
        ..Default::default()
    };
    let mut renderer = HtmlRenderer::with_options(renderer_options.clone());
    let html = renderer.render(&document);
    let mut abbreviation_renderer = HtmlRenderer::with_options_and_abbreviations(
        renderer_options,
        AbbreviationOptions::default(),
    );
    let html_with_abbreviations = abbreviation_renderer.render(&document);
    let unsanitized_options = HtmlRendererOptions {
        sanitize: false,
        autolink_patterns: vec!["ssh://".into()].into(),
        autolink_target_blank: false,
        ..Default::default()
    };
    let mut unsanitized_renderer = HtmlRenderer::with_options(unsanitized_options);
    let html_without_sanitization = unsanitized_renderer.render(&document);

    let expected = concat!(
        "<p><a href=\"#\" target=\"_blank\" rel=\"noopener noreferrer\">ssh://danger.example</a> ",
        "<a href=\"#\">manual</a> ",
        "<a href=\"http://example.com\" target=\"_blank\" rel=\"noopener noreferrer\">http://example.com</a> ",
        "<a href=\"https://example.org\" target=\"_blank\" rel=\"noopener noreferrer\">https://example.org</a> ",
        "<a href=\"mailto:help@example.com\" target=\"_blank\" rel=\"noopener noreferrer\">mailto:help@example.com</a></p>\n"
    );
    assert_eq!(html, expected);
    assert_eq!(html_with_abbreviations, expected);
    assert!(
        html_without_sanitization
            .contains("<a href=\"ssh://danger.example\">ssh://danger.example</a>"),
        "sanitize: false should preserve custom autolink URLs: {html_without_sanitization}"
    );
    assert!(
        html_without_sanitization.contains("<a href=\"ssh://danger.example\">manual</a>"),
        "sanitize: false should preserve ordinary links: {html_without_sanitization}"
    );
}

#[test]
fn whitespace_destinations_do_not_become_links_at_all() {
    // Per CommonMark, whitespace in a bare destination means the bracketed
    // run is not a link. The former scheme obfuscation via embedded spaces
    // now yields plain text — there is no href left to sanitize.
    let html = render(
        "[run](  JaVa ScRiPt:alert(1))",
        ParserOptions::default(),
        HtmlRendererOptions {
            sanitize: true,
            ..Default::default()
        },
    );

    assert!(!html.contains("<a"), "no anchor should be emitted: {html}");
    assert_eq!(html, "<p>[run](  JaVa ScRiPt:alert(1))</p>\n");
}

#[test]
fn unsafe_image_urls_are_cleared_when_sanitize_is_enabled() {
    let html = render(
        "![x](data:text/html,<script>alert(1)</script>)",
        ParserOptions::default(),
        HtmlRendererOptions {
            sanitize: true,
            ..Default::default()
        },
    );

    assert_eq!(html, "<p><img src=\"\" alt=\"x\"></p>\n");
}

#[test]
fn sanitize_keeps_relative_and_allowed_url_schemes() {
    let html = render(
        "[guide](./guide.md) [mail](mailto:hi@example.com) [phone](tel:+123)",
        ParserOptions::default(),
        HtmlRendererOptions {
            sanitize: true,
            ..Default::default()
        },
    );

    insta::assert_snapshot!(html);
}

#[test]
fn inline_raw_html_is_escaped_when_sanitize_is_enabled() {
    let html = render(
        "<span>ok</span>",
        ParserOptions::default(),
        HtmlRendererOptions {
            sanitize: true,
            ..Default::default()
        },
    );

    assert_eq!(html, "<p>&lt;span&gt;ok&lt;/span&gt;</p>\n");
}

#[test]
fn disallow_raw_html_filters_only_the_gfm_tag_list() {
    let html = render(
        "<strong> <title> <style> <em>\n",
        ParserOptions::gfm(),
        HtmlRendererOptions {
            disallow_raw_html: true,
            ..Default::default()
        },
    );

    // Only the leading `<` of a disallowed tag is escaped; `<strong>` and
    // `<em>` pass through untouched.
    assert_eq!(html, "<p><strong> &lt;title> &lt;style> <em></p>\n");
}

#[test]
fn disallow_raw_html_is_off_by_default() {
    let html = render(
        "<strong> <title>\n",
        ParserOptions::gfm(),
        HtmlRendererOptions::default(),
    );

    assert_eq!(html, "<p><strong> <title></p>\n");
}

#[test]
fn disallow_raw_html_filters_closing_tags_and_html_blocks() {
    let html = render(
        "<blockquote>\n  <xmp> is disallowed.  <XMP> is also disallowed.\n</blockquote>\n",
        ParserOptions::gfm(),
        HtmlRendererOptions {
            disallow_raw_html: true,
            ..Default::default()
        },
    );

    assert!(
        html.contains("&lt;xmp> is disallowed.  &lt;XMP> is also disallowed."),
        "{html}"
    );
    assert!(html.contains("<blockquote>"), "{html}");
}

#[test]
fn disallow_raw_html_leaves_similar_tag_names_alone() {
    let html = render(
        "<titlebar> and <scripted>\n",
        ParserOptions::gfm(),
        HtmlRendererOptions {
            disallow_raw_html: true,
            ..Default::default()
        },
    );

    assert_eq!(html, "<p><titlebar> and <scripted></p>\n");
}
