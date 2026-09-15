//! Module-level `import` / `export` (`MdxjsEsm`) when `ParserOptions.mdx` is true.
//!
//! This slice stores raw ESM source. It does not evaluate JavaScript, resolve
//! imports, or hydrate components. Fences and inline code are not ESM.

use ferromark_allocator::Allocator;
use ferromark_parser::{Parser, ParserOptions};

#[path = "support/pretty.rs"]
mod pretty;

fn pretty_ast(source: &str, options: ParserOptions) -> String {
    let allocator = Allocator::new();
    let doc = Parser::with_options(&allocator, source, options)
        .parse()
        .expect("parser should not fail on MDX ESM fixtures");
    let mut out = String::new();
    pretty::format_document(&doc, source, &mut out);
    out
}

fn mdx_tree(source: &str) -> String {
    pretty_ast(source, ParserOptions::mdx())
}

fn assert_esm_value(tree: &str, value: &str) {
    let needle = format!("MdxjsEsm value={value:?}");
    assert!(tree.contains(&needle), "expected {needle} in:\n{tree}");
}

#[test]
fn mdx_false_import_stays_paragraph() {
    let tree = pretty_ast(
        "import { Chart } from './Chart'\n",
        ParserOptions::default(),
    );
    assert!(
        tree.contains("Paragraph"),
        "mdx=false import stays a paragraph:\n{tree}"
    );
    assert!(
        tree.contains("Text \"import { Chart } from './Chart'\""),
        "expected text:\n{tree}"
    );
    assert!(
        !tree.contains("MdxjsEsm"),
        "mdx=false must not emit MdxjsEsm:\n{tree}"
    );
}

#[test]
fn mdx_false_export_stays_paragraph() {
    let tree = pretty_ast(
        "export const meta = { title: 'Hi' }\n",
        ParserOptions::default(),
    );
    assert!(
        tree.contains("Paragraph"),
        "mdx=false export stays a paragraph:\n{tree}"
    );
    assert!(
        !tree.contains("MdxjsEsm"),
        "mdx=false must not emit MdxjsEsm:\n{tree}"
    );
}

#[test]
fn import_named_is_esm() {
    let tree = mdx_tree("import { Chart } from './Chart'\n");
    assert_esm_value(&tree, "import { Chart } from './Chart'");
    assert!(
        !tree.contains("Paragraph"),
        "top-level import must not wrap in a paragraph:\n{tree}"
    );
}

#[test]
fn export_const_is_esm() {
    let tree = mdx_tree("export const meta = { title: 'Hi' }\n");
    assert_esm_value(&tree, "export const meta = { title: 'Hi' }");
    assert!(
        !tree.contains("Paragraph"),
        "top-level export must not wrap in a paragraph:\n{tree}"
    );
}

#[test]
fn consecutive_import_then_export() {
    let tree = mdx_tree("import { Chart } from './Chart'\nexport const meta = { title: 'Hi' }\n");
    assert_esm_value(&tree, "import { Chart } from './Chart'");
    assert_esm_value(&tree, "export const meta = { title: 'Hi' }");
    assert_eq!(
        tree.matches("MdxjsEsm").count(),
        2,
        "expected two ESM nodes:\n{tree}"
    );
}

#[test]
fn multiline_import() {
    let source = "import {\n  Chart\n} from './Chart'\n";
    let tree = mdx_tree(source);
    assert_esm_value(&tree, "import {\n  Chart\n} from './Chart'");
    assert!(
        !tree.contains("Paragraph"),
        "multiline import is one ESM node:\n{tree}"
    );
}

#[test]
fn multiline_export() {
    let source = "export const meta = {\n  title: 'Hi',\n}\n";
    let tree = mdx_tree(source);
    assert_esm_value(&tree, "export const meta = {\n  title: 'Hi',\n}");
}

#[test]
fn fenced_import_is_not_esm() {
    let tree = mdx_tree("```js\nimport { Chart } from './Chart'\n```\n");
    assert!(tree.contains("Code"), "expected a fence:\n{tree}");
    assert!(
        !tree.contains("MdxjsEsm"),
        "fence contents are not ESM:\n{tree}"
    );
}

#[test]
fn inline_code_import_is_not_esm() {
    let tree = mdx_tree("Use `import { Chart } from './Chart'` in prose.\n");
    assert!(tree.contains("InlineCode"), "expected inline code:\n{tree}");
    assert!(
        !tree.contains("MdxjsEsm"),
        "inline code is not ESM:\n{tree}"
    );
}

#[test]
fn hostile_script_string_stores_source_without_panic() {
    let source = "import x from \"<script>\"\n";
    let tree = mdx_tree(source);
    assert_esm_value(&tree, "import x from \"<script>\"");
    assert!(
        !tree.contains("<script>alert"),
        "source is stored, not evaluated:\n{tree}"
    );
}

#[test]
fn important_word_is_not_esm() {
    let tree = mdx_tree("important note\n");
    assert!(
        tree.contains("Paragraph"),
        "identifier prefix must stay prose:\n{tree}"
    );
    assert!(
        !tree.contains("MdxjsEsm"),
        "`important` is not `import`:\n{tree}"
    );
}

#[test]
fn esm_then_heading_and_jsx() {
    let tree = mdx_tree("import { Chart } from './Chart'\n\n# Title\n\n<Chart />\n");
    assert_esm_value(&tree, "import { Chart } from './Chart'");
    assert!(
        tree.contains("Heading"),
        "markdown after ESM still parses:\n{tree}"
    );
    assert!(
        tree.contains("MdxJsxFlowElement name=Some(\"Chart\")"),
        "JSX after ESM must stay green:\n{tree}"
    );
}

#[test]
fn value_is_source_not_evaluated() {
    let tree = mdx_tree("export const n = 1 + 1\n");
    assert_esm_value(&tree, "export const n = 1 + 1");
    assert!(
        !tree.contains("value=\"2\""),
        "must not evaluate JS:\n{tree}"
    );
}

#[test]
fn unclosed_import_does_not_panic() {
    let tree = mdx_tree("import {\n");
    assert!(
        tree.contains("MdxjsEsm") || tree.contains("Paragraph"),
        "unclosed import must not panic:\n{tree}"
    );
}

#[test]
fn esm_balances_delimiters_without_interpreting_strings_or_comments() {
    use ferromark_ast::Node;

    let statements = [
        "export const x = call(\n [1, 2], { key: '};\\\'still string' }\n); \t",
        "export const x = {\n // } ] ) ; ignored until newline\n key: \"quote\\\";still string\",\n};",
        "export const x = [\n /* ] } ) ;\n ignored */\n `template\\`;still string`,\n];",
        "export const x = 6 / 2;",
        "import \"package\" // ; } trailing comment",
        "export const x = 'single\\\\slash';",
    ];
    for ending in ["\n", "\r\n", "\r"] {
        for statement in statements {
            let statement = statement.replace('\n', ending);
            let source = format!("{statement}{ending}{ending}# Following{ending}");
            let allocator = Allocator::new();
            let doc = Parser::with_options(&allocator, &source, ParserOptions::mdx())
                .parse()
                .unwrap();
            let [Node::MdxjsEsm(esm), Node::Heading(heading)] = doc.children.as_slice() else {
                panic!("ESM swallowed or split following Markdown: {source:?}: {doc:?}");
            };
            assert_eq!(esm.value, statement.trim_end());
            assert_eq!(
                esm.span.source_text(&source).trim_end(),
                statement.trim_end()
            );
            assert_eq!(heading.depth, 1);
            assert!(heading.span.source_text(&source).contains("Following"));
        }
    }
}

#[test]
fn unterminated_esm_preserves_the_complete_source() {
    use ferromark_ast::Node;

    for source in [
        "export const x = 'unterminated\\",
        "export const x = \"unterminated\\",
        "export const x = `unterminated\\",
        "export const x = { /* unterminated",
        "export const x = [1, (2",
        "export",
        "import",
    ] {
        let allocator = Allocator::new();
        let doc = Parser::with_options(&allocator, source, ParserOptions::mdx())
            .parse()
            .unwrap();
        let [Node::MdxjsEsm(esm)] = doc.children.as_slice() else {
            panic!("expected one source-only ESM node: {doc:?}");
        };
        assert_eq!(esm.value, source);
        assert_eq!(esm.span.source_text(source), source);
    }
}
