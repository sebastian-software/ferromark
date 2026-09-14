use ferromark_allocator::Allocator;
use ferromark_ast::Node;
use ferromark_parser::ParserOptions;

use super::{flatten_text, parse_with_options};

#[test]
fn superscript_and_subscript_are_opt_in_and_outside_gfm() {
    let allocator = Allocator::new();
    let gfm = parse_with_options(&allocator, "H~2~O and x^2^", ParserOptions::gfm());
    let Node::Paragraph(paragraph) = &gfm.children[0] else {
        panic!("expected paragraph");
    };
    assert!(
        !paragraph
            .children
            .iter()
            .any(|node| matches!(node, Node::Superscript(_) | Node::Subscript(_)))
    );

    let allocator = Allocator::new();
    let options = ParserOptions { superscript: true, subscript: true, ..ParserOptions::default() };
    let doc = parse_with_options(&allocator, "H~2~O and x^2^", options);
    let Node::Paragraph(paragraph) = &doc.children[0] else {
        panic!("expected paragraph");
    };

    assert!(matches!(
        &paragraph.children[1],
        Node::Subscript(node) if node.children.iter().map(flatten_text).collect::<String>() == "2"
    ));
    assert!(paragraph.children.iter().any(|node| matches!(node, Node::Superscript(_))));
}

#[test]
fn script_spans_do_not_steal_strikethrough_or_code_spans() {
    let allocator = Allocator::new();
    let options = ParserOptions { superscript: true, subscript: true, ..ParserOptions::gfm() };
    let doc = parse_with_options(&allocator, "~~gone~~ and `x^2^`", options);
    let Node::Paragraph(paragraph) = &doc.children[0] else {
        panic!("expected paragraph");
    };

    assert!(paragraph.children.iter().any(|node| matches!(node, Node::Delete(_))));
    assert!(
        paragraph
            .children
            .iter()
            .any(|node| matches!(node, Node::InlineCode(code) if code.value == "x^2^"))
    );
}

#[test]
fn script_span_closing_delimiters_ignore_code_spans() {
    let allocator = Allocator::new();
    let options = ParserOptions { superscript: true, subscript: true, ..ParserOptions::default() };
    let doc = parse_with_options(&allocator, "^a `^ code` b^ and ~H `~ code` O~", options);
    let Node::Paragraph(paragraph) = &doc.children[0] else {
        panic!("expected paragraph");
    };

    let Node::Superscript(superscript) = &paragraph.children[0] else {
        panic!("expected superscript, got {:?}", paragraph.children[0]);
    };
    assert_eq!(superscript.children.iter().map(flatten_text).collect::<String>(), "a ^ code b");
    assert!(
        superscript
            .children
            .iter()
            .any(|node| matches!(node, Node::InlineCode(code) if code.value == "^ code"))
    );

    let Node::Subscript(subscript) = &paragraph.children[2] else {
        panic!("expected subscript, got {:?}", paragraph.children[2]);
    };
    assert_eq!(subscript.children.iter().map(flatten_text).collect::<String>(), "H ~ code O");
    assert!(
        subscript
            .children
            .iter()
            .any(|node| matches!(node, Node::InlineCode(code) if code.value == "~ code"))
    );
}

#[test]
fn text_punctuation_is_preserved_in_every_profile() {
    for options in [
        ParserOptions::commonmark(),
        ParserOptions::gfm(),
        ParserOptions::gfm_spec(),
        ParserOptions::mdx(),
    ] {
        let allocator = Allocator::new();
        let source =
            "\"Quoted\" -- --- ... '90s 'tis don't 'quoted' „Deutsch“ «Français» `\"raw\" --`";
        let doc = parse_with_options(&allocator, source, options);
        assert_eq!(
            flatten_text(&doc.children[0]),
            "\"Quoted\" -- --- ... '90s 'tis don't 'quoted' „Deutsch“ «Français» \"raw\" --"
        );
    }
}

#[test]
fn link_text_preserves_authored_punctuation() {
    let allocator = Allocator::new();
    let options = ParserOptions { autolinks: true, ..ParserOptions::default() };
    let doc = parse_with_options(
        &allocator,
        "https://example.com/a--b and [\"label\"](/x) -- ok",
        options,
    );
    let Node::Paragraph(paragraph) = &doc.children[0] else {
        panic!("expected paragraph");
    };

    let Node::Link(autolink) = &paragraph.children[0] else {
        panic!("expected GFM autolink, got {:?}", paragraph.children[0]);
    };
    assert_eq!(autolink.url, "https://example.com/a--b");
    assert_eq!(autolink.children.iter().map(flatten_text).collect::<String>(), autolink.url);

    let Node::Link(authored_link) = &paragraph.children[2] else {
        panic!("expected authored link, got {:?}", paragraph.children[2]);
    };
    assert_eq!(authored_link.children.iter().map(flatten_text).collect::<String>(), "\"label\"");
    assert_eq!(flatten_text(&doc.children[0]), "https://example.com/a--b and \"label\" -- ok");
}

#[test]
fn math_nodes_are_opt_in_and_preserve_escaped_dollars() {
    let allocator = Allocator::new();
    let off = parse_with_options(&allocator, "Energy: $E=mc^2$", ParserOptions::default());
    assert!(!matches!(
        &off.children[0],
        Node::Paragraph(paragraph)
            if paragraph.children.iter().any(|node| matches!(node, Node::InlineMath(_)))
    ));

    let allocator = Allocator::new();
    let options = ParserOptions { math: true, ..ParserOptions::default() };
    let doc = parse_with_options(&allocator, "Energy: $E=mc^2$ and \\$5", options.clone());
    let Node::Paragraph(paragraph) = &doc.children[0] else {
        panic!("expected paragraph");
    };
    assert!(
        paragraph
            .children
            .iter()
            .any(|node| matches!(node, Node::InlineMath(math) if math.value == "E=mc^2"))
    );
    assert_eq!(flatten_text(&doc.children[0]), "Energy: E=mc^2 and $5");

    let block = parse_with_options(&allocator, "$$\na + b\n$$\n", options);
    assert!(matches!(&block.children[0], Node::MathBlock(math) if math.value == "\na + b\n"));
}

#[test]
fn digit_prefixed_inline_math_preserves_emphasis_markers_as_tex() {
    let allocator = Allocator::new();
    let options = ParserOptions { math: true, ..ParserOptions::default() };

    for (source, expected) in
        [("$2*3*4 = 24$", "2*3*4 = 24"), ("$2_3_4$", "2_3_4"), ("$a_1 * b_2$", "a_1 * b_2")]
    {
        let doc = parse_with_options(&allocator, source, options.clone());
        let Node::Paragraph(paragraph) = &doc.children[0] else {
            panic!("expected paragraph for {source}");
        };
        assert!(matches!(
            &paragraph.children[0],
            Node::InlineMath(math) if math.value == expected
        ));
    }

    let money = parse_with_options(&allocator, "Costs $5 today", options);
    assert_eq!(flatten_text(&money.children[0]), "Costs $5 today");
}

#[test]
fn inline_math_closing_delimiters_ignore_code_spans() {
    let allocator = Allocator::new();
    let options = ParserOptions { math: true, ..ParserOptions::default() };
    let doc = parse_with_options(&allocator, "before $a `$ code` b$ after", options);
    let Node::Paragraph(paragraph) = &doc.children[0] else {
        panic!("expected paragraph");
    };

    assert!(matches!(
        &paragraph.children[1],
        Node::InlineMath(math) if math.value == "a `$ code` b"
    ));
    assert_eq!(flatten_text(&doc.children[0]), "before a `$ code` b after");
}

#[test]
fn definition_lists_parse_terms_definitions_and_block_body() {
    let allocator = Allocator::new();
    let options = ParserOptions { definition_lists: true, ..ParserOptions::default() };
    let source = "HTTP\n: Hypertext **Transfer** Protocol\n\nTCP\n: Transmission\n    - reliable\n";
    let doc = parse_with_options(&allocator, source, options);

    let Node::DefinitionList(list) = &doc.children[0] else {
        panic!("expected definition list, got {:?}", doc.children[0]);
    };
    assert!(matches!(
        &list.children[0],
        Node::DefinitionListTerm(term)
            if term.children.iter().map(flatten_text).collect::<String>() == "HTTP"
    ));
    assert!(matches!(
        &list.children[1],
        Node::DefinitionListDefinition(definition)
            if definition.children.iter().map(flatten_text).collect::<String>()
                == "Hypertext Transfer Protocol"
    ));
    assert!(matches!(
        &list.children[2],
        Node::DefinitionListTerm(term)
            if term.children.iter().map(flatten_text).collect::<String>() == "TCP"
    ));
    assert!(
        matches!(&list.children[3], Node::DefinitionListDefinition(definition) if definition.children.iter().any(|node| matches!(node, Node::List(_))))
    );
}
