//! End-to-end footnote behaviour (GFM extension).

#[path = "support/edge.rs"]
mod edge_support;

use edge_support::render;
use ferromark::parser::ParserOptions;
use ferromark::renderer::HtmlRendererOptions;

fn gfm(source: &str) -> String {
    render(source, ParserOptions::gfm(), HtmlRendererOptions::default())
}

/// A renderer with semantic footnotes off never writes the three containers
/// the semantic path owns, so its per-render reset skips them. Reusing such a
/// renderer across footnote-heavy documents — including ones that repeat a
/// slug, fall back to a positional slug, and have no footnotes at all — must
/// still match a renderer built fresh for each document, in both modes.
#[test]
fn reused_renderers_reset_footnote_state_in_both_modes() {
    use ferromark::allocator::Allocator;
    use ferromark::parser::Parser;
    use ferromark::renderer::HtmlRenderer;

    let sources = [
        "A[^1] and again[^1].\n\n[^1]: Shared note.\n",
        "Plain paragraph with no footnotes.\n",
        "X[^a] Y[^b] Z[^a]\n\n[^a]: First.\n\n[^b]: Second.\n",
        // Two identifiers that slugify to the same base, forcing the
        // uniquifier to hand out a suffix.
        "P[^one two] Q[^one-two]\n\n[^one two]: Spaced.\n\n[^one-two]: Hyphenated.\n",
        // An identifier with no alphanumerics falls back to a positional slug.
        "R[^!!!]\n\n[^!!!]: Symbolic.\n",
        "",
    ];

    let allocator = Allocator::new();
    let documents: Vec<_> = sources
        .iter()
        .map(|source| {
            Parser::with_options(&allocator, source, ParserOptions::gfm())
                .parse()
                .unwrap()
        })
        .collect();

    for semantic_footnotes in [false, true] {
        let options = HtmlRendererOptions {
            semantic_footnotes,
            ..HtmlRendererOptions::default()
        };
        let mut reused = HtmlRenderer::with_options(options.clone());
        // Rotating the start point means no ordering can leave stale state.
        for offset in 0..documents.len() {
            for step in 0..documents.len() {
                let document = &documents[(offset + step) % documents.len()];
                let fresh = HtmlRenderer::with_options(options.clone()).render(document);
                assert_eq!(
                    reused.render_borrowed(document),
                    fresh,
                    "semantic_footnotes={semantic_footnotes} offset={offset}"
                );
            }
        }
    }
}

#[test]
fn reference_and_definition_render_as_linked_pair() {
    let html = gfm("Here is a note[^1].\n\n[^1]: The note text.\n");

    assert!(
        html.contains("<sup><a href=\"#fn-1\" id=\"fnref-1\">1</a></sup>"),
        "{html}"
    );
    assert!(
        html.contains("<div id=\"fn-1\" class=\"footnote\">"),
        "{html}"
    );
    assert!(html.contains("<p>The note text.</p>"), "{html}");
    assert!(html.contains("<a href=\"#fnref-1\">↩</a>"), "{html}");
}

#[test]
fn definition_is_not_treated_as_a_link_reference() {
    // Regression: `[^1]: text` used to parse as a link reference
    // definition with label `^1`, turning every `[^1]` into a link
    // pointing at the definition text.
    let html = gfm("A[^1] and B[^1].\n\n[^1]: Shared.\n");

    assert!(!html.contains("href=\"Shared.\""), "{html}");
    assert!(html.contains("href=\"#fn-1\""), "{html}");
}

#[test]
fn repeated_references_get_unique_ids() {
    let html = gfm("A[^1] and B[^1] and C[^1].\n\n[^1]: Shared.\n");

    assert!(html.contains("id=\"fnref-1\""), "{html}");
    assert!(html.contains("id=\"fnref-1-2\""), "{html}");
    assert!(html.contains("id=\"fnref-1-3\""), "{html}");
}

#[test]
fn heading_ids_reserve_legacy_and_semantic_footnote_ids() {
    let source = "# fn-1\n\n# fnref-1\n\nA[^1] and B[^1].\n\n[^1]: Shared.\n";

    for semantic_footnotes in [false, true] {
        let allocator = ferromark::allocator::Allocator::new();
        let document =
            ferromark::parser::Parser::with_options(&allocator, source, ParserOptions::gfm())
                .parse()
                .unwrap();
        let mut renderer = ferromark::renderer::HtmlRenderer::with_options(HtmlRendererOptions {
            semantic_footnotes,
            heading_permalinks: true,
            ..HtmlRendererOptions::default()
        });
        let html = renderer.render(&document);

        assert!(html.contains("<h1 id=\"fn-1\">"), "{html}");
        assert!(html.contains("<h1 id=\"fnref-1\">"), "{html}");
        assert!(html.contains("href=\"#fn-1-1\""), "{html}");
        assert!(html.contains("id=\"fnref-1-1\""), "{html}");
        assert!(html.contains("id=\"fnref-1-2\""), "{html}");
        assert!(html.contains("href=\"#fnref-1-1\""), "{html}");
        assert!(html.contains("id=\"fn-1-1\""), "{html}");
    }
}

#[test]
fn footnote_ids_claimed_before_headings_keep_their_ids() {
    let source = "A[^1].\n\n[^1]: Shared.\n\n# fn-1\n";
    let html = gfm(source);

    assert!(html.contains("<div id=\"fn-1\""), "{html}");
    assert!(html.contains("<h1 id=\"fn-1-1\">"), "{html}");
}

#[test]
fn legacy_definition_reserves_its_later_reference_backlink_id() {
    let source = "[^1]: Shared.\n\n# fnref-1\n\nUse[^1].\n";
    let html = gfm(source);

    assert!(html.contains("<h1 id=\"fnref-1-1\">"), "{html}");
    assert!(html.contains("id=\"fnref-1\">"), "{html}");
    assert!(html.contains("<a href=\"#fnref-1\">↩</a>"), "{html}");

    let allocator = ferromark::allocator::Allocator::new();
    let document =
        ferromark::parser::Parser::with_options(&allocator, source, ParserOptions::gfm())
            .parse()
            .unwrap();
    let mut hooked_renderer = ferromark::renderer::HtmlRenderer::new();
    let hooked_html =
        hooked_renderer.render_with_hooks(&document, &mut ferromark::renderer::NoHtmlRenderHooks);
    assert_eq!(hooked_html, html);
}

#[test]
fn incremental_fragments_share_footnote_and_heading_ids() {
    use ferromark::allocator::Allocator;
    use ferromark::parser::Parser;
    use ferromark::renderer::HtmlRenderer;

    let first_source = "A[^1].\n\n[^1]: Shared.\n";
    let second_source = "# fn-1\n";
    let first_allocator = Allocator::new();
    let first = Parser::with_options(&first_allocator, first_source, ParserOptions::gfm())
        .parse()
        .unwrap();
    let second_allocator = Allocator::new();
    let second = Parser::with_options(&second_allocator, second_source, ParserOptions::gfm())
        .parse()
        .unwrap();
    let full_source = format!("{first_source}\n{second_source}");
    let full = gfm(&full_source);
    let mut renderer = HtmlRenderer::new();

    let first_html = renderer.render_incremental_fragment(&first);
    assert!(first_html.contains("<div id=\"fn-1\""), "{first_html}");
    let provisional = renderer.render_provisional_fragment(&second);
    assert!(provisional.contains("<h1 id=\"fn-1-1\">"), "{provisional}");
    let committed = renderer.render_incremental_fragment(&second);
    assert_eq!(committed, provisional);
    assert_eq!(format!("{first_html}{committed}"), full);
}

#[test]
fn prefixed_headings_reserve_footnote_ids_in_the_emitted_namespace() {
    let allocator = ferromark::allocator::Allocator::new();
    let document = ferromark::parser::Parser::with_options(
        &allocator,
        "# 1\n\nA[^1].\n\n[^1]: Shared.\n",
        ParserOptions::gfm(),
    )
    .parse()
    .unwrap();
    let mut renderer = ferromark::renderer::HtmlRenderer::with_options(HtmlRendererOptions {
        heading_permalinks: true,
        ..HtmlRendererOptions::default()
    })
    .try_with_heading_id_prefix("fn-")
    .unwrap();
    let html = renderer.render(&document);

    assert!(html.contains("<h1 id=\"fn-1\">"), "{html}");
    assert!(html.contains("href=\"#fn-1-1\""), "{html}");
    assert!(html.contains("<div id=\"fn-1-1\""), "{html}");
}

#[test]
fn undefined_reference_stays_literal_text() {
    let html = gfm("Missing[^nope].\n");

    assert_eq!(html, "<p>Missing[^nope].</p>\n");
}

#[test]
fn definition_body_takes_indented_continuation_blocks() {
    let html = gfm("A[^x].\n\n[^x]: First para.\n\n    Second para.\n");

    assert!(html.contains("<p>First para.</p>"), "{html}");
    assert!(html.contains("<p>Second para.</p>"), "{html}");
    // The continuation must land inside the footnote, not become code.
    assert!(!html.contains("<pre>"), "{html}");
}

#[test]
fn definition_body_supports_block_content() {
    let html = gfm("C[^l].\n\n[^l]: - item one\n    - item two\n");

    assert!(html.contains("<ul>"), "{html}");
    assert!(html.contains("<li>item one</li>"), "{html}");
    assert!(html.contains("<li>item two</li>"), "{html}");
}

#[test]
fn labels_match_case_insensitively() {
    let html = gfm("Ref[^Note].\n\n[^note]: Body.\n");

    assert!(html.contains("href=\"#fn-note\""), "{html}");
    assert!(html.contains("<p>Body.</p>"), "{html}");
}

#[test]
fn footnotes_stay_literal_without_the_extension() {
    // With footnotes disabled, `[^1]: url` is a valid CommonMark link
    // reference definition and `[^1]` a shortcut reference to it.
    let html = render(
        "A[^1].\n\n[^1]: /url\n",
        ParserOptions::default(),
        HtmlRendererOptions::default(),
    );

    assert!(html.contains("href=\"/url\""), "{html}");
    assert!(!html.contains("footnote"), "{html}");
}

#[test]
fn definition_inside_fenced_code_is_not_collected() {
    let html = gfm("```\n[^x]: not a definition\n```\n\nPlain[^x].\n");

    assert!(html.contains("<pre><code>[^x]: not a definition"), "{html}");
    assert!(html.contains("Plain[^x]."), "{html}");
    assert!(!html.contains("<sup>"), "{html}");
}

#[test]
fn nested_definitions_resolve_across_the_document() {
    for container in [
        "<Outer>[^note]: Nested body.\n\nLocal[^NOTE].\n</Outer>",
        "<Outer>\n\tLocal[^NOTE].\n\t\n\t[^note]: Nested body.\n</Outer>",
        "> Local[^NOTE].\n>\n> [^note]: Nested body.",
        "- Local[^NOTE].\n\n  [^note]: Nested body.",
    ] {
        for newline in ["\n", "\r\n", "\r"] {
            for allow_link_refs in [false, true] {
                let source = format!("Before[^note].\n\n{container}\n\nAfter[^note].")
                    .replace('\n', newline);
                let html = render(
                    &source,
                    ParserOptions {
                        mdx: true,
                        allow_link_refs,
                        ..ParserOptions::gfm()
                    },
                    HtmlRendererOptions::default(),
                );
                assert_eq!(
                    html.matches("href=\"#fn-note\"").count(),
                    3,
                    "{source:?}: {html}"
                );
                assert!(html.contains("<p>Nested body.</p>"), "{html}");
            }
        }
    }
}

#[test]
fn definition_lookalikes_in_non_markdown_blocks_stay_literal() {
    for block in [
        "<Outer>\n\t```\n\t[^note]: Hidden.\n\t```\n</Outer>",
        "> ```\n> [^note]: Hidden.\n> ```",
        "    [^note]: Hidden.",
        "<script>\n[^note]: Hidden.\n</script>",
    ] {
        let source = format!("Before[^note].\n\n{block}");
        let html = render(
            &source,
            ParserOptions {
                mdx: !block.starts_with("<script>"),
                ..ParserOptions::gfm()
            },
            HtmlRendererOptions::default(),
        );
        assert!(html.contains("Before[^note]."), "{html}");
        assert!(!html.contains("<sup>"), "{html}");
    }
}

#[test]
fn nested_footnote_bodies_contribute_labels_without_scanning_code() {
    let html = gfm(
        "Use[^INNER] and [^hidden].\n\n[^outer]: Outer body.\n\n    [^inner]: Inner body.\n\n    ```\n    [^hidden]: Code only.\n    ```\n",
    );
    assert!(html.contains("href=\"#fn-inner\""), "{html}");
    assert!(html.contains("<p>Inner body.</p>"), "{html}");
    assert!(html.contains("and [^hidden]."), "{html}");
    assert!(!html.contains("href=\"#fn-hidden\""), "{html}");
}
