use ferromark::ast::{Image, Node};
use ferromark::{Allocator, HtmlRenderer, JsxRenderer, Parser, ParserOptions};

fn image<'tree, 'a>(nodes: &'tree [Node<'a>]) -> &'tree Image<'a> {
    match &nodes[0] {
        Node::Image(value) => value,
        Node::Paragraph(value) => image(&value.children),
        Node::BlockQuote(value) => image(&value.children),
        Node::List(value) => image(&value.children[0].children),
        Node::MdxJsxFlowElement(value) => image(&value.children),
        other => panic!("expected image container, got {other:?}"),
    }
}

#[test]
fn retains_authored_markup_without_changing_flattening_or_output() {
    for (alt, expected) in [
        ("Code `sample`", "Code sample"),
        ("Code sample", "Code sample"),
        ("é **bold *deep*** &amp; \\*", "é bold deep & *"),
        ("", ""),
    ] {
        for source in [
            format!("![{alt}](./sample.png)"),
            format!("![{alt}][ref]\n\n[ref]: ./sample.png"),
            format!("![{alt}][]\n\n[{alt}]: ./sample.png"),
            format!("![{alt}]\n\n[{alt}]: ./sample.png"),
        ] {
            // Empty shortcut/collapsed reference labels cannot define images.
            if alt.is_empty() && !source.contains("](./") && !source.contains("][ref]") {
                continue;
            }
            let arena = Allocator::new();
            let ordinary = Parser::new(&arena, &source).parse().unwrap();
            let analyzed = Parser::with_options(
                &arena,
                &source,
                ParserOptions {
                    image_alt_content: true,
                    ..ParserOptions::default()
                },
            )
            .parse()
            .unwrap();
            let before = image(&ordinary.children);
            let after = image(&analyzed.children);
            assert_eq!(before.alt, expected, "{source}");
            assert_eq!(after.alt, before.alt);
            assert!(before.alt_content.is_none());
            let content = after.alt_content.as_ref().unwrap();
            assert_eq!(content.span.source_text(&source), alt);
            assert_eq!(content.children.is_empty(), alt.is_empty());
            if alt == "Code `sample`" {
                let Node::InlineCode(code) = &content.children[1] else {
                    panic!("expected code")
                };
                assert_eq!(code.value, "sample");
                assert_eq!(code.span.source_text(&source), "`sample`");
            }
            if alt.starts_with('é') {
                let Node::Strong(strong) = &content.children[1] else {
                    panic!("expected strong")
                };
                assert_eq!(strong.span.source_text(&source), "**bold *deep***");
                assert!(
                    strong
                        .children
                        .iter()
                        .any(|node| matches!(node, Node::Emphasis(_)))
                );
                assert!(content.children.iter().any(|node| matches!(node, Node::Text(text) if text.value == "&" && text.span.source_text(&source) == "&amp;")));
                assert!(content.children.iter().any(|node| matches!(node, Node::Text(text) if text.value == "*" && text.span.source_text(&source) == "\\*")));
            }
            assert_eq!(
                HtmlRenderer::new().render(&ordinary),
                HtmlRenderer::new().render(&analyzed)
            );
            assert_eq!(
                JsxRenderer::new().render(&ordinary, &source).body,
                JsxRenderer::new().render(&analyzed, &source).body
            );
        }
    }
}

#[test]
fn maps_alt_children_through_containers_and_line_endings() {
    for source in [
        "- ![é `sample`](u)",
        "> ![é `sample`](u)",
        "> - ![é `sample`](u)",
        "> ![é\n> `sample`](u)",
        "> ![é\r\n> `sample`](u)",
        "\u{feff}> ![é `sample`](u)",
        "\0\n\n> ![é `sample`](u)",
    ] {
        let arena = Allocator::new();
        let document = Parser::with_options(
            &arena,
            source,
            ParserOptions {
                image_alt_content: true,
                ..ParserOptions::default()
            },
        )
        .parse()
        .unwrap();
        let nodes = if source.starts_with('\0') {
            &document.children[1..]
        } else {
            &document.children[..]
        };
        let content = image(nodes).alt_content.as_ref().unwrap();
        let code = content
            .children
            .iter()
            .find_map(|node| match node {
                Node::InlineCode(code) => Some(code),
                _ => None,
            })
            .unwrap();
        assert_eq!(code.span.source_text(source), "`sample`", "{source:?}");
        assert_eq!(content.span.start as usize, source.find("é").unwrap());
        assert_eq!(content.span.end as usize, source.find("](u)").unwrap());
    }
}

#[cfg(feature = "jsx")]
#[test]
fn retains_expressions_and_spans_inside_mdx_containers() {
    let source = "<Card>\r\n  > ![é `code` {label} <Badge />](u)\r\n</Card>\r\n";
    let arena = Allocator::new();
    let options = ParserOptions {
        mdx: true,
        mdx_compatible: true,
        ..ParserOptions::gfm()
    };
    let ordinary = Parser::with_options(&arena, source, options.clone())
        .parse()
        .unwrap();
    let analyzed = Parser::with_options(
        &arena,
        source,
        ParserOptions {
            image_alt_content: true,
            ..options
        },
    )
    .parse()
    .unwrap();
    let value = image(&analyzed.children);
    let content = value.alt_content.as_ref().unwrap();
    assert!(content.children.iter().any(|node| matches!(node, Node::MdxTextExpression(expr) if expr.span.source_text(source) == "{label}")));
    assert!(content.children.iter().any(|node| matches!(node, Node::MdxJsxTextElement(element) if element.span.source_text(source) == "<Badge />")));
    assert_eq!(value.alt, image(&ordinary.children).alt);
    assert_eq!(
        JsxRenderer::new().render(&ordinary, source).body,
        JsxRenderer::new().render(&analyzed, source).body
    );
}

#[test]
fn nested_image_metadata_is_remapped_with_the_outer_alt() {
    let source = "> ![outer ![inner `code`](inner.png)](outer.png)";
    let arena = Allocator::new();
    let document = Parser::with_options(
        &arena,
        source,
        ParserOptions {
            image_alt_content: true,
            ..ParserOptions::default()
        },
    )
    .parse()
    .unwrap();
    let outer = image(&document.children);
    assert_eq!(outer.alt, "outer inner code");
    let content = outer.alt_content.as_ref().unwrap();
    let Node::Image(inner) = &content.children[1] else {
        panic!("expected nested image")
    };
    assert_eq!(inner.span.source_text(source), "![inner `code`](inner.png)");
    let Node::InlineCode(code) = &inner.alt_content.as_ref().unwrap().children[1] else {
        panic!("expected code")
    };
    assert_eq!(code.span.source_text(source), "`code`");
}
