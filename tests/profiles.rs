use ferromark::allocator::Allocator;
use ferromark::parser::{Parser, ParserOptions};
use ferromark::renderer::{HtmlRenderer, HtmlRendererOptions, NoHtmlRenderHooks};

const PROFILE_SOURCE: &str =
    "# Hello\n\n> [!NOTE]\n> body\n\n[[toc]]\n\n<script>alert(1)</script>\n\n```js{1}\ncode\n```\n";

fn parse<'a>(allocator: &'a Allocator, options: ParserOptions) -> ferromark::ast::Document<'a> {
    parse_source(allocator, PROFILE_SOURCE, options)
}

fn parse_source<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    options: ParserOptions,
) -> ferromark::ast::Document<'a> {
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
    let html = HtmlRenderer::with_options(HtmlRendererOptions::gfm_spec()).render(&document);
    assert!(!html.contains("id=\"hello\""), "{html}");
    assert!(!html.contains("ox-callout"), "{html}");
    assert!(html.contains("&lt;script>alert(1)&lt;/script>"), "{html}");
    assert!(html.contains("class=\"language-js{1}\""), "{html}");
}

#[test]
fn spec_profile_flags_match_normal_hooks_and_incremental_paths() {
    let allocator = Allocator::new();
    let document = parse(&allocator, ParserOptions::gfm_spec());
    let options = HtmlRendererOptions::gfm_spec();

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

/// The renderer flags the profiles disagree about, in a comparable shape.
fn renderer_profile_flags(options: &HtmlRendererOptions) -> [bool; 8] {
    [
        options.disallow_raw_html,
        options.heading_ids,
        options.callouts,
        options.inline_toc,
        options.code_fence_metadata,
        options.autolink_urls,
        options.autolink_target_blank,
        options.link_target_blank,
    ]
}

#[test]
fn parser_profiles_set_the_documented_fields() {
    let commonmark = ParserOptions::commonmark();
    assert!(!commonmark.footnotes);
    assert!(!commonmark.task_lists);
    assert!(!commonmark.tables);
    assert!(!commonmark.strikethrough);
    assert!(!commonmark.autolinks);
    assert!(!commonmark.mdx);
    assert!(!commonmark.highlight);
    assert!(!commonmark.inline_footnotes);
    assert!(commonmark.allow_link_refs);
    assert_eq!(commonmark.max_nesting_depth, 100);

    // `gfm()` is the convenience profile: GFM plus Ferromark's semantic
    // footnotes. `gfm_spec()` is the same set without them.
    let gfm = ParserOptions::gfm();
    assert!(gfm.footnotes);
    assert!(gfm.task_lists);
    assert!(gfm.tables);
    assert!(gfm.strikethrough);
    assert!(gfm.autolinks);
    assert!(!gfm.mdx);
    assert!(!gfm.highlight);
    assert!(!gfm.inline_footnotes);
    assert!(!gfm.merged_table_cells);
    assert!(!gfm.table_attributes);
    assert!(!gfm.cjk_emphasis);
    assert!(gfm.allow_link_refs);

    let gfm_spec = ParserOptions::gfm_spec();
    assert!(!gfm_spec.footnotes);
    assert!(gfm_spec.task_lists);
    assert!(gfm_spec.tables);
    assert!(gfm_spec.strikethrough);
    assert!(gfm_spec.autolinks);
    assert!(!gfm_spec.mdx);

    let mdx = ParserOptions::mdx();
    assert!(mdx.mdx);
    assert!(!mdx.footnotes);
    assert!(!mdx.tables);
    assert!(!mdx.strikethrough);
    assert!(!mdx.autolinks);
    assert_eq!(mdx.max_nesting_depth, 100);
}

#[test]
fn renderer_profiles_set_the_documented_fields() {
    // `new()` keeps every product convenience and filters nothing.
    let new = HtmlRendererOptions::new();
    assert_eq!(
        renderer_profile_flags(&new),
        [false, true, true, true, true, true, true, true]
    );

    // `commonmark()` drops the conveniences and still passes raw HTML through.
    let commonmark = HtmlRendererOptions::commonmark();
    assert_eq!(
        renderer_profile_flags(&commonmark),
        [false, false, false, false, false, false, false, false]
    );

    // `gfm()` is `new()` plus the GFM tag filter, matching the parser's
    // convenience profile of the same name.
    let gfm = HtmlRendererOptions::gfm();
    assert_eq!(
        renderer_profile_flags(&gfm),
        [true, true, true, true, true, true, true, true]
    );

    // `gfm_spec()` is `commonmark()` plus the GFM tag filter.
    let gfm_spec = HtmlRendererOptions::gfm_spec();
    assert_eq!(
        renderer_profile_flags(&gfm_spec),
        [true, false, false, false, false, false, false, false]
    );

    // No profile turns on sanitization, XHTML output, or link conversion, and
    // none of them changes a documented string default.
    for (label, options) in [
        ("new", &new),
        ("commonmark", &commonmark),
        ("gfm", &gfm),
        ("gfm_spec", &gfm_spec),
    ] {
        assert!(!options.sanitize, "{label}");
        assert!(!options.xhtml, "{label}");
        assert!(!options.convert_md_links, "{label}");
        assert!(!options.source_spans, "{label}");
        assert!(!options.heading_permalinks, "{label}");
        assert!(!options.semantic_footnotes, "{label}");
        assert!(!options.code_annotations, "{label}");
        assert!(!options.table_colgroup, "{label}");
        assert!(!options.table_column_names, "{label}");
        assert_eq!(&*options.soft_break, "\n", "{label}");
        assert_eq!(&*options.hard_break, "<br>\n", "{label}");
        assert_eq!(&*options.base_url, "/", "{label}");
        assert_eq!(&*options.source_path, "", "{label}");
        assert_eq!(&*options.code_annotation_meta_key, "annotate", "{label}");
        assert_eq!(options.toc_max_depth, 3, "{label}");
    }
}

#[test]
fn the_gfm_renderer_profiles_differ_only_in_the_product_conveniences() {
    let allocator = Allocator::new();
    let document = parse(&allocator, ParserOptions::gfm());

    let convenience = HtmlRenderer::with_options(HtmlRendererOptions::gfm()).render(&document);
    let strict = HtmlRenderer::with_options(HtmlRendererOptions::gfm_spec()).render(&document);

    // Both filter the disallowed raw HTML tag; only the convenience profile
    // adds heading IDs, callouts and TOC substitution.
    for html in [&convenience, &strict] {
        assert!(html.contains("&lt;script>alert(1)&lt;/script>"), "{html}");
    }
    assert!(convenience.contains("id=\"hello\""), "{convenience}");
    assert!(convenience.contains("ox-callout"), "{convenience}");
    assert!(convenience.contains("ox-toc"), "{convenience}");
    assert!(!strict.contains("id=\"hello\""), "{strict}");
    assert!(!strict.contains("ox-callout"), "{strict}");
    assert!(!strict.contains("ox-toc"), "{strict}");
}
