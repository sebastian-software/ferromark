//! The plain fenced-code path must not allocate per block.
//!
//! `render_code_block`'s default route resolves a fence's language into a
//! borrow of the source and reserves the output once before writing, so a
//! renderer whose buffer is already warm should emit a fence-heavy document
//! without touching the allocator at all. A regression that reintroduces an
//! owned language string, a joined class list, or an unreserved growth step
//! would show up here as a non-zero count.
//!
//! The counting allocator is global, so this file deliberately holds a single
//! test: nothing else in the binary runs while the counters are armed.

#![allow(unsafe_code)]

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use ferromark::{Allocator, HtmlRenderer, Parser};

struct CountingSystem;

static COUNTING: AtomicBool = AtomicBool::new(false);
static ALLOCATIONS: AtomicU64 = AtomicU64::new(0);

unsafe impl GlobalAlloc for CountingSystem {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if COUNTING.load(Ordering::Relaxed) {
            ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        }
        // SAFETY: forwarding the caller's valid layout to the system allocator.
        unsafe { System.alloc(layout) }
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        if COUNTING.load(Ordering::Relaxed) {
            ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        }
        // SAFETY: forwarding the caller's valid layout to the system allocator.
        unsafe { System.alloc_zeroed(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // SAFETY: forwarding the allocation and its original layout unchanged.
        unsafe { System.dealloc(ptr, layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        // A `String` grow arrives here rather than at `alloc`, so buffer growth
        // has to count too.
        if COUNTING.load(Ordering::Relaxed) {
            ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        }
        // SAFETY: forwarding the allocation, original layout, and requested size unchanged.
        unsafe { System.realloc(ptr, layout, new_size) }
    }
}

#[global_allocator]
static GLOBAL: CountingSystem = CountingSystem;

/// A document of nothing but bare-language fences, so every allocation the
/// measurement sees belongs to the plain fenced-code path.
fn bare_language_fences(count: usize) -> String {
    const LANGUAGES: [&str; 5] = ["ts", "js", "bash", "rust", "json"];

    let mut source = String::new();
    for index in 0..count {
        source.push_str("```");
        source.push_str(LANGUAGES[index % LANGUAGES.len()]);
        source.push_str("\nconst value = ");
        source.push_str(&index.to_string());
        source.push_str(";\nconst other = value < 2 && value > 0;\n```\n\n");
    }
    source
}

/// Allocation count of one render by a renderer that has already rendered the
/// same document, so its output buffer and per-render maps are warm.
fn steady_state_allocations(source: &str) -> u64 {
    let allocator = Allocator::new();
    let document = Parser::new(&allocator, source).parse().expect("parses");
    let mut renderer = HtmlRenderer::new();

    // `render_borrowed` keeps the output buffer instead of handing it away, so
    // repeated renders reach a steady state. Four warm-ups are more than the
    // buffer needs to stop growing.
    let mut warm = 0;
    for _ in 0..4 {
        warm = renderer.render_borrowed(&document).len();
    }
    assert!(warm > 0, "the fixture should render to something");

    ALLOCATIONS.store(0, Ordering::SeqCst);
    COUNTING.store(true, Ordering::SeqCst);
    let rendered = renderer.render_borrowed(&document).len();
    COUNTING.store(false, Ordering::SeqCst);

    assert_eq!(rendered, warm, "steady-state renders must agree");
    ALLOCATIONS.load(Ordering::SeqCst)
}

/// Guards the harness itself: a zero count has to mean "did not allocate",
/// not "the counter was never armed".
fn assert_the_counter_observes_allocations() {
    ALLOCATIONS.store(0, Ordering::SeqCst);
    COUNTING.store(true, Ordering::SeqCst);
    let mut probe: Vec<u8> = Vec::new();
    probe.push(1);
    probe.reserve(4096);
    COUNTING.store(false, Ordering::SeqCst);

    assert_eq!(probe.len(), 1);
    assert!(
        ALLOCATIONS.load(Ordering::SeqCst) >= 2,
        "the counting allocator is not observing allocations"
    );
}

#[test]
fn a_warm_renderer_emits_bare_language_fences_without_allocating() {
    assert_the_counter_observes_allocations();

    for count in [1, 32, 256] {
        let source = bare_language_fences(count);
        let allocations = steady_state_allocations(&source);
        assert_eq!(
            allocations, 0,
            "rendering {count} bare-language fences allocated {allocations} times"
        );
    }
}
