//! A nesting error raised inside a container reports a document offset.
//!
//! Block quotes, list items, footnote bodies and definition bodies parse a
//! stripped copy of their content, and an error raised in that copy used to
//! carry the copy's offsets: `intro\n\n> > > > a` under a cap of two failed
//! at `Span { 0, 0 }`. The error now takes the same map its sibling nodes
//! take on the way out, so the span points at the construct that went too
//! deep in the source the caller handed over.

use ferromark::{Allocator, ParseErrorKind, Parser, ParserOptions};

fn error_offset(source: &str, options: ParserOptions) -> usize {
    let allocator = Allocator::new();
    let Err(error) = Parser::with_options(&allocator, source, options).parse() else {
        panic!("the nesting cap should fail the parse of {source:?}");
    };
    assert!(matches!(
        error.kind(),
        ParseErrorKind::NestingTooDeep { max_depth: 2, .. }
    ));
    error.span().start as usize
}

fn shallow() -> ParserOptions {
    ParserOptions {
        max_nesting_depth: 2,
        ..ParserOptions::gfm()
    }
}

#[test]
fn block_quote_errors_point_into_the_document() {
    let source = "intro paragraph\n\n> > > > a";
    let offset = error_offset(source, shallow());
    assert!(offset > "intro paragraph\n\n".len(), "offset {offset}");
    assert!(
        source[offset..].starts_with("> a"),
        "the span should start at the quote that went too deep, got {offset}"
    );
}

#[test]
fn list_item_errors_point_into_the_document() {
    let source = "intro paragraph\n\n- - - - a";
    let offset = error_offset(source, shallow());
    assert!(source[offset..].starts_with("- a"), "offset {offset}");
}

#[test]
fn inline_errors_inside_a_container_point_into_the_document() {
    let quoted = "intro\n\n> [[[[a]]]]";
    let offset = error_offset(quoted, shallow());
    assert!(quoted[offset..].starts_with("[a]"), "offset {offset}");

    // The same bracket depth at the root lands on the same bracket.
    let plain = "intro\n\n[[[[a]]]]";
    let plain_offset = error_offset(plain, shallow());
    assert!(
        plain[plain_offset..].starts_with("[a]"),
        "offset {plain_offset}"
    );
}

#[test]
fn errors_inside_footnote_and_definition_bodies_point_into_the_document() {
    let footnote = "[^1]\n\n[^1]: > > > a";
    let offset = error_offset(footnote, shallow());
    assert!(footnote[offset..].starts_with("> a"), "offset {offset}");

    let definition = "term\n: > > > a";
    let options = ParserOptions {
        definition_lists: true,
        ..shallow()
    };
    let offset = error_offset(definition, options);
    assert!(definition[offset..].starts_with("> a"), "offset {offset}");
}
