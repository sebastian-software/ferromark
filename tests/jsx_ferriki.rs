#![cfg(feature = "ferriki")]

use ferromark::ferriki::{ErrorKind, Highlighter};
use ferromark::{
    Allocator, FerrikiJsxHooks, JsxCodeBlockInput, JsxHighlightedCodeBlock, JsxRenderHooks,
    JsxRenderer, JsxRendererOptions, Parser,
};

const GRAMMAR: &str = r#"{
  "name": "rust", "scopeName": "source.rust",
  "patterns": [{"match": "\\b(let|fn)\\b", "name": "keyword.control.rust"}]
}"#;
const LIGHT_THEME: &str = r##"{
  "name": "test-light", "type": "light",
  "colors": {"editor.foreground": "#202020", "editor.background": "#ffffff"},
  "tokenColors": [{"scope": "keyword.control.rust", "settings": {"foreground": "#0055aa", "fontStyle": "italic"}}]
}"##;
const DARK_THEME: &str = r##"{
  "name": "test-dark", "type": "dark",
  "colors": {"editor.foreground": "#eeeeee", "editor.background": "#101820"},
  "tokenColors": [{"scope": "keyword.control.rust", "settings": {"foreground": "#ee00ff", "fontStyle": "bold"}}]
}"##;

fn highlighter() -> Highlighter {
    let mut highlighter = Highlighter::builder().build().unwrap();
    highlighter.register_language_json(GRAMMAR).unwrap();
    highlighter.register_theme_json(LIGHT_THEME).unwrap();
    highlighter.register_theme_json(DARK_THEME).unwrap();
    highlighter
}

fn parse<'a>(allocator: &'a Allocator, source: &'a str) -> ferromark::ast::Document<'a> {
    Parser::new(allocator, source).parse().unwrap()
}

#[test]
fn renders_single_theme_tokens_as_safe_jsx_children() {
    let source = "```RUST\nlet value = \"<é>\";\n```\n";
    let allocator = Allocator::new();
    let document = parse(&allocator, source);
    let mut highlighter = highlighter();
    let mut hooks = FerrikiJsxHooks::new(&mut highlighter, "test-light");
    let output = JsxRenderer::new().render_with_hooks(&document, source, &mut hooks);

    assert!(
        output.body.contains("backgroundColor: \"#ffffff\""),
        "{}",
        output.body
    );
    assert!(
        output.body.contains("color: \"#0055AA\""),
        "{}",
        output.body
    );
    assert!(
        output.body.contains("{\" value = \\\"<é>\\\";\"}"),
        "{}",
        output.body
    );
    assert!(
        output.body.contains("className={\"language-RUST\"}"),
        "{}",
        output.body
    );
}

#[test]
fn dual_theme_output_keeps_light_and_dark_colors_and_font_styles() {
    let source = "```rust\nlet value = 1;\n```\n";
    let allocator = Allocator::new();
    let document = parse(&allocator, source);
    let mut highlighter = highlighter();
    let mut hooks =
        FerrikiJsxHooks::with_light_dark_themes(&mut highlighter, "test-light", "test-dark");
    let output = JsxRenderer::new().render_with_hooks(&document, source, &mut hooks);

    assert!(
        output.body.contains("\"--shiki-light\": \"#0055AA\""),
        "{}",
        output.body
    );
    assert!(
        output.body.contains("\"--shiki-dark\": \"#EE00FF\""),
        "{}",
        output.body
    );
    assert!(
        output
            .body
            .contains("\"--shiki-light-font-style\": \"italic\""),
        "{}",
        output.body
    );
    assert!(
        output
            .body
            .contains("\"--shiki-dark-font-weight\": \"bold\""),
        "{}",
        output.body
    );
    assert!(
        output.body.contains("\"--shiki-light-bg\": \"#ffffff\""),
        "{}",
        output.body
    );
    assert!(
        output.body.contains("\"--shiki-dark-bg\": \"#101820\""),
        "{}",
        output.body
    );
}

#[test]
fn dual_theme_highlighting_preserves_trailing_empty_lines_for_lf_and_crlf() {
    for source in [
        "```rust\nlet value = 1;\n```\n",
        "```rust\r\nlet value = 1;\r\n```\r\n",
    ] {
        let allocator = Allocator::new();
        let document = parse(&allocator, source);
        let mut highlighter = highlighter();
        let mut hooks =
            FerrikiJsxHooks::with_light_dark_themes(&mut highlighter, "test-light", "test-dark");
        let output = JsxRenderer::new().render_with_hooks(&document, source, &mut hooks);

        assert!(
            output.body.contains("className={\"shiki\"}"),
            "{}",
            output.body
        );
        assert!(output.body.contains("{\"\\n\"}"), "{}", output.body);
        assert!(
            output.body.contains("<span className={\"line\"}"),
            "{}",
            output.body
        );
    }
}

#[test]
fn generic_component_receives_code_metadata_and_line_number_settings() {
    let source = concat!(
        "```rust:line-numbers=7 [src/main.rs] {1,3}\n",
        "let first = 1;\n",
        "let second = 2;\n",
        "let third = 3;\n",
        "```\n"
    );
    let allocator = Allocator::new();
    let document = parse(&allocator, source);
    let mut highlighter = highlighter();
    let mut hooks =
        FerrikiJsxHooks::with_light_dark_themes(&mut highlighter, "test-light", "test-dark");
    let renderer = JsxRenderer::with_options(JsxRendererOptions {
        component_prefix: Some("_components".into()),
        code_block_component: Some("CodeBlock".into()),
        ..JsxRendererOptions::default()
    });
    let output = renderer.render_with_hooks(&document, source, &mut hooks);

    assert!(
        output.body.contains(
            "<CodeBlock code={\"let first = 1;\\nlet second = 2;\\nlet third = 3;\\n\"} language={\"rust\"} title={\"src/main.rs\"} data-label={\"src/main.rs\"} lineNumbers={true}>"
        ),
        "{}",
        output.body
    );
    assert!(output.body.contains("<_components.pre"), "{}", output.body);
    assert!(output.body.contains("<_components.code"), "{}", output.body);
    assert!(
        output.body.contains("data-line-number-start={\"7\"}"),
        "{}",
        output.body
    );
    assert!(output.body.contains("data-ln={\"7\"}"), "{}", output.body);
    assert!(
        output.body.contains("ox-code-line--highlight highlighted"),
        "{}",
        output.body
    );
    assert_eq!(output.components, ["CodeBlock"]);
    assert!(output.elements.contains(&"pre".to_owned()));
    assert!(output.elements.contains(&"code".to_owned()));
    assert!(output.elements.contains(&"span".to_owned()));
}

#[test]
fn global_line_numbers_and_metadata_overrides_are_applied() {
    let source = "```rust {2}\nlet first = 1;\nlet second = 2;\n```\n";
    let allocator = Allocator::new();
    let document = parse(&allocator, source);
    let renderer = JsxRenderer::with_options(JsxRendererOptions {
        show_line_numbers: true,
        ..JsxRendererOptions::default()
    });
    let output = renderer.render(&document, source);

    assert!(
        output.body.contains("data-line-number-start={\"1\"}"),
        "{}",
        output.body
    );
    assert!(output.body.contains("data-ln={\"2\"}"), "{}", output.body);
    assert!(
        output.body.contains("ox-code-line--highlight highlighted"),
        "{}",
        output.body
    );

    let no_numbers_source = "```rust :no-line-numbers\nlet value = 1;\n```\n";
    let allocator = Allocator::new();
    let document = parse(&allocator, no_numbers_source);
    let output = renderer.render(&document, no_numbers_source);
    assert!(
        !output.body.contains("data-line-numbers"),
        "{}",
        output.body
    );
}

#[test]
fn missing_language_falls_back_and_only_non_language_errors_are_observed() {
    let source = "```missing\n<plain>\n```\n";
    let allocator = Allocator::new();
    let document = parse(&allocator, source);
    let mut highlighter = highlighter();
    let mut errors = Vec::new();
    let mut record = |error: &ferromark::ferriki::Error| errors.push(error.kind());
    let mut hooks =
        FerrikiJsxHooks::new(&mut highlighter, "test-light").with_error_handler(&mut record);
    let output = JsxRenderer::new().render_with_hooks(&document, source, &mut hooks);
    assert!(output.body.contains("{\"<plain>\\n\"}"), "{}", output.body);
    assert!(errors.is_empty());

    let source = "```rust\nlet value = 1;\n```\n";
    let allocator = Allocator::new();
    let document = parse(&allocator, source);
    let mut errors = Vec::new();
    let mut record = |error: &ferromark::ferriki::Error| errors.push(error.kind());
    let mut hooks =
        FerrikiJsxHooks::new(&mut highlighter, "missing-theme").with_error_handler(&mut record);
    let output = JsxRenderer::new().render_with_hooks(&document, source, &mut hooks);
    assert!(
        output.body.contains("{\"let value = 1;\\n\"}"),
        "{}",
        output.body
    );
    assert_eq!(errors, [ErrorKind::UnknownTheme]);
}

#[test]
fn render_code_override_runs_before_highlighter_and_language_component() {
    struct Hooks {
        highlighted: bool,
    }
    impl JsxRenderHooks for Hooks {
        fn render_code_block(&mut self, _input: JsxCodeBlockInput<'_>) -> Option<String> {
            Some("<CodePreview />".into())
        }

        fn highlight_code_block(
            &mut self,
            _input: JsxCodeBlockInput<'_>,
        ) -> Option<JsxHighlightedCodeBlock> {
            self.highlighted = true;
            None
        }
    }

    let source = "```rust\nlet value = 1;\n```\n";
    let allocator = Allocator::new();
    let document = parse(&allocator, source);
    let mut hooks = Hooks { highlighted: false };
    let output = JsxRenderer::with_options(
        JsxRendererOptions::default().with_code_block_component("rust", "RustCode"),
    )
    .render_with_hooks(&document, source, &mut hooks);

    assert!(output.body.contains("<CodePreview />"), "{}", output.body);
    assert!(!hooks.highlighted);
    assert!(output.components.is_empty());
}
