use ferromark::allocator::Allocator;
use ferromark::ast::Document;
use ferromark::parser::{Parser, ParserOptions};
use ferromark::renderer::{HtmlRenderer, HtmlRendererOptions};
use ferromark::{OutlineEntry, OutlineOptions};

fn parse<'a>(allocator: &'a Allocator, source: &'a str, options: ParserOptions) -> Document<'a> {
    Parser::with_options(allocator, source, options)
        .parse()
        .unwrap()
}

fn render(document: &Document<'_>, semantic_footnotes: bool, offset: i32, prefix: &str) -> String {
    let mut renderer = HtmlRenderer::with_options(HtmlRendererOptions {
        semantic_footnotes,
        ..HtmlRendererOptions::default()
    })
    .with_heading_level_offset(offset)
    .try_with_heading_id_prefix(prefix)
    .unwrap();
    renderer.render(document)
}

fn assert_entries_match_html(entries: &[OutlineEntry], html: &str) {
    for entry in entries {
        let id = entry.id.as_deref().expect("heading IDs are enabled");
        assert!(
            html.contains(&format!("<h{} id=\"{id}\"", entry.level)),
            "outline entry {entry:?} was not present in rendered HTML:\n{html}"
        );
    }
}

#[test]
fn outline_collects_unicode_text_levels_and_source_spans_in_containers() {
    let source = "> ## Café **東京**\n>\n> - ### Nested\n\n# Root\n";
    let allocator = Allocator::new();
    let document = parse(&allocator, source, ParserOptions::gfm());

    let entries = document.outline(&OutlineOptions::default().with_heading_level_offset(1));

    assert_eq!(entries.len(), 3);
    assert_eq!(entries[0].level, 3);
    assert_eq!(entries[0].text, "Café 東京");
    assert_eq!(entries[0].id.as_deref(), Some("café-東京"));
    assert_eq!(entries[1].level, 4);
    assert_eq!(entries[1].text, "Nested");
    assert_eq!(entries[2].level, 2);
    assert_eq!(entries[2].text, "Root");

    for (entry, expected_text) in entries.iter().zip(["Café **東京**", "Nested", "Root"]) {
        let source_slice = entry.span.source_text(source);
        assert!(source_slice.contains(expected_text), "{source_slice:?}");
        assert!(entry.span.start < entry.span.end);
    }
}

#[test]
fn offsets_clamp_before_filtering_and_filtered_headings_still_claim_ids() {
    let source = concat!(
        "# First {#duplicate}\n\n",
        "## Visible {#duplicate}\n\n",
        "###### Last\n",
    );
    let allocator = Allocator::new();
    let document = parse(
        &allocator,
        source,
        ParserOptions {
            heading_attributes: true,
            ..ParserOptions::gfm()
        },
    );

    let entries = document.outline(
        &OutlineOptions::default()
            .with_heading_level_offset(10)
            .with_level_filter(6..=6),
    );

    assert_eq!(entries.len(), 3);
    assert_eq!(entries[0].level, 6);
    assert_eq!(entries[0].id.as_deref(), Some("duplicate"));
    assert_eq!(entries[1].level, 6);
    assert_eq!(entries[1].id.as_deref(), Some("duplicate-1"));
    assert_eq!(entries[2].level, 6);
    assert_eq!(entries[2].id.as_deref(), Some("last"));

    let filtered = document.outline(&OutlineOptions::default().with_level_filter(2..=6));
    assert_eq!(filtered.len(), 2);
    assert_eq!(filtered[0].text, "Visible");
    assert_eq!(filtered[0].id.as_deref(), Some("duplicate-1"));
}

#[test]
fn explicit_duplicate_ids_unicode_slugs_and_prefixes_match_rendered_html() {
    let source = concat!(
        "# Café 東京\n\n",
        "## First {#custom}\n\n",
        "### Duplicate {#custom}\n\n",
        "# Café 東京\n",
    );
    let allocator = Allocator::new();
    let document = parse(
        &allocator,
        source,
        ParserOptions {
            heading_attributes: true,
            ..ParserOptions::gfm()
        },
    );
    let options = OutlineOptions::default()
        .with_heading_level_offset(1)
        .try_with_heading_id_prefix("docs-")
        .unwrap();
    let entries = document.outline(&options);
    let html = render(&document, false, 1, "docs-");

    assert_eq!(entries[0].id.as_deref(), Some("docs-café-東京"));
    assert_eq!(entries[1].id.as_deref(), Some("docs-custom"));
    assert_eq!(entries[2].id.as_deref(), Some("docs-custom-1"));
    assert_eq!(entries[3].id.as_deref(), Some("docs-café-東京-1"));
    assert_entries_match_html(&entries, &html);
}

#[test]
fn footnote_ids_share_the_outline_heading_namespace_in_both_modes() {
    let source = "A[^1] and B[^1].\n\n# 1\n\n# fnref-1\n\n[^1]: Shared.\n";
    let allocator = Allocator::new();
    let document = parse(&allocator, source, ParserOptions::gfm());

    for semantic_footnotes in [false, true] {
        let options = OutlineOptions::default()
            .with_semantic_footnotes(semantic_footnotes)
            .try_with_heading_id_prefix("fn-")
            .unwrap();
        let entries = document.outline(&options);
        let html = render(&document, semantic_footnotes, 0, "fn-");

        assert_eq!(entries[0].id.as_deref(), Some("fn-1-1"));
        assert_eq!(entries[1].id.as_deref(), Some("fn-fnref-1"));
        assert_entries_match_html(&entries, &html);
        assert!(html.contains("id=\"fn-1\""), "{html}");
        assert!(html.contains("id=\"fnref-1\""), "{html}");
    }
}

#[test]
fn semantic_slug_collisions_match_the_renderer() {
    let source = concat!(
        "A[^foo_bar] and B[^foo-bar].\n\n",
        "# fn-foo-bar\n\n",
        "# fn-foo-bar-2\n\n",
        "[^foo_bar]: First note.\n\n",
        "[^foo-bar]: Second note.\n",
    );
    let allocator = Allocator::new();
    let document = parse(&allocator, source, ParserOptions::gfm());
    let options = OutlineOptions::default().with_semantic_footnotes(true);
    let entries = document.outline(&options);
    let html = render(&document, true, 0, "");

    assert_eq!(entries.len(), 2, "{entries:?}\n{html}");
    assert_eq!(entries[0].id.as_deref(), Some("fn-foo-bar-1"));
    assert_eq!(
        entries[1].id.as_deref(),
        Some("fn-foo-bar-2-1"),
        "{entries:?}\n{html}"
    );
    assert_entries_match_html(&entries, &html);
    assert!(html.contains("id=\"fn-foo-bar\""), "{html}");
    assert!(html.contains("id=\"fn-foo-bar-2\""), "{html}");
}

#[test]
fn definition_claim_order_matches_html_before_later_headings_and_references() {
    let source = "[^1]: Shared.\n\n# fn-1\n\n# fnref-1\n\nUse[^1].\n";
    let allocator = Allocator::new();
    let document = parse(&allocator, source, ParserOptions::gfm());

    for semantic_footnotes in [false, true] {
        let options = OutlineOptions::default().with_semantic_footnotes(semantic_footnotes);
        let entries = document.outline(&options);
        let html = render(&document, semantic_footnotes, 0, "");

        assert_eq!(entries[0].id.as_deref(), Some("fn-1-1"));
        if semantic_footnotes {
            assert_eq!(entries[1].id.as_deref(), Some("fnref-1"));
        } else {
            assert_eq!(entries[1].id.as_deref(), Some("fnref-1-1"));
        }
        assert_entries_match_html(&entries, &html);
    }
}

#[test]
fn headings_inside_footnote_definitions_keep_renderer_order() {
    let source = "[^note]: Intro.\n\n    # fn-note\n\n# fn-note\n\nUse[^note].\n";
    let allocator = Allocator::new();
    let document = parse(&allocator, source, ParserOptions::gfm());

    for semantic_footnotes in [false, true] {
        let options = OutlineOptions::default().with_semantic_footnotes(semantic_footnotes);
        let entries = document.outline(&options);
        let html = render(&document, semantic_footnotes, 0, "");

        assert_eq!(entries.len(), 2, "{entries:?}\n{html}");
        assert_eq!(entries[0].text, "fn-note");
        assert_eq!(entries[0].id.as_deref(), Some("fn-note-1"));
        assert_eq!(entries[1].id.as_deref(), Some("fn-note-2"));
        assert_entries_match_html(&entries, &html);
    }
}

#[test]
fn disabled_heading_ids_return_none_and_outline_is_read_only() {
    let source = "# A heading\n";
    let allocator = Allocator::new();
    let document = parse(&allocator, source, ParserOptions::gfm());
    let before = render(&document, false, 0, "");

    let entries = document.outline(&OutlineOptions::default().with_heading_ids(false));
    let after = render(&document, false, 0, "");

    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].text, "A heading");
    assert_eq!(entries[0].id, None);
    assert!(entries[0].span.source_text(source).contains("A heading"));
    assert_eq!(before, after);
}

#[test]
fn heading_prefix_validation_matches_renderer_validation() {
    assert!(
        OutlineOptions::default()
            .try_with_heading_id_prefix("api-v2_")
            .is_ok()
    );
    assert!(
        OutlineOptions::default()
            .try_with_heading_id_prefix("bad prefix")
            .is_err()
    );
}
