use std::collections::BTreeMap;

use crate::allocator::Allocator;
use crate::ast::Node;
use crate::parser::Parser;

use super::{JsxCodeBlockRenderOutput, JsxRenderer, JsxRendererOptions};

fn assert_fence_matches(source: &str, standalone_code: Option<&str>) -> JsxCodeBlockRenderOutput {
    let allocator = Allocator::new();
    let document = Parser::new(&allocator, source).parse().unwrap();
    let Some(Node::CodeBlock(block)) = document.children.first() else {
        panic!(
            "expected one fenced code block, got {:?}",
            document.children
        );
    };
    let renderer = JsxRenderer::new();
    let standalone = renderer.render_code_block(
        standalone_code.unwrap_or(block.value),
        block.lang,
        block.meta,
    );
    let document_output = renderer.render(&document, source);
    assert_eq!(document_output.body, format!("<>\n{}</>", standalone.jsx));
    standalone
}

#[test]
fn standalone_markup_matches_fences_for_code_shapes_and_line_endings() {
    for source in [
        "```\n```\n",
        "```rust\n",
        "```rust\nconst value = 1;",
        "```unknown [sample]\n日本語 <safe> {literal} &\n\n",
        "```rust title=\"main.rs\" [sample] {2,4} :line-numbers=7\none\ntwo\nthree\nfour\n",
        "```rust [label] title=\"CRLF sample\" :line-numbers\r\nfirst\r\nsecond\r\n\r\n",
    ] {
        assert_fence_matches(source, None);
    }

    let source = "```rust\r\nfirst\r\nsecond\r\n";
    assert_fence_matches(source, Some("first\r\nsecond\r\n"));
}

#[test]
fn standalone_metadata_reflects_overrides_and_selection_markup() {
    let rendered = assert_fence_matches(
        "```rust [demo] title=\"override.rs\" {2,4} :line-numbers=7\none\ntwo\nthree\nfour\n",
        None,
    );
    assert_eq!(rendered.language.as_deref(), Some("rust"));
    assert_eq!(rendered.title.as_deref(), Some("override.rs"));
    assert_eq!(rendered.label.as_deref(), Some("demo"));
    assert!(rendered.line_numbers);
    assert!(rendered.jsx.contains("data-line-number-start={\"7\"}"));
    assert!(rendered.jsx.contains("data-ln={\"8\"}"));
    assert!(rendered.jsx.contains("data-ln={\"10\"}"));
    assert!(rendered.jsx.contains("ox-code-line--highlight highlighted"));

    let default_off = assert_fence_matches("```rust title=\"plain.rs\"\ncode\n", None);
    assert!(!default_off.line_numbers);
    let override_on = assert_fence_matches("```rust:no-line-numbers :line-numbers=4\ncode\n", None);
    assert!(override_on.line_numbers);
    let override_off = JsxRenderer::with_options(JsxRendererOptions {
        show_line_numbers: true,
        ..JsxRendererOptions::default()
    })
    .render_code_block("code\n", Some("rust"), Some(":no-line-numbers"));
    assert!(!override_off.line_numbers);
}

#[test]
fn standalone_markup_is_always_intrinsic_and_ignores_component_wrappers() {
    let renderer = JsxRenderer::with_options(JsxRendererOptions {
        component_prefix: Some("_components".to_owned()),
        code_block_components: BTreeMap::from([("rust".to_owned(), "RustView".to_owned())]),
        code_block_component: Some("CodeBlock".to_owned()),
        ..JsxRendererOptions::default()
    });
    let rendered = renderer.render_code_block("fn main() {}\n", Some("rust"), None);
    assert!(rendered.jsx.starts_with("<pre>"));
    assert!(
        rendered
            .jsx
            .contains("<code className={\"language-rust\"}>")
    );
    assert!(!rendered.jsx.contains("_components."));
    assert!(!rendered.jsx.contains("RustView"));
    assert!(!rendered.jsx.contains("CodeBlock"));
}
