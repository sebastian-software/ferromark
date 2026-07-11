use criterion::{BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main};
use ferromark_pulldown_comparison::{
    Corpus, ParserKind, RunConfig, render_ferromark_config_into, render_pulldown_config_into,
};

fn parity_parsers() -> [ParserKind; 2] {
    match std::env::var("FERROMARK_PARITY_ORDER").as_deref() {
        Ok("pulldown-first") => [ParserKind::PulldownCmark, ParserKind::Ferromark],
        _ => [ParserKind::Ferromark, ParserKind::PulldownCmark],
    }
}

fn bench_lane(c: &mut Criterion, corpus: Corpus, configuration: RunConfig, parsers: &[ParserKind]) {
    let data = corpus.materialize();
    let input = data.input();
    let mut group = c.benchmark_group(format!(
        "profiling/{}/{}",
        corpus.as_str(),
        configuration.as_str()
    ));
    group.throughput(Throughput::Bytes(input.len() as u64));

    for parser in parsers {
        match parser {
            ParserKind::Ferromark => {
                let mut output = Vec::with_capacity(input.len() + input.len() / 4);
                group.bench_function(BenchmarkId::from_parameter(parser.as_str()), |b| {
                    b.iter(|| {
                        render_ferromark_config_into(
                            black_box(input),
                            configuration,
                            black_box(&mut output),
                        );
                        black_box(output.len());
                    });
                });
            }
            ParserKind::PulldownCmark => {
                let mut output = String::with_capacity(input.len() + input.len() / 4);
                group.bench_function(BenchmarkId::from_parameter(parser.as_str()), |b| {
                    b.iter(|| {
                        render_pulldown_config_into(
                            black_box(input),
                            configuration,
                            black_box(&mut output),
                        )
                        .expect("pulldown benchmark uses only parity configurations");
                        black_box(output.len());
                    });
                });
            }
        }
    }
    group.finish();
}

fn profiling_benches(c: &mut Criterion) {
    let both = parity_parsers();
    for corpus in [
        Corpus::CommonMark5K,
        Corpus::CommonMark20K,
        Corpus::CommonMark50K,
        Corpus::Mixed250K,
    ] {
        bench_lane(c, corpus, RunConfig::CommonMark, &both);
    }

    // Extended is covered by the secure-default size matrix below. Keeping it
    // out of this loop ensures one stable Criterion identifier per lane.
    for configuration in [RunConfig::EssentialsSecure, RunConfig::FullSecure] {
        bench_lane(
            c,
            Corpus::CommonMark50K,
            configuration,
            &[ParserKind::Ferromark],
        );
    }

    for corpus in [
        Corpus::CommonMark5K,
        Corpus::CommonMark20K,
        Corpus::CommonMark50K,
    ] {
        bench_lane(
            c,
            corpus,
            RunConfig::ExtendedSecure,
            &[ParserKind::Ferromark],
        );
    }

    for corpus in [
        Corpus::Simple,
        Corpus::Code,
        Corpus::SafeUrls,
        Corpus::UnsafeUrls,
        Corpus::References,
        Corpus::Tables,
        Corpus::Containers,
        Corpus::Delimiters,
        Corpus::Html,
        Corpus::UnicodeEntities,
    ] {
        bench_lane(
            c,
            corpus,
            RunConfig::ExtendedSecure,
            &[ParserKind::Ferromark],
        );
    }
}

criterion_group!(benches, profiling_benches);
criterion_main!(benches);
