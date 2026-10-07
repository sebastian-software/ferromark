#![cfg(feature = "jsx")]

use ferromark::allocator::Allocator;
use ferromark::ast::{MdxJsxAttributeEntry, MdxJsxAttributeValue, Node};
use ferromark::parser::{ParseErrorKind, Parser, ParserOptions};

fn options() -> ParserOptions {
    ParserOptions {
        mdx: true,
        mdx_compatible: true,
        ..ParserOptions::gfm_spec()
    }
}

#[test]
fn intrinsic_member_and_custom_elements_keep_attributes_and_markdown() {
    let source = "<section {/* spread */ ...props} data-label=\"test\" onClick={() => /}/.test(value)}>\n  ## Inside\n\n  **Markdown** and <ui.Badge title={`a${/}/.source}`} />.\n\n  <custom-element />\n</section>\n";
    let allocator = Allocator::new();
    let document = Parser::with_options(&allocator, source, options())
        .parse()
        .unwrap();
    let Node::MdxJsxFlowElement(section) = &document.children[0] else {
        panic!("expected intrinsic JSX")
    };
    assert_eq!(section.name, Some("section"));
    let MdxJsxAttributeEntry::Expression(spread) = &section.attributes[0] else {
        panic!("expected spread")
    };
    assert_eq!(spread.value, "/* spread */ ...props");
    let MdxJsxAttributeEntry::Attribute(click) = &section.attributes[2] else {
        panic!("expected onClick")
    };
    let Some(MdxJsxAttributeValue::Expression(expression)) = &click.value else {
        panic!("expected expression")
    };
    assert_eq!(expression.value, "() => /}/.test(value)");
    assert!(matches!(&section.children[0], Node::Heading(heading) if heading.depth == 2));
    let Node::Paragraph(paragraph) = &section.children[1] else {
        panic!("expected paragraph")
    };
    assert!(matches!(&paragraph.children[0], Node::Strong(_)));
    assert!(paragraph.children.iter().any(
        |node| matches!(node, Node::MdxJsxTextElement(element) if element.name == Some("ui.Badge"))
    ));
    assert!(
        matches!(&section.children[2], Node::MdxJsxFlowElement(element) if element.name == Some("custom-element"))
    );
}

#[test]
fn regex_templates_and_comments_do_not_end_expressions_early() {
    for expression in [
        " /}/.test(value) ",
        " /[};]/.source ",
        " `outer ${`inner ${/}/.source}`} tail` ",
        " ({pattern: /}/, value: `a${value}`}) ",
        " /* a } inside a comment */ ",
        " value // } is a comment\n ",
    ] {
        let source = format!("before {{{expression}}} after\n");
        let allocator = Allocator::new();
        let document = Parser::with_options(&allocator, &source, options())
            .parse()
            .unwrap();
        let Node::Paragraph(paragraph) = &document.children[0] else {
            panic!("expected paragraph")
        };
        assert!(
            paragraph.children.iter().any(
                |node| matches!(node, Node::MdxTextExpression(node) if node.value == expression)
            ),
            "{source}"
        );
    }
}

#[test]
fn multiline_module_blocks_preserve_regex_and_blank_lines_inside_javascript() {
    let esm = "import {\n  first,\n  second\n} from './values.js';\nexport const pattern = /[};]/;\nexport const label = `first\n\n    second ${`${pattern.source}`}\n`;\nexport function example() {\n\n  return /}/;\n}\nexport const commented = /* a\n\n comment } */ /;/;\nexport const conditional = condition\n  ? /[};]/\n  : /}/;\n";
    let source = format!("{esm}\n# After\n");
    let allocator = Allocator::new();
    let document = Parser::with_options(&allocator, &source, options())
        .parse()
        .unwrap();
    assert_eq!(document.children.len(), 2);
    assert!(matches!(&document.children[0], Node::MdxjsEsm(node) if node.value == esm.trim_end()));
    assert!(matches!(&document.children[1], Node::Heading(_)));
}

#[test]
fn jsx_dedent_preserves_multiline_template_bytes_and_spans() {
    let source = "<Panel>\n  ## Heading\n\n  {`first\n\n      exact indentation ${`nested ${value}`}\n  last`}\n\n  Text before {`inline\n\n      # still JavaScript\n    end`} after.\n</Panel>\n";
    let allocator = Allocator::new();
    let document = Parser::with_options(&allocator, source, options())
        .parse()
        .unwrap();
    let Node::MdxJsxFlowElement(panel) = &document.children[0] else {
        panic!("expected Panel")
    };
    assert!(matches!(&panel.children[0], Node::Heading(_)));
    let Node::MdxFlowExpression(expression) = &panel.children[1] else {
        panic!("expected flow expression")
    };
    assert_eq!(
        expression.value,
        "`first\n\n      exact indentation ${`nested ${value}`}\n  last`"
    );
    assert!(
        source[expression.span.start as usize..expression.span.end as usize]
            .contains(expression.value)
    );
    let Node::Paragraph(paragraph) = &panel.children[2] else {
        panic!("expected paragraph")
    };
    assert!(paragraph.children.iter().any(|node| matches!(node, Node::MdxTextExpression(expression) if expression.value == "`inline\n\n      # still JavaScript\n    end`")));
}

#[test]
fn indented_unicode_esm_span_is_exactly_its_preserved_source() {
    let source = "  export const café = '日本語';  \n\n# After\n";
    let allocator = Allocator::new();
    let document = Parser::with_options(&allocator, source, options())
        .parse()
        .unwrap();
    let Node::MdxjsEsm(esm) = &document.children[0] else {
        panic!("expected ESM")
    };
    assert_eq!(esm.value, "export const café = '日本語';");
    assert_eq!(
        &source[esm.span.start as usize..esm.span.end as usize],
        esm.value
    );
}

#[test]
fn invalid_javascript_and_jsx_report_source_errors() {
    for source in [
        "{foo +}",
        "{...props}",
        "{/*comment*/...props}",
        "text {foo +} after",
        "<div value={foo +} />",
        "<div {props} />",
        "<div><span></div></span>",
        "<div>",
        "text <span",
        "</div>",
        "<div></div>\n</div>",
        "export const x = ;\n\n# Heading",
    ] {
        let allocator = Allocator::new();
        let error = Parser::with_options(&allocator, source, options())
            .parse()
            .unwrap_err();
        assert!(
            matches!(error.kind(), ParseErrorKind::InvalidMdx { .. }),
            "{source}: {error}"
        );
        assert!(
            error.span().end as usize <= source.len(),
            "{source}: {error}"
        );
        assert!(error.to_string().contains("invalid MDX"));
    }
}

#[test]
fn typescript_in_module_blocks_is_named_as_the_cause() {
    for source in [
        "import type { Props } from './types'\n\n# Heading",
        "export interface Props { label: string }\n\n# Heading",
        "export const count: number = 1\n\n# Heading",
        "export function id<T>(value: T): T {\n\n  return value\n}\n\n# Heading",
    ] {
        let allocator = Allocator::new();
        let error = Parser::with_options(&allocator, source, options())
            .parse()
            .unwrap_err();
        assert!(
            error
                .to_string()
                .contains("TypeScript syntax is not supported in MDX module blocks"),
            "{source}: {error}"
        );
    }

    // Invalid JavaScript that is also invalid TypeScript keeps its diagnostic.
    let allocator = Allocator::new();
    let error = Parser::with_options(&allocator, "export const x = ;\n\n# Heading", options())
        .parse()
        .unwrap_err();
    assert!(!error.to_string().contains("TypeScript"), "{error}");
}

#[test]
fn code_fences_hide_jsx_closers_and_javascript_braces() {
    let source = "<div>\n  ~~~jsx\n  </div>\n  {invalid +}\n  ~~~\n\n  ```jsx\n  <span>\n  </div>\n  ```\n</div>\n";
    let allocator = Allocator::new();
    let document = Parser::with_options(&allocator, source, options())
        .parse()
        .unwrap();
    let Node::MdxJsxFlowElement(element) = &document.children[0] else {
        panic!("expected div")
    };
    assert_eq!(element.children.len(), 2);
    assert!(
        element
            .children
            .iter()
            .all(|node| matches!(node, Node::CodeBlock(_)))
    );
}

#[test]
fn intrinsic_jsx_quotes_treat_backslashes_as_literal_bytes() {
    let source = r#"<section><span title="a\" α="β" /></section>"#;
    let allocator = Allocator::new();
    let document = Parser::with_options(&allocator, source, options())
        .parse()
        .unwrap();
    let Node::MdxJsxFlowElement(element) = &document.children[0] else {
        panic!("expected section")
    };
    let Node::MdxJsxFlowElement(span) = &element.children[0] else {
        panic!("expected span")
    };
    let MdxJsxAttributeEntry::Attribute(title) = &span.attributes[0] else {
        panic!("expected title")
    };
    assert!(matches!(
        title.value,
        Some(MdxJsxAttributeValue::Literal("a\\"))
    ));
}

#[test]
fn ordinary_less_than_autolinks_and_escaped_braces_stay_markdown() {
    let allocator = Allocator::new();
    let document = Parser::with_options(
        &allocator,
        "a < b and \\{literal} and <https://example.com>\n",
        options(),
    )
    .parse()
    .unwrap();
    let Node::Paragraph(paragraph) = &document.children[0] else {
        panic!("expected paragraph")
    };
    assert!(
        paragraph
            .children
            .iter()
            .any(|node| matches!(node, Node::Link(_)))
    );
    assert!(!paragraph.children.iter().any(|node| matches!(
        node,
        Node::MdxTextExpression(_) | Node::MdxJsxTextElement(_)
    )));
    assert!(
        Parser::with_options(
            &allocator,
            "<div>\n  escaped \\{invalid +} and \\<span>\n</div>\n",
            options()
        )
        .parse()
        .is_ok()
    );
}

#[test]
fn nested_mdx_diagnostics_remap_into_original_container_source() {
    let source = "before\n\n> <Panel>\n>   text {invalid +}\n> </Panel>\n";
    let allocator = Allocator::new();
    let error = Parser::with_options(&allocator, source, options())
        .parse()
        .unwrap_err();
    let span = error.span();
    assert!(span.start as usize >= source.find("{invalid").unwrap());
    assert!(span.end as usize <= source.len());
    assert!(source[span.start as usize..span.end as usize].contains('{'));
}

#[test]
fn autolinks_in_jsx_markdown_children_are_not_element_openers() {
    let allocator = Allocator::new();
    let source = "<div>\n  <https://example.com> and <a{b}@example.com>\n</div>\n";
    let document = Parser::with_options(&allocator, source, options())
        .parse()
        .unwrap();
    let Node::MdxJsxFlowElement(element) = &document.children[0] else {
        panic!("expected div")
    };
    let Node::Paragraph(paragraph) = &element.children[0] else {
        panic!("expected paragraph")
    };
    assert_eq!(
        paragraph
            .children
            .iter()
            .filter(|node| matches!(node, Node::Link(_)))
            .count(),
        2
    );
}

#[test]
fn legacy_mdx_still_keeps_intrinsic_html_and_recovers_invalid_braces() {
    let allocator = Allocator::new();
    let document = Parser::with_options(
        &allocator,
        "<div>hello</div>\n\n{invalid +}\n",
        ParserOptions::mdx(),
    )
    .parse()
    .unwrap();
    assert!(matches!(&document.children[0], Node::Html(_)));
    assert!(
        Parser::with_options(&allocator, "{unclosed", ParserOptions::mdx())
            .parse()
            .is_ok()
    );
}
