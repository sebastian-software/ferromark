//! The Ferromark Flavored Markdown parser profile.

use ferromark::{HtmlRendererOptions, ParserOptions, to_html_with_options};

const SOURCE: &str = "![Pipeline](pipeline.svg){.diagram}

: The processing pipeline

> A memorable passage.
: Jane Doe

We ++added++ this ==today== with x^2^ and a note.^[Inline]

// A source-only note
[Styled]{.term lang=en} text.

Term
: Definition
";

fn render(options: ParserOptions) -> String {
    to_html_with_options(SOURCE, options, HtmlRendererOptions::default()).unwrap()
}

#[test]
fn ffm_extends_gfm_with_authoring_syntax() {
    let ffm = ParserOptions::ffm();
    let gfm = ParserOptions::gfm();
    for (name, enabled) in [
        ("highlight", ffm.highlight),
        ("inline_footnotes", ffm.inline_footnotes),
        ("merged_table_cells", ffm.merged_table_cells),
        ("table_attributes", ffm.table_attributes),
        ("image_attributes", ffm.image_attributes),
        ("image_captions", ffm.image_captions),
        ("blockquote_attributions", ffm.blockquote_attributions),
        ("line_comments", ffm.line_comments),
        ("insertions", ffm.insertions),
        ("superscript", ffm.superscript),
        ("definition_lists", ffm.definition_lists),
        ("heading_attributes", ffm.heading_attributes),
        ("extended_attributes", ffm.extended_attributes),
        ("bracketed_spans", ffm.bracketed_spans),
        ("guillemet_digraphs", ffm.guillemet_digraphs),
    ] {
        assert!(enabled, "{name} is part of FFM");
    }
    // GFM stays the base, and syntax that changes GFM input stays off.
    assert_eq!(ffm.tables, gfm.tables);
    assert_eq!(ffm.task_lists, gfm.task_lists);
    assert_eq!(ffm.strikethrough, gfm.strikethrough);
    assert_eq!(ffm.autolinks, gfm.autolinks);
    assert_eq!(ffm.footnotes, gfm.footnotes);
    assert_eq!(ffm.max_nesting_depth, gfm.max_nesting_depth);
    for (name, enabled) in [
        ("subscript", ffm.subscript),
        ("math", ffm.math),
        ("wiki_links", ffm.wiki_links),
        ("front_matter", ffm.front_matter),
        ("cjk_emphasis", ffm.cjk_emphasis),
        ("mdx", ffm.mdx),
    ] {
        assert!(!enabled, "{name} is not part of FFM");
    }
}

#[test]
fn ffm_renders_publishing_syntax() {
    let html = render(ParserOptions::ffm());
    for expected in [
        "<figcaption>The processing pipeline</figcaption>",
        "class=\"diagram\"",
        "<figcaption>Jane Doe</figcaption>",
        "<ins>added</ins>",
        "<mark>today</mark>",
        "<sup>2</sup>",
        "<span class=\"term\" lang=\"en\">Styled</span>",
        "<dt>Term</dt>",
    ] {
        assert!(html.contains(expected), "missing {expected}: {html}");
    }
    assert!(!html.contains("source-only note"), "{html}");
}

#[test]
fn gfm_leaves_ffm_syntax_literal() {
    let html = render(ParserOptions::gfm());
    for unexpected in ["<figure>", "<ins>", "<mark>", "<sup>", "<span", "<dl"] {
        assert!(
            !html.contains(unexpected),
            "unexpected {unexpected}: {html}"
        );
    }
    assert!(html.contains("source-only note"), "{html}");
}
