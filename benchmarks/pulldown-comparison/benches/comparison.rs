use criterion::{BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main};
use ferromark_pulldown_comparison::{ParityConfig, render_ferromark_into, render_pulldown_into};

const COMMONMARK_5K: &str = include_str!("../../../benches/fixtures/commonmark-5k.md");
const COMMONMARK_50K: &str = include_str!("../../../benches/fixtures/commonmark-50k.md");

const EXTENDED_SECTION: &str = r#"
## Scientific note

The result is ^squared^ in this synthetic example, with inline math $a+b=c$
and a reference to the [method][method].

> [!NOTE]
> The shared extended lane enables callouts in both parsers.

The result has a supporting footnote.[^result]

[^result]: Repeated measurements were stable.
[method]: https://example.com/method

| Feature | Enabled |
| --- | --- |
| Math | **yes** |
| Footnotes | ~~pending~~ yes |

"#;

fn bench_pair(c: &mut Criterion, group_name: &str, input: &str, config: ParityConfig) {
    let mut group = c.benchmark_group(group_name);
    group.throughput(Throughput::Bytes(input.len() as u64));

    let mut ferromark_output = Vec::with_capacity(input.len() + input.len() / 4);
    group.bench_function(BenchmarkId::from_parameter("ferromark"), |b| {
        b.iter(|| {
            render_ferromark_into(black_box(input), config, black_box(&mut ferromark_output));
            black_box(&ferromark_output);
        });
    });

    let mut pulldown_output = String::with_capacity(input.len() + input.len() / 4);
    group.bench_function(BenchmarkId::from_parameter("pulldown-cmark"), |b| {
        b.iter(|| {
            render_pulldown_into(black_box(input), config, black_box(&mut pulldown_output));
            black_box(&pulldown_output);
        });
    });

    group.finish();
}

fn parity_benches(c: &mut Criterion) {
    bench_pair(
        c,
        "parity/commonmark/5k",
        COMMONMARK_5K,
        ParityConfig::CommonMark,
    );
    bench_pair(
        c,
        "parity/commonmark/50k",
        COMMONMARK_50K,
        ParityConfig::CommonMark,
    );
    bench_pair(
        c,
        "parity/gfm_overlap/5k",
        COMMONMARK_5K,
        ParityConfig::GfmOverlap,
    );
    bench_pair(
        c,
        "parity/gfm_overlap/50k",
        COMMONMARK_50K,
        ParityConfig::GfmOverlap,
    );

    let extended = EXTENDED_SECTION.repeat(128);
    bench_pair(
        c,
        "parity/extended_overlap",
        &extended,
        ParityConfig::ExtendedOverlap,
    );
}

criterion_group!(benches, parity_benches);
criterion_main!(benches);
