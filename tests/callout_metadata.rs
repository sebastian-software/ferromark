#[cfg(feature = "jsx")]
use ferromark::ParserOptions;
use ferromark::ast::{BlockQuote, Node};
use ferromark::{Allocator, CalloutKind, HtmlRenderer, Parser};

fn quote<'tree, 'a>(nodes: &'tree [Node<'a>]) -> &'tree BlockQuote<'a> {
    match &nodes[0] {
        Node::BlockQuote(quote) if matches!(&quote.children[0], Node::Paragraph(_)) => quote,
        Node::BlockQuote(value) => quote(&value.children),
        Node::List(value) => quote(&value.children[0].children),
        Node::MdxJsxFlowElement(value) => quote(&value.children),
        other => panic!("expected quote, got {other:?}"),
    }
}

#[test]
fn exposes_decoded_marker_and_ast_body_boundaries() {
    for marker in [
        "[!NOTE]",
        "\\[!NOTE]",
        "&#91;!NOTE]",
        "&lbrack;!NOTE]",
        "[!&#78;OTE]",
        "[!NOTE&#93;",
        "[! note ]",
    ] {
        for source in [
            format!("> {marker}\n> Keep this **note**.\n>\n> Second block."),
            format!("- > {marker}\r\n  > Keep this **note**.\r\n  >\r\n  > Second block."),
            format!("> > {marker} \tKeep this **note**.\n> >\n> > Second block."),
        ] {
            let arena = Allocator::new();
            let document = Parser::new(&arena, &source).parse().unwrap();
            let callout = quote(&document.children)
                .callout(&source)
                .unwrap_or_else(|| panic!("{source:?}: {document:#?}"));
            assert_eq!(callout.kind, CalloutKind::Note);
            assert_eq!(
                callout.marker_span.source_text(&source),
                marker,
                "{source:?}"
            );
            let Node::Text(first) = &callout.body.inlines[0] else {
                panic!("expected body text")
            };
            assert_eq!(&first.value[callout.body.text_offset..], "Keep this ");
            assert!(matches!(callout.body.inlines[1], Node::Strong(_)));
            assert_eq!(callout.body.blocks.len(), 1, "{source:?}");
            assert!(
                HtmlRenderer::new()
                    .render(&document)
                    .contains("ox-callout--note")
            );
        }
    }
}

#[test]
fn recognizes_all_kinds_and_empty_or_formatted_body() {
    for (name, kind) in [
        ("NOTE", CalloutKind::Note),
        ("TIP", CalloutKind::Tip),
        ("IMPORTANT", CalloutKind::Important),
        ("WARNING", CalloutKind::Warning),
        ("CAUTION", CalloutKind::Caution),
    ] {
        let source = format!("> [!{name}]");
        let arena = Allocator::new();
        let document = Parser::new(&arena, &source).parse().unwrap();
        let result = quote(&document.children)
            .callout(&source)
            .unwrap_or_else(|| panic!("{source:?}: {document:#?}"));
        assert_eq!(result.kind, kind);
        assert!(result.body.inlines.is_empty());
        assert!(result.body.blocks.is_empty());
    }
    let source = "é前\r\n\r\n> [!TIP]\r\n> **body**";
    let arena = Allocator::new();
    let document = Parser::new(&arena, source).parse().unwrap();
    let result = quote(&document.children[1..])
        .callout(source)
        .unwrap_or_else(|| panic!("{source:?}: {document:#?}"));
    assert_eq!(result.marker_span.source_text(source), "[!TIP]");
    assert_eq!(result.body.text_offset, 0);
    assert!(matches!(result.body.inlines[0], Node::Strong(_)));
}

#[test]
fn ordinary_quotes_agree_with_renderer_recognition() {
    for source in [
        "> ordinary",
        "> [!UNKNOWN]\n> body",
        "> `[!NOTE]`\n> body",
        "> [!NOTE]: /u\n> body",
        "> [!NOTE]\n> ===",
        "[!note]: /u\n\n> [!NOTE]\n> body",
    ] {
        let arena = Allocator::new();
        let document = Parser::new(&arena, source).parse().unwrap();
        let Node::BlockQuote(value) = document
            .children
            .iter()
            .find(|node| matches!(node, Node::BlockQuote(_)))
            .unwrap()
        else {
            panic!("expected quote")
        };
        assert!(value.callout(source).is_none(), "{source:?}");
        assert!(
            !HtmlRenderer::new()
                .render(&document)
                .contains("ox-callout--")
        );
    }
}

#[cfg(feature = "jsx")]
#[test]
fn callout_ranges_survive_mdx_indentation_and_root_normalization() {
    let source = "\u{feff}\0\r\n\r\n<Card>\r\n  - > &#91;!TIP]\r\n    > 日本語\r\n</Card>\r\n";
    let arena = Allocator::new();
    let document = Parser::with_options(
        &arena,
        source,
        ParserOptions {
            mdx: true,
            mdx_compatible: true,
            ..ParserOptions::gfm()
        },
    )
    .parse()
    .unwrap();
    let callout = quote(&document.children[1..])
        .callout(source)
        .unwrap_or_else(|| panic!("{source:?}: {document:#?}"));
    assert_eq!(callout.marker_span.source_text(source), "&#91;!TIP]");
    let Node::Text(first) = &callout.body.inlines[0] else {
        panic!("expected text")
    };
    assert_eq!(&first.value[callout.body.text_offset..], "日本語");
}

#[test]
fn multiline_markers_keep_original_bounding_ranges() {
    for source in [
        "> [!\n> NOTE] body",
        "> [!NOTE\r\n> ] body",
        "> [!NOTE]\u{2003}日本語",
    ] {
        let arena = Allocator::new();
        let document = Parser::new(&arena, source).parse().unwrap();
        let callout = quote(&document.children).callout(source).unwrap();
        assert_eq!(callout.marker_span.start, 2);
        assert_eq!(
            callout.marker_span.end as usize,
            source.find(']').unwrap() + 1
        );
        let Node::Text(text) = &callout.body.inlines[0] else {
            panic!("expected text")
        };
        assert_eq!(
            &text.value[callout.body.text_offset..],
            if source.contains("日本語") {
                "日本語"
            } else {
                "body"
            }
        );
        assert!(
            HtmlRenderer::new()
                .render(&document)
                .contains("ox-callout--note")
        );
    }
}
