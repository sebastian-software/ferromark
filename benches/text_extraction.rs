//! Measure parsing and prepared text extraction in separate regions.
use std::hint::black_box;

use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use ferromark::{Allocator, JsxRenderer, Parser, ParserOptions, TextExtractionOptions};

fn bench_text_extraction(c: &mut Criterion) {
    let source = include_str!("fixtures/text-extraction.mdx").repeat(64);
    let parser_options = ParserOptions {
        mdx: true,
        mdx_compatible: true,
        ..ParserOptions::gfm()
    };
    let allocator = Allocator::for_source_len(source.len());
    let document = Parser::with_options(&allocator, &source, parser_options.clone())
        .parse()
        .unwrap();
    let with_code = TextExtractionOptions::default();
    let without_code = TextExtractionOptions {
        include_block_code: false,
        ..TextExtractionOptions::default()
    };
    // Check final heading planning before any timing; no highlighting hook runs.
    let rendered = JsxRenderer::new().render(&document, &source);
    for options in [&with_code, &without_code] {
        let extraction = document.extract_text(options);
        assert_eq!(extraction.sections.len(), rendered.headings.len());
        for (section, heading) in extraction.sections.iter().zip(&rendered.headings) {
            assert_eq!(section.id, heading.id);
        }
    }
    let mut group = c.benchmark_group("text_extraction");
    group.throughput(Throughput::Bytes(source.len() as u64));
    group.bench_function("parse", |b| {
        b.iter(|| {
            let allocator = Allocator::for_source_len(source.len());
            black_box(
                Parser::with_options(&allocator, black_box(&source), parser_options.clone())
                    .parse()
                    .unwrap(),
            );
        });
    });
    for (name, options) in [
        ("prepared_with_code", &with_code),
        ("prepared_without_code", &without_code),
    ] {
        group.bench_function(name, |b| {
            b.iter(|| black_box(document.extract_text(black_box(options))));
        });
    }
    group.finish();
}

criterion_group!(benches, bench_text_extraction);
criterion_main!(benches);
