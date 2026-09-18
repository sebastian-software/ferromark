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
//!
//! The delimiter-run cases are issue #371, and they fail one step later:
//! pairing `*`×2n `a` `*`×2n is iterative, so the parse itself survived, but
//! the tree it built was n levels deep and the HTML renderer and the public
//! `ast::visit` walkers recurse over it. 4 KB aborted a 1 MB stack and 40 KB
//! an 8 MB one, so those cases render what they parse rather than stopping
//! at the parse.

use std::thread;

use ferromark::allocator::Allocator;
use ferromark::ast::{Node, Visit, walk_node};
use ferromark::parser::{ParseError, ParseErrorKind, Parser, ParserOptions};
use ferromark::renderer::{HtmlRenderer, HtmlRendererOptions};

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

/// Parses *and renders*, which is what the delimiter-run cases need: the
/// tree pairing builds is walked recursively by the renderer and by the
/// public visitor, and that walk is where a too-deep tree aborts.
fn render_result(source: &str, options: ParserOptions) -> Result<(), ParseError> {
    let allocator = Allocator::new();
    let document = Parser::with_options(&allocator, source, options).parse()?;
    let mut renderer = HtmlRenderer::with_options(HtmlRendererOptions::new());
    let html = renderer.render(&document);
    assert!(!html.is_empty(), "every source here renders something");
    // The public walkers recurse over the same tree, so exercise them too.
    let mut depth = EmphasisDepth::default();
    depth.visit_document(&document);
    Ok(())
}

/// How deep the emphasis-like containers nest — the depth `process_emphasis`
/// bounds, measured on the finished tree rather than assumed from the input.
#[derive(Default)]
struct EmphasisDepth {
    current: usize,
    max: usize,
}

impl<'a> Visit<'a> for EmphasisDepth {
    fn visit_node(&mut self, node: &Node<'a>) {
        let nests = matches!(
            node,
            Node::Emphasis(_) | Node::Strong(_) | Node::Delete(_) | Node::Highlight(_)
        );
        if nests {
            self.current += 1;
            self.max = self.max.max(self.current);
        }
        walk_node(self, node);
        if nests {
            self.current -= 1;
        }
    }
}

fn emphasis_depth(source: &str, options: ParserOptions) -> usize {
    let allocator = Allocator::new();
    let document = Parser::with_options(&allocator, source, options)
        .parse()
        .expect("the source is inside the cap");
    let mut depth = EmphasisDepth::default();
    depth.visit_document(&document);
    depth.max
}

fn assert_too_deep(label: &str, source: &str, options: ParserOptions) {
    expect_too_deep(label, parse_result(source, options));
}

fn expect_too_deep(label: &str, outcome: Result<(), ParseError>) {
    match outcome {
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
    expect_bounded(label, parse_result(source, options));
}

/// The same property for a source that has to survive rendering as well.
fn assert_bounded_rendering(label: &str, source: &str, options: ParserOptions) {
    expect_bounded(label, render_result(source, options));
}

fn expect_bounded(label: &str, outcome: Result<(), ParseError>) {
    match outcome {
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

/// The delimiter-run shapes from issue #371: a marker repeated, one letter,
/// and the mirrored closers. `*`, `_` and `**` nest in every profile;
/// `==`, `^` and `~` need their extension, and in this shape they collapse
/// into one long run that opens nothing — which is exactly why they are
/// here, since "settles" is the property, not "is refused".
fn delimiter_run_shapes(count: usize) -> Vec<(&'static str, String)> {
    let mirrored = |marker: &str| marker.repeat(count) + "a" + &marker.repeat(count);
    vec![
        ("stars", mirrored("*")),
        ("underscores", mirrored("_")),
        ("double stars", mirrored("**")),
        ("highlight runs", mirrored("==")),
        ("superscript runs", mirrored("^")),
        ("subscript runs", mirrored("~")),
        ("strikethrough runs", mirrored("~~")),
        (
            // East Asian punctuation on the outside, which `cjk_emphasis`
            // reclassifies so that these runs may pair at all.
            "cjk emphasis",
            "。".to_string() + &"*".repeat(count) + "強" + &"*".repeat(count) + "。",
        ),
    ]
}

/// Runs separated by text, one level per run pair. `==` and `~~` runs are
/// capped at two characters, so a single run cannot nest and only this
/// shape reaches `Highlight` and `Delete` nesting.
fn nested_run_shapes(depth: usize) -> Vec<(&'static str, String)> {
    let nested = |marker: &str| {
        format!(" {marker}x").repeat(depth) + " a " + &format!("x{marker} ").repeat(depth)
    };
    vec![
        ("nested stars", nested("*")),
        ("nested highlight", nested("==")),
        ("nested strikethrough", nested("~~")),
        ("nested tildes", nested("~")),
    ]
}

fn shape(shapes: Vec<(&'static str, String)>, label: &str) -> String {
    shapes
        .into_iter()
        .find(|(name, _)| *name == label)
        .expect("the shape is in the list")
        .1
}

#[test]
fn deeply_nested_emphasis_fails_closed() {
    // 40 KB of `*` built a 10,000-level tree that aborted an 8 MB stack;
    // 4 KB was enough for a 1 MB one.
    for (profile, options) in [
        ("default", ParserOptions::default()),
        ("gfm", ParserOptions::gfm()),
        ("all options", all_options()),
    ] {
        for label in ["stars", "underscores", "double stars", "cjk emphasis"] {
            let source = shape(delimiter_run_shapes(INLINE_OVER_LIMIT), label);
            assert_too_deep(&format!("{profile}: {label}"), &source, options.clone());
        }
    }
}

#[test]
fn deep_delimiter_runs_stay_bounded_in_every_profile() {
    // The extension markers depend on the profile: with the extension off
    // they are literal text, and in this shape they are one long run even
    // with it on. Either way the document has to settle and render.
    for (profile, options) in [
        ("default", ParserOptions::default()),
        ("gfm", ParserOptions::gfm()),
        ("all options", all_options()),
    ] {
        for (label, source) in delimiter_run_shapes(INLINE_OVER_LIMIT) {
            assert_bounded_rendering(&format!("{profile}: {label}"), &source, options.clone());
        }
        for (label, source) in nested_run_shapes(INLINE_OVER_LIMIT) {
            assert_bounded_rendering(&format!("{profile}: {label}"), &source, options.clone());
        }
    }
}

#[test]
fn deep_delimiter_runs_survive_a_small_stack() {
    // The renderer walk is the one that used to abort, so these render as
    // well as parse. Returning at all is the result being checked.
    let worker = thread::Builder::new()
        .stack_size(SMALL_STACK)
        .spawn(|| {
            for (profile, options) in [
                ("default", ParserOptions::default()),
                ("gfm", ParserOptions::gfm()),
                ("all options", all_options()),
            ] {
                for (label, source) in delimiter_run_shapes(INLINE_OVER_LIMIT) {
                    assert_bounded_rendering(
                        &format!("{SMALL_STACK} stack, {profile}: {label}"),
                        &source,
                        options.clone(),
                    );
                }
                for (label, source) in nested_run_shapes(INLINE_OVER_LIMIT) {
                    assert_bounded_rendering(
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
fn the_emphasis_cap_counts_nesting_levels_exactly() {
    // Two delimiter characters per level of strong emphasis, so 200 of them
    // are the documented cap and 202 are one past it — the same counting as
    // the block and bracket bounds.
    let at_cap = "*".repeat(200) + "a" + &"*".repeat(200);
    assert_eq!(
        emphasis_depth(&at_cap, ParserOptions::gfm()),
        100,
        "100 levels are the documented cap, not one past it"
    );
    assert_too_deep(
        "101 levels of emphasis",
        &("*".repeat(202) + "a" + &"*".repeat(202)),
        ParserOptions::gfm(),
    );

    // The same boundary for the markers that only nest through separate
    // runs, which is where `Highlight` and `Delete` nodes come from.
    for (label, options) in [
        ("nested highlight", all_options()),
        ("nested strikethrough", ParserOptions::gfm()),
        ("nested tildes", ParserOptions::gfm()),
    ] {
        assert_eq!(
            emphasis_depth(&shape(nested_run_shapes(100), label), options.clone()),
            100,
            "{label}: 100 levels are inside the cap"
        );
        assert_too_deep(
            &format!("{label}: 101 levels"),
            &shape(nested_run_shapes(101), label),
            options,
        );
    }
}

#[test]
fn flat_delimiter_runs_are_not_nesting() {
    // Depth is the depth of the tree, not the length of a run: a run with
    // nothing to close it builds no node at all, and pairs that sit next to
    // one another are all one level deep however many there are.
    let unclosed = "*".repeat(200_000) + "a";
    assert_eq!(
        emphasis_depth(&unclosed, ParserOptions::gfm()),
        0,
        "a run with no closer stays literal text"
    );
    let adjacent = "*a* ".repeat(INLINE_OVER_LIMIT);
    assert_eq!(
        emphasis_depth(&adjacent, ParserOptions::gfm()),
        1,
        "adjacent pairs are siblings, not nesting"
    );
}

#[test]
fn emphasis_and_bracket_nesting_share_the_inline_budget() {
    // Brackets and emphasis stack up along one path — an emphasis node
    // inside link text, image alt or JSX phrasing is that much deeper than
    // the tree it sits in — so unlike blocks and inline they are counted
    // against the limit together. 50 JSX elements leave room for 50 levels
    // of emphasis; the letter in front keeps them phrasing rather than the
    // flow elements a line of their own would make, which are blocks and
    // are counted as blocks.
    let jsx = |elements: usize, delimiters: usize| {
        "x".to_string()
            + &"<A>".repeat(elements)
            + &"*".repeat(delimiters)
            + "a"
            + &"*".repeat(delimiters)
            + &"</A>".repeat(elements)
    };
    assert_eq!(
        emphasis_depth(&jsx(50, 100), ParserOptions::mdx()),
        50,
        "50 elements plus 50 emphasis levels are exactly the cap"
    );
    assert_too_deep(
        "51 emphasis levels under 50 elements",
        &jsx(50, 102),
        ParserOptions::mdx(),
    );

    // Link text counts the same way, although CommonMark keeps only the
    // innermost bracket as a link: what the bound has to fit is the parse's
    // own recursion, as for the bracket cap itself.
    let links = |levels: usize, delimiters: usize| {
        "[".repeat(levels)
            + &"*".repeat(delimiters)
            + "a"
            + &"*".repeat(delimiters)
            + &"](/u)".repeat(levels)
    };
    assert!(
        parse_result(&links(50, 100), ParserOptions::gfm()).is_ok(),
        "50 bracket levels plus 50 emphasis levels are exactly the cap"
    );
    assert_too_deep(
        "51 emphasis levels under 50 brackets",
        &links(50, 102),
        ParserOptions::gfm(),
    );

    // The mixtures the cap exists for: 100 levels of a nesting construct,
    // each holding 100 levels of emphasis, is 40 KB that used to build a
    // 10,000-level tree and abort.
    for (label, source) in [
        (
            "jsx",
            ("x<A>".to_string() + &"*".repeat(200)).repeat(100)
                + "a"
                + &("*".repeat(200) + "</A>").repeat(100),
        ),
        (
            "images",
            ("![".to_string() + &"*".repeat(200)).repeat(100)
                + "a"
                + &("*".repeat(200) + "](u)").repeat(100),
        ),
        (
            "inline notes",
            ("^[".to_string() + &"*".repeat(200)).repeat(100)
                + "a"
                + &("*".repeat(200) + "]").repeat(100),
        ),
    ] {
        assert_too_deep(&format!("all options: {label}"), &source, all_options());
    }
}

#[test]
fn a_custom_cap_bounds_emphasis_too() {
    let options = ParserOptions {
        max_nesting_depth: 3,
        ..ParserOptions::gfm()
    };
    match parse_result(&("*".repeat(8) + "a" + &"*".repeat(8)), options.clone()) {
        Err(error)
            if matches!(
                error.kind(),
                ParseErrorKind::NestingTooDeep { max_depth: 3, .. }
            ) => {}
        other => panic!("expected the custom cap to bite, got {other:?}"),
    }
    // Three levels are the cap itself and must still parse.
    assert_eq!(
        emphasis_depth(&("*".repeat(6) + "a" + &"*".repeat(6)), options),
        3
    );
}

#[test]
fn zero_still_means_unlimited_for_emphasis() {
    // The documented escape hatch, on the one construct that now counts
    // where it did not before. Modest depth: with the cap off, the caller
    // owns the stack again.
    let options = ParserOptions {
        max_nesting_depth: 0,
        ..ParserOptions::gfm()
    };
    assert_eq!(
        emphasis_depth(&("*".repeat(400) + "a" + &"*".repeat(400)), options),
        200
    );
}
