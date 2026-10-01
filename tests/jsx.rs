use ferromark::ast::Document;
use ferromark::renderer::{JsxCodeBlockInput, JsxRenderHooks, JsxRenderer, JsxRendererOptions};
use ferromark::{Allocator, Parser, ParserOptions};

fn parse<'a>(allocator: &'a Allocator, source: &'a str, options: ParserOptions) -> Document<'a> {
    Parser::with_options(allocator, source, options)
        .parse()
        .expect("fixture should parse")
}

struct CodeHook;

impl JsxRenderHooks for CodeHook {
    fn render_code_block(&mut self, input: JsxCodeBlockInput<'_>) -> Option<String> {
        assert_eq!(input.code, "graph TD\n");
        assert_eq!(input.language, Some("mermaid"));
        assert_eq!(input.meta, Some("title=flow"));
        Some("<CodePreview />".into())
    }
}

#[test]
fn renders_markdown_directly_as_wrapped_jsx_and_reports_intrinsics() {
    let source = "# Hello\n\nUse {value} &amp; < 3.\n";
    let allocator = Allocator::new();
    let document = parse(&allocator, source, ParserOptions::gfm());
    let output = JsxRenderer::new().render(&document, source);

    assert_eq!(
        output.body,
        "<>\n<h1 id={\"hello\"}>{\"Hello\"}</h1>\n<p>{\"Use {value} \"}{\"&\"}{\" \"}{\"<\"}{\" 3.\"}</p>\n</>"
    );
    assert_eq!(output.elements, ["h1", "p"]);
    assert!(output.components.is_empty());
    assert_eq!(output.headings[0].text, "Hello");
    assert_eq!(output.headings[0].id.as_deref(), Some("hello"));
    assert!(output.mappings.iter().any(|mapping| {
        mapping.generated_line == 1
            && mapping.generated_column == 0
            && mapping.source_line == 0
            && mapping.source_column == 0
    }));
}

#[test]
fn prefixes_markdown_intrinsics_without_changing_authored_component_names() {
    let source = "Hello <Card><b>there</b></Card>.\n";
    let allocator = Allocator::new();
    let document = parse(&allocator, source, ParserOptions::mdx());
    let options = JsxRendererOptions {
        component_prefix: Some("_components".into()),
        ..JsxRendererOptions::default()
    };
    let output = JsxRenderer::with_options(options).render(&document, source);

    assert!(output.body.contains("<_components.p>"), "{}", output.body);
    assert!(output.body.contains("</_components.p>"), "{}", output.body);
    assert!(output.body.contains("<Card>"), "{}", output.body);
    assert!(output.body.contains("<b>"), "{}", output.body);
    assert_eq!(output.components, ["Card"]);
    assert_eq!(output.elements, ["p"]);
}

#[test]
fn extracts_esm_and_preserves_authored_jsx_attribute_source() {
    let source = concat!(
        "import { Chart } from './Chart'\n",
        "export const meta = { title: 'Charts' }\n\n",
        "# <Badge title=\"a &amp; b\" value={chart} {...props}>Chart</Badge>\n"
    );
    let allocator = Allocator::new();
    let document = parse(&allocator, source, ParserOptions::mdx());
    let output = JsxRenderer::new().render(&document, source);

    assert_eq!(output.esm.len(), 2);
    assert_eq!(output.esm[0].value, "import { Chart } from './Chart'");
    assert_eq!(
        output.esm[1].value,
        "export const meta = { title: 'Charts' }"
    );
    assert!(!output.body.contains("import { Chart }"));
    assert!(
        output.body.contains("title=\"a &amp; b\""),
        "{}",
        output.body
    );
    assert!(output.body.contains("value={chart}"), "{}", output.body);
    assert!(output.body.contains("{...props}"), "{}", output.body);
    assert!(
        output.body.contains(" value={chart} {...props}"),
        "{}",
        output.body
    );
    assert_eq!(output.components, ["Badge"]);
    assert_eq!(output.headings[0].text, "Chart");
    assert_eq!(output.headings[0].id.as_deref(), Some("chart"));
}

#[test]
fn normalizes_raw_html_for_jsx_without_rewriting_it_as_markdown() {
    let source = concat!(
        "<div class=\"x\" style=\"color: red; margin-top: 2px\">",
        "<br><span>raw &amp; {value}</span><!--safe--></div>\n"
    );
    let allocator = Allocator::new();
    let document = parse(&allocator, source, ParserOptions::gfm());
    let options = JsxRendererOptions {
        component_prefix: Some("_components".into()),
        ..JsxRendererOptions::default()
    };
    let output = JsxRenderer::with_options(options).render(&document, source);

    assert!(
        output.body.contains("<div className=\"x\""),
        "{}",
        output.body
    );
    assert!(
        output
            .body
            .contains("style={{color: \"red\", marginTop: \"2px\"}}"),
        "{}",
        output.body
    );
    assert!(output.body.contains("<br />"), "{}", output.body);
    assert!(
        output.body.contains("{\"raw & {value}\"}"),
        "{}",
        output.body
    );
    assert!(output.body.contains("{/*safe*/}"), "{}", output.body);
    assert!(output.body.contains("</div>"), "{}", output.body);
    assert!(output.elements.is_empty());
}

#[test]
fn renders_callouts_gfm_tables_tasks_and_footnotes() {
    let source = concat!(
        "> [!NOTE] Read this\n> More detail.\n\n",
        "| Name | Value |\n| --- | ---: |\n| A | 2 |\n\n",
        "- [x] done\n\n",
        "See[^one].\n\n[^one]: Footnote body.\n"
    );
    let allocator = Allocator::new();
    let document = parse(&allocator, source, ParserOptions::gfm());
    let output = JsxRenderer::new().render(&document, source);

    assert!(output.body.contains("ox-callout ox-callout--note"));
    assert!(output.body.contains("ox-callout-title"));
    assert!(output.body.contains("<table>"));
    assert!(output.body.contains("<thead>"));
    assert!(output.body.contains("<tbody>"));
    assert!(output.body.contains("align={\"right\"}"));
    assert!(output.body.contains("type={\"checkbox\"}"));
    assert!(output.body.contains("checked={true}"));
    assert!(output.body.contains("href={\"#fn-one\"}"));
    assert!(output.body.contains("id={\"fn-one\"}"));
    assert!(
        output.body.contains("<sup><a href={\"#fn-one\"}"),
        "{}",
        output.body
    );
    assert!(output.body.contains("</a></sup>"), "{}", output.body);

    let prefixed = JsxRenderer::with_options(JsxRendererOptions {
        component_prefix: Some("_components".into()),
        ..JsxRendererOptions::default()
    })
    .render(&document, source);
    assert!(
        prefixed
            .body
            .contains("<_components.sup><_components.a href={\"#fn-one\"}"),
        "{}",
        prefixed.body
    );
    assert!(
        prefixed.body.contains("</_components.a></_components.sup>"),
        "{}",
        prefixed.body
    );

    let no_callouts = JsxRenderer::with_options(JsxRendererOptions {
        callouts: false,
        ..JsxRendererOptions::default()
    })
    .render(&document, source);
    // The parser can split the marker across adjacent text nodes; React renders
    // those string children consecutively in the ordinary block quote path.
    assert!(
        no_callouts
            .body
            .contains("{\"[\"}{\"!\"}{\"NOTE] Read this"),
        "{}",
        no_callouts.body
    );
    assert!(!no_callouts.body.contains("ox-callout"));
}

#[test]
fn code_component_mapping_and_hook_report_code_descriptors() {
    let source = "```mermaid title=flow\ngraph TD\n```\n";
    let allocator = Allocator::new();
    let document = parse(&allocator, source, ParserOptions::gfm());
    let options = JsxRendererOptions::default().with_code_block_component("mermaid", "Mermaid");
    let output = JsxRenderer::with_options(options).render(&document, source);

    assert!(output.body.contains("<Mermaid language={\"mermaid\"}"));
    assert!(output.body.contains("title=flow"));
    assert_eq!(output.components, ["Mermaid"]);
    assert_eq!(output.code_blocks.len(), 1);
    assert_eq!(output.code_blocks[0].language.as_deref(), Some("mermaid"));
    assert_eq!(output.code_blocks[0].meta.as_deref(), Some("title=flow"));
    assert_eq!(output.code_blocks[0].component.as_deref(), Some("Mermaid"));
    assert_eq!(output.code_blocks[0].value, "graph TD\n");

    let hooked = JsxRenderer::new().render_with_hooks(&document, source, &mut CodeHook);
    assert!(hooked.body.contains("<CodePreview />"));
    assert_eq!(hooked.code_blocks[0].component, None);
}

#[test]
fn heading_omission_is_exact_and_does_not_claim_an_id_or_disappear_from_metadata() {
    let source = "## Section\n\n# Title {#chosen}\n\n# Title\n";
    let allocator = Allocator::new();
    let document = parse(
        &allocator,
        source,
        ParserOptions {
            heading_attributes: true,
            ..ParserOptions::gfm()
        },
    );
    let options = JsxRendererOptions {
        omit_title_heading: Some(" Title ".into()),
        ..JsxRendererOptions::default()
    };
    let output = JsxRenderer::with_options(options).render(&document, source);

    assert_eq!(
        output
            .omitted_title_heading
            .map(|span| span.source_text(source)),
        Some("# Title {#chosen}\n")
    );
    assert_eq!(output.headings.len(), 2);
    assert_eq!(output.headings[0].text, "Section");
    assert_eq!(output.headings[1].id.as_deref(), Some("title"));
    assert!(!output.body.contains("chosen"));
    assert!(output.body.contains("id={\"title\"}"));

    let first_does_not_match = "# Other\n\n# Title\n";
    let allocator = Allocator::new();
    let document = parse(&allocator, first_does_not_match, ParserOptions::gfm());
    let output = JsxRenderer::with_options(JsxRendererOptions {
        omit_title_heading: Some("Title".into()),
        ..JsxRendererOptions::default()
    })
    .render(&document, first_does_not_match);
    assert_eq!(output.omitted_title_heading, None);
    assert_eq!(output.headings.len(), 2);
}

#[test]
fn source_mappings_count_utf16_and_each_line_of_preserved_expressions() {
    let source = "😀 {choose(\n  value)}\n";
    let allocator = Allocator::new();
    let document = parse(&allocator, source, ParserOptions::mdx());
    let output = JsxRenderer::new().render(&document, source);

    assert!(
        output.body.contains("{choose(\n  value)}"),
        "{}",
        output.body
    );
    assert!(output.mappings.iter().any(|mapping| {
        mapping.source_line == 0 && mapping.source_column == 4 && mapping.generated_line == 1
    }));
    assert!(output.mappings.iter().any(|mapping| {
        mapping.source_line == 1 && mapping.source_column == 0 && mapping.generated_line == 2
    }));
}

#[test]
fn authored_intrinsic_heading_markup_does_not_break_native_heading_rules() {
    let source = "# <span>a</span><strong>b</strong>\n";
    let allocator = Allocator::new();
    let document = parse(&allocator, source, ParserOptions::mdx());
    let output = JsxRenderer::new().render(&document, source);

    assert!(output.body.contains("<span>"), "{}", output.body);
    assert!(output.body.contains("<strong>"), "{}", output.body);
    assert_eq!(output.headings[0].text, "ab");
    assert_eq!(output.headings[0].id.as_deref(), Some("ab"));
    assert!(output.components.is_empty());
}
