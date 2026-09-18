//! CommonMark forbids a link inside a link, so `parse_link` parses the
//! bracket text before it can tell whether the outer bracket is a link at
//! all. When it is not, the bracket stays literal and the caller re-scans
//! the same bytes — so every nesting level used to parse its inner text
//! twice, and `[[[[a](/u)](/u)]...` cost 2^depth.
//!
//! Bracket text that holds another bracket is now parsed once, where it
//! stands, and the bracket matches one walk finds are kept for the walks
//! below it (issue #350). Before that, every level probed its whole text and
//! then parsed it again, and every level walked for its own `]`: nesting a
//! link N deep cost O(N^3), so 8 KB of `[`x1600 `a` `](u)`x1600 took 2.5 s
//! and 25 KB took minutes.
//!
//! These tests pin all of it: the run has to finish, the cost has to grow
//! with the document rather than with the nesting, and the output has to
//! stay exactly the nesting the spec asks for — down to the text nodes,
//! since a link's children and a literal run either side of it are the same
//! bytes in a different shape.

use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use ferromark::allocator::Allocator;
use ferromark::parser::{Parser, ParserOptions};
use ferromark::renderer::HtmlRenderer;

#[path = "support/pretty.rs"]
mod pretty;

/// Generous enough that a slow shared runner never trips it, and far below
/// what the old exponential path needed: at depth 64 that path was 2^44
/// times the depth-20 cost, which already measured 45 ms.
const BUDGET: Duration = Duration::from_secs(15);

fn render(source: &str, options: ParserOptions) -> String {
    let allocator = Allocator::new();
    let document = Parser::with_options(&allocator, source, options)
        .parse()
        .expect("source should parse");
    HtmlRenderer::new().render(&document).trim().to_string()
}

/// Parses `source` on a worker thread so a regression fails the suite in
/// bounded time instead of hanging it. Returns the best of three parses, so a
/// scheduling stall on a busy runner has to hit every repetition to fail.
fn parse_within_budget(source: String) -> Duration {
    let (best, parsed) = outcome_within_budget(source);
    assert!(parsed, "nested brackets should parse to a document");
    best
}

/// The same bounded parse, without requiring a document: past the nesting
/// cap the expected answer is an error, and it still has to arrive quickly.
fn outcome_within_budget(source: String) -> (Duration, bool) {
    outcome_within_budget_with(source, ParserOptions::gfm())
}

/// The same, under a caller's options — the nesting cap has to be lifted to
/// measure how the cost grows past it.
fn outcome_within_budget_with(source: String, options: ParserOptions) -> (Duration, bool) {
    let mut best = BUDGET;
    let mut parsed_all = true;
    for _ in 0..3 {
        let owned = source.clone();
        let options = options.clone();
        let (sender, receiver) = mpsc::channel();
        thread::spawn(move || {
            let started = Instant::now();
            let allocator = Allocator::new();
            let parsed = Parser::with_options(&allocator, &owned, options)
                .parse()
                .is_ok();
            let _ = sender.send((parsed, started.elapsed()));
        });
        let (parsed, elapsed) = receiver
            .recv_timeout(BUDGET)
            .expect("nested brackets should parse in bounded time, not exponential time");
        parsed_all &= parsed;
        best = best.min(elapsed);
    }
    (best, parsed_all)
}

fn nested_inline_links(depth: usize) -> String {
    "[".repeat(depth) + "a" + &"](/u)".repeat(depth)
}

fn nested_reference_links(depth: usize) -> String {
    "[".repeat(depth) + "a" + &"][r]".repeat(depth) + "\n\n[r]: /r\n"
}

/// One paragraph of nested-link groups, at least `bytes` long. Each group
/// closes before the next one opens, so the document holds many independent
/// nestings of the same depth — the shape a hostile document takes once the
/// cap refuses one deep nesting.
fn nested_link_groups(depth: usize, bytes: usize) -> String {
    let group = nested_inline_links(depth);
    let mut out = String::with_capacity(bytes + group.len());
    while out.len() < bytes {
        out.push_str(&group);
    }
    out
}

/// The nesting cap lifted, so a shape past it measures its own cost instead
/// of the refusal. `0` is the documented "no limit".
fn uncapped() -> ParserOptions {
    ParserOptions {
        max_nesting_depth: 0,
        ..ParserOptions::gfm()
    }
}

/// Parses past the cap on a worker with room for the recursion, and reports
/// the best of three. Debug frames are several times release frames, so the
/// size is the debug suite's, not the parser's bound.
fn parse_uncapped_within_budget(source: String) -> Duration {
    let mut best = BUDGET;
    for _ in 0..3 {
        let owned = source.clone();
        let (sender, receiver) = mpsc::channel();
        thread::Builder::new()
            .stack_size(64 << 20)
            .spawn(move || {
                let started = Instant::now();
                let allocator = Allocator::new();
                let parsed = Parser::with_options(&allocator, &owned, uncapped())
                    .parse()
                    .is_ok();
                let _ = sender.send((parsed, started.elapsed()));
            })
            .expect("worker thread");
        let (parsed, elapsed) = receiver
            .recv_timeout(BUDGET)
            .expect("nested brackets should parse in bounded time, not cubic time");
        assert!(parsed, "nested brackets should parse to a document");
        best = best.min(elapsed);
    }
    best
}

fn ratio(large: Duration, small: Duration) -> f64 {
    large.as_secs_f64() / small.as_secs_f64().max(1e-9)
}

#[test]
fn deeply_nested_inline_links_finish_in_bounded_time() {
    parse_within_budget(nested_inline_links(64));
}

#[test]
fn deeply_nested_reference_links_finish_in_bounded_time() {
    parse_within_budget(nested_reference_links(64));
}

#[test]
fn deeply_nested_unclosed_brackets_finish_in_bounded_time() {
    // The closing run is one short, so no bracket ever becomes a link and
    // every level takes the literal-text fallback.
    parse_within_budget("[".repeat(64) + "a" + &"](/u)".repeat(63));
}

#[test]
fn deeply_nested_images_in_links_finish_in_bounded_time() {
    parse_within_budget("[![".repeat(48) + "a" + &"](/i)](/u)".repeat(48));
}

#[test]
fn nested_bracket_cost_grows_far_slower_than_it_doubles() {
    // Each added level used to double the work. Sixteen more levels would
    // therefore cost 65536x; anything under 100x proves the doubling is
    // gone without pinning an absolute time on a shared runner.
    let shallow = parse_within_budget(nested_inline_links(32)).max(Duration::from_micros(1));
    let deep = parse_within_budget(nested_inline_links(48));
    assert!(
        deep < shallow * 100,
        "depth 48 took {deep:?} against {shallow:?} at depth 32; the doubling is back"
    );
}

#[test]
fn only_the_innermost_inline_link_survives() {
    assert_eq!(
        render("[a [b [c](/c)](/b)](/a)", ParserOptions::gfm()),
        "<p>[a [b <a href=\"/c\">c</a>](/b)](/a)</p>"
    );
}

#[test]
fn only_the_innermost_reference_link_survives() {
    assert_eq!(
        render(
            "[a [b [c][ref]][ref]][ref]\n\n[ref]: /r",
            ParserOptions::gfm()
        ),
        "<p>[a [b <a href=\"/r\">c</a>]<a href=\"/r\">ref</a>]<a href=\"/r\">ref</a></p>"
    );
}

#[test]
fn bracket_text_without_a_link_still_becomes_link_children() {
    // The reused probe nodes have to be the same children a fresh parse
    // would have produced, brackets and emphasis included.
    assert_eq!(
        render("[a [b] *c*](/u)", ParserOptions::gfm()),
        "<p><a href=\"/u\">a [b] <em>c</em></a></p>"
    );
}

#[test]
fn an_image_inside_link_text_is_still_allowed() {
    assert_eq!(
        render("[![alt](img.png)](/u)", ParserOptions::gfm()),
        "<p><a href=\"/u\"><img src=\"img.png\" alt=\"alt\"></a></p>"
    );
}

#[test]
fn identical_bracket_text_is_judged_per_occurrence() {
    // The memoized verdict is keyed by the slice, so repeating the same
    // characters in a different position must not inherit an answer.
    assert_eq!(
        render(
            "[same](/1) and [same](/2) and [[same](/3)](/4)",
            ParserOptions::gfm()
        ),
        concat!(
            "<p><a href=\"/1\">same</a> and <a href=\"/2\">same</a> ",
            "and [<a href=\"/3\">same</a>](/4)</p>"
        )
    );
}

#[test]
fn nesting_inside_a_block_quote_and_a_list_item_behaves_the_same() {
    // Sub-parsers get their own cache; the verdict must not change.
    assert_eq!(
        render("> [a [b](/b)](/a)", ParserOptions::gfm()),
        "<blockquote>\n<p>[a <a href=\"/b\">b</a>](/a)</p>\n</blockquote>"
    );
    assert_eq!(
        render("- [a [b](/b)](/a)", ParserOptions::gfm()),
        "<ul>\n<li>[a <a href=\"/b\">b</a>](/a)</li>\n</ul>"
    );
}

#[test]
fn nesting_past_the_cap_fails_closed_in_bounded_time() {
    // Depth 64 above is well inside `max_nesting_depth`; this is the other
    // side of it. Without the inline bound this input did not run long, it
    // overflowed the stack and aborted the test binary (issue #349).
    let (_, parsed) = outcome_within_budget(nested_inline_links(20_000));
    assert!(!parsed, "20,000 levels should be refused, not parsed");

    let allocator = Allocator::new();
    let error = Parser::with_options(
        &allocator,
        &nested_inline_links(20_000),
        ParserOptions::gfm(),
    )
    .parse()
    .expect_err("20,000 levels are past the cap");
    assert!(
        matches!(
            error.kind(),
            ferromark::parser::ParseErrorKind::NestingTooDeep { max_depth: 100, .. }
        ),
        "expected the nesting cap, got {error}"
    );
}

#[test]
fn nested_link_groups_cost_the_same_per_byte_at_any_depth() {
    // Two documents of the same size, one nesting four times as deep. The
    // per-level probe made the cost per byte grow with the depth, so the
    // deep one measured 53 ms against 9 ms for the shallow one at 32 KiB;
    // parsing each level once makes them cost the same per byte.
    let shallow = parse_within_budget(nested_link_groups(25, 32 * 1024));
    let deep = parse_within_budget(nested_link_groups(100, 32 * 1024));

    let ratio = ratio(deep, shallow);
    assert!(
        ratio < 3.0,
        "depth 100 took {deep:?} against {shallow:?} at depth 25 for the same 32 KiB (x{ratio:.1}); \
         the same cost per byte is about x1, the per-level probe about x6"
    );
}

#[test]
fn deep_nesting_finishes_when_the_cap_is_lifted() {
    // The cap refuses this shape, so lifting it is the only way to see what
    // the parser does with the nesting itself. Depth 3200 cost 8 times
    // depth 1600, which measured 2.5 s in a release build: cubic, and far
    // past the budget here. The stack is the parser's own bound at this
    // depth — `max_nesting_depth` is what keeps a default parse off it —
    // so the worker gets room for the recursion it opted into.
    let elapsed = parse_uncapped_within_budget(nested_inline_links(3_200));
    assert!(elapsed < BUDGET, "depth 3200 took {elapsed:?}");
}

#[test]
fn a_run_of_openers_with_one_closer_costs_linear_time() {
    // `[`xN `a](u)`: one bracket closes, so every opener before it walked to
    // the end of the content looking for its own `]`. 100 KB took 2.0 s and
    // grew x4 for every x2 of input.
    let small = parse_within_budget("[".repeat(32 * 1024) + "a](u)");
    let large = parse_within_budget("[".repeat(128 * 1024) + "a](u)");

    let ratio = ratio(large, small);
    assert!(
        ratio < 8.0,
        "128 Ki openers took {large:?} against {small:?} for 32 Ki (x{ratio:.1}); \
         linear is about x4, quadratic about x16"
    );
}

#[test]
fn the_literal_bracket_fallback_keeps_its_node_shape() {
    // The bracket text is now parsed where it stands, so the run after the
    // inner link has to stay one text node that reaches past the outer `]`,
    // exactly as re-parsing from the literal bracket produced it. Splitting
    // it at the bracket would render the same and shift every span after it.
    assert_eq!(
        tree("[[a](/a)x](/b)"),
        "Document [0..14]\n  Paragraph [0..14]\n    Text \"[\" [0..1]\n    \
         Link url=\"/a\" title=None [1..8]\n      Text \"a\" [2..3]\n    \
         Text \"x](/b)\" [8..14]\n"
    );
    // Same shape one level further in, where the trailing run spans two
    // closing groups.
    assert_eq!(
        tree("[[[a](/a)](/b)](/c)"),
        "Document [0..19]\n  Paragraph [0..19]\n    Text \"[\" [0..1]\n    \
         Text \"[\" [1..2]\n    Link url=\"/a\" title=None [2..9]\n      \
         Text \"a\" [3..4]\n    Text \"](/b)](/c)\" [9..19]\n"
    );
    // Bracket text that holds no link keeps the same children a fresh parse
    // produced, brackets and all.
    assert_eq!(
        tree("[a [b] c](/u)"),
        "Document [0..13]\n  Paragraph [0..13]\n    Link url=\"/u\" title=None [0..13]\n      \
         Text \"a \" [1..3]\n      Text \"[\" [3..4]\n      Text \"b] c\" [4..8]\n"
    );
}

#[test]
fn a_destination_that_reaches_past_the_outer_bracket_still_decides_the_run() {
    // The brackets inside a region are balanced, but a destination is not:
    // the inner `(u]x)` closes after the outer `]`, so the region cannot be
    // decided where it stands and the probe has to settle it.
    for (source, expected) in [
        ("[[a](u]x)]", r#"<p>[<a href="u%5Dx">a</a>]</p>"#),
        ("[[a](u]x)](v)", r#"<p>[<a href="u%5Dx">a</a>](v)</p>"#),
        (
            "[[a](u \"t]\")](v)",
            r#"<p>[<a href="u" title="t]">a</a>](v)</p>"#,
        ),
    ] {
        assert_eq!(
            render(source, ParserOptions::gfm()),
            expected,
            "for {source:?}"
        );
    }
}

#[test]
fn markup_inside_bracket_text_keeps_pairing_across_the_bracket() {
    // Emphasis pairs across a bracket, so bracket text holding a delimiter
    // is not a region that can be decided on its own. These are the shapes
    // that prove the parser still reads them in the surrounding context.
    for (source, expected) in [
        ("[*[a](/a)]*", r#"<p>[<em><a href="/a">a</a>]</em></p>"#),
        (
            "[[a](/a)*b*](/c)",
            r#"<p>[<a href="/a">a</a><em>b</em>](/c)</p>"#,
        ),
        (
            "[*a* [b](/b)](/c)",
            r#"<p>[<em>a</em> <a href="/b">b</a>](/c)</p>"#,
        ),
        (
            "[`[a](/a)`](/c)",
            r#"<p><a href="/c"><code>[a](/a)</code></a></p>"#,
        ),
        ("[[a](/a)\n](/c)", "<p>[<a href=\"/a\">a</a>\n](/c)</p>"),
    ] {
        assert_eq!(
            render(source, ParserOptions::gfm()),
            expected,
            "for {source:?}"
        );
    }
}

fn tree(source: &str) -> String {
    let allocator = Allocator::new();
    let document = Parser::with_options(&allocator, source, ParserOptions::gfm())
        .parse()
        .expect("source should parse");
    let mut out = String::new();
    pretty::format_document(&document, source, &mut out);
    out
}
