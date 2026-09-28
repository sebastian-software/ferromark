//! Parser cost of disabled and enabled image syntax on representative shapes.

use std::hint::black_box;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use ferromark::allocator::Allocator;
use ferromark::parser::{Parser, ParserOptions};

fn parse(source: &str, attributes: bool, captions: bool) {
    let allocator = Allocator::for_source_len(source.len());
    let options = ParserOptions {
        image_attributes: attributes,
        image_captions: captions,
        ..ParserOptions::gfm()
    };
    black_box(
        Parser::with_options(&allocator, black_box(source), options)
            .parse()
            .unwrap(),
    );
}

fn bench_image_captions(c: &mut Criterion) {
    let mut group = c.benchmark_group("image_syntax_parse");
    let prose =
        "# Publishing\n\nThis document explains normal Markdown prose and links.\n\n".repeat(64);
    let images = "![Alt](image.svg){.diagram}\n\n: A **caption** {#figure .wide}\n\n".repeat(64);
    let malformed = "![Alt](image.svg){#one #two}\n\n: Caption {#bad #other}\n\n".repeat(64);

    for (workload, source) in [
        ("prose", &prose),
        ("images", &images),
        ("malformed", &malformed),
    ] {
        group.throughput(Throughput::Bytes(source.len() as u64));
        for (mode, attributes, captions) in [
            ("off", false, false),
            ("attributes", true, false),
            ("captions", false, true),
            ("both", true, true),
        ] {
            group.bench_with_input(
                BenchmarkId::new(format!("{workload}/{mode}"), source.len()),
                source,
                |b, source| b.iter(|| parse(source, attributes, captions)),
            );
        }
    }
    group.finish();
}

criterion_group!(benches, bench_image_captions);
criterion_main!(benches);
