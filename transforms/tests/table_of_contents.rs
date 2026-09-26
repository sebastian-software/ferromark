#![allow(clippy::panic, clippy::unwrap_used)]

use ferromark::ast::{Node, Span};
use ferromark::{
    Allocator, HtmlRenderer, OutlineEntry, OutlineOptions, Parser, ParserOptions, parse,
};
use ferromark_transforms::build_table_of_contents;

#[test]
fn caller_places_a_nested_toc_using_the_outline_ids() {
    let source = "Before\n\n# Root\n\n#### Deep\n\n## Sibling\n";
    let allocator = Allocator::new();
    let mut document = parse(&allocator, source).unwrap();
    let outline = document.outline(&OutlineOptions::default());
    let toc = build_table_of_contents(&allocator, &outline)
        .unwrap()
        .unwrap();

    let Node::List(list) = &toc else {
        panic!("expected a list node");
    };
    assert_eq!(list.span, Span::empty());
    assert_eq!(list.children[0].span, Span::empty());
    document.children.insert(1, toc);

    let html = HtmlRenderer::new().render(&document);
    assert!(html.starts_with("<p>Before</p>\n<ul>\n"), "{html}");
    assert!(
        html.contains(
            "<li><a href=\"#root\">Root</a>\n<ul>\n<li><a href=\"#deep\">Deep</a></li>\n<li><a href=\"#sibling\">Sibling</a></li>\n</ul>\n</li>\n</ul>\n<h1 id=\"root\">Root</h1>"
        ),
        "{html}"
    );
}

#[test]
fn toc_preserves_prefixed_explicit_and_duplicate_heading_ids() {
    let source = concat!(
        "# Café 東京\n\n",
        "## Explicit {#custom}\n\n",
        "### Duplicate {#custom}\n\n",
        "# Café 東京\n",
    );
    let allocator = Allocator::new();
    let mut parser_options = ParserOptions::gfm();
    parser_options.heading_attributes = true;
    let mut document = Parser::with_options(&allocator, source, parser_options)
        .parse()
        .unwrap();
    let outline_options = OutlineOptions::default()
        .try_with_heading_id_prefix("docs-")
        .unwrap();
    let outline = document.outline(&outline_options);
    let toc = build_table_of_contents(&allocator, &outline)
        .unwrap()
        .unwrap();
    document.children.insert(0, toc);

    let html = HtmlRenderer::new()
        .try_with_heading_id_prefix("docs-")
        .unwrap()
        .render(&document);

    for (id, href) in [
        ("docs-café-東京", "docs-caf%C3%A9-%E6%9D%B1%E4%BA%AC"),
        ("docs-custom", "docs-custom"),
        ("docs-custom-1", "docs-custom-1"),
        ("docs-café-東京-1", "docs-caf%C3%A9-%E6%9D%B1%E4%BA%AC-1"),
    ] {
        assert!(html.contains(&format!("href=\"#{href}\"")), "{html}");
        assert!(html.contains(&format!("id=\"{id}\"")), "{html}");
    }
}

#[test]
fn empty_outline_returns_no_node() {
    let allocator = Allocator::new();
    let document = parse(&allocator, "No headings here.").unwrap();
    let outline = document.outline(&OutlineOptions::default());

    assert!(
        build_table_of_contents(&allocator, &outline)
            .unwrap()
            .is_none()
    );
}

#[test]
fn heading_ids_are_required_for_toc_links() {
    let allocator = Allocator::new();
    let entries = [OutlineEntry {
        level: 1,
        text: "Heading".to_owned(),
        id: None,
        span: Span::empty(),
    }];

    let error = build_table_of_contents(&allocator, &entries).unwrap_err();
    assert_eq!(error.entry_index(), 0);
    assert_eq!(
        error.to_string(),
        "outline entry 0 has no heading ID; enable heading IDs when building the outline"
    );
}
