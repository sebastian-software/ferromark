//! Local characterization of default, per-node, and code-only render paths.
use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use ferromark::{Allocator, HtmlRenderer, NoHtmlRenderHooks, Parser};
use std::hint::black_box;

fn code_block_hooks(c: &mut Criterion) {
    let fixtures = [
        ("short-prose", "A **small** paragraph with a [link](https://example.com).".to_owned()),
        ("long-prose", "# Section\n\nA paragraph with **emphasis**, a [link](https://example.com), and `code`.\n\n- one\n- two\n\n".repeat(80)),
        ("code-fences", "# Code\n\n```rust\nfn main() {}\n```\n\n".repeat(40)),
    ];
    let mut group = c.benchmark_group("code_block_hooks");
    for (name, source) in fixtures {
        let arena = Allocator::for_source_len(source.len());
        let document = Parser::new(&arena, &source).parse().unwrap();
        let mut renderer = HtmlRenderer::new();
        let expected = renderer.render(&document);
        assert_eq!(
            renderer.render_with_hooks(&document, &mut NoHtmlRenderHooks),
            expected
        );
        assert_eq!(
            renderer.render_with_code_hooks(&document, &mut NoHtmlRenderHooks),
            expected
        );
        group.bench_function(BenchmarkId::new(name, "default"), |b| {
            b.iter(|| {
                black_box(renderer.render_borrowed(black_box(&document)));
            });
        });
        group.bench_function(BenchmarkId::new(name, "node-hooks"), |b| {
            b.iter(|| {
                black_box(
                    renderer
                        .render_borrowed_with_hooks(black_box(&document), &mut NoHtmlRenderHooks),
                );
            });
        });
        group.bench_function(BenchmarkId::new(name, "code-hooks"), |b| {
            b.iter(|| {
                black_box(
                    renderer.render_borrowed_with_code_hooks(
                        black_box(&document),
                        &mut NoHtmlRenderHooks,
                    ),
                );
            });
        });
    }
    group.finish();
}
criterion_group!(benches, code_block_hooks);
criterion_main!(benches);
