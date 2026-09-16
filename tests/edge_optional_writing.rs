//! Optional writing syntax and reference-policy regressions.
use ferromark::allocator::Allocator;
use ferromark::ast::{Node, Visit};
use ferromark::parser::{Parser, ParserOptions};
use ferromark::renderer::{HtmlRenderer, HtmlRendererOptions, NoHtmlRenderHooks};

fn options() -> ParserOptions {
    ParserOptions {
        highlight: true,
        inline_footnotes: true,
        ..ParserOptions::gfm()
    }
}
fn render(source: &str, options: ParserOptions) -> String {
    let arena = Allocator::new();
    let doc = Parser::with_options(&arena, source, options)
        .parse()
        .unwrap();
    HtmlRenderer::with_options(HtmlRendererOptions::commonmark()).render(&doc)
}
#[test]
fn extensions_are_opt_in_in_every_preset() {
    for options in [
        ParserOptions::default(),
        ParserOptions::commonmark(),
        ParserOptions::gfm(),
        ParserOptions::gfm_spec(),
        ParserOptions::mdx(),
    ] {
        assert!(!options.highlight && !options.inline_footnotes && options.allow_link_refs);
        assert_eq!(
            render("==text== ^[note]", options),
            "<p>==text== ^[note]</p>\n"
        );
    }
}
#[test]
fn highlight_respects_inline_boundaries_and_flanking() {
    for (source, expected) in [
        ("==important==", "<mark>important</mark>"),
        (
            "==a **bold** and *soft*==",
            "<mark>a <strong>bold</strong> and <em>soft</em></mark>",
        ),
        (
            "**==bold mark==**",
            "<strong><mark>bold mark</mark></strong>",
        ),
        ("==[a](url)==", "<mark><a href=\"url\">a</a></mark>"),
        ("[==a==](url)", "<a href=\"url\"><mark>a</mark></a>"),
        ("==`a==b`==", "<mark><code>a==b</code></mark>"),
        ("==a [b](x==y)==", "<mark>a <a href=\"x==y\">b</a></mark>"),
        (
            "==a <i title=\"==\">b</i>==",
            "<mark>a <i title=\"==\">b</i></mark>",
        ),
        ("==空 白==", "<mark>空 白</mark>"),
        ("x==yes==y", "x<mark>yes</mark>y"),
        ("== x==", "== x=="),
        ("==x ==", "==x =="),
        ("===x===", "===x==="),
        (r"\==x==", "==x=="),
        ("`==x==`", "<code>==x==</code>"),
        ("==a\nb==", "<mark>a\nb</mark>"),
    ] {
        assert_eq!(
            render(source, options()),
            format!("<p>{expected}</p>\n"),
            "{source}"
        );
    }
}
#[test]
fn marked_text_participates_in_autolinks_and_heading_metadata() {
    let source = "# ==Heading==\n\n==https://example.org==";
    let arena = Allocator::new();
    let doc = Parser::with_options(&arena, source, options())
        .parse()
        .unwrap();
    let html = HtmlRenderer::with_options(HtmlRendererOptions {
        heading_ids: true,
        ..HtmlRendererOptions::commonmark()
    })
    .render(&doc);
    assert!(html.contains("id=\"heading\""), "{html}");
    assert!(
        html.contains("<mark><a href=\"https://example.org\">https://example.org</a></mark>"),
        "{html}"
    );
}
#[test]
fn inline_note_uses_existing_footnote_nodes_and_deferred_definition() {
    let html = render("Text^[a *note* with [link](/url)] after.", options());
    assert!(
        html.starts_with("<p>Text<sup><a href=\"#fn-1\" id=\"fnref-1\">1</a></sup> after.</p>\n"),
        "{html}"
    );
    assert!(
        html.contains("<p>a <em>note</em> with <a href=\"/url\">link</a></p>"),
        "{html}"
    );
}
#[test]
fn inline_notes_work_without_reference_footnotes_or_link_references() {
    let html = render(
        "A^[note] [link](/url)",
        ParserOptions {
            footnotes: false,
            allow_link_refs: false,
            ..options()
        },
    );
    assert!(html.contains("<p>note</p>"), "{html}");
    assert!(html.contains("<a href=\"/url\">link</a>"), "{html}");
}
#[test]
fn mixed_notes_avoid_identifiers_and_share_semantic_numbering() {
    let arena = Allocator::new();
    let doc = Parser::with_options(
        &arena,
        "A^[inline] B[^1] C^[last]\n\n[^1]: named",
        options(),
    )
    .parse()
    .unwrap();
    let html = HtmlRenderer::with_options(HtmlRendererOptions {
        semantic_footnotes: true,
        ..HtmlRendererOptions::commonmark()
    })
    .render(&doc);
    assert!(html.contains("id=\"fnref-2\">1</a>"), "{html}");
    assert!(html.contains("id=\"fnref-1\">2</a>"), "{html}");
    assert!(html.contains("id=\"fnref-3\">3</a>"), "{html}");
    assert_eq!(html.matches("<li id=").count(), 3);
}
#[test]
fn notes_in_containers_have_document_wide_identifiers() {
    let html = render(
        "> Quote^[first]\n\n- List^[second]\n\nText^[third]",
        options(),
    );
    for id in 1..=3 {
        assert_eq!(
            html.matches(&format!("id=\"fn-{id}\"")).count(),
            1,
            "{html}"
        );
    }
    assert!(html.rfind("class=\"footnote\"").unwrap() > html.find("</ul>").unwrap());
}
#[test]
fn speculative_links_and_images_do_not_leave_phantom_notes() {
    let html = render(
        "[text^[note]](/outer) ![alt^[hidden]](/img) after^[real]",
        options(),
    );
    assert!(!html.contains("href=\"/outer\""), "{html}");
    assert_eq!(html.matches("class=\"footnote\"").count(), 2, "{html}");
    assert!(!html.contains("<p>hidden</p>"), "{html}");
}
#[test]
fn notes_respect_escapes_code_balanced_brackets_and_superscript() {
    let html = render(
        r"\^[literal] `^[code]` A^[a \] and `]` and [nested]] x^2^",
        ParserOptions {
            superscript: true,
            ..options()
        },
    );
    assert!(html.contains("^[literal] <code>^[code]</code>"), "{html}");
    assert!(
        html.contains("a ] and <code>]</code> and [nested]"),
        "{html}"
    );
    assert!(html.contains("x<sup>2</sup>"), "{html}");
}
#[test]
fn malformed_empty_and_nested_notes_have_bounded_literal_fallback() {
    for source in ["^[]", "^[missing", "^[first\n\nsecond]"] {
        assert!(!render(source, options()).contains("class=\"footnote\""));
    }
    let html = render("A^[outer ^[inner]]", options());
    assert_eq!(html.matches("class=\"footnote\"").count(), 1, "{html}");
    assert!(html.contains("outer ^[inner]"), "{html}");
    let unclosed = "^[".repeat(5000);
    assert!(!render(&unclosed, options()).contains("class=\"footnote\""));
}
#[test]
fn no_op_hooks_and_renderer_reuse_preserve_both_extensions() {
    let arena = Allocator::new();
    let doc = Parser::with_options(&arena, "==Text==^[==note==]", options())
        .parse()
        .unwrap();
    for semantic in [false, true] {
        let mut renderer = HtmlRenderer::with_options(HtmlRendererOptions {
            semantic_footnotes: semantic,
            ..HtmlRendererOptions::commonmark()
        });
        let expected = renderer.render(&doc);
        assert_eq!(
            renderer.render_with_hooks(&doc, &mut NoHtmlRenderHooks),
            expected
        );
        assert_eq!(renderer.render(&doc), expected);
    }
}
#[test]
fn reference_toggle_preserves_definitions_and_inline_links() {
    let source = "[full][key] [key][] [key] ![img][key]\n\n[key]: /target \"title\"\n\n[inline](/ok) ![image](/img)";
    let html = render(
        source,
        ParserOptions {
            allow_link_refs: false,
            ..options()
        },
    );
    assert!(
        html.contains("[full][key] [key][] [key] ![img][key]"),
        "{html}"
    );
    assert!(html.contains("[key]: /target &quot;title&quot;"), "{html}");
    assert!(html.contains("<a href=\"/ok\">inline</a>"), "{html}");
    assert!(html.contains("<img src=\"/img\" alt=\"image\""), "{html}");
    assert!(!html.contains("href=\"/target\""), "{html}");
}
#[test]
fn reference_toggle_keeps_footnotes_wiki_links_and_nested_contexts() {
    let source = "> [key] and [^n]\n\n[key]: /target\n\n[^n]: footnote\n\n[[wiki]]";
    let html = render(
        source,
        ParserOptions {
            allow_link_refs: false,
            wiki_links: true,
            ..options()
        },
    );
    assert!(html.contains("[key]"), "{html}");
    assert!(html.contains("href=\"#fn-n\""), "{html}");
    assert!(html.contains("href=\"wiki\""), "{html}");
}
#[test]
fn visitors_and_spans_cover_marks_and_lifted_notes() {
    struct Check<'s> {
        source: &'s str,
        highlights: usize,
        definitions: usize,
    }
    impl<'a> Visit<'a> for Check<'_> {
        fn visit_node(&mut self, node: &Node<'a>) {
            let span = node.span();
            assert!(
                self.source
                    .get(span.start as usize..span.end as usize)
                    .is_some(),
                "{node:?}"
            );
            if let Node::Highlight(_) = node {
                self.highlights += 1;
                assert!(self.source[span.start as usize..span.end as usize].starts_with("=="));
            }
            if let Node::FootnoteDefinition(_) = node {
                self.definitions += 1;
                assert!(
                    self.source[span.start as usize..span.end as usize].starts_with("^["),
                    "source={:?}, node={:?}",
                    self.source,
                    node
                );
            }
            ferromark::ast::walk_node(self, node);
        }
    }
    for source in [
        "\u{feff}> ==é==^[==中==]\r\n",
        "- ==x==^[a\0b]\r\n",
        "| a |\n|---|\n| ==a\\|b==^[n\\|x] |\n",
    ] {
        let arena = Allocator::new();
        let doc = Parser::with_options(&arena, source, options())
            .parse()
            .unwrap();
        let mut check = Check {
            source,
            highlights: 0,
            definitions: 0,
        };
        check.visit_document(&doc);
        assert!(check.highlights > 0);
        assert_eq!(check.definitions, 1);
    }
}

#[test]
fn notes_are_lifted_from_inline_definition_list_table_caption_and_mdx_containers() {
    for source in [
        "**strong^[note]**",
        "*emphasis^[note]*",
        "~~deleted^[note]~~",
        "==marked^[note]==",
        "~subscript^[note]~",
        "Term^[note]\n: Definition",
        "Term\n: Definition^[note]",
        "| Column |\n|---|\n| cell |\n: Caption^[note] {#table}",
        "<Box>Text^[note]</Box>",
        "Text <Span>inline^[note]</Span>",
    ] {
        let html = render(
            source,
            ParserOptions {
                definition_lists: true,
                table_attributes: true,
                subscript: true,
                mdx: true,
                ..options()
            },
        );
        assert_eq!(
            html.matches("class=\"footnote\"").count(),
            1,
            "{source}: {html}"
        );
        assert!(html.contains("<p>note</p>"), "{source}: {html}");
        assert!(!html.contains("^[note]"), "{source}: {html}");
    }
}

#[test]
fn inline_identifiers_do_not_collide_with_nested_unreferenced_definitions() {
    let html = render(
        "> [^1]: quoted definition\n\n- Item\n\n  [^2]: list definition\n\nText^[inline]",
        options(),
    );
    for id in 1..=3 {
        assert_eq!(
            html.matches(&format!("id=\"fn-{id}\"")).count(),
            1,
            "{html}"
        );
    }
    assert!(html.contains("href=\"#fn-3\""), "{html}");
}
