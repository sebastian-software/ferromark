//! A leading UTF-8 BOM is parser metadata, while later BOMs remain content.
use ferromark_allocator::Allocator;
use ferromark_ast::{Node, Span};
use ferromark_parser::Parser;

#[test]
fn strips_one_leading_bom_and_preserves_original_spans() {
    let source = "\u{feff}BOM and nbsp\u{00a0}space\n";
    let allocator = Allocator::new();
    let document = Parser::new(&allocator, source).parse().unwrap();

    assert_eq!(document.span, Span::new(0, source.len() as u32));
    let Node::Paragraph(paragraph) = &document.children[0] else {
        panic!("expected paragraph");
    };
    let Node::Text(text) = &paragraph.children[0] else {
        panic!("expected text");
    };
    assert_eq!(text.value, "BOM and nbsp\u{00a0}space");
    assert_eq!(text.span.source_text(source), text.value);
    assert_eq!(text.span.start, 3);
}

#[test]
fn embedded_bom_and_nul_remain_content_after_leading_bom() {
    let source = "\u{feff}a\u{feff} b\0c\n";
    let allocator = Allocator::new();
    let document = Parser::new(&allocator, source).parse().unwrap();

    assert_eq!(document.span, Span::new(0, source.len() as u32));
    let Node::Paragraph(paragraph) = &document.children[0] else {
        panic!("expected paragraph");
    };
    let Node::Text(text) = &paragraph.children[0] else {
        panic!("expected text");
    };
    assert_eq!(text.value, "a\u{feff} b�c");
    assert_eq!(text.span.source_text(source), "a\u{feff} b\0c");
    assert_eq!(text.span.start, 3);
    assert!(source.is_char_boundary(text.span.end as usize));
}

#[test]
fn bom_only_input_has_an_original_document_span() {
    let source = "\u{feff}";
    let allocator = Allocator::new();
    let document = Parser::new(&allocator, source).parse().unwrap();

    assert!(document.children.is_empty());
    assert_eq!(document.span, Span::new(0, source.len() as u32));
}

#[test]
fn strips_only_the_first_of_two_boms() {
    let source = "\u{feff}\u{feff}content\n";
    let allocator = Allocator::new();
    let document = Parser::new(&allocator, source).parse().unwrap();

    assert_eq!(document.span, Span::new(0, source.len() as u32));
    let Node::Paragraph(paragraph) = &document.children[0] else {
        panic!("expected paragraph");
    };
    let Node::Text(text) = &paragraph.children[0] else {
        panic!("expected text");
    };
    assert_eq!(text.value, "\u{feff}content");
    assert_eq!(text.span.source_text(source), text.value);
    assert_eq!(text.span.start, 3);
}

#[test]
fn bom_before_nested_link_and_nul_keeps_link_source_span() {
    let source = "\u{feff}> [x\0y](/url)\n";
    let allocator = Allocator::new();
    let document = Parser::new(&allocator, source).parse().unwrap();

    assert_eq!(document.span, Span::new(0, source.len() as u32));
    let Node::BlockQuote(quote) = &document.children[0] else {
        panic!("expected block quote");
    };
    let Node::Paragraph(paragraph) = &quote.children[0] else {
        panic!("expected paragraph");
    };
    let Node::Link(link) = &paragraph.children[0] else {
        panic!("expected link");
    };
    assert_eq!(link.span.source_text(source), "[x\0y](/url)");
    assert_eq!(link.span.start, 5);
}
