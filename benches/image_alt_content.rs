//! Characterize the opt-in cost of retaining image alternative content.

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use ferromark::{Allocator, HtmlRenderer, Parser, ParserOptions};
use std::hint::black_box;

fn options(retain: bool) -> ParserOptions {
    ParserOptions {
        image_alt_content: retain,
        ..ParserOptions::gfm()
    }
}

fn bench_image_alt_content(c: &mut Criterion) {
    let mut group = c.benchmark_group("image_alt_content");
    for (name, source) in [
        (
            "prose",
            "A paragraph with **emphasis** and ordinary prose.\n\n".repeat(64),
        ),
        (
            "plain",
            "![A descriptive image label](image.png)\n\n".repeat(64),
        ),
        (
            "rich",
            "![Code `sample` and **é &amp; prose**](image.png)\n\n".repeat(64),
        ),
    ] {
        let arena = Allocator::for_source_len(source.len());
        let baseline = Parser::with_options(&arena, &source, options(false))
            .parse()
            .unwrap();
        let candidate = Parser::with_options(&arena, &source, options(true))
            .parse()
            .unwrap();
        assert_eq!(
            HtmlRenderer::new().render(&baseline),
            HtmlRenderer::new().render(&candidate)
        );
        group.throughput(Throughput::Bytes(source.len() as u64));
        for retain in [false, true] {
            // Keep all allocations in one pre-sized chunk, so consumed arena
            // bytes are exactly initial capacity minus remaining capacity.
            let arena = Allocator::with_capacity(source.len() * 128);
            let capacity = arena.chunk_capacity();
            let document = Parser::with_options(&arena, &source, options(retain))
                .parse()
                .unwrap();
            assert_eq!(arena.allocated_bytes(), capacity);
            #[allow(clippy::print_stderr)]
            {
                eprintln!(
                    "arena {name} retain={retain}: {} bytes",
                    capacity - arena.chunk_capacity()
                );
            }
            black_box(document);
            group.bench_with_input(
                BenchmarkId::new(name, if retain { "on" } else { "off" }),
                &source,
                |b, source| {
                    b.iter(|| {
                        let arena = Allocator::for_source_len(source.len());
                        black_box(
                            Parser::with_options(&arena, black_box(source), options(retain))
                                .parse()
                                .unwrap(),
                        );
                    });
                },
            );
        }
    }
    group.finish();
}

criterion_group!(benches, bench_image_alt_content);
criterion_main!(benches);
