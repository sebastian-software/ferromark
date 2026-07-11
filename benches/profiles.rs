use criterion::{BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main};
use ferromark::{Options, Profile};

const ESSENTIALS_SECTION: &str = r#"
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

fn profile_cost_benches(c: &mut Criterion) {
    let input = ESSENTIALS_SECTION.repeat(96);
    let mut group = c.benchmark_group("profiles/shared_essentials_corpus");
    group.throughput(Throughput::Bytes(input.len() as u64));

    for (name, profile) in [
        ("essentials", Profile::Essentials),
        ("extended", Profile::Extended),
        ("full", Profile::Full),
    ] {
        let options = Options::from(profile);
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

criterion_group!(benches, profile_cost_benches);
criterion_main!(benches);
