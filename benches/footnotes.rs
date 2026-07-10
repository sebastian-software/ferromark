//! Scaling benchmark for distinct footnote references.
//!
//! Run with: `cargo bench --bench footnotes`

use criterion::{BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main};
use ferromark::{Options, to_html_with_options};

fn document_with_distinct_footnotes(count: usize) -> String {
    let mut markdown = String::with_capacity(count * 32);
    for index in 0..count {
        markdown.push_str("[^n");
        markdown.push_str(&index.to_string());
        markdown.push_str("] ");
    }
    markdown.push('\n');
    for index in 0..count {
        markdown.push_str("[^n");
        markdown.push_str(&index.to_string());
        markdown.push_str("]: note\n");
    }
    markdown
}

fn benchmark_footnote_scaling(c: &mut Criterion) {
    let options = Options {
        footnotes: true,
        ..Options::default()
    };
    let mut group = c.benchmark_group("footnotes/distinct_references");
    group.sample_size(10);

    for count in [1_000usize, 10_000, 100_000] {
        let markdown = document_with_distinct_footnotes(count);
        group.throughput(Throughput::Elements(count as u64));
        group.bench_with_input(BenchmarkId::from_parameter(count), &markdown, |b, input| {
            b.iter(|| to_html_with_options(black_box(input), black_box(&options)));
        });
    }
    group.finish();
}

criterion_group!(benches, benchmark_footnote_scaling);
criterion_main!(benches);
