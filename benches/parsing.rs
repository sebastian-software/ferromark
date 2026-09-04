//! Performance benchmarks for ferromark
//!
//! Run with: cargo bench

use std::hint::black_box;

use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use ferromark::escape_text_into;

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
    pub const COMMONMARK_1M: &str = include_str!("fixtures/commonmark-1m.md");

    /// Keep each pathological case directly comparable at the same input size.
    pub const PATHOLOGICAL_BYTES: usize = 64 * 1024;

    /// Table-heavy document (~5KB)
    pub const TABLES_5K: &str = include_str!("fixtures/tables-5k.md");

    /// Definition-list-heavy document (~5KB)
    pub const DEFLISTS_5K: &str = include_str!("fixtures/deflists-5k.md");

    /// Tables with merged cells and column-width delimiter hints (~5KB)
    pub const TABLES_MERGED_5K: &str = include_str!("fixtures/tables-merged-5k.md");

    /// Mixed document exercising every extension at once (~5KB)
    pub const MIXED_EXTENDED_5K: &str = include_str!("fixtures/mixed-extended-5k.md");

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

    /// Generate many natural and generated suffix collisions. The first
    /// repeated `foo` scans the occupied natural suffixes; the per-base
    /// cursor keeps all later repeats amortized linear.
    pub fn heading_collision_suffixes() -> String {
        let mut input = String::with_capacity(64 * 1024);
        input.push_str("# foo\n\n");
        for suffix in 1..=1024 {
            input.push_str("# foo-");
            input.push_str(&suffix.to_string());
            input.push_str("\n\n");
        }
        for _ in 0..1024 {
            input.push_str("# foo\n\n");
        }
        input
    }

    /// Pathological document with many potential delimiters
    pub fn pathological_emphasis() -> String {
        // Many potential opener/closer pairs
        "*a ".repeat(1000) + &"b* ".repeat(1000)
    }

    /// Mark-capped emphasis followed by link boundaries found by full-text scans.
    pub fn pathological_emphasis_boundaries() -> String {
        "*a ".repeat(2048) + &"b* ".repeat(2048) + &"<https://example.com/path> ".repeat(4096)
    }

    /// Document with deeply nested structures
    pub fn pathological_nested() -> String {
        "> ".repeat(100) + "deep\n"
    }

    /// Classic nested-bracket stress case for link/reference resolution.
    pub fn pathological_brackets() -> String {
        let half = PATHOLOGICAL_BYTES / 2;
        "[".repeat(half) + &"]".repeat(half)
    }

    /// Unmatched backtick runs of every supported length, split into paragraphs.
    pub fn pathological_backticks() -> String {
        let mut paragraph = String::new();
        for run_length in 1..=ferromark::limits::MAX_CODE_SPAN_BACKTICKS {
            paragraph.push_str(&"`".repeat(run_length));
            paragraph.push('x');
        }
        paragraph.push_str("\n\n");
        repeat_to_budget(&paragraph)
    }

    /// Many unique, valid definitions exercise label normalization and storage.
    pub fn pathological_reference_definitions() -> String {
        let mut input = String::with_capacity(PATHOLOGICAL_BYTES);
        let mut index = 0;
        loop {
            let definition =
                format!("[reference-{index:05}-xxxxxxxxxxxxxxxx]: https://example.com/{index}\n");
            if input.len() + definition.len() > PATHOLOGICAL_BYTES {
                break;
            }
            input.push_str(&definition);
            index += 1;
        }
        input.push_str(&" ".repeat(PATHOLOGICAL_BYTES - input.len()));
        input
    }

    /// Repeated invalid tag starts exercise the inline HTML candidate scanner.
    pub fn pathological_html_starts() -> String {
        repeat_to_budget("<not-a-tag ")
    }

    fn repeat_to_budget(pattern: &str) -> String {
        debug_assert!(pattern.is_ascii());
        let repetitions = PATHOLOGICAL_BYTES / pattern.len();
        let remainder = PATHOLOGICAL_BYTES % pattern.len();
        let mut input = pattern.repeat(repetitions);
        input.push_str(&pattern[..remainder]);
        debug_assert_eq!(input.len(), PATHOLOGICAL_BYTES);
        input
    }

    /// Presentation-like input that exercises semantic slide boundaries.
    pub fn thematic_breaks() -> String {
        "Slide content with **formatting**.\n\n---\n\n".repeat(500)
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

    let commonmark_1m = samples::COMMONMARK_1M;
    group.throughput(Throughput::Bytes(commonmark_1m.len() as u64));
    group.bench_function("commonmark_1m", |b| {
        b.iter(|| ferromark::to_html(black_box(commonmark_1m)))
    });

    // Table-heavy document
    let tables_5k = samples::TABLES_5K;
    group.throughput(Throughput::Bytes(tables_5k.len() as u64));
    group.bench_function("tables_5k", |b| {
        b.iter(|| ferromark::to_html(black_box(tables_5k)))
    });

    let thematic_breaks = samples::thematic_breaks();
    group.throughput(Throughput::Bytes(thematic_breaks.len() as u64));
    group.bench_function("thematic_breaks", |b| {
        b.iter(|| ferromark::to_html(black_box(&thematic_breaks)))
    });

    group.finish();
}

/// Benchmarks for extension-heavy inputs under the options they require.
fn bench_extensions(c: &mut Criterion) {
    use ferromark::{Options, RenderPolicy};

    let mut group = c.benchmark_group("extensions");

    let deflist_options = ferromark::options!(Options::gfm();
        definition_lists: true,);
    group.throughput(Throughput::Bytes(samples::DEFLISTS_5K.len() as u64));
    group.bench_function("deflists_5k", |b| {
        b.iter(|| {
            ferromark::to_html_with_options(black_box(samples::DEFLISTS_5K), &deflist_options)
        })
    });

    let merged_options = ferromark::options!(Options::gfm();
        merged_table_cells: true,
        table_column_widths: true,);
    group.throughput(Throughput::Bytes(samples::TABLES_MERGED_5K.len() as u64));
    group.bench_function("tables_merged_5k", |b| {
        b.iter(|| {
            ferromark::to_html_with_options(black_box(samples::TABLES_MERGED_5K), &merged_options)
        })
    });

    let all_options = ferromark::options!(Options::default();
        render_policy: RenderPolicy::Untrusted,
        allow_html: true,
        allow_link_refs: true,
        tables: true,
        merged_table_cells: true,
        table_column_widths: true,
        strikethrough: true,
        highlight: true,
        superscript: true,
        subscript: true,
        task_lists: true,
        autolink_literals: true,
        disallowed_raw_html: true,
        footnotes: true,
        inline_footnotes: true,
        front_matter: true,
        heading_ids: true,
        math: true,
        callouts: true,
        definition_lists: true,
        line_comments: true,
        indented_code_blocks: true,
        link_base_path: None,);
    group.throughput(Throughput::Bytes(samples::MIXED_EXTENDED_5K.len() as u64));
    group.bench_function("mixed_extended_5k", |b| {
        b.iter(|| {
            ferromark::to_html_with_options(black_box(samples::MIXED_EXTENDED_5K), &all_options)
        })
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
            escape_text_into(&mut out, black_box(plain.as_bytes()));
            out
        })
    });

    // Text with HTML that needs escaping
    let html_heavy = "<script>alert('xss')</script> & more <tags> here! ".repeat(100);
    group.throughput(Throughput::Bytes(html_heavy.len() as u64));
    group.bench_function("html_heavy", |b| {
        b.iter(|| {
            let mut out = Vec::with_capacity(html_heavy.len() * 2);
            escape_text_into(&mut out, black_box(html_heavy.as_bytes()));
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

    let emphasis_boundaries = samples::pathological_emphasis_boundaries();
    group.throughput(Throughput::Bytes(emphasis_boundaries.len() as u64));
    group.bench_function("emphasis_many_link_boundaries", |b| {
        b.iter(|| ferromark::to_html(black_box(&emphasis_boundaries)))
    });

    let nested = samples::pathological_nested();
    group.throughput(Throughput::Bytes(nested.len() as u64));
    group.bench_function("deep_nesting", |b| {
        b.iter(|| ferromark::to_html(black_box(&nested)))
    });

    let brackets = samples::pathological_brackets();
    group.throughput(Throughput::Bytes(brackets.len() as u64));
    group.bench_function("bracket_explosion_64k", |b| {
        b.iter(|| ferromark::to_html(black_box(&brackets)))
    });

    let heading_collisions = samples::heading_collision_suffixes();
    group.throughput(Throughput::Bytes(heading_collisions.len() as u64));
    group.bench_function("heading_collision_suffixes", |b| {
        b.iter(|| ferromark::to_html(black_box(&heading_collisions)))
    });

    let backticks = samples::pathological_backticks();
    group.throughput(Throughput::Bytes(backticks.len() as u64));
    group.bench_function("unmatched_backticks_64k", |b| {
        b.iter(|| ferromark::to_html(black_box(&backticks)))
    });

    let reference_definitions = samples::pathological_reference_definitions();
    group.throughput(Throughput::Bytes(reference_definitions.len() as u64));
    group.bench_function("reference_definitions_64k", |b| {
        b.iter(|| ferromark::to_html(black_box(&reference_definitions)))
    });

    let html_starts = samples::pathological_html_starts();
    group.throughput(Throughput::Bytes(html_starts.len() as u64));
    group.bench_function("invalid_html_starts_64k", |b| {
        b.iter(|| ferromark::to_html(black_box(&html_starts)))
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

fn bench_renderer_session(c: &mut Criterion) {
    let mut group = c.benchmark_group("renderer_session");

    let input = samples::TINY;
    group.throughput(Throughput::Bytes(input.len() as u64));

    group.bench_function("output_buffer_only", |b| {
        let mut buffer = Vec::with_capacity(input.len() * 2);
        b.iter(|| {
            ferromark::to_html_into(black_box(input), &mut buffer);
            black_box(&buffer);
        })
    });

    group.bench_function("parser_and_output_buffers", |b| {
        let mut renderer = ferromark::Renderer::new();
        let mut buffer = Vec::with_capacity(input.len() * 2);
        b.iter(|| {
            renderer.render_into(black_box(input), &mut buffer);
            black_box(&buffer);
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_parsing,
    bench_extensions,
    bench_escaping,
    bench_pathological,
    bench_buffer_reuse,
    bench_renderer_session
);
criterion_main!(benches);
