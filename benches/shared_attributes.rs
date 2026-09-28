//! Enabled and disabled parser cost of shared attributes and bracketed spans.

use std::hint::black_box;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use ferromark::allocator::Allocator;
use ferromark::parser::{Parser, ParserOptions};

fn parse(source: &str, extended_attributes: bool, bracketed_spans: bool) {
    let allocator = Allocator::for_source_len(source.len());
    let options = ParserOptions {
        extended_attributes,
        bracketed_spans,
        ..ParserOptions::gfm()
    };
    black_box(
        Parser::with_options(&allocator, black_box(source), options)
            .parse()
            .unwrap(),
    );
}

fn bench_shared_attributes(c: &mut Criterion) {
    let mut group = c.benchmark_group("shared_attributes_parse");
    let prose =
        "# Notes\n\nOrdinary Markdown text with a [link](/docs) and **emphasis**.\n\n".repeat(64);
    let annotated = "[Product **offer**]{.product lang=en sku=\"A-17\"}\n\n[Docs](/docs){.external hreflang=en}\n\n".repeat(64);
    let malformed = "[Text]{bare #one #two}\n\n[Text]{title=\"never closed}\n\n".repeat(64);
    let distant = format!("{} }}", "[Text]{title=\"".repeat(512));
    for (workload, source) in [
        ("prose", &prose),
        ("annotated", &annotated),
        ("malformed", &malformed),
        ("distant", &distant),
    ] {
        group.throughput(Throughput::Bytes(source.len() as u64));
        for (mode, attributes, spans) in [
            ("off", false, false),
            ("attributes", true, false),
            ("spans", false, true),
            ("both", true, true),
        ] {
            group.bench_with_input(
                BenchmarkId::new(format!("{workload}/{mode}"), source.len()),
                source,
                |b, source| b.iter(|| parse(source, attributes, spans)),
            );
        }
    }
    group.finish();
}

criterion_group!(benches, bench_shared_attributes);
criterion_main!(benches);
