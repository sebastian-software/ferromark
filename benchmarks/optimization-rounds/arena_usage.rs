//! Parse-only arena occupancy diagnostic; deliberately separate from timings.
use ferromark::{Allocator, Parser, ParserOptions};

fn main() {
    for path in std::env::args().skip(1) {
        let input = std::fs::read_to_string(path).expect("UTF-8 input");
        let arena = Allocator::for_source_len(input.len());
        let document = Parser::with_options(&arena, &input, ParserOptions {
            footnotes: false,
            ..ParserOptions::gfm()
        }).parse().expect("parse input");
        std::hint::black_box(&document);
        // SAFETY: inspect only chunk lengths, never dereference raw pointers;
        // no arena allocations occur while the iterator is alive.
        let used: usize = unsafe {
            arena.bump().iter_allocated_chunks_raw().map(|(_, len)| len).sum()
        };
        println!("{} {} {}", input.len(), arena.allocated_bytes(), used);
    }
}
