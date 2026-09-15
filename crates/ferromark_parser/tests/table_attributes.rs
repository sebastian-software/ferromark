use ferromark_allocator::Allocator;
use ferromark_ast::{Node, Span, Table, Text, Visit};
use ferromark_parser::{Parser, ParserOptions};

fn options() -> ParserOptions {
    ParserOptions {
        table_attributes: true,
        merged_table_cells: true,
        ..ParserOptions::gfm()
    }
}

#[test]
fn table_metadata_accepts_caption_or_attributes_only_with_all_line_endings() {
    for newline in ["\n", "\r\n", "\r"] {
        for gap in ["", newline] {
            for caption in ["", "*Größe* "] {
                let source = format!(
                    "| A | B |{newline}| --- | --- |{newline}| merged ||{newline}{gap}: {caption}{{#preise .wide .striped}}"
                );
                let allocator = Allocator::new();
                let document = Parser::with_options(&allocator, &source, options())
                    .parse()
                    .unwrap();
                assert_eq!(document.children.len(), 1, "{source:?}");
                let Node::Table(table) = &document.children[0] else {
                    panic!("expected table");
                };
                let attributes = table.attributes.as_ref().unwrap();
                assert_eq!(attributes.id, Some("preise"));
                assert_eq!(attributes.classes.as_slice(), &["wide", "striped"]);
                assert_eq!(attributes.caption.is_empty(), caption.is_empty());
                assert_eq!(table.span, Span::new(0, source.len() as u32));
                assert_eq!(table.children.len(), 2);
                assert_eq!(table.children[1].children[0].colspan, 2);
            }
        }
    }
}

#[test]
fn table_metadata_is_opt_in_and_requires_tables() {
    let source = "| A | B |\n| --- | --- |\n\n: Caption {#prices .wide}";
    for preset in [ParserOptions::gfm(), ParserOptions::gfm_spec()] {
        let allocator = Allocator::new();
        let document = Parser::with_options(&allocator, source, preset)
            .parse()
            .unwrap();
        let Node::Table(table) = &document.children[0] else {
            panic!("expected table");
        };
        assert!(table.attributes.is_none());
        assert!(matches!(document.children[1], Node::Paragraph(_)));
    }
    let allocator = Allocator::new();
    let document = Parser::with_options(
        &allocator,
        source,
        ParserOptions {
            table_attributes: true,
            merged_table_cells: true,
            ..ParserOptions::default()
        },
    )
    .parse()
    .unwrap();
    assert!(
        document
            .children
            .iter()
            .all(|node| !matches!(node, Node::Table(_)))
    );
    for preset in [
        ParserOptions::commonmark(),
        ParserOptions::gfm(),
        ParserOptions::gfm_spec(),
        ParserOptions::mdx(),
    ] {
        assert!(!preset.table_attributes && !preset.merged_table_cells);
    }
}

#[test]
fn malformed_metadata_preserves_the_following_markdown() {
    for metadata in [
        ": Caption {}",
        ": Caption {#}",
        ": Caption {.}",
        ": Caption {#one #two}",
        ": Caption {#ok style=width:50%}",
        ": Caption {onclick=run()}",
        ": Caption {#bad\"id}",
        ": Caption {.bad<class}",
        ": Caption {#a\\b}",
        ": Caption {#ok} trailing",
        ": Caption{#ok}",
        ":{#ok}",
        "    : Caption {#ok}",
        "\t: Caption {#ok}",
        ": Caption without attributes",
    ] {
        let source = format!("| A | B |\n| --- | --- |\n\n{metadata}");
        let allocator = Allocator::new();
        let document = Parser::with_options(&allocator, &source, options())
            .parse()
            .unwrap();
        let Node::Table(table) = &document.children[0] else {
            panic!("expected table");
        };
        assert!(table.attributes.is_none(), "{metadata:?}");
        assert!(document.children.len() > 1, "{metadata:?}");
        assert_eq!(table.span.end, 24);
    }
}

#[test]
fn metadata_attaches_once_and_only_to_an_adjacent_table() {
    for separator in ["\n\n", "\nparagraph\n\n", "\n# Heading\n\n"] {
        let source = format!("| A | B |\n| --- | --- |\n{separator}: {{#distant}}");
        let allocator = Allocator::new();
        let document = Parser::with_options(&allocator, &source, options())
            .parse()
            .unwrap();
        let Node::Table(table) = &document.children[0] else {
            panic!("expected table");
        };
        assert!(table.attributes.is_none());
    }
    let source = "| A | B |\n| --- | --- |\n: {#first}\n: {#second}";
    let allocator = Allocator::new();
    let document = Parser::with_options(&allocator, source, options())
        .parse()
        .unwrap();
    let Node::Table(table) = &document.children[0] else {
        panic!("expected table");
    };
    assert_eq!(table.attributes.as_ref().unwrap().id, Some("first"));
    assert!(matches!(document.children[1], Node::Paragraph(_)));
}

#[derive(Default)]
struct CaptionSpans {
    spans: Vec<Span>,
}

impl<'a> Visit<'a> for CaptionSpans {
    fn visit_table(&mut self, table: &Table<'a>) {
        let attributes = table.attributes.as_ref().expect("caption metadata");
        for child in &attributes.caption {
            self.visit_node(child);
        }
    }
    fn visit_text(&mut self, text: &Text<'a>) {
        if text.value == "Größe" {
            self.spans.push(text.span);
        }
    }
}

#[test]
fn caption_spans_map_back_through_containers_and_normalization() {
    for source in [
        "| A | B |\n| --- | --- |\n\n: *Größe* {#id}",
        "> | A | B |\n> | --- | --- |\n>\n> : *Größe* {#id}",
        "- | A | B |\n  | --- | --- |\n\n  : *Größe* {#id}",
        "<Panel>\n\n| A | B |\n| --- | --- |\n\n: *Größe* {#id}\n\n</Panel>",
        "\u{feff}| A\0 | B |\n| --- | --- |\n\n: *Größe* {#id}",
    ] {
        let allocator = Allocator::new();
        let document = Parser::with_options(
            &allocator,
            source,
            ParserOptions {
                mdx: true,
                ..options()
            },
        )
        .parse()
        .unwrap();
        let mut visitor = CaptionSpans::default();
        visitor.visit_document(&document);
        assert_eq!(visitor.spans.len(), 1, "{source:?}");
        assert_eq!(visitor.spans[0].source_text(source), "Größe", "{source:?}");
    }
}

#[test]
fn default_ast_visitor_includes_caption_text() {
    struct TextCollector(String);
    impl<'a> Visit<'a> for TextCollector {
        fn visit_text(&mut self, text: &Text<'a>) {
            self.0.push_str(text.value);
        }
    }
    let allocator = Allocator::new();
    let source = "| A | B |\n| --- | --- |\n: Caption {#id}";
    let document = Parser::with_options(&allocator, source, options())
        .parse()
        .unwrap();
    let mut visitor = TextCollector(String::new());
    visitor.visit_document(&document);
    assert_eq!(visitor.0, "CaptionAB");
}
