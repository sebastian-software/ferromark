//! Comparison benchmarks: md-fast vs other Rust Markdown parsers
//!
//! Run with: cargo bench --bench comparison
//!
//! Parsers compared:
//! - md-fast (this crate)
//! - pulldown-cmark (most popular, used by rustdoc)
//! - comrak (100% CommonMark compliant, GFM support)
//! - markdown (markdown-rs, wooorm's parser)

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};

/// Sample documents for benchmarking
mod samples {
    /// Tiny document - baseline measurement
    pub const TINY: &str = "Hello, **world**!";

    /// Small README-style document
    pub const SMALL: &str = r#"# Heading

This is a paragraph with *emphasis* and **strong** text.

- Item 1
- Item 2
- Item 3

`inline code` and [a link](https://example.com).
"#;

    /// Medium-sized README
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

    /// Simple document: headers, lists, paragraphs, basic inline formatting
    pub const SIMPLE: &str = r#"# Title

## Section A

This is a paragraph with *emphasis* and **strong**.

- Item one
- Item two
- Item three

Another paragraph.
"#;

    /// Link-heavy document: autolinks, inline links, entities, images
    pub const LINKS: &str = r#"# Links

Visit <https://example.com> or <mailto:test@example.com>.

Inline [link](https://example.com/path?query=1&x=2) with &amp; entity.

![Image alt](https://example.com/image.png "Title")

Text with `code` and [another link](https://example.com).
"#;

    /// Reference link definitions and uses
    pub const REFS: &str = r#"[ref-1]: https://example.com "Example"
[ref-2]: /relative/path 'Rel'

This uses [ref-1] and [ref-2].

[Another ref][ref-1] and [short][ref-2].
"#;

    /// Nested lists and mixed block elements
    pub const LISTS: &str = r#"# Lists

1. Ordered
   1. Nested ordered
   2. Nested ordered
2. Ordered
   - Nested unordered
     - Deep nested

> Blockquote
> - Quoted list item
>   - Nested in quote
"#;

    /// HTML blocks and inline HTML
    pub const HTML: &str = r#"<div class="note">
<p>Inline <em>HTML</em> inside a block.</p>
</div>

Paragraph with <span class="hi">inline HTML</span> and &amp; entity.

<script>
var x = 1;
</script>
"#;

    /// Mixed realistic document with multiple features
    pub const MIXED: &str = r#"# Mixed Sample

Intro paragraph with *emphasis*, **strong**, and `code`.

[ref]: https://example.com "Title"

## Section

> Blockquote with [link][ref] and <https://example.com>.

- List item with ![image](https://example.com/x.png)
- List item with `<code>` and <span>HTML</span>

```rust
fn example() {
    println!("Hello");
}
```

Paragraph after code.
"#;

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
}

/// Parse with md-fast
fn parse_md_fast(input: &str) -> String {
    md_fast::to_html(input)
}

/// Parse with pulldown-cmark
fn parse_pulldown_cmark(input: &str) -> String {
    use pulldown_cmark::{html, Parser};
    let parser = Parser::new(input);
    let mut output = String::new();
    html::push_html(&mut output, parser);
    output
}

/// Parse with comrak
fn parse_comrak(input: &str) -> String {
    comrak::markdown_to_html(input, &comrak::Options::default())
}

/// Parse with markdown-rs
fn parse_markdown_rs(input: &str) -> String {
    markdown::to_html(input)
}

fn bench_tiny(c: &mut Criterion) {
    let mut group = c.benchmark_group("tiny");
    let input = samples::TINY;
    group.throughput(Throughput::Bytes(input.len() as u64));

    group.bench_function("md-fast", |b| {
        b.iter(|| parse_md_fast(black_box(input)))
    });
    group.bench_function("pulldown-cmark", |b| {
        b.iter(|| parse_pulldown_cmark(black_box(input)))
    });
    group.bench_function("comrak", |b| {
        b.iter(|| parse_comrak(black_box(input)))
    });
    group.bench_function("markdown-rs", |b| {
        b.iter(|| parse_markdown_rs(black_box(input)))
    });

    group.finish();
}

fn bench_small(c: &mut Criterion) {
    let mut group = c.benchmark_group("small");
    let input = samples::SMALL;
    group.throughput(Throughput::Bytes(input.len() as u64));

    group.bench_function("md-fast", |b| {
        b.iter(|| parse_md_fast(black_box(input)))
    });
    group.bench_function("pulldown-cmark", |b| {
        b.iter(|| parse_pulldown_cmark(black_box(input)))
    });
    group.bench_function("comrak", |b| {
        b.iter(|| parse_comrak(black_box(input)))
    });
    group.bench_function("markdown-rs", |b| {
        b.iter(|| parse_markdown_rs(black_box(input)))
    });

    group.finish();
}

fn bench_medium(c: &mut Criterion) {
    let mut group = c.benchmark_group("medium");
    let input = samples::MEDIUM;
    group.throughput(Throughput::Bytes(input.len() as u64));

    group.bench_function("md-fast", |b| {
        b.iter(|| parse_md_fast(black_box(input)))
    });
    group.bench_function("pulldown-cmark", |b| {
        b.iter(|| parse_pulldown_cmark(black_box(input)))
    });
    group.bench_function("comrak", |b| {
        b.iter(|| parse_comrak(black_box(input)))
    });
    group.bench_function("markdown-rs", |b| {
        b.iter(|| parse_markdown_rs(black_box(input)))
    });

    group.finish();
}

fn bench_large(c: &mut Criterion) {
    let mut group = c.benchmark_group("large");
    let input = samples::large();
    group.throughput(Throughput::Bytes(input.len() as u64));

    group.bench_function("md-fast", |b| {
        b.iter(|| parse_md_fast(black_box(&input)))
    });
    group.bench_function("pulldown-cmark", |b| {
        b.iter(|| parse_pulldown_cmark(black_box(&input)))
    });
    group.bench_function("comrak", |b| {
        b.iter(|| parse_comrak(black_box(&input)))
    });
    group.bench_function("markdown-rs", |b| {
        b.iter(|| parse_markdown_rs(black_box(&input)))
    });

    group.finish();
}

/// Complexity comparison across representative feature sets
fn bench_complexity(c: &mut Criterion) {
    let mut group = c.benchmark_group("complexity");

    let cases: Vec<(&str, &str)> = vec![
        ("simple", samples::SIMPLE),
        ("links", samples::LINKS),
        ("refs", samples::REFS),
        ("lists", samples::LISTS),
        ("html", samples::HTML),
        ("mixed", samples::MIXED),
    ];

    for (name, input) in &cases {
        group.throughput(Throughput::Bytes(input.len() as u64));

        group.bench_with_input(BenchmarkId::new("md-fast", name), input, |b, s| {
            b.iter(|| parse_md_fast(black_box(s)))
        });
        group.bench_with_input(BenchmarkId::new("pulldown-cmark", name), input, |b, s| {
            b.iter(|| parse_pulldown_cmark(black_box(s)))
        });
        group.bench_with_input(BenchmarkId::new("comrak", name), input, |b, s| {
            b.iter(|| parse_comrak(black_box(s)))
        });
        group.bench_with_input(BenchmarkId::new("markdown-rs", name), input, |b, s| {
            b.iter(|| parse_markdown_rs(black_box(s)))
        });
    }

    group.finish();
}

/// Throughput comparison across all document sizes
fn bench_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("throughput");

    let sizes: Vec<(&str, String)> = vec![
        ("tiny", samples::TINY.to_string()),
        ("small", samples::SMALL.to_string()),
        ("medium", samples::MEDIUM.to_string()),
        ("large", samples::large()),
    ];

    for (name, input) in &sizes {
        group.throughput(Throughput::Bytes(input.len() as u64));

        group.bench_with_input(BenchmarkId::new("md-fast", name), input, |b, s| {
            b.iter(|| parse_md_fast(black_box(s)))
        });
        group.bench_with_input(BenchmarkId::new("pulldown-cmark", name), input, |b, s| {
            b.iter(|| parse_pulldown_cmark(black_box(s)))
        });
        group.bench_with_input(BenchmarkId::new("comrak", name), input, |b, s| {
            b.iter(|| parse_comrak(black_box(s)))
        });
        group.bench_with_input(BenchmarkId::new("markdown-rs", name), input, |b, s| {
            b.iter(|| parse_markdown_rs(black_box(s)))
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_tiny,
    bench_small,
    bench_medium,
    bench_large,
    bench_complexity,
    bench_throughput
);
criterion_main!(benches);
