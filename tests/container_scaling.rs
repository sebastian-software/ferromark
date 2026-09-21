//! Container and inline shapes that used to cost far more than their size.
//!
//! Two kinds of regression are pinned here. Every container level
//! re-materializes the blank lines of its content for its sub-parser, and
//! every blank line used to add a forty-byte source-map entry per level, so
//! a hundred-level list followed by a hundred thousand blank lines — 100 KB
//! of input — held four hundred megabytes of source maps. And several inline
//! scans walked to the end of the content once per opener: image alt text,
//! reference labels, inline notes, wiki links, the trailing-punctuation trim
//! of a GFM autolink, and the retired delimiters an unequal strikethrough
//! pair leaves behind, so 128 KB of one of those shapes took seconds.
//!
//! Heap use is measured through a counting allocator, so the memory guard
//! holds on every platform; time is compared between two sizes of the same
//! shape, best of three, so a slow runner cannot fail it and a quadratic
//! regression cannot pass it.

// The counting allocator is the one `unsafe` item in the test suite: it
// forwards every call to the system allocator unchanged and only keeps two
// counters. The size ratio is a test diagnostic, where converting a length to
// a float is exact for any input this file builds.
#![allow(unsafe_code, clippy::cast_precision_loss)]

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use ferromark::allocator::Allocator;
use ferromark::parser::{Parser, ParserOptions};
use ferromark::renderer::HtmlRenderer;

struct CountingAllocator;

static HEAP_IN_USE: AtomicUsize = AtomicUsize::new(0);
static HEAP_PEAK: AtomicUsize = AtomicUsize::new(0);

// SAFETY: every method forwards to `System` unchanged and only adds
// bookkeeping; the bookkeeping never allocates.
unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let in_use = HEAP_IN_USE.fetch_add(layout.size(), Ordering::Relaxed) + layout.size();
        HEAP_PEAK.fetch_max(in_use, Ordering::Relaxed);
        // SAFETY: forwarded unchanged.
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        HEAP_IN_USE.fetch_sub(layout.size(), Ordering::Relaxed);
        // SAFETY: forwarded unchanged.
        unsafe { System.dealloc(ptr, layout) }
    }
}

#[global_allocator]
static GLOBAL: CountingAllocator = CountingAllocator;

/// Generous enough that a slow shared runner never trips it, and far below
/// what the quadratic shapes needed.
const BUDGET: Duration = Duration::from_secs(20);

/// Parses and renders on a worker thread so a regression fails the suite in
/// bounded time instead of hanging it. Returns the best of three runs.
fn best_of_three(source: &str, options: &ParserOptions) -> Duration {
    let mut best = BUDGET;
    for _ in 0..3 {
        let owned = source.to_owned();
        let options = options.clone();
        let (sender, receiver) = mpsc::channel();
        thread::spawn(move || {
            let started = Instant::now();
            let allocator = Allocator::for_source_len(owned.len());
            let document = Parser::with_options(&allocator, &owned, options)
                .parse()
                .expect("the shape should parse");
            let html = HtmlRenderer::new().render(&document);
            let _ = sender.send((html.len(), started.elapsed()));
        });
        let (_, elapsed) = receiver
            .recv_timeout(BUDGET)
            .expect("the shape should parse in bounded time, not quadratic time");
        best = best.min(elapsed);
    }
    best
}

/// Asserts that four times the input costs well under sixteen times the
/// time: a linear shape stays near 4x, a quadratic one lands near 16x.
fn assert_linear(label: &str, small: &str, large: &str, options: &ParserOptions) {
    let size_ratio = large.len() as f64 / small.len() as f64;
    assert!(
        (3.9..=4.1).contains(&size_ratio),
        "{label}: the large input should be 4x, got x{size_ratio:.2}"
    );
    let small_time = best_of_three(small, options).max(Duration::from_millis(1));
    let large_time = best_of_three(large, options);
    let ratio = large_time.as_secs_f64() / small_time.as_secs_f64();
    assert!(
        ratio < 8.0,
        "{label}: {} bytes took {small_time:?}, {} bytes took {large_time:?} (x{ratio:.1})",
        small.len(),
        large.len()
    );
}

fn nested_list_with_blank_lines(blank_lines: usize) -> String {
    let mut source = "- ".repeat(100);
    source.push_str("a\n");
    source.push_str(&"\n".repeat(blank_lines));
    source.push_str(&" ".repeat(200));
    source.push_str("x\n");
    source
}

#[test]
fn blank_lines_under_nested_containers_cost_their_own_size() {
    let source = nested_list_with_blank_lines(100_000);
    assert!(source.len() > 100_000);
    let options = ParserOptions::commonmark();

    HEAP_PEAK.store(HEAP_IN_USE.load(Ordering::Relaxed), Ordering::Relaxed);
    let before = HEAP_PEAK.load(Ordering::Relaxed);
    let started = Instant::now();
    let allocator = Allocator::for_source_len(source.len());
    let (elapsed, peak_heap) = {
        let document = Parser::with_options(&allocator, &source, options)
            .parse()
            .expect("the nested list should parse");
        let html = HtmlRenderer::new().render(&document);
        let elapsed = started.elapsed();
        assert!(!html.is_empty());
        (
            elapsed,
            HEAP_PEAK.load(Ordering::Relaxed).saturating_sub(before),
        )
    };

    // The arena holds one copy of the blank run per level, which is bounded
    // by depth × input, and its chunks come from the global allocator too.
    // Everything else on the heap — the source maps above all — has to stay
    // small next to it; before blank runs coalesced, the maps alone held
    // twenty times the arena.
    let arena_bytes = allocator.allocated_bytes();
    assert!(
        arena_bytes < 64 * 1024 * 1024,
        "arena {arena_bytes} bytes for a 100 KB nested list"
    );
    assert!(
        peak_heap < 3 * arena_bytes + 8 * 1024 * 1024,
        "a 100 KB nested list took {peak_heap} bytes of heap next to {arena_bytes} bytes of arena"
    );
    assert!(elapsed < BUDGET, "took {elapsed:?}");

    // Footnote and definition bodies re-materialize blank lines the same way.
    let mut footnotes = String::from("[^1]\n\n");
    for _ in 0..100 {
        footnotes.push_str("[^1]: ");
    }
    footnotes.push_str("a\n");
    footnotes.push_str(&"\n".repeat(50_000));
    footnotes.push_str(&" ".repeat(400));
    footnotes.push_str("x\n");
    HEAP_PEAK.store(HEAP_IN_USE.load(Ordering::Relaxed), Ordering::Relaxed);
    let before = HEAP_PEAK.load(Ordering::Relaxed);
    let allocator = Allocator::for_source_len(footnotes.len());
    let peak_heap = {
        let document = Parser::with_options(&allocator, &footnotes, ParserOptions::gfm())
            .parse()
            .expect("the nested footnotes should parse");
        assert!(!document.children.is_empty());
        HEAP_PEAK.load(Ordering::Relaxed).saturating_sub(before)
    };
    let arena_bytes = allocator.allocated_bytes();
    assert!(
        peak_heap < 3 * arena_bytes + 8 * 1024 * 1024,
        "nested footnote bodies took {peak_heap} bytes of heap next to {arena_bytes} bytes of arena"
    );
}

#[test]
fn nested_blank_runs_scale_linearly_in_time() {
    assert_linear(
        "nested list blank lines",
        &nested_list_with_blank_lines(25_000),
        &nested_list_with_blank_lines(100_000),
        &ParserOptions::commonmark(),
    );
}

#[test]
fn image_openers_with_one_closer_scale_linearly() {
    let shape = |n: usize| "![".repeat(n) + "a]";
    assert_linear(
        "image openers",
        &shape(8_000),
        &shape(32_000),
        &ParserOptions::default(),
    );
}

#[test]
fn reference_label_chains_scale_linearly() {
    let shape = |n: usize| "[a][".repeat(n);
    assert_linear(
        "reference label chain",
        &shape(4_000),
        &shape(16_000),
        &ParserOptions::default(),
    );
}

#[test]
fn inline_note_openers_scale_linearly() {
    let options = ParserOptions {
        inline_footnotes: true,
        ..ParserOptions::default()
    };
    let shape = |n: usize| "^[".repeat(n) + "a]";
    assert_linear("inline notes", &shape(8_000), &shape(32_000), &options);
}

#[test]
fn wiki_link_openers_scale_linearly() {
    let options = ParserOptions {
        wiki_links: true,
        ..ParserOptions::default()
    };
    for (label, shape) in [
        (
            "single brackets",
            (|n: usize| "[".repeat(n) + "a]") as fn(usize) -> String,
        ),
        ("double brackets", |n: usize| "[[".repeat(n / 2) + "]"),
        ("spaced double brackets", |n: usize| {
            "[[a ".repeat(n / 4) + "]"
        }),
    ] {
        assert_linear(label, &shape(8_000), &shape(32_000), &options);
    }
}

#[test]
fn autolink_trailing_punctuation_scales_linearly() {
    let options = ParserOptions::gfm();
    let parens = |n: usize| "http://x.com/".to_owned() + &")".repeat(n);
    assert_linear(
        "trailing parentheses",
        &parens(8_000),
        &parens(32_000),
        &options,
    );
    let semicolons = |n: usize| "http://x.com/".to_owned() + &";".repeat(n);
    assert_linear(
        "trailing semicolons",
        &semicolons(8_000),
        &semicolons(32_000),
        &options,
    );
}

#[test]
fn retired_strikethrough_delimiters_scale_linearly() {
    let options = ParserOptions::gfm();
    let shape = |n: usize| " ~a".repeat(n) + &"b_".repeat(n * 4) + &" a~~".repeat(n);
    assert_linear(
        "unequal strikethrough pairs",
        &shape(2_000),
        &shape(8_000),
        &options,
    );
}

#[test]
fn the_shapes_still_render_as_before() {
    // The memoized scans must reach the same verdicts as the plain walks
    // did; these outputs are what the previous release produced.
    let render = |source: &str, options: ParserOptions| {
        let allocator = Allocator::new();
        let document = Parser::with_options(&allocator, source, options)
            .parse()
            .expect("source should parse");
        HtmlRenderer::new().render(&document)
    };
    assert_eq!(
        render(
            "![![a](u)](v) ![b][c] ![d]\n\n[c]: /c\n[d]: /d",
            ParserOptions::default()
        ),
        "<p><img src=\"v\" alt=\"a\"> <img src=\"/c\" alt=\"b\"> <img src=\"/d\" alt=\"d\"></p>\n"
    );
    assert_eq!(
        render(
            "[a][b] [a][[b]] [x][]\n\n[b]: /b\n[x]: /x",
            ParserOptions::default()
        ),
        "<p><a href=\"/b\">a</a> [a][<a href=\"/b\">b</a>] <a href=\"/x\">x</a></p>\n"
    );
    let wiki = ParserOptions {
        wiki_links: true,
        ..ParserOptions::default()
    };
    assert_eq!(
        render("[[Page]] [[a|b]] [[x", wiki),
        "<p><a href=\"Page\">Page</a> <a href=\"a\">b</a> [[x</p>\n"
    );
    assert_eq!(
        render(
            "see https://x.test/a(b)) and https://x.test/c&amp;.",
            ParserOptions::gfm()
        ),
        concat!(
            "<p>see <a href=\"https://x.test/a(b)\" target=\"_blank\" rel=\"noopener noreferrer\">https://x.test/a(b)</a>) ",
            "and <a href=\"https://x.test/c&amp;\" target=\"_blank\" rel=\"noopener noreferrer\">https://x.test/c&amp;</a>.</p>\n"
        )
    );
    assert_eq!(
        render("~a~~ ~~b~~ *c* d_e_f", ParserOptions::gfm()),
        "<p>~a~~ <del>b</del> <em>c</em> d_e_f</p>\n"
    );
    let notes = ParserOptions {
        inline_footnotes: true,
        ..ParserOptions::default()
    };
    let html = render("^[note] ^[a ^[b]] text", notes);
    assert!(
        html.starts_with(
            "<p><sup><a href=\"#fn-1\" id=\"fnref-1\">1</a></sup> <sup><a href=\"#fn-2\""
        ),
        "{html}"
    );
    assert!(html.contains("<p>a ^[b]</p>"), "{html}");
}
