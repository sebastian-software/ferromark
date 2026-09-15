//! Public-output coverage for TOC discovery, heading IDs, and renderer reuse.

#[path = "support/edge.rs"]
mod edge_support;

use edge_support::render;
use ferromark::allocator::Allocator;
use ferromark::parser::{Parser, ParserOptions};
use ferromark::renderer::{HtmlRenderer, HtmlRendererOptions};

#[test]
fn toc_markers_in_block_containers_discover_document_headings() {
    for container in [
        "> [[toc]]\n>\n> ## Nested",
        "- [[toc]]\n\n  ## Nested",
        "[^note]: [[toc]]\n\n    ## Nested",
        "<Notice>[[toc]]\n\n## Nested\n</Notice>",
    ] {
        for semantic_footnotes in [false, true] {
            let source = format!("# Root\n\n{container}\n");
            let html = render(
                &source,
                ParserOptions {
                    mdx: true,
                    ..ParserOptions::gfm()
                },
                HtmlRendererOptions {
                    semantic_footnotes,
                    ..HtmlRendererOptions::default()
                },
            );
            assert_eq!(html.matches("<nav class=\"ox-toc\"").count(), 1, "{html}");
            for (id, title) in [("root", "Root"), ("nested", "Nested")] {
                assert!(
                    html.contains(&format!("href=\"#{id}\">{title}</a>")),
                    "{html}"
                );
                assert!(html.contains(&format!("id=\"{id}\"")), "{html}");
            }
        }
    }
}

#[test]
fn excluded_headings_still_reserve_automatic_ids() {
    let html = render(
        "[[toc]]\n\n#### Repeated\n\n#### Repeated\n\n## Repeated\n\n#### Deep {#fixed}\n\n## Visible {#fixed}\n",
        ParserOptions {
            heading_attributes: true,
            ..ParserOptions::default()
        },
        HtmlRendererOptions {
            toc_max_depth: 2,
            ..HtmlRendererOptions::default()
        },
    );
    let nav = html.split("</nav>").next().unwrap();
    assert!(nav.contains("href=\"#repeated-2\">Repeated</a>"), "{html}");
    assert!(
        html.contains("<h2 id=\"repeated-2\">Repeated</h2>"),
        "{html}"
    );
    assert!(nav.contains("href=\"#fixed\">Visible</a>"), "{html}");
    assert!(!nav.contains(">Deep</a>"), "{html}");
    assert_eq!(nav.matches("<li ").count(), 2, "{html}");
}

#[test]
fn toc_requires_a_standalone_text_marker() {
    for marker in [
        "[[toc]] suffix",
        "[[to c]]",
        "**[[toc]]**",
        "[[toc]] x",
        "[[toc]] [[toc]]",
    ] {
        let html = render(
            &format!("{marker}\n\n# Heading"),
            ParserOptions::default(),
            HtmlRendererOptions::default(),
        );
        assert!(!html.contains("<nav "), "{html}");
    }
    let html = render(
        "  [[ToC]]  \n\n# Heading",
        ParserOptions::default(),
        HtmlRendererOptions::default(),
    );
    assert!(html.contains("href=\"#heading\">Heading</a>"), "{html}");
}

#[test]
fn reused_renderer_does_not_retain_previous_toc_entries() {
    let allocator = Allocator::new();
    let first = Parser::new(&allocator, "[[toc]]\n\n# Earlier")
        .parse()
        .unwrap();
    let second = Parser::new(&allocator, "[[toc]]\n\n# Later")
        .parse()
        .unwrap();
    let empty = Parser::new(&allocator, "[[toc]]").parse().unwrap();
    let mut renderer = HtmlRenderer::new();
    assert!(renderer.render(&first).contains("href=\"#earlier\""));
    let html = renderer.render(&second);
    assert!(html.contains("href=\"#later\""), "{html}");
    assert!(!html.contains("earlier"), "{html}");
    assert_eq!(renderer.render(&empty), "");
}
