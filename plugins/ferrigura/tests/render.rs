use ferromark::{Allocator, HtmlRenderer, Parser};
use ferromark_ferrigura::{
    Error, FerriguraHooks,
    ferrigura::{Renderer, Value},
};

fn hooks(prefix: &str) -> FerriguraHooks {
    FerriguraHooks::new(Renderer::new(&Value::empty_object()), prefix).unwrap()
}

#[test]
fn renders_parsed_mermaid_fences_and_keeps_other_code_and_trust_policy() {
    let source = "# Demo\n\n> ```merm&#97;id title=\"Flow\"\n> flowchart LR\n> A[Hello] --> B[World]\n> ```\n\n```rust\n<safe>\n```\n\n<script>unsafe</script>\n";
    let allocator = Allocator::new();
    let document = Parser::new(&allocator, source).parse().unwrap();
    let mut plugin = hooks("article");
    let options = ferromark::HtmlRendererOptions {
        sanitize: true,
        ..Default::default()
    };
    let html = HtmlRenderer::with_options(options).render_with_code_hooks(&document, &mut plugin);
    assert!(plugin.take_error().is_none());
    assert!(html.contains("<blockquote>\n<svg "));
    assert!(html.contains("id=\"article-1\""));
    assert!(html.contains("Hello") && html.contains("World"));
    assert!(html.contains("class=\"language-rust\""));
    assert!(html.contains("&lt;safe&gt;"));
    assert!(!html.contains("<script>"));
}

#[test]
fn repeated_diagrams_and_renders_receive_distinct_ids() {
    let source = "```mermaid\npie\n\"A\" : 1\n```\n\n```mermaid\npie\n\"A\" : 1\n```";
    let allocator = Allocator::new();
    let document = Parser::new(&allocator, source).parse().unwrap();
    let mut plugin = hooks("page");
    let mut renderer = HtmlRenderer::new();
    let first = renderer.render_with_code_hooks(&document, &mut plugin);
    let second = renderer.render_with_code_hooks(&document, &mut plugin);
    assert!(first.contains("id=\"page-1\"") && first.contains("id=\"page-2\""));
    assert!(second.contains("id=\"page-3\"") && second.contains("id=\"page-4\""));
    assert!(plugin.take_error().is_none());
}

#[test]
fn errors_are_observable_and_later_documents_can_render() {
    let source = "```mermaid\nnot a valid diagram\n```";
    let allocator = Allocator::new();
    let document = Parser::new(&allocator, source).parse().unwrap();
    let mut plugin = hooks("errors");
    let mut renderer = HtmlRenderer::new();
    let html = renderer.render_with_code_hooks(&document, &mut plugin);
    assert!(html.contains("not a valid diagram"));
    assert!(matches!(plugin.take_error(), Some(Error::Render(_))));
    let source = "```mermaid\npie\n\"A\" : 1\n```";
    let document = Parser::new(&allocator, source).parse().unwrap();
    assert!(
        renderer
            .render_with_code_hooks(&document, &mut plugin)
            .contains("<svg ")
    );
    assert!(plugin.take_error().is_none());
    assert!(matches!(
        FerriguraHooks::new(Renderer::new(&Value::empty_object()), "0bad"),
        Err(Error::InvalidIdPrefix)
    ));
}

#[cfg(feature = "jsx")]
#[test]
fn emits_intrinsic_jsx_and_a_valid_module_without_html_injection() {
    use ferromark::{JsxModuleOptions, JsxRenderer};
    let source = "```mermaid\nflowchart LR\nA[\"Hello {literal} & world\"] --> B[World]\n```";
    let allocator = Allocator::new();
    let document = Parser::new(&allocator, source).parse().unwrap();
    let mut plugin = hooks("jsx");
    let renderer = JsxRenderer::new();
    let result = renderer.render_with_hooks(&document, source, &mut plugin);
    assert!(plugin.take_error().is_none());
    assert!(result.body.contains("<svg ") && result.body.contains("viewBox="));
    assert!(result.body.contains("className=") && result.body.contains("style={{"));
    assert!(result.body.contains("{literal}"));
    assert!(!result.body.contains("dangerouslySetInnerHTML"));
    assert_eq!(result.code_blocks.len(), 1);
    let module = renderer
        .render_module_with_hooks(&document, source, &JsxModuleOptions::default(), &mut plugin)
        .unwrap();
    assert!(plugin.take_error().is_none());
    assert!(module.code.contains("<svg"));
}
