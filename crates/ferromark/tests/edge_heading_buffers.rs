use ferromark::allocator::Allocator;
use ferromark::ast::Node;
use ferromark::parser::{Parser, ParserOptions};
use ferromark::renderer::{HtmlRenderer, HtmlRendererOptions};

#[test]
fn explicit_heading_ids_escape_identically_in_ids_and_permalinks() {
    let allocator = Allocator::new();
    let mut document = Parser::new(&allocator, "## Title").parse().unwrap();
    let Node::Heading(heading) = &mut document.children[0] else {
        panic!("expected heading");
    };
    heading.id = Some("日本語<&>\"'\r\n");
    let mut renderer = HtmlRenderer::with_options(HtmlRendererOptions {
        heading_permalinks: true,
        ..Default::default()
    });
    let expected = concat!(
        "<h2 id=\"日本語&lt;&amp;&gt;&quot;&#39;&#13;&#10;\">Title",
        "<a class=\"header-anchor\" href=\"#日本語&lt;&amp;&gt;&quot;&#39;&#13;&#10;\" ",
        "aria-label=\"Permalink to &quot;Title&quot;\">#</a></h2>\n"
    );
    assert_eq!(renderer.render_borrowed(&document), expected);
    assert_eq!(renderer.render(&document), expected);
}

/// The heading scratch buffers are allocated on first use rather than in the
/// constructor, so the first heading (or footnote slug) a renderer ever sees
/// takes a different path than later ones. A renderer built per document and a
/// renderer reused across documents must still agree byte for byte, in every
/// order the two paths can be reached.
#[test]
fn lazily_sized_heading_buffers_match_a_reused_renderer_in_any_order() {
    let allocator = Allocator::new();
    let sources = [
        // No heading at all: nothing must touch the scratch buffers.
        "Just a paragraph with https://example.com in it.\n",
        // Footnote slugs borrow the slug scratch without any heading.
        "Ref[^one] and[^one] and[^!!!]\n\n[^one]: First\n\n[^!!!]: Symbolic\n",
        // Explicit ids skip slugification but still use the id scratch.
        "## Explicit {#fixed}\n\n## Explicit {#fixed}\n",
        // Generated ids, duplicates, and a non-ASCII slug.
        "# Root\n\n## Dup\n\n## Dup\n\n## 日本語の見出し\n",
        // A heading whose text slugifies to nothing.
        "## ---\n\n## ---\n",
        // Headings and footnotes together, sharing the slug scratch.
        "# Shared\n\nBody[^shared]\n\n[^shared]: Shared note\n",
        "",
    ];
    for semantic_footnotes in [false, true] {
        let options = HtmlRendererOptions {
            semantic_footnotes,
            heading_permalinks: true,
            ..HtmlRendererOptions::default()
        };
        let parser_options = ParserOptions {
            heading_attributes: true,
            ..ParserOptions::gfm()
        };
        let documents: Vec<_> = sources
            .iter()
            .map(|source| {
                Parser::with_options(&allocator, source, parser_options.clone())
                    .parse()
                    .unwrap()
            })
            .collect();

        // Rotating the start point exercises every "which shape warmed the
        // buffers first" ordering, including footnotes before headings.
        for offset in 0..documents.len() {
            let mut reused = HtmlRenderer::with_options(options.clone());
            for step in 0..documents.len() {
                let document = &documents[(offset + step) % documents.len()];
                let fresh = HtmlRenderer::with_options(options.clone()).render(document);
                assert_eq!(reused.render_borrowed(document), fresh, "offset {offset}");
            }
        }
    }
}

#[test]
fn heading_buffers_do_not_leak_long_ids_or_duplicate_counts_across_renders() {
    let allocator = Allocator::new();
    let long_title = "日本語の長い見出し".repeat(64);
    let long_source = format!("## {long_title}\n\n## {long_title}");
    let long_document = Parser::new(&allocator, &long_source).parse().unwrap();
    let short_document = Parser::new(&allocator, "## A\n\n## A").parse().unwrap();
    let empty_document = Parser::new(&allocator, "").parse().unwrap();
    let options = HtmlRendererOptions {
        heading_permalinks: true,
        ..Default::default()
    };
    let mut renderer = HtmlRenderer::with_options(options.clone());
    let expected = concat!(
        "<h2 id=\"a\">A<a class=\"header-anchor\" href=\"#a\" ",
        "aria-label=\"Permalink to &quot;A&quot;\">#</a></h2>\n",
        "<h2 id=\"a-1\">A<a class=\"header-anchor\" href=\"#a-1\" ",
        "aria-label=\"Permalink to &quot;A&quot;\">#</a></h2>\n"
    );
    for _ in 0..3 {
        assert_eq!(
            renderer.render_borrowed(&long_document),
            HtmlRenderer::with_options(options.clone()).render(&long_document)
        );
        assert_eq!(renderer.render_borrowed(&short_document), expected);
        assert_eq!(renderer.render_borrowed(&empty_document), "");
        assert_eq!(renderer.render(&short_document), expected);
    }
}
