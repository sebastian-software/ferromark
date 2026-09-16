//! Allocation accounting for `HtmlRendererOptions`.
//!
//! The documented renderer defaults are compile-time constants, so building,
//! cloning and handing them to a renderer must not reach the allocator. That is
//! an exact property rather than a timing one, so it is asserted directly with a
//! counting global allocator. This needs its own test binary: `#[global_allocator]`
//! is per-binary, and every other test target must keep the ordinary allocator.
#![allow(unsafe_code)]

use std::alloc::{GlobalAlloc, Layout, System};
use std::borrow::Cow;
use std::cell::Cell;
use std::hint::black_box;

use ferromark::{HtmlRenderer, HtmlRendererOptions};

// Allocation counter, scoped to the measuring thread.
//
// The test harness runs tests in parallel, so a process-wide counter would
// bill another test's allocations to whichever measurement happens to overlap.
// Both cells are `const`-initialized and hold no `Drop` value, so reading them
// from inside the allocator cannot allocate or recurse.
thread_local! {
    static COUNTING: Cell<bool> = const { Cell::new(false) };
    static ALLOCATIONS: Cell<u64> = const { Cell::new(0) };
}

struct CountingAllocator;

impl CountingAllocator {
    /// Records one allocation when the current thread is measuring.
    fn record() {
        if COUNTING.try_with(Cell::get).unwrap_or(false) {
            let _ = ALLOCATIONS.try_with(|count| count.set(count.get() + 1));
        }
    }
}

// SAFETY: every method forwards the caller's pointer, layout and size to the
// system allocator unchanged; the counter is the only added effect.
unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        Self::record();
        // SAFETY: forwarding the caller's valid layout to the system allocator.
        unsafe { System.alloc(layout) }
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        Self::record();
        // SAFETY: forwarding the caller's valid layout to the system allocator.
        unsafe { System.alloc_zeroed(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // SAFETY: forwarding the allocation and its original layout unchanged.
        unsafe { System.dealloc(ptr, layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        Self::record();
        // SAFETY: forwarding the allocation, original layout and requested size
        // unchanged.
        unsafe { System.realloc(ptr, layout, new_size) }
    }
}

#[global_allocator]
static GLOBAL: CountingAllocator = CountingAllocator;

/// Runs `work` and reports how many allocations it made on this thread.
///
/// The value is returned so the caller can `black_box` it: dropping it inside
/// the measured region would make the count depend on the drop, not on the
/// construction under test.
fn allocations<T>(work: impl FnOnce() -> T) -> (T, u64) {
    ALLOCATIONS.with(|count| count.set(0));
    COUNTING.with(|flag| flag.set(true));
    let value = work();
    COUNTING.with(|flag| flag.set(false));
    (value, ALLOCATIONS.with(Cell::get))
}

/// Guards the guard: a zero count must mean "nothing allocated", not "the
/// counter never fired". Without this, every other assertion here would pass
/// vacuously if the hook or the thread-local ever stopped working.
#[test]
fn the_counter_observes_a_real_allocation() {
    let (owned, count) = allocations(|| black_box(String::from("not a borrowed default")));
    assert!(
        count >= 1,
        "the counting allocator missed an obvious heap allocation"
    );
    drop(owned);

    let (_, idle) = allocations(|| black_box(7_u32));
    assert_eq!(idle, 0, "counting a value that never reaches the heap");
}

#[test]
fn default_options_never_allocate() {
    // Warm any lazily initialized machinery before measuring.
    drop(black_box(HtmlRendererOptions::new()));

    for (label, build) in [
        (
            "new",
            (|| HtmlRendererOptions::new()) as fn() -> HtmlRendererOptions,
        ),
        ("default", HtmlRendererOptions::default),
        ("commonmark", HtmlRendererOptions::commonmark),
        ("gfm", HtmlRendererOptions::gfm),
    ] {
        let (options, count) = allocations(|| black_box(build()));
        assert_eq!(count, 0, "`HtmlRendererOptions::{label}()` allocated");
        drop(options);
    }
}

#[test]
fn cloning_default_options_never_allocates() {
    let options = HtmlRendererOptions::new();
    drop(black_box(options.clone()));

    let (clone, count) = allocations(|| black_box(options.clone()));
    assert_eq!(count, 0, "cloning default options allocated");
    // The clone must still carry the documented defaults, borrowed.
    assert!(matches!(clone.soft_break, Cow::Borrowed("\n")));
    assert!(matches!(clone.hard_break, Cow::Borrowed("<br>\n")));
    assert!(matches!(clone.base_url, Cow::Borrowed("/")));
    assert!(matches!(clone.source_path, Cow::Borrowed("")));
    assert!(matches!(
        clone.code_annotation_meta_key,
        Cow::Borrowed("annotate")
    ));
    assert!(matches!(clone.autolink_patterns, Cow::Borrowed(_)));
    assert_eq!(
        clone.autolink_patterns.as_ref(),
        [Cow::Borrowed("http://"), Cow::Borrowed("https://")]
    );
}

#[test]
fn every_static_profile_clone_is_allocation_free() {
    for (label, options) in [
        ("commonmark", HtmlRendererOptions::commonmark()),
        ("gfm", HtmlRendererOptions::gfm()),
    ] {
        drop(black_box(options.clone()));
        let (clone, count) = allocations(|| black_box(options.clone()));
        assert_eq!(count, 0, "cloning `{label}` options allocated");
        drop(clone);
    }
}

#[test]
fn building_a_renderer_from_default_options_adds_no_allocation() {
    // The renderer allocates its own reusable scratch buffers, which this
    // change does not touch. What must hold is that routing through
    // `HtmlRendererOptions` adds nothing on top of the plain constructor.
    drop(black_box(HtmlRenderer::new()));
    drop(black_box(HtmlRenderer::with_options(
        HtmlRendererOptions::new(),
    )));

    let (plain, plain_count) = allocations(|| black_box(HtmlRenderer::new()));
    drop(plain);
    let (configured, configured_count) =
        allocations(|| black_box(HtmlRenderer::with_options(HtmlRendererOptions::new())));
    drop(configured);

    assert_eq!(
        configured_count, plain_count,
        "`with_options(HtmlRendererOptions::new())` allocated {configured_count} times \
         against {plain_count} for `HtmlRenderer::new()`"
    );
}

#[test]
fn owned_values_stay_owned_and_survive_a_clone() {
    let runtime_base = String::from("/generated-at-runtime/");
    let options = HtmlRendererOptions {
        base_url: runtime_base.into(),
        autolink_patterns: vec![Cow::Owned(String::from("ftp://"))].into(),
        ..HtmlRendererOptions::new()
    };
    assert!(matches!(options.base_url, Cow::Owned(_)));
    assert!(matches!(options.autolink_patterns, Cow::Owned(_)));

    let clone = options.clone();
    assert_eq!(&*clone.base_url, "/generated-at-runtime/");
    assert_eq!(clone.autolink_patterns.as_ref(), [Cow::Borrowed("ftp://")]);
    // Static values assigned from literals borrow rather than allocate.
    assert!(matches!(clone.hard_break, Cow::Borrowed("<br>\n")));

    // The clone owns its own copy, so the original is untouched and both
    // outlive the assertion independently.
    drop(clone);
    assert_eq!(&*options.base_url, "/generated-at-runtime/");
    assert_eq!(
        options.autolink_patterns.as_ref(),
        [Cow::Borrowed("ftp://")]
    );
}

#[test]
fn static_literals_assigned_by_callers_stay_borrowed() {
    let options = HtmlRendererOptions {
        hard_break: "<br />\n".into(),
        base_url: "/docs/".into(),
        source_path: "guide/index.md".into(),
        code_annotation_meta_key: "markers".into(),
        ..HtmlRendererOptions::new()
    };
    assert!(matches!(options.hard_break, Cow::Borrowed("<br />\n")));
    assert!(matches!(options.base_url, Cow::Borrowed("/docs/")));
    assert!(matches!(
        options.source_path,
        Cow::Borrowed("guide/index.md")
    ));
    assert!(matches!(
        options.code_annotation_meta_key,
        Cow::Borrowed("markers")
    ));

    drop(black_box(options.clone()));
    let (clone, count) = allocations(|| black_box(options.clone()));
    assert_eq!(count, 0, "cloning borrowed caller literals allocated");
    drop(clone);
}
