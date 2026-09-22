use crate::allocator::Allocator;
use crate::parser::{Parser, ParserOptions};
use crate::renderer::html::{HtmlRenderer, HtmlRendererOptions, NoHtmlRenderHooks};

#[test]
fn test_render_paragraph() {
    let allocator = Allocator::new();
    let doc = Parser::new(&allocator, "Hello world").parse().unwrap();
    let mut renderer = HtmlRenderer::new();
    let html = renderer.render(&doc);
    assert_eq!(html, "<p>Hello world</p>\n");
}

#[test]
fn test_render_heading() {
    let allocator = Allocator::new();
    let doc = Parser::new(&allocator, "# Hello").parse().unwrap();
    let mut renderer = HtmlRenderer::new();
    let html = renderer.render(&doc);
    assert_eq!(html, "<h1 id=\"hello\">Hello</h1>\n");
}

#[test]
fn test_render_heading_attributes_use_explicit_id_and_classes() {
    let html = render_html_with_options(
        "### Custom identifier {#custom-heading-id .highlight .wide}",
        ParserOptions {
            heading_attributes: true,
            ..ParserOptions::default()
        },
    );

    assert_eq!(
        html,
        "<h3 id=\"custom-heading-id\" class=\"highlight wide\">Custom identifier</h3>\n"
    );
}

#[test]
fn test_render_heading_attributes_escape_explicit_attrs() {
    let html = render_html_with_options(
        "### Custom {#a\"b .x<y}",
        ParserOptions {
            heading_attributes: true,
            ..ParserOptions::default()
        },
    );

    assert_eq!(html, "<h3 id=\"a&quot;b\" class=\"x&lt;y\">Custom</h3>\n");
}

#[test]
fn test_explicit_heading_ids_keep_escaping_in_id_and_permalink_href() {
    // Generated slugs bypass attribute escaping because their alphabet cannot
    // produce an escapable byte. Author-supplied ids have no such guarantee, so
    // both the `id` attribute and the permalink `href` must still escape them.
    let allocator = Allocator::new();
    let doc = Parser::with_options(
        &allocator,
        "## Custom {#a\"b&c<d>e'f}",
        ParserOptions {
            heading_attributes: true,
            ..ParserOptions::default()
        },
    )
    .parse()
    .unwrap();
    let mut renderer = HtmlRenderer::with_options(HtmlRendererOptions {
        heading_permalinks: true,
        ..Default::default()
    });
    let html = renderer.render(&doc);

    let escaped = "a&quot;b&amp;c&lt;d&gt;e&#39;f";
    assert!(html.contains(&format!("<h2 id=\"{escaped}\">")), "{html}");
    assert!(html.contains(&format!("href=\"#{escaped}\"")), "{html}");
    // The raw characters must not reach the output through either path.
    assert!(!html.contains("a\"b"), "{html}");
    assert!(!html.contains("c<d>e"), "{html}");
}

#[test]
fn test_generated_heading_ids_are_emitted_verbatim() {
    // Duplicate, Unicode, and fallback slugs all take the no-escape path; the
    // emitted bytes must equal the slugifier's own output.
    let html = render_with_permalinks(
        "# Options & Defaults\n\n# Options & Defaults\n\n# はじめに\n\n# !!!\n",
    );

    assert!(html.contains("<h1 id=\"options-defaults\">"), "{html}");
    assert!(html.contains("<h1 id=\"options-defaults-1\">"), "{html}");
    assert!(html.contains("href=\"#options-defaults-1\""), "{html}");
    assert!(html.contains("<h1 id=\"はじめに\">"), "{html}");
    assert!(html.contains("href=\"#はじめに\""), "{html}");
    assert!(html.contains("<h1 id=\"section\">"), "{html}");
}

#[test]
fn test_render_crlf_fenced_code_like_lf() {
    let lf = render_html("```rust\nfn main() {}\n```\n");
    let crlf = render_html("```rust\r\nfn main() {}\r\n```\r\n");

    assert_eq!(crlf, lf);
    assert_eq!(
        crlf,
        "<pre><code class=\"language-rust\">fn main() {}\n</code></pre>\n"
    );
}

#[test]
fn test_render_heading_ids_are_unique_and_unicode() {
    let allocator = Allocator::new();
    let doc = Parser::new(&allocator, "## はじめに\n## はじめに")
        .parse()
        .unwrap();
    let mut renderer = HtmlRenderer::new();
    let html = renderer.render(&doc);
    insta::assert_snapshot!(html);
}

#[test]
fn heading_ids_skip_suffix_collisions_inside_containers_and_for_explicit_ids() {
    let allocator = Allocator::new();
    let doc = Parser::with_options(
        &allocator,
        "> # a\n>\n> # a\n>\n> # a-1 {#a-1}\n\n# b {#b}\n\n# b {#b}",
        ParserOptions {
            heading_attributes: true,
            ..ParserOptions::default()
        },
    )
    .parse()
    .unwrap();
    let mut renderer = HtmlRenderer::with_options(HtmlRendererOptions {
        heading_permalinks: true,
        ..Default::default()
    });
    let html = renderer.render(&doc);

    for id in ["a", "a-1", "a-1-1", "b", "b-1"] {
        assert!(html.contains(&format!("id=\"{id}\"")), "{id}: {html}");
        assert!(html.contains(&format!("href=\"#{id}\"")), "{id}: {html}");
    }
    assert_eq!(html.matches("id=\"a-1\"").count(), 1, "{html}");
    assert_eq!(html.matches("id=\"a-1-1\"").count(), 1, "{html}");
    assert_eq!(html.matches("id=\"b-1\"").count(), 1, "{html}");

    let mut hooked_renderer = HtmlRenderer::with_options(HtmlRendererOptions {
        heading_permalinks: true,
        ..Default::default()
    });
    let hooked = hooked_renderer.render_with_hooks(&doc, &mut NoHtmlRenderHooks);
    assert_eq!(hooked, html);
}

fn render_html(source: &str) -> String {
    let allocator = Allocator::new();
    let doc = Parser::new(&allocator, source).parse().unwrap();
    let mut renderer = HtmlRenderer::new();
    renderer.render(&doc)
}

fn render_html_with_options(source: &str, options: ParserOptions) -> String {
    let allocator = Allocator::new();
    let doc = Parser::with_options(&allocator, source, options)
        .parse()
        .unwrap();
    let mut renderer = HtmlRenderer::new();
    renderer.render(&doc)
}

#[test]
fn test_render_heading_id_uses_inline_text() {
    let allocator = Allocator::new();
    let doc = Parser::new(&allocator, "## **API** `Index` [Guide](./guide.md)")
        .parse()
        .unwrap();
    let mut renderer = HtmlRenderer::new();
    let html = renderer.render(&doc);
    insta::assert_snapshot!(html);
}

fn render_with_permalinks(source: &str) -> String {
    let allocator = Allocator::new();
    let doc = Parser::new(&allocator, source).parse().unwrap();
    let mut renderer = HtmlRenderer::with_options(HtmlRendererOptions {
        heading_permalinks: true,
        ..Default::default()
    });
    renderer.render(&doc)
}

#[test]
fn test_heading_permalinks_default_off_keeps_html() {
    let allocator = Allocator::new();
    let doc = Parser::new(&allocator, "# Hello").parse().unwrap();
    let html = HtmlRenderer::new().render(&doc);
    assert_eq!(html, "<h1 id=\"hello\">Hello</h1>\n");
}

#[test]
fn test_heading_permalinks_reuse_generated_id() {
    let html = render_with_permalinks("# Hello World");
    assert_eq!(
        html,
        "<h1 id=\"hello-world\">Hello World<a class=\"header-anchor\" href=\"#hello-world\" aria-label=\"Permalink to &quot;Hello World&quot;\">#</a></h1>\n"
    );
}

#[test]
fn test_heading_permalinks_unicode_and_duplicates() {
    let html = render_with_permalinks("## はじめに\n## はじめに");
    insta::assert_snapshot!(html);
}

#[test]
fn test_heading_permalinks_skip_existing_hash_link() {
    let html = render_with_permalinks("## Hello [#](#hello)");
    assert!(html.contains("<h2 id=\"hello\">"), "{html}");
    assert_eq!(html.matches("href=\"#hello\"").count(), 1, "{html}");
    assert!(!html.contains("class=\"header-anchor\""), "{html}");
}

#[test]
fn test_heading_permalinks_skip_existing_header_anchor_html() {
    let html = render_with_permalinks(
        "## Hello <a class=\"header-anchor\" href=\"#hello\" aria-label=\"Permalink to &quot;Hello&quot;\">#</a>",
    );
    assert_eq!(html.matches("class=\"header-anchor\"").count(), 1, "{html}");
}

#[test]
fn test_heading_permalinks_empty_heading_uses_section_id() {
    let html = render_with_permalinks("#");
    assert!(
        html.contains("<h1 id=\"section\"><a class=\"header-anchor\" href=\"#section\" aria-label=\"Permalink to this section\">#</a></h1>"),
        "{html}"
    );
}

#[test]
fn test_heading_permalinks_are_real_links_without_js() {
    let html = render_with_permalinks("## API");
    assert!(
        html.contains("<a class=\"header-anchor\" href=\"#api\""),
        "{html}"
    );
    assert!(!html.contains("onclick="), "{html}");
    assert!(!html.contains("<script"), "{html}");
}

#[test]
fn test_render_toc_marker_as_literal_without_extension() {
    let allocator = Allocator::new();
    let doc = Parser::new(&allocator, "# Title\n\n[[toc]]\n\n## Intro")
        .parse()
        .unwrap();
    assert_eq!(
        HtmlRenderer::new().render(&doc),
        "<h1 id=\"title\">Title</h1>\n<p>[[toc]]</p>\n<h2 id=\"intro\">Intro</h2>\n"
    );
}

#[test]
fn test_render_block_quote() {
    let allocator = Allocator::new();
    let doc = Parser::new(&allocator, "> Hello world").parse().unwrap();
    let mut renderer = HtmlRenderer::new();
    let html = renderer.render(&doc);
    assert_eq!(html, "<blockquote>\n<p>Hello world</p>\n</blockquote>\n");
}

#[test]
fn test_render_block_quote_with_inline() {
    let allocator = Allocator::new();
    let doc = Parser::new(&allocator, "> **Note:** This is important")
        .parse()
        .unwrap();
    let mut renderer = HtmlRenderer::new();
    let html = renderer.render(&doc);
    insta::assert_snapshot!(html);
}

#[test]
fn test_render_github_style_important_callout() {
    let allocator = Allocator::new();
    let doc = Parser::new(&allocator, "> [!IMPORTANT]\n> This is important.")
        .parse()
        .unwrap();
    let mut renderer = HtmlRenderer::new();
    let html = renderer.render(&doc);

    insta::assert_snapshot!(html);
}

#[test]
fn test_render_github_style_callout_with_inline_content_after_marker() {
    let allocator = Allocator::new();
    let doc = Parser::new(&allocator, "> [!NOTE] Supports **inline** content")
        .parse()
        .unwrap();
    let mut renderer = HtmlRenderer::new();
    let html = renderer.render(&doc);

    insta::assert_snapshot!(html);
}
