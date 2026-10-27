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
    let error = plugin.take_error();
    assert!(error.is_none(), "{error:?}");
    assert!(html.contains("<blockquote>\n<svg "));
    assert!(html.contains("id=\"article-10\""));
    assert!(html.contains("Hello") && html.contains("World"));
    assert!(html.contains("class=\"language-rust\""));
    assert!(html.contains("&lt;safe&gt;"));
    assert!(!html.contains("<script>"));
}

#[test]
fn repeated_renders_are_identical_and_diagrams_receive_distinct_ids() {
    let source = "```mermaid\npie\n\"A\" : 1\n```\n\n```mermaid\npie\n\"A\" : 1\n```";
    let allocator = Allocator::new();
    let document = Parser::new(&allocator, source).parse().unwrap();
    let mut plugin = hooks("page");
    let mut renderer = HtmlRenderer::new();
    let first = renderer.render_with_code_hooks(&document, &mut plugin);
    let second = renderer.render_with_code_hooks(&document, &mut plugin);
    assert_eq!(first, second);
    let wrapped = format!("<root>{first}</root>");
    let xml = roxmltree::Document::parse(&wrapped).unwrap();
    let ids: Vec<_> = xml
        .descendants()
        .filter_map(|node| node.attribute("id"))
        .collect();
    let unique: std::collections::HashSet<_> = ids.iter().collect();
    assert_eq!(unique.len(), ids.len(), "IDs collide: {ids:?}");
    assert!(first.contains("id=\"page-0\""));
    let error = plugin.take_error();
    assert!(error.is_none(), "{error:?}");
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
    let error = plugin.take_error();
    assert!(error.is_none(), "{error:?}");
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
    let error = plugin.take_error();
    assert!(error.is_none(), "{error:?}");
    assert!(result.body.contains("<svg ") && result.body.contains("viewBox="));
    assert!(result.body.contains("className=") && result.body.contains("style={{"));
    assert!(result.body.contains("{literal}"));
    assert!(!result.body.contains("dangerouslySetInnerHTML"));
    assert_eq!(result.code_blocks.len(), 1);
    let module = renderer
        .render_module_with_hooks(&document, source, &JsxModuleOptions::default(), &mut plugin)
        .unwrap();
    let error = plugin.take_error();
    assert!(error.is_none(), "{error:?}");
    assert!(module.code.contains("<svg"));
}

#[test]
fn fixture_bytes_are_pinned_across_processes_and_platforms() {
    use sha2::{Digest, Sha256};
    let fixtures: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/diagrams.json")).unwrap();
    let mut actual = Vec::new();
    let mut expected = Vec::new();
    for fixture in fixtures.as_array().unwrap() {
        let name = fixture["name"].as_str().unwrap();
        let source = format!("```mermaid\n{}\n```", fixture["source"].as_str().unwrap());
        let allocator = Allocator::new();
        let document = Parser::new(&allocator, &source).parse().unwrap();
        let mut plugin = hooks("fixture");
        let mut renderer = HtmlRenderer::new();
        let first = renderer.render_with_code_hooks(&document, &mut plugin);
        assert_eq!(plugin.take_diagnostics(), [], "{name}");
        assert!(first.starts_with("<svg "), "{name}: {first}");
        assert!(first.contains("prefers-color-scheme"), "{name}");
        let xml = roxmltree::Document::parse(&first).unwrap();
        let text: String = xml
            .descendants()
            .filter(|node| {
                node.is_text()
                    && node
                        .parent_element()
                        .is_some_and(|parent| parent.tag_name().name() != "style")
            })
            .filter_map(|node| node.text())
            .collect();
        for label in fixture["labels"].as_array().unwrap() {
            assert!(
                text.contains(label.as_str().unwrap()),
                "{name}: missing {label}"
            );
        }
        assert_eq!(
            first,
            renderer.render_with_code_hooks(&document, &mut plugin)
        );
        assert_eq!(
            first,
            renderer.render_with_code_hooks(&document, &mut hooks("fixture"))
        );
        actual.push((name, format!("{:x}", Sha256::digest(first.as_bytes()))));
        expected.push((name, fixture["sha256"].as_str().unwrap().to_owned()));
    }
    assert_eq!(
        actual, expected,
        "Engine output changed; inspect fixture output before updating hashes"
    );
}

#[test]
fn every_failed_fence_reports_its_original_utf8_span_and_keeps_later_diagrams() {
    let invalid = "```mermaid\nflowchart LR\nA[unterminated\n```";
    let unsupported = "```mermaid\nfutureDiagram\nA --> B\n```";
    let source = format!("é\n\n{invalid}\n\n{unsupported}\n\n```mermaid\npie\n\"OK\" : 1\n```");
    let allocator = Allocator::new();
    let document = Parser::new(&allocator, &source).parse().unwrap();
    let mut plugin = hooks("errors");
    let html = HtmlRenderer::new().render_with_code_hooks(&document, &mut plugin);
    assert!(html.contains("A[unterminated"));
    assert!(html.contains("futureDiagram"));
    assert!(html.contains("<svg "));
    let diagnostics = plugin.take_diagnostics();
    assert_eq!(diagnostics.len(), 2, "{diagnostics:?}");
    assert_eq!(diagnostics[0].span.source_text(&source).trim_end(), invalid);
    assert_eq!(
        diagnostics[1].span.source_text(&source).trim_end(),
        unsupported
    );
    assert!(
        diagnostics
            .iter()
            .all(|diagnostic| diagnostic.message.contains("Ferrigura"))
    );
    assert!(plugin.take_diagnostics().is_empty());
}

#[test]
fn explicit_light_and_dark_schemes_need_no_client_runtime() {
    let allocator = Allocator::new();
    let source = "```mermaid\npie\n\"A\" : 1\n```";
    let document = Parser::new(&allocator, source).parse().unwrap();
    let mut renderer = HtmlRenderer::new();
    let mut light =
        hooks("scheme").with_color_scheme(ferromark_ferrigura::ferrigura::ColorScheme::Light);
    let mut dark =
        hooks("scheme").with_color_scheme(ferromark_ferrigura::ferrigura::ColorScheme::Dark);
    let light_html = renderer.render_with_code_hooks(&document, &mut light);
    let dark_html = renderer.render_with_code_hooks(&document, &mut dark);
    assert!(light_html.starts_with("<svg ") && dark_html.starts_with("<svg "));
    assert_ne!(light_html, dark_html);
    assert_eq!(light.take_diagnostics(), []);
    assert_eq!(dark.take_diagnostics(), []);
}
