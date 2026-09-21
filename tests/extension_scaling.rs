//! Three opt-in extensions answered "nothing closes this" by reading to the
//! end of the document, once for every opener, so ordinary text cost O(n²).
//!
//! Measured on the release build before the fix, at 32 KiB and 128 KiB —
//! x16 for every x4 of input, which is the signature:
//!
//! - math: `$a ` 0.37 s / 5.9 s, `$1 ` 0.38 s / 6.2 s, `$$a ` 0.54 s /
//!   6.7 s, the same runs with their only closing `$` inside a code span
//!   0.47 s / 6.0 s, and `$$ a` lines 0.29 s / 3.7 s;
//! - MDX: `<A>` with one closer behind the run 0.72 s / 9.4 s, the same
//!   inline 0.72 s / 11.4 s, `<A>` over a `{` run 0.41 s / 5.3 s and over
//!   one with a `}` behind it 0.83 s / 10.7 s (0.98 s / 17.5 s when the tag
//!   closes too), a brace run with one `}` behind it 0.33 s / 5.4 s,
//!   0.33 s / 5.3 s inline, and an unclosed attribute expression —
//!   `<A {>` 0.09 s / 1.29 s, `<A x={>` 0.05 s / 0.81 s, `</A {>` inside an
//!   element 0.12 s / 2.00 s;
//! - definition lists: lazy body lines 0.04 s / 0.82 s.
//!
//! All of them are 0.1 to 1.7 ms and 0.5 to 4.0 ms now.
//!
//! None of these is a crafted document. `$5 for a $10 book` is prose, a
//! shell snippet outside a fence is full of braces, and a list of terms
//! under a definition body is what the extension is for.
//!
//! Each test pins the cost back to linear and the parse the memo now
//! short-circuits: the shape an extension specifies has to survive the
//! early answer, and the literal fallbacks have to stay literal.

use std::sync::{Mutex, mpsc};
use std::thread;
use std::time::{Duration, Instant};

use ferromark::allocator::Allocator;
use ferromark::parser::{Parser, ParserOptions};
use ferromark::renderer::HtmlRenderer;

#[path = "support/pretty.rs"]
mod pretty;

/// Generous enough that a slow shared runner never trips it, and far below
/// what the quadratic paths needed at these sizes.
const BUDGET: Duration = Duration::from_secs(30);

/// Every shape here is a few milliseconds of real work at 128 KiB, in a
/// debug build too. A second is an absolute floor under the ratio test, so
/// a uniformly slow machine cannot pass by being slow at both sizes.
const SANITY: Duration = Duration::from_secs(5);

const SMALL: usize = 32 * 1024;
const LARGE: usize = 128 * 1024;

/// The test harness runs these functions in parallel, and a measurement
/// that shares four cores with another one measures the scheduler. One
/// shape is timed at a time; the parses that only check output are too
/// short to disturb them.
static MEASURING: Mutex<()> = Mutex::new(());

fn math_options() -> ParserOptions {
    ParserOptions {
        math: true,
        ..ParserOptions::gfm()
    }
}

fn mdx_options() -> ParserOptions {
    ParserOptions::mdx()
}

fn definition_options() -> ParserOptions {
    ParserOptions {
        definition_lists: true,
        ..ParserOptions::gfm()
    }
}

fn render(source: &str, options: ParserOptions) -> String {
    let allocator = Allocator::new();
    let document = Parser::with_options(&allocator, source, options)
        .parse()
        .expect("source should parse");
    HtmlRenderer::new().render(&document).trim().to_string()
}

/// An MDX expression renders to nothing, so its nodes are pinned instead.
fn tree(source: &str, options: ParserOptions) -> String {
    let allocator = Allocator::new();
    let document = Parser::with_options(&allocator, source, options)
        .parse()
        .expect("source should parse");
    let mut out = String::new();
    pretty::format_document(&document, source, &mut out);
    out
}

/// `unit` repeated to at least `bytes`, between a prefix and a suffix. The
/// suffix is what the cheap "is there a closer at all" guards cannot answer
/// with: one closer behind the run leaves every opener in it to be decided
/// on its own.
fn run_to(prefix: &str, unit: &str, bytes: usize, suffix: &str) -> String {
    let mut out = String::with_capacity(bytes + prefix.len() + unit.len() + suffix.len());
    out.push_str(prefix);
    while out.len() < bytes {
        out.push_str(unit);
    }
    out.push_str(suffix);
    out
}

/// A definition body followed by lazy continuation lines, with a second
/// item far behind them so the marker cache cannot settle the question.
fn definition_run(bytes: usize) -> String {
    let mut out = String::with_capacity(bytes + 32);
    out.push_str("term\n: body\n");
    let mut line = 0;
    while out.len() < bytes {
        out.push_str("lazy line ");
        out.push_str(&line.to_string());
        out.push('\n');
        line += 1;
    }
    out.push_str("\npara\n\n: x\n");
    out
}

/// Parses on a worker thread so a regression fails the suite in bounded
/// time instead of hanging it. Best of three, so a scheduling stall on a
/// busy runner has to hit every repetition to fail the build.
fn parse_within_budget(source: &str, options: &ParserOptions) -> Duration {
    let mut best = BUDGET;
    for _ in 0..3 {
        let owned = source.to_string();
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
            .expect("an extension run should parse in bounded time");
        assert!(parsed, "an extension run should parse to a document");
        best = best.min(elapsed);
    }
    best
}

/// 4x the input. Linear costs about 4x the time; quadratic costs 16x, which
/// is what every shape here measured before the fix.
fn assert_linear(name: &str, options: &ParserOptions, shape: impl Fn(usize) -> String) {
    let measuring = MEASURING
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let small = parse_within_budget(&shape(SMALL), options);
    let large = parse_within_budget(&shape(LARGE), options);
    drop(measuring);

    assert!(
        large < SANITY,
        "{name}: 128 KiB took {large:?}, past the {SANITY:?} bound on its own"
    );
    let ratio = large.as_secs_f64() / small.as_secs_f64().max(1e-9);
    assert!(
        ratio < 8.0,
        "{name}: 128 KiB took {large:?} against {small:?} for 32 KiB (x{ratio:.1}); \
         linear is about x4, quadratic about x16"
    );
}

#[test]
fn inline_math_runs_cost_linear_time() {
    // A `$` opener used to scan every later `$` to the end of the content,
    // and — unlike the `^`/`~` script spans — kept going past a `$` that
    // cannot close. The digit-prefixed form pays the same scan before it
    // can even decide to open.
    for (name, unit) in [
        ("bare", "$a "),
        ("digit", "$1 "),
        ("display", "$$a "),
        ("spaced", "$ a "),
        ("after text", "a$1 "),
    ] {
        assert_linear(name, &math_options(), |bytes| run_to("", unit, bytes, ""));
    }
}

#[test]
fn inline_math_runs_with_one_closer_cost_linear_time() {
    // One closing `$` behind the run is what a cheap last-closer guard
    // cannot answer with.
    for (name, unit) in [("bare", "$a "), ("digit", "$1 "), ("display", "$$a ")] {
        assert_linear(name, &math_options(), |bytes| {
            run_to("", unit, bytes, "$x$")
        });
    }
}

#[test]
fn inline_math_runs_with_a_hidden_candidate_cost_linear_time() {
    // The one `$` that could close sits inside a code span the scan steps
    // over, so the candidate memo cannot refuse the run and every opener
    // walked to the end for a `None`.
    for (name, unit, suffix) in [
        ("bare", "$a ", "`$`"),
        ("digit", "$1 ", "`$`"),
        ("display", "$$a ", "`$`"),
        ("after a code span", "$a ", "`x` `$`"),
    ] {
        assert_linear(name, &math_options(), |bytes| {
            run_to("", unit, bytes, suffix)
        });
    }
}

#[test]
fn display_math_lines_cost_linear_time() {
    // Every line opening with `$$` asks for the terminator twice: once
    // through the block dispatch, once through the block-start probe.
    for (name, unit) in [
        ("text", "$$ a\n"),
        ("bare", "$$x\n"),
        ("indented", "  $$a\n"),
    ] {
        assert_linear(name, &math_options(), |bytes| run_to("", unit, bytes, ""));
    }
}

#[test]
fn jsx_open_runs_cost_linear_time() {
    // `<A>` repeated nests one level per tag, so a single `</A>` closes the
    // innermost opener only and every other one has to learn that for
    // itself.
    for (name, prefix, suffix) in [
        ("flow, one closer", "", "</A>"),
        ("text, one closer", "x", "</A>"),
        ("flow, no closer", "", ""),
        ("text, no closer", "x", ""),
    ] {
        assert_linear(name, &mdx_options(), |bytes| {
            run_to(prefix, "<A>", bytes, suffix)
        });
    }
}

#[test]
fn jsx_tags_over_brace_runs_cost_linear_time() {
    // The tag walk steps over `{...}` whole, and an unclosed `{` reports
    // that only after reading to the end of the slice. The walk takes that
    // answer from the same record the expression parse does, so a `}`
    // behind the run — which no cheap guard can refuse — costs one walk for
    // the whole run, whether or not the tag itself closes.
    for (name, prefix, unit, suffix) in [
        ("bare", "<A>", "{", ""),
        ("spaced", "<A>", "{ ", ""),
        ("nested", "<A>", "{a{b", ""),
        ("one closer", "<A>", "{", "}"),
        ("one closer, in text", "x<A>", "{", "}"),
        ("one closer, tag closes", "<A>", "{", "}</A>"),
        ("one closer, spaced", "<A>", "{ ", "}"),
    ] {
        assert_linear(name, &mdx_options(), |bytes| {
            run_to(prefix, unit, bytes, suffix)
        });
    }
}

#[test]
fn tags_with_unclosed_attribute_braces_cost_linear_time() {
    // An attribute expression that never closes is read by the opening-tag
    // scan, which runs for every `<` the inline dispatch reaches and again
    // for every tag a closing-tag walk steps over — before any closer walk
    // starts. It asks the same record as the rest.
    for (name, prefix, unit, suffix) in [
        ("spread", "", "<A {>", ""),
        ("spread, one closer", "", "<A {>", "}"),
        ("spread, tag closes", "", "<A {>", "</A>"),
        ("attribute value", "", "<A x={>", ""),
        ("closing tag", "<A>", "</A {>", ""),
    ] {
        assert_linear(name, &mdx_options(), |bytes| {
            run_to(prefix, unit, bytes, suffix)
        });
    }
}

#[test]
fn brace_runs_with_one_closer_cost_linear_time() {
    // `tests/mdx_brace_scaling.rs` covers the run with nothing to close it,
    // which the last-closer guard answers. One `}` behind the run defeats
    // that guard and leaves every brace to the balanced scan.
    for (name, prefix, unit) in [
        ("flow", "", "{"),
        ("flow, spaced", "", "{ "),
        ("text", "x", "{"),
        ("text, spaced", "x", "{ "),
        ("lines", "", "{\n"),
        ("nested", "", "{a{b"),
    ] {
        assert_linear(name, &mdx_options(), |bytes| {
            run_to(prefix, unit, bytes, "}")
        });
    }
}

#[test]
fn lazy_definition_body_lines_cost_linear_time() {
    // Every non-indented continuation line of a body asks whether a new
    // item starts there, and the term scan walks to the next blank line to
    // answer. A `:` marker further down the document keeps the marker cache
    // from settling it.
    assert_linear("lazy lines", &definition_options(), definition_run);
}

#[test]
fn inline_math_keeps_its_shape() {
    // The early answers replace scans that ended in the literal fallback,
    // so each of these has to render exactly as it did.
    for (source, expected) in [
        ("$a $a $a ", "<p>$a $a $a</p>"),
        ("$1 $1 ", "<p>$1 $1</p>"),
        ("$$a $$a ", "<p>$$a $$a</p>"),
        ("$ a $", "<p>$ a $</p>"),
        ("a $5 and $10 b", "<p>a $5 and $10 b</p>"),
        (
            "$a$",
            "<p><span class=\"ox-math ox-math-inline\" data-ox-tex=\"a\">\
             <math><mtext>a</mtext></math></span></p>",
        ),
        (
            "x $1*2$ y",
            "<p>x <span class=\"ox-math ox-math-inline\" data-ox-tex=\"1*2\">\
             <math><mtext>1*2</mtext></math></span> y</p>",
        ),
        (
            "$a `$` b$",
            "<p><span class=\"ox-math ox-math-inline\" data-ox-tex=\"a `$` b\">\
             <math><mtext>a `$` b</mtext></math></span></p>",
        ),
        // The only `$` that could close is inside a code span, so the run
        // stays literal and the code span stays a code span.
        ("$a $a `$`", "<p>$a $a <code>$</code></p>"),
        ("$1 $1 `$`", "<p>$1 $1 <code>$</code></p>"),
        (
            "$a $a `x` `$`",
            "<p>$a $a <code>x</code> <code>$</code></p>",
        ),
    ] {
        assert_eq!(render(source, math_options()), expected, "for {source:?}");
    }
}

#[test]
fn math_hidden_from_one_scan_still_closes_for_a_later_opener() {
    // The backtick inside the attribute is a code-span opener to the math
    // scan and part of a raw HTML tag to the parse, so the first `$` steps
    // over a region the second one opens inside. What one scan read is what
    // it may answer for; a `$` it never looked at scans for itself.
    assert_eq!(
        render("$a <i t=\"`\">$b$ x` z", math_options()),
        "<p>$a <i t=\"`\"><span class=\"ox-math ox-math-inline\" data-ox-tex=\"b\">\
         <math><mtext>b</mtext></math></span> x` z</p>"
    );
    // The same skip the other way round: the scan steps over the code span
    // and closes on the `$` behind it, so the span is math, not a code
    // span with math after it.
    assert_eq!(
        render("$a `x$y` $b$", math_options()),
        "<p><span class=\"ox-math ox-math-inline\" data-ox-tex=\"a `x$y` $b\">\
         <math><mtext>a `x$y` $b</mtext></math></span></p>"
    );
}

#[test]
fn display_math_keeps_its_shape() {
    for (source, expected) in [
        ("$$ a\n$$ a\n", "<p>$$ a\n$$ a</p>"),
        (
            "$$ a\n$$\n",
            "<div class=\"ox-math ox-math-block\" data-ox-tex=\" a&#10;\">\
             <math display=\"block\"><mtext> a\n</mtext></math></div>",
        ),
        (
            "$$\na\n$$\n",
            "<div class=\"ox-math ox-math-block\" data-ox-tex=\"&#10;a&#10;\">\
             <math display=\"block\"><mtext>\na\n</mtext></math></div>",
        ),
    ] {
        assert_eq!(render(source, math_options()), expected, "for {source:?}");
    }
}

#[test]
fn jsx_runs_keep_their_shape() {
    for (source, expected) in [
        (
            "<A><A></A>",
            "<p><A><span class=\"ox-island\" data-ox-island=\"A\"></span></p>",
        ),
        (
            "x<A><A></A>",
            "<p>x<A><span class=\"ox-island\" data-ox-island=\"A\"></span></p>",
        ),
        ("<A><A>", "<p><A><A></p>"),
        ("<A>{{{", "<p><A>{{{</p>"),
        (
            "<A>x</A>",
            "<div class=\"ox-island\" data-ox-island=\"A\"><p>x</p>\n</div>",
        ),
        // The brace run the tag walk steps over, with one `}` behind it.
        ("<A>{{{}", "<p><A>{{</p>"),
        ("x<A>{{{}", "<p>x<A>{{</p>"),
        (
            "<A>{{{}</A>",
            "<div class=\"ox-island\" data-ox-island=\"A\"><p>{{</p>\n</div>",
        ),
        // An attribute expression that never closes leaves the tag text.
        ("<A {>", "<p>&lt;A {&gt;</p>"),
        ("<A {><A {>", "<p>&lt;A {&gt;&lt;A {&gt;</p>"),
        ("<A x={>", "<A x={>"),
        ("<A></A {>", "<p><A>&lt;/A {&gt;</p>"),
    ] {
        assert_eq!(render(source, mdx_options()), expected, "for {source:?}");
    }
}

#[test]
fn attribute_expressions_still_parse() {
    // The opening-tag scan takes its braces from the record now, so the
    // attributes it builds from them have to be the same ones.
    assert_eq!(
        render("<A {x}>t</A>", mdx_options()),
        "<div class=\"ox-island\" data-ox-island=\"A\" data-ox-props=\"{&quot;expressions&quot;:{},\
         &quot;props&quot;:{},&quot;spreads&quot;:[&quot;x&quot;]}\">\
         <script type=\"application/json\">{\"expressions\":{},\"props\":{},\"spreads\":[\"x\"]}\
         </script><p>t</p>\n</div>"
    );
    assert_eq!(
        tree("a <A x={1} {...y}/> b", mdx_options()),
        "Document [0..21]\n  Paragraph [0..21]\n    Text \"a \" [0..2]\n    \
         MdxJsxTextElement name=Some(\"A\") self_closing=true [2..19]\n      \
         Attr name=\"x\" value=expression(\"1\") [5..10]\n      \
         AttrExpr value=\"...y\" [11..17]\n    Text \" b\" [19..21]\n"
    );
}

#[test]
fn a_closer_hidden_from_one_tag_still_closes_a_later_one() {
    // The memo is keyed by the opener it answers for, not by the slice
    // alone. The first `<A>` never sees the `</A>` below it — the tag walk
    // steps over the backtick run whole — and that answer must not reach
    // the second `<A>`, which the same `</A>` does close.
    assert_eq!(
        render("<A>`\n<A>x</A>\n`\n", mdx_options()),
        "<p><A>`</p>\n<div class=\"ox-island\" data-ox-island=\"A\"><p>x</p>\n</div>\n<p>`</p>"
    );
    assert_eq!(
        render("<A>{\n<A>x</A>\n}\n", mdx_options()),
        "<p><A>{</p>\n<div class=\"ox-island\" data-ox-island=\"A\"><p>x</p>\n</div>\n<p>}</p>"
    );
}

#[test]
fn braces_keep_their_shape() {
    // An expression renders to nothing — its source is stored, never
    // evaluated — so the nodes and their spans are what has to be pinned:
    // the braces that stay literal, and the one that does close.
    for (source, expected) in [
        (
            "{{{}",
            "Document [0..4]\n  Paragraph [0..4]\n    Text \"{\" [0..1]\n    \
             Text \"{\" [1..2]\n    MdxTextExpression value=\"\" [2..4]\n",
        ),
        (
            "x{{{}",
            "Document [0..5]\n  Paragraph [0..5]\n    Text \"x\" [0..1]\n    \
             Text \"{\" [1..2]\n    Text \"{\" [2..3]\n    MdxTextExpression value=\"\" [3..5]\n",
        ),
        (
            "{ { { }",
            "Document [0..7]\n  Paragraph [0..7]\n    Text \"{\" [0..1]\n    \
             Text \" \" [1..2]\n    Text \"{\" [2..3]\n    Text \" \" [3..4]\n    \
             MdxTextExpression value=\" \" [4..7]\n",
        ),
        (
            "{a}",
            "Document [0..3]\n  MdxFlowExpression value=\"a\" [0..3]\n",
        ),
        (
            "x{a}y",
            "Document [0..5]\n  Paragraph [0..5]\n    Text \"x\" [0..1]\n    \
             MdxTextExpression value=\"a\" [1..4]\n    Text \"y\" [4..5]\n",
        ),
        (
            "{a{b}}",
            "Document [0..6]\n  MdxFlowExpression value=\"a{b}\" [0..6]\n",
        ),
    ] {
        assert_eq!(tree(source, mdx_options()), expected, "for {source:?}");
    }
}

#[test]
fn definition_lists_keep_their_shape() {
    // The window the term scan records must not swallow the item that does
    // start after it.
    assert_eq!(
        render(
            "term\n: body\nlazy one\nlazy two\n\npara\n\n: x\n",
            definition_options()
        ),
        "<dl class=\"ox-definition-list\">\n<dt>term</dt>\n\
         <dd>body\nlazy one\nlazy two</dd>\n<dt>para</dt>\n<dd>x</dd>\n</dl>"
    );
    assert_eq!(
        render("term\n: body\n\nnext\n: also\n", definition_options()),
        "<dl class=\"ox-definition-list\">\n<dt>term</dt>\n<dd>body</dd>\n\
         <dt>next</dt>\n<dd>also</dd>\n</dl>"
    );
    assert_eq!(
        render("lazy only\nmore lazy\n\npara\n", definition_options()),
        "<p>lazy only\nmore lazy</p>\n<p>para</p>"
    );
}
