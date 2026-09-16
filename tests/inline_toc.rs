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

/// The setup scan is skipped or reduced depending on the renderer options, so
/// every container it can descend into needs coverage: a marker nested two
/// levels deep must still be found, and must still be suppressed when the
/// document has no TOC entries to show.
#[test]
fn toc_markers_nested_two_levels_deep_are_still_discovered() {
    for container in [
        "> - [[toc]]\n>\n> - other",
        "- > [[toc]]",
        "- - [[toc]]\n\n  - other",
        "> > [[toc]]",
        "[^note]: - [[toc]]\n\n    - other",
        "<Notice>\n<Card>\n[[toc]]\n</Card>\n</Notice>",
        "> [^note]: [[toc]]",
    ] {
        let parser_options = ParserOptions {
            mdx: true,
            ..ParserOptions::gfm()
        };

        let source = format!("# Root\n\n{container}\n\n## Nested\n");
        let html = render(
            &source,
            parser_options.clone(),
            HtmlRendererOptions::default(),
        );
        assert_eq!(
            html.matches("<nav class=\"ox-toc\"").count(),
            1,
            "{container:?} -> {html}"
        );
        for (id, title) in [("root", "Root"), ("nested", "Nested")] {
            assert!(
                html.contains(&format!("href=\"#{id}\">{title}</a>")),
                "{container:?} -> {html}"
            );
        }
        assert!(!html.contains("[[toc]]"), "{container:?} -> {html}");

        // A marker with nothing to list still suppresses the literal text.
        let empty = render(
            &format!("{container}\n"),
            parser_options,
            HtmlRendererOptions::default(),
        );
        assert!(!empty.contains("<nav "), "{container:?} -> {empty}");
        assert!(!empty.contains("[[toc]]"), "{container:?} -> {empty}");
    }
}

/// Renderers that cannot act on a marker skip part or all of the setup scan.
/// They must still emit the literal paragraph, exactly as when the scan ran
/// and the result was discarded.
#[test]
fn disabled_toc_options_emit_the_literal_marker_from_any_container() {
    for container in ["[[toc]]", "> [[toc]]", "- - [[toc]]", "[^note]: [[toc]]"] {
        for (inline_toc, heading_ids) in [(false, true), (true, false), (false, false)] {
            let html = render(
                &format!("# Root\n\n{container}\n\n## Nested\n"),
                ParserOptions::gfm(),
                HtmlRendererOptions {
                    inline_toc,
                    heading_ids,
                    ..HtmlRendererOptions::default()
                },
            );
            assert!(!html.contains("<nav "), "{container:?} -> {html}");
            assert!(html.contains("[[toc]]"), "{container:?} -> {html}");
            assert_eq!(
                html.contains("id=\"root\""),
                heading_ids,
                "{container:?} -> {html}"
            );
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
