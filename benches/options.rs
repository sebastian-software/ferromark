use criterion::{BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main};
use ferromark::{Options, RenderPolicy};

const SHARED_SECTION: &str = r#"
## Release notes

The parser keeps **important text**, *supporting detail*, and `inline code`
readable. Visit [the guide](https://example.com/guide) for more information.

- [x] Parse the document
- [ ] Publish the result

| Parser | Status |
| --- | --- |
| ferromark | ~~planned~~ shipped |

> Simple blockquotes, lists, links, and code cover most documentation.

```rust
fn throughput(bytes: usize, seconds: f64) -> f64 { bytes as f64 / seconds }
```

"#;

fn all_extensions() -> Options {
    Options {
        render_policy: RenderPolicy::Untrusted,
        allow_html: true,
        allow_link_refs: true,
        tables: true,
        strikethrough: true,
        highlight: true,
        superscript: true,
        subscript: true,
        task_lists: true,
        autolink_literals: true,
        disallowed_raw_html: true,
        footnotes: true,
        front_matter: true,
        heading_ids: true,
        math: true,
        callouts: true,
        line_comments: true,
    }
}

fn options_cost_benches(c: &mut Criterion) {
    let input = SHARED_SECTION.repeat(96);
    let mut group = c.benchmark_group("options/shared_corpus");
    group.throughput(Throughput::Bytes(input.len() as u64));

    for (name, options) in [
        ("minimal", Options::minimal()),
        ("commonmark", Options::commonmark()),
        ("gfm", Options::gfm()),
        ("default", Options::default()),
        (
            "default_line_comments",
            Options {
                line_comments: true,
                ..Options::default()
            },
        ),
        ("all_extensions", all_extensions()),
    ] {
        let mut output = Vec::with_capacity(input.len() + input.len() / 4);
        group.bench_with_input(BenchmarkId::from_parameter(name), &options, |b, options| {
            b.iter(|| {
                output.clear();
                ferromark::to_html_into_with_options(
                    black_box(&input),
                    &mut output,
                    black_box(options),
                );
                black_box(&output);
            });
        });
    }

    group.finish();
}

criterion_group!(benches, options_cost_benches);
criterion_main!(benches);
