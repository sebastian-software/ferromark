use ferromark_allocator::Allocator;
use ferromark_ast::{AlignKind, Node};
use ferromark_parser::ParserOptions;

use super::parse_with_options;

#[test]
fn table_alignment_variants_are_parsed() {
    let allocator = Allocator::new();
    let doc = parse_with_options(
        &allocator,
        "| a | b | c |\n| :-- | :-: | --: |\n| 1 | 2 | 3 |",
        ParserOptions::gfm(),
    );

    match &doc.children[0] {
        Node::Table(table) => {
            assert_eq!(table.align.len(), 3);
            assert_eq!(table.align[0], AlignKind::Left);
            assert_eq!(table.align[1], AlignKind::Center);
            assert_eq!(table.align[2], AlignKind::Right);
        }
        other => panic!("expected table, got {other:?}"),
    }
}

#[test]
fn table_header_and_delimiter_must_have_matching_valid_cells() {
    let allocator = Allocator::new();
    for source in [
        "| a | b |\n| --- |",
        "| a | b |\n| --- | --- | --- |",
        "| a | b |\n| : | --- |",
        "| a | b |\n| --:-- | --- |",
    ] {
        let doc = parse_with_options(&allocator, source, ParserOptions::gfm());
        assert!(
            matches!(doc.children.first(), Some(Node::Paragraph(_))),
            "invalid delimiter must remain paragraph text: {source:?}"
        );
    }
}

#[test]
fn table_body_rows_are_normalized_to_header_width() {
    let allocator = Allocator::new();
    let doc = parse_with_options(
        &allocator,
        "| a | b |\n| --- | --- |\n| one |\n| two | three | ignored |",
        ParserOptions::gfm(),
    );

    let Node::Table(table) = &doc.children[0] else {
        panic!("expected table, got {:?}", doc.children[0]);
    };
    assert_eq!(table.children.len(), 3);
    assert!(table.children.iter().all(|row| row.children.len() == 2));
    assert!(table.children[1].children[1].children.is_empty());
}

#[test]
fn merged_table_cells_are_opt_in() {
    let allocator = Allocator::new();
    let source = "| a | b |\n| --- | --- |\n| one ||";
    let doc = parse_with_options(&allocator, source, ParserOptions::gfm());

    let Node::Table(table) = &doc.children[0] else {
        panic!("expected table, got {:?}", doc.children[0]);
    };
    assert_eq!(table.children[1].children.len(), 2);
    assert_eq!(table.children[1].children[0].colspan, 1);
    assert_eq!(table.children[1].children[1].colspan, 1);
}

#[test]
fn merged_table_cells_cover_header_and_body_logical_columns() {
    let allocator = Allocator::new();
    let options = ParserOptions {
        merged_table_cells: true,
        ..ParserOptions::gfm()
    };
    let source = "| Name || Price |\n| --- | --- | --- |\n| Widget || 10$ |";
    let doc = parse_with_options(&allocator, source, options);

    let Node::Table(table) = &doc.children[0] else {
        panic!("expected table, got {:?}", doc.children[0]);
    };
    assert_eq!(table.align.len(), 3);
    assert_eq!(
        table.children[0]
            .children
            .iter()
            .map(|cell| cell.colspan)
            .collect::<Vec<_>>(),
        [2, 1]
    );
    assert_eq!(
        table.children[1]
            .children
            .iter()
            .map(|cell| cell.colspan)
            .collect::<Vec<_>>(),
        [2, 1]
    );
}

#[test]
fn merged_table_terminal_header_spans_do_not_create_phantom_cells() {
    let allocator = Allocator::new();
    for (source, expected_span) in [
        ("| Group ||\n| --- | --- |", 2),
        ("| Group |||\n| --- | --- | --- |", 3),
    ] {
        let options = ParserOptions {
            merged_table_cells: true,
            ..ParserOptions::gfm()
        };
        let doc = parse_with_options(&allocator, source, options);
        let Node::Table(table) = &doc.children[0] else {
            panic!("expected table, got {:?}", doc.children[0]);
        };
        assert_eq!(table.children[0].children.len(), 1);
        assert_eq!(table.children[0].children[0].colspan, expected_span);
    }
}

#[test]
fn merged_table_empty_terminal_cells_keep_their_span() {
    let allocator = Allocator::new();
    let options = ParserOptions {
        merged_table_cells: true,
        ..ParserOptions::gfm()
    };
    let source = "| ||\n| --- | --- |";
    let doc = parse_with_options(&allocator, source, options);

    let Node::Table(table) = &doc.children[0] else {
        panic!("expected table, got {:?}", doc.children[0]);
    };
    assert_eq!(table.children[0].children.len(), 1);
    assert_eq!(table.children[0].children[0].colspan, 2);
    assert!(table.children[0].children[0].children.is_empty());
}

#[test]
fn merged_table_header_spans_must_match_delimiter_width() {
    let allocator = Allocator::new();
    let options = ParserOptions {
        merged_table_cells: true,
        ..ParserOptions::gfm()
    };
    let doc = parse_with_options(
        &allocator,
        "| Name || Price |\n| --- | --- |\n| Widget | 10$ |",
        options,
    );

    assert!(matches!(doc.children.first(), Some(Node::Paragraph(_))));
}

#[test]
fn merged_table_cells_keep_whitespace_empty_cells_and_escaped_pipes() {
    let allocator = Allocator::new();
    let options = ParserOptions {
        merged_table_cells: true,
        ..ParserOptions::gfm()
    };
    let source = "| Name || Price |\n| --- | --- | --- |\n| α | | β \\|\\| γ |";
    let doc = parse_with_options(&allocator, source, options);

    let Node::Table(table) = &doc.children[0] else {
        panic!("expected table, got {:?}", doc.children[0]);
    };
    let body = &table.children[1];
    assert_eq!(
        body.children
            .iter()
            .map(|cell| cell.colspan)
            .collect::<Vec<_>>(),
        [1, 1, 1]
    );
    assert_eq!(super::flatten_text(&body.children[2].children[0]), "β || γ");
    assert!(body.children[2].span.start < body.children[2].span.end);
}

#[test]
fn merged_table_code_escaped_pipes_preserve_exact_source_spans() {
    let allocator = Allocator::new();
    let options = ParserOptions {
        merged_table_cells: true,
        ..ParserOptions::gfm()
    };
    let source = "| Name || Price |\n| --- | --- | --- |\n| `β \\|\\| γ` ||";
    let doc = parse_with_options(&allocator, source, options);

    let Node::Table(table) = &doc.children[0] else {
        panic!("expected table, got {:?}", doc.children[0]);
    };
    let cell = &table.children[1].children[0];
    assert_eq!(cell.colspan, 2);
    assert_eq!(cell.span.source_text(source), "`β \\|\\| γ`");
    let Node::InlineCode(code) = &cell.children[0] else {
        panic!("expected inline code, got {:?}", cell.children[0]);
    };
    assert_eq!(code.value, "β || γ");
    assert_eq!(code.span.source_text(source), "`β \\|\\| γ`");
}

#[test]
fn merged_table_cells_clamp_and_pad_ragged_rows() {
    let allocator = Allocator::new();
    let options = ParserOptions {
        merged_table_cells: true,
        ..ParserOptions::gfm()
    };
    let source = "| a || b |\n| --- | --- | --- |\n| wide |||| ignored |\n| short ||";
    let doc = parse_with_options(&allocator, source, options);

    let Node::Table(table) = &doc.children[0] else {
        panic!("expected table, got {:?}", doc.children[0]);
    };
    assert_eq!(
        table.children[1]
            .children
            .iter()
            .map(|cell| cell.colspan)
            .collect::<Vec<_>>(),
        [3]
    );
    assert_eq!(
        table.children[2]
            .children
            .iter()
            .map(|cell| cell.colspan)
            .collect::<Vec<_>>(),
        [2, 1]
    );
}

#[test]
fn table_accepts_pipe_less_rows_and_stops_at_block_starts() {
    let allocator = Allocator::new();
    let doc = parse_with_options(
        &allocator,
        "| a | b |\n| --- | --- |\nplain\n> quote | value",
        ParserOptions::gfm(),
    );

    let Node::Table(table) = &doc.children[0] else {
        panic!("expected table, got {:?}", doc.children[0]);
    };
    assert_eq!(table.children.len(), 2);
    assert_eq!(table.children[1].children.len(), 2);
    assert!(matches!(doc.children.get(1), Some(Node::BlockQuote(_))));
}
