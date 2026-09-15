use ferromark_allocator::Allocator;
use ferromark_parser::{Parser, ParserOptions};
use ferromark_renderer::{HtmlRenderer, HtmlRendererOptions, NoHtmlRenderHooks};

fn options() -> HtmlRendererOptions {
    HtmlRendererOptions {
        table_colgroup: true,
        table_column_names: true,
        ..HtmlRendererOptions::gfm()
    }
}

fn render(source: &str) -> String {
    let allocator = Allocator::new();
    let document = Parser::with_options(
        &allocator,
        source,
        ParserOptions {
            merged_table_cells: true,
            ..ParserOptions::gfm()
        },
    )
    .parse()
    .unwrap();
    HtmlRenderer::with_options(options()).render(&document)
}

fn column_classes(html: &str) -> Vec<&str> {
    html.split("<col class=\"")
        .skip(1)
        .map(|part| part.split('"').next().unwrap())
        .collect()
}

#[test]
fn column_names_use_heading_text_and_unicode_slug_rules() {
    let html = render(
        "| **Netto Preis** | Größe | 東京 | [Mehr Infos](/url) | `Order_ID` |\n| --- | --- | --- | --- | --- |\n| Other content | | | | |",
    );
    assert_eq!(
        column_classes(&html),
        [
            "col-1 col-name-netto-preis",
            "col-2 col-name-größe",
            "col-3 col-name-東京",
            "col-4 col-name-mehr-infos",
            "col-5 col-name-order-id",
        ]
    );
}

#[test]
fn blank_headers_fall_back_and_numeric_headers_cannot_alias_positions() {
    let html = render("| | !!! | 2 | Section |\n| --- | --- | --- | --- |");
    assert_eq!(
        column_classes(&html),
        [
            "col-1",
            "col-2",
            "col-3 col-name-2",
            "col-4 col-name-section"
        ]
    );
}

#[test]
fn duplicate_names_and_natural_suffixes_never_collide() {
    let html =
        render("| Price | Price | Price 1 | Price | PRICE |\n| --- | --- | --- | --- | --- |");
    assert_eq!(
        column_classes(&html),
        [
            "col-1 col-name-price",
            "col-2 col-name-price-1",
            "col-3 col-name-price-1-1",
            "col-4 col-name-price-2",
            "col-5 col-name-price-3",
        ]
    );
    let html = render("| Price 1 | Price | Price |\n| --- | --- | --- |");
    assert_eq!(
        column_classes(&html),
        [
            "col-1 col-name-price-1",
            "col-2 col-name-price",
            "col-3 col-name-price-2"
        ]
    );
}

#[test]
fn merged_headers_name_each_covered_logical_column() {
    let html = render("| Group || Group |\n| --- | --- | --- |\n| first | second | third |");
    assert_eq!(
        column_classes(&html),
        [
            "col-1 col-name-group",
            "col-2 col-name-group",
            "col-3 col-name-group-1"
        ]
    );
}

#[test]
fn naming_is_table_local_and_does_not_change_heading_ids_or_renderer_reuse() {
    let source = "# Price\n\n| Price | Price |\n| --- | --- |\n\n# Price\n\n| Price | Price |\n| --- | --- |";
    let allocator = Allocator::new();
    let document = Parser::with_options(&allocator, source, ParserOptions::gfm())
        .parse()
        .unwrap();
    let opts = HtmlRendererOptions {
        heading_ids: true,
        ..options()
    };
    let mut renderer = HtmlRenderer::with_options(opts);
    let html = renderer.render(&document);
    assert_eq!(
        column_classes(&html),
        [
            "col-1 col-name-price",
            "col-2 col-name-price-1",
            "col-1 col-name-price",
            "col-2 col-name-price-1",
        ]
    );
    assert!(html.contains("<h1 id=\"price\">Price</h1>"));
    assert!(html.contains("<h1 id=\"price-1\">Price</h1>"));
    assert_eq!(renderer.render_borrowed(&document), html);
    assert_eq!(
        renderer.render_with_hooks(&document, &mut NoHtmlRenderHooks),
        html
    );
}

#[test]
fn names_are_opt_in_and_require_colgroup() {
    let source = "| Price |\n| --- |";
    let allocator = Allocator::new();
    let document = Parser::with_options(&allocator, source, ParserOptions::gfm())
        .parse()
        .unwrap();
    let plain = HtmlRenderer::with_options(HtmlRendererOptions::gfm()).render(&document);
    let names_without_columns = HtmlRendererOptions {
        table_column_names: true,
        ..HtmlRendererOptions::gfm()
    };
    assert_eq!(
        HtmlRenderer::with_options(names_without_columns).render(&document),
        plain
    );
    let columns_only = HtmlRendererOptions {
        table_colgroup: true,
        ..HtmlRendererOptions::gfm()
    };
    assert_eq!(
        column_classes(&HtmlRenderer::with_options(columns_only).render(&document)),
        ["col-1"]
    );
    for preset in [
        HtmlRendererOptions::default(),
        HtmlRendererOptions::commonmark(),
        HtmlRendererOptions::gfm(),
    ] {
        assert!(!preset.table_column_names);
    }
}

#[test]
fn named_columns_keep_xhtml_and_hook_output_identical() {
    let source = "| A &amp; B | `\"quoted\"` |\n| --- | --- |";
    let allocator = Allocator::new();
    let document = Parser::with_options(&allocator, source, ParserOptions::gfm())
        .parse()
        .unwrap();
    let mut renderer = HtmlRenderer::with_options(HtmlRendererOptions {
        xhtml: true,
        ..options()
    });
    let html = renderer.render(&document);
    assert!(html.contains("<col class=\"col-1 col-name-a-b\" />"));
    assert!(html.contains("<col class=\"col-2 col-name-quoted\" />"));
    assert_eq!(
        renderer.render_with_hooks(&document, &mut NoHtmlRenderHooks),
        html
    );
}
