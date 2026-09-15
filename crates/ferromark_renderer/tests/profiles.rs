use ferromark_allocator::Allocator;
use ferromark_parser::{Parser, ParserOptions};
use ferromark_renderer::{HtmlRenderer, HtmlRendererOptions, NoHtmlRenderHooks};

const PROFILE_SOURCE: &str =
    "# Hello\n\n> [!NOTE]\n> body\n\n[[toc]]\n\n<script>alert(1)</script>\n\n```js{1}\ncode\n```\n";

fn parse<'a>(allocator: &'a Allocator, options: ParserOptions) -> ferromark_ast::Document<'a> {
    parse_source(allocator, PROFILE_SOURCE, options)
}

fn parse_source<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    options: ParserOptions,
) -> ferromark_ast::Document<'a> {
    Parser::with_options(allocator, source, options)
        .parse()
        .unwrap()
}

#[test]
fn spec_profiles_disable_product_rendering_extras() {
    let allocator = Allocator::new();
    let document = parse(&allocator, ParserOptions::commonmark());
    let html = HtmlRenderer::with_options(HtmlRendererOptions::commonmark()).render(&document);

    assert!(html.contains("<h1>Hello</h1>"), "{html}");
    assert!(!html.contains("id=\"hello\""), "{html}");
    assert!(html.contains("<blockquote>"), "{html}");
    assert!(!html.contains("ox-callout"), "{html}");
    assert!(html.contains("<p>[[toc]]</p>"), "{html}");
    assert!(!html.contains("ox-toc"), "{html}");
    assert!(html.contains("class=\"language-js{1}\""), "{html}");
}

#[test]
fn gfm_spec_uses_tagfilter_without_footnotes_or_ids() {
    let parser_options = ParserOptions::gfm_spec();
    assert!(!parser_options.footnotes);
    assert!(parser_options.tables);
    assert!(parser_options.strikethrough);

    let allocator = Allocator::new();
    let document = parse(&allocator, parser_options);
    let html = HtmlRenderer::with_options(HtmlRendererOptions::gfm()).render(&document);
    assert!(!html.contains("id=\"hello\""), "{html}");
    assert!(!html.contains("ox-callout"), "{html}");
    assert!(html.contains("&lt;script>alert(1)&lt;/script>"), "{html}");
    assert!(html.contains("class=\"language-js{1}\""), "{html}");
}

#[test]
fn spec_profile_flags_match_normal_hooks_and_incremental_paths() {
    let allocator = Allocator::new();
    let document = parse(&allocator, ParserOptions::gfm_spec());
    let options = HtmlRendererOptions::gfm();

    let expected = HtmlRenderer::with_options(options.clone()).render(&document);
    let mut hooks_renderer = HtmlRenderer::with_options(options.clone());
    let hooked = hooks_renderer.render_with_hooks(&document, &mut NoHtmlRenderHooks);
    assert_eq!(hooked, expected);

    let mut borrowed_renderer = HtmlRenderer::with_options(options.clone());
    assert_eq!(borrowed_renderer.render_borrowed(&document), expected);

    let mut incremental_renderer = HtmlRenderer::with_options(options);
    let incremental = incremental_renderer.render_incremental_fragment(&document);
    assert_eq!(incremental, expected);
}

#[test]
fn heading_attributes_match_across_renderer_paths_and_disabled_ids() {
    let allocator = Allocator::new();
    let source = "# Custom {#custom-id .highlight .wide}\n";
    let document = parse_source(
        &allocator,
        source,
        ParserOptions {
            heading_attributes: true,
            ..ParserOptions::commonmark()
        },
    );

    let expected = HtmlRenderer::new().render(&document);
    assert_eq!(
        expected,
        "<h1 id=\"custom-id\" class=\"highlight wide\">Custom</h1>\n"
    );

    let mut hooks_renderer = HtmlRenderer::new();
    assert_eq!(
        hooks_renderer.render_with_hooks(&document, &mut NoHtmlRenderHooks),
        expected
    );

    let mut borrowed_renderer = HtmlRenderer::new();
    assert_eq!(borrowed_renderer.render_borrowed(&document), expected);

    let mut incremental_renderer = HtmlRenderer::new();
    assert_eq!(
        incremental_renderer.render_incremental_fragment(&document),
        expected
    );

    let mut strict_options = HtmlRendererOptions::commonmark();
    strict_options.heading_permalinks = true;
    strict_options.inline_toc = true;
    let strict_source = "# Custom {#custom-id .highlight .wide}\n\n[[toc]]\n";
    let strict_document = parse_source(
        &allocator,
        strict_source,
        ParserOptions {
            heading_attributes: true,
            ..ParserOptions::commonmark()
        },
    );
    let strict = HtmlRenderer::with_options(strict_options.clone()).render(&strict_document);
    assert_eq!(
        strict,
        "<h1 class=\"highlight wide\">Custom</h1>\n<p>[[toc]]</p>\n"
    );
    assert!(!strict.contains("header-anchor"), "{strict}");
    assert!(!strict.contains("ox-toc"), "{strict}");
    let mut reused = HtmlRenderer::with_options(strict_options.clone());
    let mut incremental = HtmlRenderer::with_options(strict_options.clone());
    let mut hooked = HtmlRenderer::with_options(strict_options);
    for _ in 0..2 {
        assert_eq!(reused.render_borrowed(&strict_document), strict);
        assert_eq!(
            incremental.render_incremental_fragment(&strict_document),
            strict
        );
        assert_eq!(
            hooked.render_incremental_fragment_with_hooks(&strict_document, &mut NoHtmlRenderHooks),
            strict,
        );
        assert_eq!(
            hooked.render_provisional_fragment_with_hooks(&strict_document, &mut NoHtmlRenderHooks),
            strict,
        );
    }
}

#[test]
fn existing_defaults_keep_heading_ids_callouts_and_toc() {
    let allocator = Allocator::new();
    let document = parse(&allocator, ParserOptions::gfm());
    let html = HtmlRenderer::new().render(&document);
    assert!(html.contains("id=\"hello\""), "{html}");
    assert!(html.contains("ox-callout"), "{html}");
    assert!(html.contains("ox-toc"), "{html}");
}
