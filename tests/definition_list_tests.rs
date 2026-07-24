use ferromark::{BlockEvent, BlockParser, Options, to_html_with_options};

fn options() -> Options {
    Options {
        definition_lists: true,
        ..Options::default()
    }
}

fn render(input: &str) -> String {
    to_html_with_options(input, &options())
}

#[test]
fn definition_lists_are_disabled_by_default() {
    assert_eq!(
        to_html_with_options("Term\n: Definition", &Options::default()),
        "<p>Term\n: Definition</p>\n"
    );
}

#[test]
fn basic_definition_list_renders_semantic_html() {
    assert_eq!(
        render("Term\n: Definition"),
        "<dl>\n<dt>Term</dt>\n<dd>Definition</dd>\n</dl>\n"
    );
}

#[test]
fn terms_and_descriptions_support_inline_markdown() {
    assert_eq!(
        render("**Term**\n: A *definition*"),
        "<dl>\n<dt><strong>Term</strong></dt>\n<dd>A <em>definition</em></dd>\n</dl>\n"
    );
}

#[test]
fn multiple_terms_and_definitions_share_one_list() {
    let input = "\
Term one
Term *two*
: First definition
: Second definition

Term three
: Third definition";
    let html = render(input);

    assert_eq!(html.matches("<dl>").count(), 1);
    assert_eq!(html.matches("<dt>").count(), 3);
    assert_eq!(html.matches("<dd>").count(), 3);
    assert!(html.contains("<dt>Term <em>two</em></dt>"));
    assert!(html.contains("<dd>Second definition</dd>"));
}

#[test]
fn blank_before_definition_makes_its_paragraph_loose() {
    assert_eq!(
        render("Term\n\n: Definition"),
        "<dl>\n<dt>Term</dt>\n<dd>\n<p>Definition</p>\n</dd>\n</dl>\n"
    );
}

#[test]
fn descriptions_support_lazy_continuation() {
    assert_eq!(
        render("Term\n: First line\nlazy continuation"),
        "<dl>\n<dt>Term</dt>\n<dd>First line\nlazy continuation</dd>\n</dl>\n"
    );
}

#[test]
fn descriptions_support_multiple_blocks_and_nested_lists() {
    let input = "\
Term
: First paragraph.

    Second paragraph.

    - nested item";
    let html = render(input);

    assert_eq!(
        html,
        "<dl>\n<dt>Term</dt>\n<dd>First paragraph.<p>Second paragraph.</p>\n\n<ul>\n<li>nested item</li>\n</ul>\n</dd>\n</dl>\n"
    );
}

#[test]
fn ordinary_paragraphs_after_a_definition_list_stay_outside_the_dl() {
    assert_eq!(
        render("Term\n: Definition\n\nOrdinary paragraph."),
        "<dl>\n<dt>Term</dt>\n<dd>Definition</dd>\n</dl>\n<p>Ordinary paragraph.</p>\n"
    );
    assert_eq!(
        render("Term\n: Definition\n\nOrdinary paragraph.\n\n# Heading"),
        "<dl>\n<dt>Term</dt>\n<dd>Definition</dd>\n</dl>\n<p>Ordinary paragraph.</p>\n<h1 id=\"heading\">Heading</h1>\n"
    );
}

#[test]
fn definition_lists_work_inside_blockquotes_and_list_items() {
    assert_eq!(
        render("> Term\n> : Definition"),
        "<blockquote>\n<dl>\n<dt>Term</dt>\n<dd>Definition</dd>\n</dl>\n</blockquote>\n"
    );
    assert_eq!(
        render("- Term\n  : Definition"),
        "<ul>\n<li><dl>\n<dt>Term</dt>\n<dd>Definition</dd>\n</dl>\n</li>\n</ul>\n"
    );
}

#[test]
fn descriptions_can_contain_nested_definition_lists() {
    let html = render("Outer\n: Outer definition\n\n    Inner\n    : Inner definition");

    assert_eq!(html.matches("<dl>").count(), 2);
    assert!(
        html.contains(
            "<dd>Outer definition\n<dl>\n<dt>Inner</dt>\n<dd>Inner definition</dd>\n</dl>\n</dd>"
        ),
        "{html}"
    );
}

#[test]
fn colon_prefixed_prose_and_indented_markers_stay_commonmark() {
    assert!(!render(": ordinary prose").contains("<dl>"));
    assert!(render("Term\n   : definition").contains("<dl>"));
    assert!(!render("Term\n    : indented continuation").contains("<dl>"));
    assert!(!render("Term\n\\: escaped marker").contains("<dl>"));
}

#[test]
fn parser_emits_definition_structure_as_block_events() {
    let input = b"Term\n: Definition";
    let mut parser = BlockParser::new_with_options(input, options());
    let mut events = Vec::new();
    parser.parse(&mut events);

    assert!(matches!(events[0], BlockEvent::DefinitionListStart));
    assert!(matches!(events[1], BlockEvent::DefinitionTermStart));
    assert!(matches!(events[3], BlockEvent::DefinitionTermEnd));
    assert!(matches!(
        events[4],
        BlockEvent::DefinitionDescriptionStart { tight: true }
    ));
    assert!(matches!(
        events[events.len() - 2],
        BlockEvent::DefinitionDescriptionEnd
    ));
    assert!(matches!(
        events[events.len() - 1],
        BlockEvent::DefinitionListEnd
    ));
}
