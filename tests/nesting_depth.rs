//! `max_nesting_depth` has to bound every construct that re-enters the
//! parser, not just block quotes.
//!
//! A stack overflow aborts the process instead of unwinding, so no caller
//! can recover from one. Each case here nests far past the cap; if the cap
//! stops applying to that construct the test does not fail, it kills the
//! test binary — which is the point.
//!
//! The inline cases are the ones from issue #349: link text, image alt
//! text, wiki-link labels and inline notes re-enter inline parsing once per
//! bracket level, which used to be uncounted. 16 KB of `[` aborted an 8 MB
//! stack and 2 KB aborted a 1 MB one, so the sizes here are deliberately
//! larger than anything that used to survive.

use std::thread;

use ferromark::allocator::Allocator;
use ferromark::ast::Node;
use ferromark::parser::{ParseError, ParseErrorKind, Parser, ParserOptions};

const OVER_LIMIT: usize = 500;

/// Far past the cap and past every measured overflow in issue #349.
const INLINE_OVER_LIMIT: usize = 20_000;

/// The Windows main-thread default, and the smallest stack the parser is
/// expected to survive on.
const SMALL_STACK: usize = 1 << 20;

fn parse_result(source: &str, options: ParserOptions) -> Result<(), ParseError> {
    let allocator = Allocator::new();
    Parser::with_options(&allocator, source, options)
        .parse()
        .map(|_| ())
}

fn assert_too_deep(label: &str, source: &str, options: ParserOptions) {
    match parse_result(source, options) {
        Err(error)
            if matches!(
                error.kind(),
                ParseErrorKind::NestingTooDeep { max_depth: 100, .. }
            ) => {}
        Err(other) => panic!("{label}: expected NestingTooDeep, got {other}"),
        Ok(()) => panic!("{label}: expected NestingTooDeep, parsed instead"),
    }
}

/// The weaker property, for inputs whose shape depends on the options: the
/// parse has to finish, either with a document or with the cap refusing it.
/// A construct that stopped counting would not fail this assertion, it would
/// overflow the stack and abort the binary.
fn assert_bounded(label: &str, source: &str, options: ParserOptions) {
    match parse_result(source, options) {
        Ok(()) => {}
        Err(error)
            if matches!(
                error.kind(),
                ParseErrorKind::NestingTooDeep { max_depth: 100, .. }
            ) => {}
        Err(other) => panic!("{label}: expected a document or NestingTooDeep, got {other}"),
    }
}

/// Every extension at once, so no option combination re-enters inline
/// parsing through a path the default and GFM profiles never take.
fn all_options() -> ParserOptions {
    ParserOptions {
        highlight: true,
        inline_footnotes: true,
        merged_table_cells: true,
        table_attributes: true,
        line_comments: true,
        front_matter: true,
        superscript: true,
        subscript: true,
        math: true,
        definition_lists: true,
        heading_attributes: true,
        wiki_links: true,
        cjk_emphasis: true,
        mdx: true,
        ..ParserOptions::gfm()
    }
}

/// The inline shapes from issue #349, each paired with the construct whose
/// recursion it drives.
fn inline_shapes(depth: usize) -> Vec<(&'static str, String)> {
    vec![
        ("bracket text", "[".repeat(depth) + "a" + &"]".repeat(depth)),
        (
            "inline links",
            "[".repeat(depth) + "a" + &"](u)".repeat(depth),
        ),
        ("images", "![".repeat(depth) + "a" + &"](u)".repeat(depth)),
        ("spaced brackets", "[ ".repeat(depth) + &"]".repeat(depth)),
        (
            "images in links",
            "[![".repeat(depth) + "a" + &"](i)](u)".repeat(depth),
        ),
        (
            "inline notes",
            "^[".repeat(depth) + "a" + &"]".repeat(depth),
        ),
        (
            "wiki labels",
            "[[".repeat(depth) + "a" + &"]]".repeat(depth),
        ),
    ]
}

fn nested_lists(depth: usize) -> String {
    let mut source = String::new();
    for level in 0..depth {
        source.push_str(&" ".repeat(level * 2));
        source.push_str("- item\n");
    }
    source
}

fn nested_quotes(depth: usize) -> String {
    "> ".repeat(depth) + "item\n"
}

fn nested_footnote_definitions(depth: usize) -> String {
    let mut source = String::new();
    for level in 0..depth {
        source.push_str(&" ".repeat(level * 4));
        source.push_str("[^");
        source.push_str(&level.to_string());
        source.push_str("]:\n");
    }
    source.push_str(&" ".repeat(depth * 4));
    source.push_str("body\n");
    source
}

fn nested_quotes_around_lists(depth: usize) -> String {
    let mut source = String::new();
    for level in 0..depth {
        source.push_str(&"> ".repeat(level + 1));
        source.push_str("- item\n");
    }
    source
}

#[test]
fn deeply_nested_lists_fail_closed() {
    assert_too_deep("lists", &nested_lists(OVER_LIMIT), ParserOptions::gfm());
}

#[test]
fn deeply_nested_quotes_fail_closed() {
    assert_too_deep("quotes", &nested_quotes(OVER_LIMIT), ParserOptions::gfm());
}

#[test]
fn deeply_nested_footnote_definitions_fail_closed() {
    assert_too_deep(
        "footnotes",
        &nested_footnote_definitions(OVER_LIMIT),
        ParserOptions::gfm(),
    );
}

#[test]
fn deeply_nested_quotes_around_lists_fail_closed() {
    assert_too_deep(
        "quoted lists",
        &nested_quotes_around_lists(OVER_LIMIT),
        ParserOptions::gfm(),
    );
}

#[test]
fn the_cap_applies_without_the_gfm_profile() {
    // `ParserOptions::default()` used to leave nesting unlimited, so the
    // plain-CommonMark path could take the host process down.
    assert_eq!(ParserOptions::default().max_nesting_depth, 100);
    assert_too_deep(
        "plain lists",
        &nested_lists(OVER_LIMIT),
        ParserOptions::default(),
    );
    assert_too_deep(
        "plain quotes",
        &nested_quotes(OVER_LIMIT),
        ParserOptions::default(),
    );
}

#[test]
fn the_cap_applies_in_mdx_mode() {
    assert_eq!(ParserOptions::mdx().max_nesting_depth, 100);
    assert_too_deep("mdx lists", &nested_lists(OVER_LIMIT), ParserOptions::mdx());
}

#[test]
fn nesting_within_the_cap_still_parses() {
    let allocator = Allocator::new();
    let source = nested_lists(40);
    let document = Parser::with_options(&allocator, &source, ParserOptions::gfm())
        .parse()
        .expect("40 levels are well inside the cap");

    // Walk the whole chain so the depth is measured, not assumed.
    let mut node = document.children.first().expect("one root list");
    let mut levels = 1;
    while let Node::List(list) = node {
        let item = list.children.first().expect("each list has an item");
        match item
            .children
            .iter()
            .find(|child| matches!(child, Node::List(_)))
        {
            Some(inner) => {
                node = inner;
                levels += 1;
            }
            None => break,
        }
    }
    assert_eq!(
        levels, 40,
        "every source level should survive as a nested list"
    );
}

#[test]
fn a_custom_cap_is_honored_for_every_construct() {
    let options = ParserOptions {
        max_nesting_depth: 3,
        ..ParserOptions::gfm()
    };
    for (label, source) in [
        ("lists", nested_lists(12)),
        ("quotes", nested_quotes(12)),
        ("footnotes", nested_footnote_definitions(12)),
    ] {
        match parse_result(&source, options.clone()) {
            Err(error)
                if matches!(
                    error.kind(),
                    ParseErrorKind::NestingTooDeep { max_depth: 3, .. }
                ) => {}
            other => panic!("{label}: expected the custom cap to bite, got {other:?}"),
        }
    }
    assert!(parse_result(&nested_lists(2), options.clone()).is_ok());
    assert!(parse_result(&nested_quotes(2), options).is_ok());
}

#[test]
fn zero_still_means_unlimited() {
    // Documented escape hatch: shallow input must keep parsing with it.
    let options = ParserOptions {
        max_nesting_depth: 0,
        ..ParserOptions::gfm()
    };
    assert!(parse_result(&nested_lists(120), options.clone()).is_ok());
    assert!(parse_result(&nested_quotes(120), options).is_ok());
}

#[test]
fn deeply_nested_inline_brackets_fail_closed() {
    for (profile, options) in [
        ("default", ParserOptions::default()),
        ("gfm", ParserOptions::gfm()),
    ] {
        for (label, source) in inline_shapes(INLINE_OVER_LIMIT) {
            assert_too_deep(&format!("{profile}: {label}"), &source, options.clone());
        }
    }
}

#[test]
fn deeply_nested_inline_brackets_stay_bounded_with_every_extension() {
    // Two of these shapes parse to a document with every extension on:
    // `wiki_links` consumes `[[...]]` up to the first `]]`, so the label it
    // hands to inline parsing holds openers with nothing to close them and
    // nothing nests. Both outcomes are fine — the property under test is
    // that a 40 KB document settles one way or the other instead of
    // aborting the process.
    for (label, source) in inline_shapes(INLINE_OVER_LIMIT) {
        assert_bounded(&format!("all options: {label}"), &source, all_options());
    }
}

#[test]
fn deeply_nested_inline_brackets_survive_a_small_stack() {
    // The cap is worth having only if the recursion it permits fits in the
    // smallest stack the parser is expected to run on. A stack overflow
    // aborts the whole test binary rather than failing this thread, so
    // returning at all is the result being checked.
    let worker = thread::Builder::new()
        .stack_size(SMALL_STACK)
        .spawn(|| {
            for (profile, options) in [
                ("default", ParserOptions::default()),
                ("gfm", ParserOptions::gfm()),
                ("all options", all_options()),
            ] {
                for (label, source) in inline_shapes(INLINE_OVER_LIMIT) {
                    assert_bounded(
                        &format!("{SMALL_STACK} stack, {profile}: {label}"),
                        &source,
                        options.clone(),
                    );
                }
            }
        })
        .expect("the worker thread should start");
    worker.join().expect("a 1 MB stack should be enough");
}

#[test]
fn the_inline_cap_counts_bracket_levels_exactly() {
    // One nesting level per bracket, so the cap allows exactly as many as
    // it says and the next one fails — the same counting as the block bound.
    let at_cap = "[".repeat(100) + "a" + &"](/u)".repeat(100);
    assert!(
        parse_result(&at_cap, ParserOptions::gfm()).is_ok(),
        "100 levels are the documented cap, not one past it"
    );
    assert_too_deep(
        "101 levels",
        &("[".repeat(101) + "a" + &"](/u)".repeat(101)),
        ParserOptions::gfm(),
    );
}

#[test]
fn block_depth_and_inline_depth_are_counted_separately() {
    // The two counts never stack up along one path — a block container
    // cannot appear inside inline content — so a document at the block cap
    // still parses ordinary links, and nesting them is bounded on its own.
    let quoted = "> ".repeat(100) + "[a [b](/b)](/u)\n";
    assert!(
        parse_result(&quoted, ParserOptions::gfm()).is_ok(),
        "the block cap must not spend the inline budget"
    );
    assert_too_deep(
        "quoted deep brackets",
        &("> ".repeat(100) + &"[".repeat(200) + "a" + &"](/u)".repeat(200) + "\n"),
        ParserOptions::gfm(),
    );
}

#[test]
fn inline_nesting_within_the_cap_still_parses() {
    let allocator = Allocator::new();
    // 64 bracket levels, the depth `nested_links.rs` pins, stay well inside
    // the cap: the bound must only refuse what no document asks for.
    let source = "[".repeat(64) + "a" + &"](/u)".repeat(64);
    let document = Parser::with_options(&allocator, &source, ParserOptions::gfm())
        .parse()
        .expect("64 levels are well inside the cap");

    // CommonMark keeps only the innermost link, so walk to it and count.
    let mut node = document.children.first().expect("one paragraph");
    let mut levels = 0;
    loop {
        let children = match node {
            Node::Paragraph(paragraph) => &paragraph.children,
            Node::Link(link) => {
                levels += 1;
                &link.children
            }
            _ => break,
        };
        match children.iter().find(|child| matches!(child, Node::Link(_))) {
            Some(inner) => node = inner,
            None => break,
        }
    }
    assert_eq!(levels, 1, "only the innermost bracket becomes a link");
}

#[test]
fn a_custom_cap_bounds_inline_nesting_too() {
    let options = ParserOptions {
        max_nesting_depth: 3,
        ..ParserOptions::gfm()
    };
    for (label, source) in inline_shapes(12) {
        match parse_result(&source, options.clone()) {
            Err(error)
                if matches!(
                    error.kind(),
                    ParseErrorKind::NestingTooDeep { max_depth: 3, .. }
                ) => {}
            other => panic!("{label}: expected the custom cap to bite, got {other:?}"),
        }
    }
    // One level of link text is inside a cap of three and must still parse.
    assert!(parse_result("[a [b](/b)](/a)", options).is_ok());
}
