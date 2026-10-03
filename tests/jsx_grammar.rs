#![cfg(feature = "jsx")]

use ferromark::{Allocator, JsxRenderer, JsxRendererOptions, Parser, ParserOptions};

fn assert_jsx(source: &str, mdx: bool, prefix: bool) {
    let allocator = Allocator::new();
    let options = ParserOptions {
        mdx,
        mdx_compatible: mdx,
        front_matter: true,
        ..ParserOptions::ffm()
    };
    let document = Parser::with_options(&allocator, source, options)
        .parse()
        .unwrap();
    let renderer = JsxRenderer::with_options(JsxRendererOptions {
        component_prefix: prefix.then(|| "_components".to_owned()),
        ..JsxRendererOptions::default()
    });
    let output = renderer.render(&document, source);
    let js_allocator = oxc_allocator::Allocator::default();
    let parsed = oxc_parser::Parser::new(&js_allocator, &output.body, oxc_span::SourceType::jsx())
        .parse_expression();
    assert!(
        parsed.is_ok(),
        "source:\n{source}\nJSX:\n{}\n{parsed:?}",
        output.body
    );
}

#[test]
fn emitted_bodies_are_real_jsx_expressions_across_markdown_constructs() {
    for source in [
        "",
        "# Heading\n\nQuoted \"text\" with \\backslash and literal {braces}.\n\n**Strong** *emphasis* ~~strike~~ `code`.",
        "---\ntitle: Native\n---\n\nTitle\n=====\n\n- [x] Complete\n- [ ] Pending\n\n3. Third\n4. Fourth\n\n---",
        "| A | B |\n| :- | -: |\n| One | Two |\n| Three | Four |\n\n[Link](https://example.com) and ![Alt](image.png).",
        "> [!WARNING] Title\n> Body **with formatting**.\n\n```js title=\"Example\"\nconst text = \"<safe>\";\n```",
        "Reference[^note].\n\n[^note]: Footnote **content**.\n\n## fn-note",
        "<div class=\"note\" style=\"color: red; --x: 1; *color: blue\"><br><img src='image.png' alt='Image'><span>Raw &amp; text</span></div>\n\n<!-- comment -->",
        "# A <em>raw</em> and <strong>heading</strong>\n\n$$x^2$$\n\nDefinition\n: Explanation",
    ] {
        assert_jsx(source, false, false);
        assert_jsx(source, false, true);
    }
}

#[test]
fn authored_mdx_survives_as_valid_jsx_with_component_prefixes() {
    for source in [
        "import Badge from './badge.js';\n\n# <Badge>Visible</Badge>\n\n<section {...props}><ui.Badge label=\"a &amp; b\" /></section>",
        "<Panel>\n  ## Nested\n\n  Markdown **children** and {/}/.test(value) ? `a${value}` : null}.\n</Panel>",
        "Text {(() => {\n  // } is a comment\n  return `value\n\n    preserved ${value}`;\n})()} after.",
        "<Widget\n label=\"first\n  second\"\n callback={() => {\n   return /}/.source;\n }}\n/>",
    ] {
        assert_jsx(source, true, false);
        assert_jsx(source, true, true);
    }
}

#[test]
fn expression_line_mappings_point_to_actual_generated_utf16_positions() {
    let source =
        "Quoted \"text\", \\ slash and 😀 {(() => {\n  throw new Error(\"boom\");\n})()} after.\n";
    let allocator = Allocator::new();
    let document = Parser::with_options(
        &allocator,
        source,
        ParserOptions {
            mdx: true,
            mdx_compatible: true,
            ..ParserOptions::gfm_spec()
        },
    )
    .parse()
    .unwrap();
    let output = JsxRenderer::new().render(&document, source);
    let mapping = output
        .mappings
        .iter()
        .find(|mapping| mapping.source_line == 1)
        .unwrap();
    assert_eq!(mapping.source_column, 0);
    let generated_line = output
        .body
        .lines()
        .nth(mapping.generated_line as usize)
        .unwrap();
    let before_throw = generated_line.split("throw").next().unwrap();
    assert_eq!(
        before_throw.encode_utf16().count() as u32,
        mapping.generated_column + 2
    );
    assert!(generated_line.contains("throw new Error(\"boom\")"));
}

#[test]
fn authored_binding_metadata_follows_jsx_identifier_rules() {
    let allocator = Allocator::new();
    let source = "<widget /><DIV /><custom-element /><foo.Bar /><α />";
    let document = Parser::with_options(
        &allocator,
        source,
        ParserOptions {
            mdx: true,
            mdx_compatible: true,
            ..ParserOptions::gfm_spec()
        },
    )
    .parse()
    .unwrap();
    let output = JsxRenderer::new().render(&document, source);
    assert_eq!(output.components, ["DIV", "foo", "α"]);
    assert_eq!(output.elements, ["p"]);
}
