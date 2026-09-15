use ferromark_allocator::Allocator;
use ferromark_ast::Node;
use ferromark_parser::{Parser, ParserOptions};
use ferromark_renderer::{
    HtmlRenderContext, HtmlRenderControl, HtmlRenderHooks, HtmlRenderer, HtmlRendererOptions,
    NoHtmlRenderHooks,
};

fn render(
    source: &str,
    parser_options: ParserOptions,
    renderer_options: HtmlRendererOptions,
) -> String {
    let allocator = Allocator::new();
    let document = Parser::with_options(&allocator, source, parser_options)
        .parse()
        .unwrap();
    let mut renderer = HtmlRenderer::with_options(renderer_options);
    renderer.render(&document)
}

#[test]
fn table_attributes_caption_and_colgroup_render_in_order() {
    let html = render(
        "| item | price |\n| :--- | ---: |\n| tea | 2 |\n: Prices *today* {#prices .wide}",
        ParserOptions {
            tables: true,
            table_attributes: true,
            ..ParserOptions::default()
        },
        HtmlRendererOptions {
            table_colgroup: true,
            ..HtmlRendererOptions::default()
        },
    );

    assert_eq!(
        html,
        concat!(
            "<table id=\"prices\" class=\"wide\">\n",
            "<caption>Prices <em>today</em></caption>\n",
            "<colgroup>\n",
            "<col class=\"col-1\">\n",
            "<col class=\"col-2\">\n",
            "</colgroup>\n",
            "<thead>\n",
            "<tr>\n",
            "<th align=\"left\">item</th>\n",
            "<th align=\"right\">price</th>\n",
            "</tr>\n",
            "</thead>\n",
            "<tbody>\n",
            "<tr>\n",
            "<td align=\"left\">tea</td>\n",
            "<td align=\"right\">2</td>\n",
            "</tr>\n",
            "</tbody>\n",
            "</table>\n",
        )
    );
}

#[test]
fn merged_cells_emit_colspan_and_align_the_following_cell_logically() {
    let html = render(
        "| first || third |\n| :--- | --- | ---: |\n| one || three |",
        ParserOptions {
            tables: true,
            merged_table_cells: true,
            ..ParserOptions::default()
        },
        HtmlRendererOptions::default(),
    );

    assert!(
        html.contains("<th align=\"left\" colspan=\"2\">first</th>"),
        "{html}"
    );
    assert!(html.contains("<th align=\"right\">third</th>"), "{html}");
    assert!(
        html.contains("<td align=\"left\" colspan=\"2\">one</td>"),
        "{html}"
    );
    assert!(html.contains("<td align=\"right\">three</td>"), "{html}");
}

#[test]
fn xhtml_colgroup_uses_self_closing_columns() {
    let html = render(
        "| a | b |\n| --- | --- |\n| c | d |",
        ParserOptions::gfm(),
        HtmlRendererOptions {
            table_colgroup: true,
            xhtml: true,
            ..Default::default()
        },
    );

    assert!(
        html.contains("<col class=\"col-1\" />\n<col class=\"col-2\" />"),
        "{html}"
    );
}

#[test]
fn hooks_traverse_caption_children_and_match_default_output() {
    let source = "| item |\n| --- |\n| value |\n: Caption **bold** {#prices}";
    let parser_options = ParserOptions {
        tables: true,
        table_attributes: true,
        ..ParserOptions::default()
    };
    let renderer_options = HtmlRendererOptions {
        table_colgroup: true,
        ..Default::default()
    };
    let allocator = Allocator::new();
    let document = Parser::with_options(&allocator, source, parser_options)
        .parse()
        .unwrap();

    let mut default_renderer = HtmlRenderer::with_options(renderer_options.clone());
    let expected = default_renderer.render(&document);
    let mut hooked_renderer = HtmlRenderer::with_options(renderer_options);
    let mut hooks = CaptionTextHook { text_nodes: 0 };
    let actual = hooked_renderer.render_with_hooks(&document, &mut hooks);
    assert_eq!(actual, expected);
    assert_eq!(
        hooks.text_nodes, 4,
        "caption and cell text must traverse hooks"
    );

    let mut no_hooks_renderer = HtmlRenderer::new();
    let mut no_hooks = NoHtmlRenderHooks;
    assert_eq!(
        no_hooks_renderer.render_with_hooks(&document, &mut no_hooks),
        HtmlRenderer::with_options(HtmlRendererOptions::default()).render(&document)
    );
}

struct CaptionTextHook {
    text_nodes: usize,
}

#[test]
fn combined_extensions_preserve_logical_columns_with_or_without_hooks() {
    let source = "| Group ||\n| :--- | ---: |\n| joined ||\n\n: {#prices .wide .compact}";
    let allocator = Allocator::new();
    let document = Parser::with_options(
        &allocator,
        source,
        ParserOptions {
            tables: true,
            merged_table_cells: true,
            table_attributes: true,
            ..ParserOptions::default()
        },
    )
    .parse()
    .unwrap();
    let options = HtmlRendererOptions {
        table_colgroup: true,
        ..HtmlRendererOptions::gfm()
    };
    let expected = concat!(
        "<table id=\"prices\" class=\"wide compact\">\n",
        "<colgroup>\n<col class=\"col-1\">\n<col class=\"col-2\">\n</colgroup>\n",
        "<thead>\n<tr>\n<th align=\"left\" colspan=\"2\">Group</th>\n</tr>\n</thead>\n",
        "<tbody>\n<tr>\n<td align=\"left\" colspan=\"2\">joined</td>\n</tr>\n</tbody>\n</table>\n",
    );
    assert_eq!(
        HtmlRenderer::with_options(options.clone()).render(&document),
        expected
    );
    assert_eq!(
        HtmlRenderer::with_options(options).render_with_hooks(&document, &mut NoHtmlRenderHooks),
        expected
    );
}

#[test]
fn table_attributes_escape_values_from_a_transformed_ast() {
    let allocator = Allocator::new();
    let mut document = Parser::with_options(
        &allocator,
        "| A |\n| --- |\n: {#id}",
        ParserOptions {
            tables: true,
            table_attributes: true,
            ..ParserOptions::default()
        },
    )
    .parse()
    .unwrap();
    let Node::Table(table) = &mut document.children[0] else {
        panic!("expected table");
    };
    let attributes = table.attributes.as_mut().unwrap();
    attributes.id = Some("id\"<&");
    attributes.classes.push("class\"<&");
    let html = HtmlRenderer::new().render(&document);
    assert!(
        html.starts_with("<table id=\"id&quot;&lt;&amp;\" class=\"class&quot;&lt;&amp;\">\n"),
        "{html}"
    );
    assert_eq!(
        HtmlRenderer::new().render_with_hooks(&document, &mut NoHtmlRenderHooks),
        html
    );
}

#[test]
fn gfm_presets_keep_plain_table_markup() {
    let html = render(
        "| a | b |\n| --- | --- |\n| value ||",
        ParserOptions::gfm_spec(),
        HtmlRendererOptions::gfm(),
    );
    assert_eq!(
        html,
        concat!(
            "<table>\n<thead>\n<tr>\n<th>a</th>\n<th>b</th>\n</tr>\n</thead>\n",
            "<tbody>\n<tr>\n<td>value</td>\n<td></td>\n</tr>\n</tbody>\n</table>\n",
        )
    );
    for preset in [
        HtmlRendererOptions::default(),
        HtmlRendererOptions::commonmark(),
        HtmlRendererOptions::gfm(),
    ] {
        assert!(!preset.table_colgroup);
    }
}

impl HtmlRenderHooks for CaptionTextHook {
    fn render_node(
        &mut self,
        node: &Node<'_>,
        _cx: &mut HtmlRenderContext<'_>,
    ) -> HtmlRenderControl {
        if matches!(node, Node::Text(_)) {
            self.text_nodes += 1;
        }
        HtmlRenderControl::Default
    }
}
