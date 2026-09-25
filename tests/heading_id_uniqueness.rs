use ferromark::renderer::{HtmlRenderer, HtmlRendererOptions, NoHtmlRenderHooks};
use ferromark::{Allocator, Parser, ParserOptions};

fn render(
    source: &str,
    parser_options: ParserOptions,
    renderer_options: HtmlRendererOptions,
) -> String {
    let allocator = Allocator::for_source_len(source.len());
    let document = Parser::with_options(&allocator, source, parser_options)
        .parse()
        .unwrap();
    HtmlRenderer::with_options(renderer_options).render(&document)
}

fn ids(html: &str) -> Vec<&str> {
    html.lines()
        .filter(|line| line.trim_start().starts_with("<h"))
        .filter_map(|line| line.split_once(" id=\"").map(|(_, rest)| rest))
        .filter_map(|rest| rest.split_once('"').map(|(id, _)| id))
        .collect()
}

#[test]
fn generated_and_explicit_heading_ids_are_unique() {
    let html = render(
        "# a\n\n# a\n\n# a-1\n\n# a-1\n\n# b {#b}\n\n# b {#b}",
        ParserOptions {
            heading_attributes: true,
            ..ParserOptions::default()
        },
        HtmlRendererOptions::default(),
    );

    assert_eq!(ids(&html), ["a", "a-1", "a-1-1", "a-1-2", "b", "b-1"]);
}

#[test]
fn unicode_and_nested_headings_use_the_same_unique_id_planner() {
    let html = render(
        "# 日本語\n\n# 日本語\n\n> # Nested\n>\n> # Nested",
        ParserOptions::default(),
        HtmlRendererOptions::default(),
    );

    assert_eq!(ids(&html), ["日本語", "日本語-1", "nested", "nested-1"]);
}

#[test]
fn heading_ids_do_not_claim_legacy_or_semantic_footnote_ids() {
    let parser_options = ParserOptions {
        footnotes: true,
        ..ParserOptions::default()
    };
    let source = "# fn-1\n\n# fnref-1-2\n\nA reference[^1] and another[^1].\n\n[^1]: note";

    let legacy = render(
        source,
        parser_options.clone(),
        HtmlRendererOptions::default(),
    );
    assert_eq!(ids(&legacy), ["fn-1-1", "fnref-1-2-1"]);
    assert!(legacy.contains("<div id=\"fn-1\""));

    let semantic = render(
        source,
        parser_options,
        HtmlRendererOptions {
            semantic_footnotes: true,
            ..HtmlRendererOptions::default()
        },
    );
    assert_eq!(ids(&semantic), ["fn-1-1", "fnref-1-2-1"]);
    assert!(semantic.contains("<li id=\"fn-1\""));
}

#[test]
fn inline_toc_and_hooked_rendering_use_the_emitted_heading_ids() {
    let source = "[[toc]]\n\n# a\n\n# a\n\n# a-1";
    let parser_options = ParserOptions::default();
    let renderer_options = HtmlRendererOptions {
        inline_toc: true,
        ..HtmlRendererOptions::default()
    };
    let allocator = Allocator::for_source_len(source.len());
    let document = Parser::with_options(&allocator, source, parser_options)
        .parse()
        .unwrap();
    let mut renderer = HtmlRenderer::with_options(renderer_options);
    let ordinary = renderer.render(&document);
    let mut hooks = NoHtmlRenderHooks;
    let hooked = renderer.render_with_hooks(&document, &mut hooks);

    assert_eq!(ordinary, hooked);
    assert!(ordinary.contains("href=\"#a-1-1\""));
    assert_eq!(ids(&ordinary), ["a", "a-1", "a-1-1"]);
}

#[test]
fn heading_permalink_fragments_use_the_unique_id() {
    let html = render(
        "# repeated\n\n# repeated",
        ParserOptions::default(),
        HtmlRendererOptions {
            heading_permalinks: true,
            ..HtmlRendererOptions::default()
        },
    );

    assert!(
        html.contains("<h1 id=\"repeated\">repeated<a class=\"header-anchor\" href=\"#repeated\"")
    );
    assert!(
        html.contains(
            "<h1 id=\"repeated-1\">repeated<a class=\"header-anchor\" href=\"#repeated-1\""
        )
    );
}

#[test]
fn committed_and_provisional_fragments_preserve_heading_id_state() {
    let source = "# repeated";
    let allocator = Allocator::for_source_len(source.len());
    let document = Parser::new(&allocator, source).parse().unwrap();
    let mut renderer = HtmlRenderer::new();

    assert_eq!(
        ids(&renderer.render_incremental_fragment(&document)),
        ["repeated"]
    );
    assert_eq!(
        ids(&renderer.render_provisional_fragment(&document)),
        ["repeated-1"]
    );
    assert_eq!(
        ids(&renderer.render_incremental_fragment(&document)),
        ["repeated-1"]
    );
    renderer.reset_incremental_state();
    assert_eq!(
        ids(&renderer.render_incremental_fragment(&document)),
        ["repeated"]
    );
}
