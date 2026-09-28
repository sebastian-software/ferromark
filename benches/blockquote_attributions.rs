//! Parse cost of enabled and disabled block quote attribution syntax.

use std::hint::black_box;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use ferromark::allocator::Allocator;
use ferromark::parser::{Parser, ParserOptions};

fn parse(source: &str, attributions: bool) {
    let allocator = Allocator::for_source_len(source.len());
    let options = ParserOptions {
        blockquote_attributions: attributions,
        ..ParserOptions::gfm()
    };
    black_box(
        Parser::with_options(&allocator, black_box(source), options)
            .parse()
            .unwrap(),
    );
}

fn bench_blockquote_attributions(c: &mut Criterion) {
    let mut group = c.benchmark_group("blockquote_attributions_parse");
    let prose = "# Publishing\n\nThis document explains Markdown prose and links.\n\n".repeat(64);
    let quotes = "> A passage with **formatting** and [a link](/guide).\n\n".repeat(64);
    let attributed =
        "> A passage with **formatting** and [a link](/guide).\n: Jane Doe {#source .byline}\n\n"
            .repeat(64);
    let malformed = "> A passage.\n: Jane {#one #two}\n\n".repeat(64);

    for (workload, source) in [
        ("prose", &prose),
        ("quotes", &quotes),
        ("attributed", &attributed),
        ("malformed", &malformed),
    ] {
        group.throughput(Throughput::Bytes(source.len() as u64));
        for enabled in [false, true] {
            group.bench_with_input(
                BenchmarkId::new(
                    format!("{workload}/{}", if enabled { "on" } else { "off" }),
                    source.len(),
                ),
                source,
                |b, source| b.iter(|| parse(source, enabled)),
            );
        }
    }
    group.finish();
}

criterion_group!(benches, bench_blockquote_attributions);
criterion_main!(benches);
