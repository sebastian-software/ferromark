use ferromark::ast::Node;
use ferromark::{
    AbbreviationOptions, Allocator, CodeHighlightInput, HighlightedCodeBlock, HtmlCodeBlockInput,
    HtmlRenderContext, HtmlRenderControl, HtmlRenderHooks, HtmlRenderer, HtmlRendererOptions,
    JsxCodeBlockInput, JsxRenderHooks, JsxRenderer, Parser, ParserOptions,
};

#[derive(Default)]
struct Diagrams {
    calls: Vec<(String, Option<String>, Option<String>)>,
}

impl HtmlRenderHooks for Diagrams {
    fn render_node(&mut self, _: &Node<'_>, _: &mut HtmlRenderContext<'_>) -> HtmlRenderControl {
        panic!("code-only rendering must not invoke per-node callbacks");
    }

    fn render_code_block(&mut self, input: HtmlCodeBlockInput<'_>) -> Option<String> {
        self.calls.push((
            input.code.to_owned(),
            input.language.map(str::to_owned),
            input.meta.map(str::to_owned),
        ));
        (input.language == Some("mermaid")).then(|| "<svg data-diagram=\"yes\"></svg>\n".to_owned())
    }
}

struct Marks;

impl HtmlRenderHooks for Marks {
    fn highlight_code_block(
        &mut self,
        input: CodeHighlightInput<'_>,
    ) -> Option<HighlightedCodeBlock> {
        (input.language == Some("rust")).then(|| {
            HighlightedCodeBlock::new(
                input
                    .code
                    .split('\n')
                    .map(|line| format!("<mark>{line}</mark>"))
                    .collect(),
            )
        })
    }
}

#[test]
fn code_plugins_compose_with_highlighting_in_nested_containers() {
    let source = "> ```merm&#97;id title=\"Flow\"\n> A --> B // [!code ++]\n> ```\n\n- ```rust\n  let x = 1;\n  ```\n\n```unknown\n<safe>\n```\n";
    let arena = Allocator::new();
    let document = Parser::new(&arena, source).parse().unwrap();
    let mut hooks = (Diagrams::default(), Marks);
    let mut renderer = HtmlRenderer::with_options(HtmlRendererOptions {
        code_annotations: true,
        ..Default::default()
    });
    let output = renderer.render_with_code_hooks(&document, &mut hooks);
    assert!(
        output.contains("<blockquote>\n<svg data-diagram=\"yes\"></svg>\n</blockquote>"),
        "{output}"
    );
    assert!(output.contains("<mark>let x = 1;</mark>"), "{output}");
    assert!(output.contains("&lt;safe&gt;"), "{output}");
    assert_eq!(hooks.0.calls.len(), 3);
    assert_eq!(
        hooks.0.calls[0],
        (
            "A --> B // [!code ++]\n".to_owned(),
            Some("mermaid".to_owned()),
            Some("title=\"Flow\"".to_owned())
        )
    );
    assert_eq!(
        output,
        renderer.render_borrowed_with_code_hooks(&document, &mut hooks)
    );
}

#[derive(Default)]
struct Decline;
impl HtmlRenderHooks for Decline {}

#[test]
fn code_only_fallback_preserves_inline_features_and_renderer_reuse() {
    let sources = [
        "# A *heading*\n\nText **strong**, [link](https://example.com), `code` and &copy;.\n",
        "> [!NOTE]\n> A *callout* with https://example.com.\n\n- one **strong**\n- two\n",
        "| A | B |\n| - | - |\n| *one* | two |\n\nText[^n].\n\n[^n]: a **footnote**\n",
        "<Panel>\n\n```unknown\n<safe>\n```\n\nA **child**.\n</Panel>\n",
        "Text <Badge>**inline**</Badge> and [a *link*](https://example.com).\n",
        "> [!NOTE]\n> API **API** and `API`.\n",
    ];
    for source in sources {
        let arena = Allocator::new();
        let document = Parser::with_options(
            &arena,
            source,
            ParserOptions {
                mdx: true,
                footnotes: true,
                ..ParserOptions::default()
            },
        )
        .parse()
        .unwrap();
        let mut renderer = HtmlRenderer::new();
        let expected = renderer.render(&document);
        let mut hooks = Decline;
        assert_eq!(
            renderer.render_with_code_hooks(&document, &mut hooks),
            expected,
            "{source}"
        );
        assert_eq!(
            renderer.render_borrowed_with_code_hooks(&document, &mut hooks),
            expected,
            "{source}"
        );
        assert_eq!(
            renderer.render_with_hooks(&document, &mut hooks),
            expected,
            "{source}"
        );
        assert_eq!(
            renderer.render_borrowed_with_hooks(&document, &mut hooks),
            expected,
            "{source}"
        );
    }
}

struct Remove;
impl HtmlRenderHooks for Remove {
    fn render_code_block(&mut self, _: HtmlCodeBlockInput<'_>) -> Option<String> {
        Some(String::new())
    }
}
impl JsxRenderHooks for Remove {
    fn render_code_block(&mut self, _: JsxCodeBlockInput<'_>) -> Option<String> {
        Some(String::new())
    }
}
struct MustNotRun;
impl HtmlRenderHooks for MustNotRun {
    fn render_code_block(&mut self, _: HtmlCodeBlockInput<'_>) -> Option<String> {
        panic!("an empty replacement handles the block");
    }
    fn highlight_code_block(&mut self, _: CodeHighlightInput<'_>) -> Option<HighlightedCodeBlock> {
        panic!("whole-block replacement must precede highlighting");
    }
}
impl JsxRenderHooks for MustNotRun {
    fn render_code_block(&mut self, _: JsxCodeBlockInput<'_>) -> Option<String> {
        panic!("an empty replacement handles the block");
    }
}

#[test]
fn an_empty_replacement_stops_html_and_jsx_chains() {
    let arena = Allocator::new();
    let document = Parser::new(&arena, "```mermaid\nA --> B\n```\n")
        .parse()
        .unwrap();
    let mut hooks = (Remove, MustNotRun);
    assert_eq!(
        HtmlRenderer::new().render_with_code_hooks(&document, &mut hooks),
        ""
    );
    let jsx =
        JsxRenderer::new().render_with_hooks(&document, "```mermaid\nA --> B\n```\n", &mut hooks);
    assert!(!jsx.body.contains("A --"), "{}", jsx.body);
    assert_eq!(jsx.code_blocks.len(), 1);
}

#[derive(Default)]
struct CountNodes(usize);
impl HtmlRenderHooks for CountNodes {
    fn render_node(&mut self, _: &Node<'_>, _: &mut HtmlRenderContext<'_>) -> HtmlRenderControl {
        self.0 += 1;
        HtmlRenderControl::Default
    }
}

#[test]
fn code_only_mode_is_scoped_to_its_render_entry_point() {
    let source = "> [!NOTE]\n> API **HTTP** and <Badge>API</Badge>.\n\nAPI outside.\n";
    let arena = Allocator::new();
    let document = Parser::with_options(
        &arena,
        source,
        ParserOptions {
            mdx: true,
            ..ParserOptions::default()
        },
    )
    .parse()
    .unwrap();
    let mut renderer = HtmlRenderer::new().with_abbreviations(AbbreviationOptions::default());
    let expected = renderer.render(&document);
    let mut hooks = CountNodes::default();
    assert_eq!(
        renderer.render_with_code_hooks(&document, &mut hooks),
        expected
    );
    assert_eq!(hooks.0, 0);
    let _ = renderer.render_incremental_fragment_with_hooks(&document, &mut hooks);
    assert!(hooks.0 > 0);
    hooks.0 = 0;
    let _ = renderer.render_borrowed_with_code_hooks(&document, &mut hooks);
    let _ = renderer.render_provisional_fragment_with_hooks(&document, &mut hooks);
    assert!(hooks.0 > 0);
}
