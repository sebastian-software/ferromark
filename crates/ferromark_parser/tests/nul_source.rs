//! Literal NUL is replaced before parsing; spans still refer to the caller's bytes.
use ferromark_allocator::Allocator;
use ferromark_ast::{Node, Span, Visit, walk_node};
use ferromark_parser::{Parser, ParserOptions};
use ferromark_renderer::{HtmlRenderer, HtmlRendererOptions};

fn render(source: &str, options: ParserOptions) -> String {
    let allocator = Allocator::new();
    let document = Parser::with_options(&allocator, source, options)
        .parse()
        .unwrap();
    let html = HtmlRendererOptions {
        autolink_urls: false,
        link_target_blank: false,
        autolink_target_blank: false,
        ..HtmlRendererOptions::new()
    };
    HtmlRenderer::with_options(html).render(&document)
}

#[test]
fn replaces_literal_nul_before_syntax_and_reference_resolution() {
    for source in [
        "a\0b\n",
        "`a\0b`\n",
        "```\na\0b\n```\n",
        "[x](/a\0b)\n",
        "<i title=\"a\0b\">x</i>\n",
        "<!-- a\0b -->\n",
        "[a\0b]: /url\n\n[a�b]\n",
        "[a�b]: /url\n\n[a\0b]\n",
        "> - a\0b\n",
        "a | b\n- | -\nc\0 | d\n",
        "\0\0é\0😀\0",
    ] {
        let replaced = source.replace('\0', "�");
        for options in [
            ParserOptions::default(),
            ParserOptions::gfm(),
            ParserOptions::mdx(),
        ] {
            assert_eq!(
                render(source, options.clone()),
                render(&replaced, options),
                "{source:?}"
            );
        }
    }
}

#[test]
fn nul_expansion_keeps_original_spans_through_nested_sources() {
    struct Check<'s> {
        source: &'s str,
        checked: usize,
    }
    impl<'a> Visit<'a> for Check<'_> {
        fn visit_node(&mut self, node: &Node<'a>) {
            let span = node.span();
            assert!(span.start <= span.end && span.end as usize <= self.source.len());
            assert!(self.source.is_char_boundary(span.start as usize));
            assert!(self.source.is_char_boundary(span.end as usize));
            match node {
                Node::Strong(strong) => {
                    assert_eq!(strong.span.source_text(self.source), "**b\0c**");
                }
                Node::Link(link) => assert_eq!(link.span.source_text(self.source), "[d\0e](/url)"),
                Node::MdxJsxFlowElement(element) => {
                    let ferromark_ast::MdxJsxAttributeEntry::Attribute(attribute) =
                        &element.attributes[0]
                    else {
                        panic!("attribute");
                    };
                    assert_eq!(attribute.span.source_text(self.source), "name=\"j\0\"");
                }
                _ => {}
            }
            self.checked += 1;
            walk_node(self, node);
        }
    }
    let source = "é\0 **b\0c**\n\n> - [d\0e](/url)\n\n| f | g |\n| - | - |\n| h\0 | i |\n\n<Widget name=\"j\0\" />\n";
    let allocator = Allocator::new();
    let options = ParserOptions {
        mdx: true,
        ..ParserOptions::gfm()
    };
    let document = Parser::with_options(&allocator, source, options)
        .parse()
        .unwrap();
    assert_eq!(document.span, Span::new(0, source.len() as u32));
    let mut check = Check { source, checked: 0 };
    check.visit_document(&document);
    assert!(check.checked > 10);
    assert!(render(source, ParserOptions::gfm()).contains('�'));
}

#[test]
fn clean_source_remains_borrowed() {
    let allocator = Allocator::new();
    let source = "ordinary text";
    let document = Parser::new(&allocator, source).parse().unwrap();
    let Node::Paragraph(paragraph) = &document.children[0] else {
        panic!("paragraph");
    };
    let Node::Text(text) = &paragraph.children[0] else {
        panic!("text");
    };
    assert_eq!(text.value.as_ptr(), source.as_ptr());
}
