//! The abbreviation dictionary is built once per reusable renderer.
#![allow(unsafe_code)]

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::collections::BTreeMap;
use std::hint::black_box;

use ferromark::ast::Node;
use ferromark::{AbbreviationOptions, Allocator, HtmlRenderer, HtmlRendererOptions, Parser};

thread_local! {
    static COUNTING: Cell<bool> = const { Cell::new(false) };
    static ALLOCATIONS: Cell<u64> = const { Cell::new(0) };
}

struct CountingAllocator;

impl CountingAllocator {
    fn record() {
        if COUNTING.try_with(Cell::get).unwrap_or(false) {
            let _ = ALLOCATIONS.try_with(|count| count.set(count.get() + 1));
        }
    }
}

// SAFETY: each operation forwards its original pointer and layout unchanged.
unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        Self::record();
        // SAFETY: the caller supplies the valid layout for this allocation.
        unsafe { System.alloc(layout) }
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        Self::record();
        // SAFETY: the caller supplies the valid layout for this allocation.
        unsafe { System.alloc_zeroed(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // SAFETY: the pointer and layout came from the matching system allocator.
        unsafe { System.dealloc(ptr, layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        Self::record();
        // SAFETY: the pointer, layout, and new size are forwarded unchanged.
        unsafe { System.realloc(ptr, layout, size) }
    }
}

#[global_allocator]
static GLOBAL: CountingAllocator = CountingAllocator;

fn allocations<T>(work: impl FnOnce() -> T) -> (T, u64) {
    ALLOCATIONS.with(|count| count.set(0));
    COUNTING.with(|flag| flag.set(true));
    let value = work();
    COUNTING.with(|flag| flag.set(false));
    (value, ALLOCATIONS.with(Cell::get))
}

#[test]
fn disabled_renderer_construction_allocates_nothing_for_abbreviations() {
    drop(black_box(HtmlRenderer::new()));
    let (renderer, construction_allocations) = allocations(|| black_box(HtmlRenderer::new()));
    drop(renderer);

    assert_eq!(
        construction_allocations, 0,
        "the default renderer must not build abbreviation state"
    );
}

#[test]
fn built_in_matcher_construction_stays_within_its_allocation_budget() {
    drop(black_box(
        HtmlRenderer::new().with_abbreviations(AbbreviationOptions::default()),
    ));
    let (renderer, construction_allocations) = allocations(|| {
        black_box(HtmlRenderer::new().with_abbreviations(AbbreviationOptions::default()))
    });
    drop(renderer);

    assert!(
        construction_allocations < 32,
        "built-in matcher setup should not allocate once per built-in term or title; got {construction_allocations} allocations"
    );
}

#[test]
fn enabled_renderer_reuses_its_dictionary_without_render_allocations() {
    let source = "x\\_API HTTP/2 XYZ ID";
    let allocator = Allocator::for_source_len(source.len());
    let document = Parser::new(&allocator, source).parse().unwrap();
    let Node::Paragraph(paragraph) = &document.children[0] else {
        panic!("expected a paragraph");
    };
    assert!(
        paragraph.children.len() > 1,
        "the test needs multiple text nodes"
    );
    assert!(
        paragraph
            .children
            .iter()
            .all(|node| matches!(node, Node::Text(_)))
    );
    let mut abbreviations = AbbreviationOptions::default();
    abbreviations.overrides = BTreeMap::from([("ID".to_string(), Some("Identifier".to_string()))]);

    let (mut renderer, construction_allocations) = allocations(|| {
        HtmlRenderer::with_options_and_abbreviations(HtmlRendererOptions::default(), abbreviations)
    });
    assert!(
        construction_allocations > 0,
        "building the enabled dictionary must be visible to the counter"
    );
    let _ = black_box(renderer.render_borrowed(&document));
    let (_, reused_render_allocations) =
        allocations(|| black_box(renderer.render_borrowed(&document)));
    assert_eq!(
        reused_render_allocations, 0,
        "a warmed reusable renderer should keep its dictionary and output capacity"
    );
}
