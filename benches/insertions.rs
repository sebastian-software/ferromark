//! Benchmarks for optional inline insertion parsing.

use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use ferromark::allocator::Allocator;
use ferromark::parser::{Parser, ParserOptions};
use std::hint::black_box;

fn bench_insertions(c: &mut Criterion) {
    let plain_plus = "C++ is still plain text. ".repeat(256);
    let unmatched_run = format!("{}tail", "+".repeat(16_384));
    let repeated_pairs = "++inserted text++ ".repeat(512);

    let mut group = c.benchmark_group("inline_insertions");
    for (name, source) in [
        ("ordinary_plus", plain_plus.as_str()),
        ("unmatched_plus_run", unmatched_run.as_str()),
        ("repeated_pairs", repeated_pairs.as_str()),
    ] {
        group.throughput(Throughput::Bytes(source.len() as u64));
        for enabled in [false, true] {
            let variant = if enabled { "enabled" } else { "disabled" };
            group.bench_function(format!("{name}/{variant}"), |b| {
                b.iter(|| {
                    let allocator = Allocator::new();
                    let options = ParserOptions {
                        insertions: enabled,
                        ..ParserOptions::default()
                    };
                    let parser = Parser::with_options(&allocator, black_box(source), options);
                    let _ = parser.parse();
                });
            });
        }
    }
    group.finish();
}

criterion_group!(benches, bench_insertions);
criterion_main!(benches);
