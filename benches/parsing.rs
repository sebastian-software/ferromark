//! Performance benchmarks for ferromark
//!
//! Run with: cargo bench

use criterion::{BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main};

/// Sample Markdown documents of various sizes
mod samples {
    pub const TINY: &str = "Hello, **world**!";

    pub const SMALL: &str = r#"# Heading

This is a paragraph with *emphasis* and **strong** text.

- Item 1
- Item 2
- Item 3

`inline code` and [a link](https://example.com).
"#;

    pub const MEDIUM: &str = r#"# Project README

This is a sample README file that demonstrates various Markdown features.

## Features

- Fast parsing
- Zero-copy design
- SIMD acceleration

### Code Example

```rust
fn main() {
    println!("Hello, world!");
}
```

## Performance

The parser achieves **high throughput** on typical documents.

> This is a blockquote with some *emphasized* text.

### Links

- [GitHub](https://github.com)
- [Documentation](https://docs.rs)

## Conclusion

Thank you for reading!
"#;

    /// CommonMark-heavy documents (wiki-style, text-heavy)
    pub const COMMONMARK_5K: &str = include_str!("fixtures/commonmark-5k.md");
    pub const COMMONMARK_20K: &str = include_str!("fixtures/commonmark-20k.md");
    pub const COMMONMARK_50K: &str = include_str!("fixtures/commonmark-50k.md");

    /// Table-heavy document (~5KB)
    pub const TABLES_5K: &str = include_str!("fixtures/tables-5k.md");

    /// Generate a large document by repeating sections
    pub fn large() -> String {
        let section = r#"
## Section Title

This paragraph contains various inline elements like *emphasis*, **strong**,
`code`, and [links](https://example.com).

- First bullet point with **bold** text
- Second bullet point with *italic* text
- Third point with `code`

> A blockquote that spans
> multiple lines.

```rust
fn example() {
    let x = 42;
    println!("{}", x);
}
```

Another paragraph to add some content. This helps test the parser's ability
to handle longer documents efficiently.

"#;
        section.repeat(50)
    }

    /// Pathological document with many potential delimiters
    pub fn pathological_emphasis() -> String {
        // Many potential opener/closer pairs
        "*a ".repeat(1000) + &"b* ".repeat(1000)
    }

    /// Document with deeply nested structures
    pub fn pathological_nested() -> String {
        "> ".repeat(100) + "deep\n"
    }
}

fn bench_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("parsing");

    // Tiny document
    group.throughput(Throughput::Bytes(samples::TINY.len() as u64));
    group.bench_function("tiny", |b| {
        b.iter(|| ferromark::to_html(black_box(samples::TINY)))
    });

    // Small document
    group.throughput(Throughput::Bytes(samples::SMALL.len() as u64));
    group.bench_function("small", |b| {
        b.iter(|| ferromark::to_html(black_box(samples::SMALL)))
    });

    // Medium document
    group.throughput(Throughput::Bytes(samples::MEDIUM.len() as u64));
    group.bench_function("medium", |b| {
        b.iter(|| ferromark::to_html(black_box(samples::MEDIUM)))
    });

    // Large document
    let large = samples::large();
    group.throughput(Throughput::Bytes(large.len() as u64));
    group.bench_function("large", |b| {
        b.iter(|| ferromark::to_html(black_box(&large)))
    });

    // CommonMark documents (wiki-style)
    let commonmark_5k = samples::COMMONMARK_5K;
    group.throughput(Throughput::Bytes(commonmark_5k.len() as u64));
    group.bench_function("commonmark_5k", |b| {
        b.iter(|| ferromark::to_html(black_box(commonmark_5k)))
    });

    let commonmark_20k = samples::COMMONMARK_20K;
    group.throughput(Throughput::Bytes(commonmark_20k.len() as u64));
    group.bench_function("commonmark_20k", |b| {
        b.iter(|| ferromark::to_html(black_box(commonmark_20k)))
    });

    let commonmark_50k = samples::COMMONMARK_50K;
    group.throughput(Throughput::Bytes(commonmark_50k.len() as u64));
    group.bench_function("commonmark_50k", |b| {
        b.iter(|| ferromark::to_html(black_box(commonmark_50k)))
    });

    // Table-heavy document
    let tables_5k = samples::TABLES_5K;
    group.throughput(Throughput::Bytes(tables_5k.len() as u64));
    group.bench_function("tables_5k", |b| {
        b.iter(|| ferromark::to_html(black_box(tables_5k)))
    });

    group.finish();
}

fn bench_escaping(c: &mut Criterion) {
    let mut group = c.benchmark_group("escaping");

    // Plain text (no escaping needed)
    let plain = "Hello, this is plain text without any special characters. ".repeat(100);
    group.throughput(Throughput::Bytes(plain.len() as u64));
    group.bench_function("plain_text", |b| {
        b.iter(|| {
            let mut out = Vec::with_capacity(plain.len());
            ferromark::escape::escape_text_into(&mut out, black_box(plain.as_bytes()));
            out
        })
    });

    // Text with HTML that needs escaping
    let html_heavy = "<script>alert('xss')</script> & more <tags> here! ".repeat(100);
    group.throughput(Throughput::Bytes(html_heavy.len() as u64));
    group.bench_function("html_heavy", |b| {
        b.iter(|| {
            let mut out = Vec::with_capacity(html_heavy.len() * 2);
            ferromark::escape::escape_text_into(&mut out, black_box(html_heavy.as_bytes()));
            out
        })
    });

    group.finish();
}

fn bench_pathological(c: &mut Criterion) {
    let mut group = c.benchmark_group("pathological");
    group.sample_size(20); // Fewer samples for slow cases

    let emphasis = samples::pathological_emphasis();
    group.throughput(Throughput::Bytes(emphasis.len() as u64));
    group.bench_function("emphasis_explosion", |b| {
        b.iter(|| ferromark::to_html(black_box(&emphasis)))
    });

    let nested = samples::pathological_nested();
    group.throughput(Throughput::Bytes(nested.len() as u64));
    group.bench_function("deep_nesting", |b| {
        b.iter(|| ferromark::to_html(black_box(&nested)))
    });

    group.finish();
}

fn bench_buffer_reuse(c: &mut Criterion) {
    let mut group = c.benchmark_group("buffer_reuse");

    let input = samples::MEDIUM;
    group.throughput(Throughput::Bytes(input.len() as u64));

    // Without buffer reuse
    group.bench_function("without_reuse", |b| {
        b.iter(|| ferromark::to_html(black_box(input)))
    });

    // With buffer reuse
    group.bench_function("with_reuse", |b| {
        let mut buffer = Vec::with_capacity(input.len() * 2);
        b.iter(|| {
            ferromark::to_html_into(black_box(input), &mut buffer);
            black_box(&buffer);
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_parsing,
    bench_escaping,
    bench_pathological,
    bench_buffer_reuse
);
criterion_main!(benches);
