#![cfg(feature = "jsx")]

use ferromark::{
    Allocator, JsxModuleError, JsxModuleOptions, JsxModuleOutput, JsxRenderer, JsxRendererOptions,
    Parser, ParserOptions,
};

fn provider() -> JsxModuleOptions {
    JsxModuleOptions {
        provider_import_source: Some("docs/provider".to_owned()),
        filename: Some("page.mdx".to_owned()),
        ..JsxModuleOptions::default()
    }
}

fn render(
    source: &str,
    renderer: JsxRendererOptions,
    module: &JsxModuleOptions,
) -> Result<JsxModuleOutput, JsxModuleError> {
    let allocator = Allocator::new();
    let options = ParserOptions {
        mdx: true,
        mdx_compatible: true,
        ..ParserOptions::gfm_spec()
    };
    let document = Parser::with_options(&allocator, source, options)
        .parse()
        .expect("fixture should parse");
    JsxRenderer::with_options(renderer).render_module(&document, source, module)
}

/// Renders a module and checks that it is a JavaScript module with JSX.
fn module(source: &str, options: &JsxModuleOptions) -> JsxModuleOutput {
    let output = render(source, JsxRendererOptions::default(), options).expect("module");
    let allocator = oxc_allocator::Allocator::default();
    let parsed =
        oxc_parser::Parser::new(&allocator, &output.code, oxc_span::SourceType::jsx()).parse();
    assert!(
        parsed.diagnostics.is_empty() && !parsed.panicked,
        "source:\n{source}\nmodule:\n{}\n{:?}",
        output.code,
        parsed.diagnostics
    );
    output
}

/// Decodes source map mappings into `[generated column, source line, source
/// column]` segments per generated line.
fn decode(mappings: &str) -> Vec<Vec<[i64; 3]>> {
    const BASE64: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let (mut source_line, mut source_column) = (0, 0);
    mappings
        .split(';')
        .map(|line| {
            let mut generated_column = 0;
            line.split(',')
                .filter(|segment| !segment.is_empty())
                .map(|segment| {
                    let mut fields = Vec::new();
                    let (mut value, mut shift) = (0i64, 0);
                    for ch in segment.chars() {
                        let digit =
                            i64::try_from(BASE64.find(ch).expect("base64 digit")).expect("digit");
                        value |= (digit & 31) << shift;
                        shift += 5;
                        if digit & 32 == 0 {
                            fields.push(if value & 1 == 1 {
                                -(value >> 1)
                            } else {
                                value >> 1
                            });
                            (value, shift) = (0, 0);
                        }
                    }
                    assert_eq!(fields.len(), 4, "segment {segment}");
                    assert_eq!(fields[1], 0, "one source file");
                    generated_column += fields[0];
                    source_line += fields[2];
                    source_column += fields[3];
                    [generated_column, source_line, source_column]
                })
                .collect()
        })
        .collect()
}

fn utf16_length(text: &str) -> i64 {
    i64::try_from(text.encode_utf16().count()).expect("length")
}

/// The source position that `needle` in the generated code maps to.
fn mapped(output: &JsxModuleOutput, needle: &str) -> (i64, i64) {
    let (line_index, line) = output
        .code
        .lines()
        .enumerate()
        .find(|(_, line)| line.contains(needle))
        .unwrap_or_else(|| panic!("`{needle}` in\n{}", output.code));
    let byte = line.find(needle).expect("needle");
    let column = utf16_length(&line[..byte]);
    let segments = decode(&output.map.mappings);
    let segment = segments[line_index]
        .iter()
        .rev()
        .find(|segment| segment[0] <= column)
        .unwrap_or_else(|| panic!("no mapping before `{needle}`"));
    assert_eq!(segment[0], column, "`{needle}` starts a mapping");
    (segment[1], segment[2])
}

#[test]
fn a_document_becomes_a_complete_mdx_module() {
    let output = module("# Hello\n\nText with <Badge />.\n", &provider());
    assert_eq!(
        output.code,
        r#"import {useMDXComponents as _provideComponents} from "docs/provider";
function _createMdxContent(props) {
  const _components = {
    "h1": "h1",
    "p": "p",
    ..._provideComponents(),
    ...props.components,
  };
  const {Badge} = _components;
  if (!Badge) _missingMdxReference("Badge", true);
  return <>
<_components.h1 id={"hello"}>{"Hello"}</_components.h1>
<_components.p>{"Text with "}<Badge />{"."}</_components.p>
</>;
}
export default function MDXContent(props = {}) {
  const {wrapper: MDXLayout} = {
    ..._provideComponents(),
    ...props.components,
  };
  return MDXLayout
    ? <MDXLayout {...props}><_createMdxContent {...props} /></MDXLayout>
    : _createMdxContent(props);
}
function _missingMdxReference(id, component) {
  throw new Error("Expected " + (component ? "component" : "object") + " `" + id + "` to be defined: import it, pass it in `components`, or provide it.");
}
"#
    );
    assert_eq!(output.headings.len(), 1);
    assert_eq!(output.map.sources, ["page.mdx"]);
    assert_eq!(
        output.map.sources_content,
        ["# Hello\n\nText with <Badge />.\n"]
    );
    let json = output.map.to_json();
    assert!(json.starts_with(
        "{\"version\":3,\"sources\":[\"page.mdx\"],\"sourcesContent\":[\"# Hello\\n\\nText with <Badge />.\\n\"],\"names\":[],\"mappings\":\""
    ));
    assert!(json.ends_with("\"}"));
}

#[test]
fn the_module_works_without_a_provider_and_without_a_default_export() {
    let options = JsxModuleOptions {
        default_export: false,
        ..JsxModuleOptions::default()
    };
    let output = module("Plain *text*.\n", &options);
    assert!(!output.code.contains("import "));
    assert!(!output.code.contains("_provideComponents"));
    assert!(!output.code.contains("export default"));
    assert!(
        output
            .code
            .contains("\nfunction MDXContent(props = {}) {\n")
    );
    assert!(
        output
            .code
            .contains("const {wrapper: MDXLayout} = props.components || {};")
    );
    // Nothing can be missing, so the module carries no check and no helper.
    assert!(!output.code.contains("_missingMdxReference"));
    assert_eq!(output.map.sources, [""]);
}

#[test]
fn authored_esm_keeps_document_order_and_reports_bindings_and_exports() {
    let source = "import Default, { named as alias } from './a.js'\nimport * as space from './b.js'\n\n# One\n\nexport const [first, { second = 1, ...rest }] = data, third = 3\nexport function helper() {}\nexport class Model {}\n\n## Two\n\nexport { alias as renamed, space }\nexport * as everything from './c.js'\nexport * from './d.js'\n";
    let output = module(source, &provider());
    assert_eq!(
        output.bindings,
        [
            "Default", "alias", "space", "first", "second", "rest", "third", "helper", "Model"
        ]
    );
    assert_eq!(
        output.exports,
        [
            "first",
            "second",
            "rest",
            "third",
            "helper",
            "Model",
            "renamed",
            "space",
            "everything"
        ]
    );
    let import = output.code.find("import Default").expect("import");
    let helper = output.code.find("export function helper").expect("helper");
    let renamed = output
        .code
        .find("export { alias as renamed")
        .expect("export");
    let content = output
        .code
        .find("function _createMdxContent")
        .expect("content");
    assert!(import < helper && helper < renamed && renamed < content);
}

#[test]
fn every_default_export_form_becomes_the_layout() {
    for (source, expected) in [
        (
            "export default function Layout({ children }) { return children }\n\ncontent",
            "\nfunction Layout({ children }) { return children }\nconst MDXLayout = Layout;\n",
        ),
        (
            "export default class Layout {}\n\ncontent",
            "\nclass Layout {}\nconst MDXLayout = Layout;\n",
        ),
        (
            "export default function ({ children }) { return children }\n\ncontent",
            "\nconst MDXLayout = function ({ children }) { return children }\n",
        ),
        (
            "export default ({ children }) => <article>{children}</article>;\n\ncontent",
            "\nconst MDXLayout = ({ children }) => <article>{children}</article>;\n",
        ),
        (
            "import Layout from './layout.js';\nexport default Layout;\n\ncontent",
            "\nimport Layout from './layout.js';\nconst MDXLayout = Layout;\n",
        ),
        (
            "export { default } from './layout.js';\n\ncontent",
            "\nimport MDXLayout from './layout.js';\n",
        ),
        (
            "export { Page as default, other } from \"./layout.js\";\n\ncontent",
            "\nexport { other } from \"./layout.js\";\nimport { Page as MDXLayout } from \"./layout.js\";\n",
        ),
        (
            "import { Page, other } from './layout.js';\nexport { other, Page as default };\nexport const helper = 1;\n\ncontent",
            "\nexport { other };\nexport const helper = 1;\nconst MDXLayout = Page;\n",
        ),
        (
            "export * as default from './layout.js';\n\ncontent",
            "\nimport * as MDXLayout from './layout.js';\n",
        ),
    ] {
        let output = module(source, &provider());
        assert!(
            output.code.contains(expected),
            "source:\n{source}\nmodule:\n{}",
            output.code
        );
        assert!(
            output.code.contains(
                "  return <MDXLayout {...props}><_createMdxContent {...props} /></MDXLayout>;\n"
            ),
            "{}",
            output.code
        );
        // An authored layout replaces the provider's wrapper.
        assert!(!output.code.contains("wrapper"), "{}", output.code);
        assert!(!output.exports.iter().any(|name| name == "default"));
    }
}

#[test]
fn a_named_default_export_stays_a_module_binding() {
    let output = module(
        "export default function Layout({ children }) { return children }\n\n<Layout />\n",
        &provider(),
    );
    assert_eq!(output.bindings, ["Layout"]);
    assert!(!output.code.contains("_missingMdxReference"));
}

#[test]
fn component_references_use_module_bindings_before_the_components_object() {
    let source = "import { Button } from './button.js'\nimport * as ui from './ui.js'\n\n<Button />\n\n<ui.Badge />\n\n<Provided />\n\n<kit.forms.Input />\n\n<props.Slot />\n\n<Reserved />\n\n<α />\n\n<section><custom-element /></section>\n";
    let options = JsxModuleOptions {
        reserved_bindings: ["Reserved".to_owned()].into(),
        ..provider()
    };
    let output = module(source, &options);
    assert!(
        output
            .code
            .contains("  const {Provided, kit, α} = _components;\n")
    );
    assert!(output.code.contains(
        r#"  if (!Provided) _missingMdxReference("Provided", true);
  if (!kit) _missingMdxReference("kit", false);
  if (!kit.forms) _missingMdxReference("kit.forms", false);
  if (!kit.forms.Input) _missingMdxReference("kit.forms.Input", true);
  if (!α) _missingMdxReference("α", true);
  return <>"#
    ));
}

#[test]
fn configured_components_resolve_like_authored_ones() {
    let renderer = JsxRendererOptions {
        code_block_component: Some("_components.CodeBlock".to_owned()),
        ..JsxRendererOptions::default()
    }
    .with_code_block_component("mermaid", "Diagram")
    .with_callout_component("note", "Note");
    let source = "> [!NOTE]\n> Body\n\n```mermaid\ngraph TD\n```\n\n```js\nlet a = 1\n```\n";
    let options = JsxModuleOptions {
        reserved_bindings: ["Diagram".to_owned()].into(),
        ..provider()
    };
    let output = render(source, renderer, &options).expect("module");
    assert!(output.code.contains("  const {Note} = _components;\n"));
    assert!(
        output
            .code
            .contains("  if (!Note) _missingMdxReference(\"Note\", true);\n")
    );
    // A member of the components object needs no destructuring.
    assert!(
        output
            .code
            .contains("  if (!_components.CodeBlock) _missingMdxReference(\"CodeBlock\", true);\n")
    );
    assert!(!output.code.contains("!Diagram"));
    assert!(output.code.contains("<Diagram language={\"mermaid\"}>"));
    assert_eq!(output.code_blocks.len(), 2);
}

#[test]
fn reserved_names_and_duplicate_layouts_are_errors() {
    for name in [
        "_components",
        "_createMdxContent",
        "_missingMdxReference",
        "_provideComponents",
        "MDXLayout",
        "MDXContent",
        "Caller",
    ] {
        let source = format!("Intro\n\nexport const {name} = 1\n");
        let options = JsxModuleOptions {
            reserved_bindings: ["Caller".to_owned()].into(),
            ..provider()
        };
        let error = render(&source, JsxRendererOptions::default(), &options).unwrap_err();
        let JsxModuleError::ReservedBinding { name: found, span } = &error else {
            panic!("{error:?}");
        };
        assert_eq!(found, name);
        assert_eq!(&source[span.start as usize..span.end as usize], name);
    }

    let error = render(
        "import _components from './a.js'\n\ntext",
        JsxRendererOptions::default(),
        &provider(),
    )
    .unwrap_err();
    assert!(matches!(error, JsxModuleError::ReservedBinding { .. }));

    let error = render(
        "export default A\n\ntext\n\nexport { B as default }\n",
        JsxRendererOptions::default(),
        &provider(),
    )
    .unwrap_err();
    assert!(matches!(
        error,
        JsxModuleError::DuplicateDefaultExport { .. }
    ));

    let renderer = JsxRendererOptions {
        component_prefix: Some("custom".to_owned()),
        ..JsxRendererOptions::default()
    };
    assert_eq!(
        render("text", renderer, &provider()).unwrap_err(),
        JsxModuleError::ComponentPrefix
    );
}

#[test]
fn authored_javascript_maps_to_its_own_utf16_column() {
    let line = "  const note = \"😀\"; throw new Error(\"broken 🧪\");";
    let source =
        format!("# Intro \"😀\"\n\nexport function fail() {{\n{line}\n}}\n\n# Body {{value}}\n");
    let output = module(&source, &provider());
    let column = utf16_length(&line[..line.find("throw").expect("throw")]);
    assert_eq!(mapped(&output, "throw new Error"), (3, column));
    assert_eq!(mapped(&output, "export function fail"), (2, 0));
    // Generated nodes map to the start of their Markdown source.
    assert_eq!(mapped(&output, "<_components.h1 id={\"intro\"}"), (0, 0));
    assert_eq!(mapped(&output, "<_components.h1 id={\"body\"}"), (6, 0));
    assert_eq!(mapped(&output, "value}</_components.h1>"), (6, 8));
}

#[test]
fn a_rewritten_default_export_maps_to_its_statement() {
    let source = "Intro\n\nexport const a = 1\nexport default ({ children }) => children\n";
    let output = module(source, &provider());
    assert_eq!(mapped(&output, "const MDXLayout = "), (3, 0));
    assert_eq!(mapped(&output, "({ children }) => children"), (3, 15));
}

#[test]
fn modules_are_valid_javascript_across_documents() {
    for source in [
        "",
        "import Badge from './badge.js';\n\n# <Badge>Visible</Badge>\n\n<section {...props}><ui.Badge label=\"a &amp; b\" /></section>",
        "<Panel>\n  ## Nested\n\n  Markdown **children** and {/}/.test(value) ? `a${value}` : null}.\n</Panel>",
        "export const meta = {\n  title: `multi\n\nline`,\n}\n\nText {(() => {\n  // } is a comment\n  return 1;\n})()} after.",
        "| A | B |\n| :- | -: |\n| One | Two |\n\n> [!WARNING] Title\n> Body.\n\n```js title=\"Example\"\nconst text = \"<safe>\";\n```",
        "export { default } from './layout.js'\n\r\nWindows\r\nlines\r\n",
    ] {
        module(source, &provider());
    }
}
