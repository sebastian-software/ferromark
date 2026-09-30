#![cfg(feature = "ferriki")]

use ferromark::ferriki::{ErrorKind, Highlighter};
use ferromark::{
    Allocator, CodeAnnotationSyntax, FerrikiHighlightHooks, HtmlRenderer, HtmlRendererOptions,
    Parser,
};

const GRAMMAR: &str = r#"{
  "name": "rust", "scopeName": "source.rust",
  "patterns": [{"match": "\\b(let|fn)\\b", "name": "keyword.control.rust"}]
}"#;
const THEME: &str = r##"{
  "name": "test-dark", "type": "dark",
  "colors": {"editor.foreground": "#eaf0ff", "editor.background": "#101820"},
  "tokenColors": [{"scope": "keyword.control.rust", "settings": {"foreground": "#ff0000"}}]
}"##;

fn highlighter() -> Highlighter {
    let mut highlighter = Highlighter::builder().build().unwrap();
    highlighter.register_language_json(GRAMMAR).unwrap();
    highlighter.register_theme_json(THEME).unwrap();
    highlighter
}

#[test]
fn renders_fenced_and_indented_blocks_with_public_apis() {
    let source = "```RUST title=\"a&b\"\r\nlet x = \"<é>\";\r\n```\r\n\r\n    plain & code\r\n";
    let allocator = Allocator::new();
    let document = Parser::new(&allocator, source).parse().unwrap();
    let mut highlighter = highlighter();
    let mut hooks = FerrikiHighlightHooks::new(&mut highlighter, "test-dark");
    let mut renderer = HtmlRenderer::new();
    let html = renderer.render_with_hooks(&document, &mut hooks);

    assert!(html.contains("class=\"language-RUST\""), "{html}");
    assert!(
        html.contains("background-color:#101820;color:#eaf0ff"),
        "{html}"
    );
    assert!(html.contains("&#x3C;é>"), "{html}");
    assert!(!html.contains("<é>"), "{html}");
    assert!(html.contains("plain &amp; code"), "{html}");
}

#[test]
fn unknown_language_and_missing_language_fall_back_without_error_callback() {
    let source = "```missing\n<unknown>\n```\n\n```\n<plain>\n```";
    let allocator = Allocator::new();
    let document = Parser::new(&allocator, source).parse().unwrap();
    let mut highlighter = highlighter();
    let mut errors = Vec::new();
    let mut record = |error: &ferromark::ferriki::Error| errors.push(error.kind());
    let mut hooks =
        FerrikiHighlightHooks::new(&mut highlighter, "test-dark").with_error_handler(&mut record);
    let html = HtmlRenderer::new().render_with_hooks(&document, &mut hooks);

    assert!(html.contains("&lt;unknown&gt;"), "{html}");
    assert!(html.contains("&lt;plain&gt;"), "{html}");
    assert!(!html.contains("background-color:"), "{html}");
    assert!(errors.is_empty());
}

#[test]
fn highlighter_error_is_reported_and_falls_back() {
    let allocator = Allocator::new();
    let document = Parser::new(&allocator, "```rust\n<code>\n```")
        .parse()
        .unwrap();
    let mut highlighter = highlighter();
    let mut errors = Vec::new();
    let mut record = |error: &ferromark::ferriki::Error| errors.push(error.kind());
    let mut hooks = FerrikiHighlightHooks::new(&mut highlighter, "missing-theme")
        .with_error_handler(&mut record);
    let html = HtmlRenderer::new().render_with_hooks(&document, &mut hooks);

    assert!(html.contains("&lt;code&gt;"), "{html}");
    assert!(!html.contains("background-color:"), "{html}");
    assert_eq!(errors, [ErrorKind::UnknownTheme]);
}

#[test]
fn vitepress_metadata_and_lines_remain_ferromarks() {
    let source = "```rust:line-numbers=7 [a&b.rs] {1}\n// [!code warning]\nlet x = 1;\n```";
    let allocator = Allocator::new();
    let document = Parser::new(&allocator, source).parse().unwrap();
    let mut highlighter = highlighter();
    let mut hooks = FerrikiHighlightHooks::new(&mut highlighter, "test-dark");
    let mut renderer = HtmlRenderer::with_options(HtmlRendererOptions {
        code_annotations: true,
        code_annotation_syntax: CodeAnnotationSyntax::VitePress,
        ..Default::default()
    });
    let html = renderer.render_with_hooks(&document, &mut hooks);

    assert!(html.contains("data-code-title=\"a&amp;b.rs\""), "{html}");
    assert!(html.contains("data-line-number-start=\"7\""), "{html}");
    assert!(html.contains("ox-code-line--warning"), "{html}");
    assert!(html.contains("let x = 1;"), "{html}");
}

#[test]
fn reuses_highlighter_and_renderer_across_documents() {
    let mut highlighter = highlighter();
    let mut renderer = HtmlRenderer::new();
    for source in ["```rust\nfn one() {}\n```", "```rust\nfn two() {}\n```"] {
        let allocator = Allocator::new();
        let document = Parser::new(&allocator, source).parse().unwrap();
        let mut hooks = FerrikiHighlightHooks::new(&mut highlighter, "test-dark");
        let html = renderer.render_with_hooks(&document, &mut hooks);
        assert!(html.contains("background-color:#101820"), "{html}");
    }
}
