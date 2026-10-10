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

fn boundary_highlighter() -> Highlighter {
    let mut highlighter = Highlighter::builder().build().unwrap();
    highlighter
        .register_language_json(
            r#"{
      "name": "boundary", "scopeName": "source.boundary",
      "patterns": [
        {"match": "a", "name": "first"},
        {"match": "😀", "name": "middle"},
        {"match": "b", "name": "last"}
      ]
    }"#,
        )
        .unwrap();
    highlighter
        .register_theme_json(
            r##"{
      "name": "boundary-light", "type": "light",
      "colors": {"editor.foreground": "#111111", "editor.background": "#ffffff"},
      "tokenColors": [
        {"scope": "last", "settings": {"foreground": "#222222", "fontStyle": "italic underline"}}
      ]
    }"##,
        )
        .unwrap();
    highlighter
        .register_theme_json(
            r##"{
      "name": "boundary-dark", "type": "dark",
      "colors": {"editor.foreground": "#bbbbbb", "editor.background": "#000000"},
      "tokenColors": [
        {"scope": "first", "settings": {"foreground": "#aaaaaa", "fontStyle": "bold strikethrough"}}
      ]
    }"##,
        )
        .unwrap();
    highlighter
}

#[test]
fn dual_theme_output_matches_the_legacy_adapter() {
    let mut highlighter = boundary_highlighter();
    let mut output = String::new();
    for code in [
        "a😀b <é東京>&\"'",
        "a😀b\n\nb\n",
        "a😀b\r\n\r\nb\r\n",
        "",
        "\n\n",
    ] {
        for dark in ["boundary-dark", "boundary-light"] {
            let mut hooks =
                FerrikiJsxHooks::with_light_dark_themes(&mut highlighter, "boundary-light", dark);
            let block = JsxRenderer::new().render_code_block_with_hooks(
                code,
                Some("boundary"),
                Some("[a&b] title=\"safe.rs\" {1,3} :line-numbers=7"),
                &mut hooks,
            );
            output.push_str(&block.jsx);
        }
    }
    // Captured with Ferriki 0.10 and the manual two-theme adapter before the
    // upgrade. Keep this exact output as the compatibility oracle.
    insta::assert_snapshot!("dual_theme_legacy_output", output);
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

#[test]
fn public_multi_theme_tokens_align_different_boundaries_with_utf8_offsets() {
    let mut highlighter = boundary_highlighter();
    let code = "a😀b\r\na😀b\n";
    let light = highlighter
        .highlight(code, "boundary", "boundary-light")
        .unwrap();
    let dark = highlighter
        .highlight(code, "boundary", "boundary-dark")
        .unwrap();
    assert_eq!(
        light.tokens[0]
            .iter()
            .map(|token| token.content.as_str())
            .collect::<Vec<_>>(),
        ["a😀", "b"]
    );
    assert_eq!(
        dark.tokens[0]
            .iter()
            .map(|token| token.content.as_str())
            .collect::<Vec<_>>(),
        ["a", "😀b"]
    );
    let highlighted = highlighter
        .highlight_with_themes(
            code,
            "boundary",
            &[("light", "boundary-light"), ("dark", "boundary-dark")],
        )
        .unwrap();
    assert_eq!(highlighted.tokens.len(), 3);
    assert!(highlighted.tokens[2].is_empty());
    assert_eq!(highlighted.themes[0].color, "light");
    assert_eq!(highlighted.themes[1].color, "dark");
    assert_eq!(
        highlighted
            .tokens
            .iter()
            .flatten()
            .map(|token| token.offset)
            .collect::<Vec<_>>(),
        [0, 1, 5, 8, 9, 13]
    );
    for token in highlighted.tokens.iter().flatten() {
        assert_eq!(
            &code[token.offset..token.offset + token.content.len()],
            token.content
        );
        assert!(token.variants.contains_key("light"));
        assert!(token.variants.contains_key("dark"));
    }
}

#[test]
fn dual_theme_errors_keep_plain_fallback_and_observer_categories() {
    for (language, light, dark, expected) in [
        ("missing", "test-light", "test-dark", None),
        (
            "rust",
            "missing",
            "test-dark",
            Some(ErrorKind::UnknownTheme),
        ),
        (
            "rust",
            "test-light",
            "missing",
            Some(ErrorKind::UnknownTheme),
        ),
    ] {
        let mut highlighter = highlighter();
        let mut errors = Vec::new();
        let mut record = |error: &ferromark::ferriki::Error| errors.push(error.kind());
        let mut hooks = FerrikiJsxHooks::with_light_dark_themes(&mut highlighter, light, dark)
            .with_error_handler(&mut record);
        let block = JsxRenderer::new().render_code_block_with_hooks(
            "<é😀> & code\n",
            Some(language),
            None,
            &mut hooks,
        );
        assert!(block.jsx.contains("{\"<é😀> & code\\n\"}"), "{}", block.jsx);
        assert!(!block.jsx.contains("shiki"), "{}", block.jsx);
        assert_eq!(errors, expected.into_iter().collect::<Vec<_>>());
    }
}
