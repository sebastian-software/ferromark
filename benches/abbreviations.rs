//! Cost of optional technical abbreviation processing.
//!
//! Parsing is outside the timed region. The reused-renderer cases build the
//! dictionary once, then measure repeated renders with the same configuration.

use std::collections::BTreeMap;
use std::hint::black_box;

use criterion::{BatchSize, Criterion, Throughput, criterion_group, criterion_main};
use ferromark::allocator::Allocator;
use ferromark::parser::Parser;
use ferromark::renderer::{AbbreviationOptions, HtmlRenderer, HtmlRendererOptions};

const MARKDOWN: &str = include_str!("fixtures/abbreviations.md");
const RAW_HTML_FRAGMENT: &str = include_str!("fixtures/abbreviations-raw-html.md");
const NO_CANDIDATES: &str = "This guide describes a renderer, a parser, and a reusable cache.\n";

fn bench_abbreviations(c: &mut Criterion) {
    let allocator = Allocator::for_source_len(MARKDOWN.len());
    let document = Parser::new(&allocator, MARKDOWN).parse().unwrap();
    {
        let mut group = c.benchmark_group("abbreviations");
        group.throughput(Throughput::Bytes(MARKDOWN.len() as u64));

        group.bench_function("disabled/fresh", |b| {
            b.iter(|| {
                let mut renderer = HtmlRenderer::new();
                black_box(renderer.render_borrowed(black_box(&document)));
            });
        });

        let abbreviations = BTreeMap::from([
            ("README".to_string(), None),
            ("ID".to_string(), Some("Identifier".to_string())),
            ("GraphQL".to_string(), Some(String::new())),
        ]);
        let mut enabled_abbreviations = AbbreviationOptions::default();
        enabled_abbreviations.overrides = abbreviations;
        let enabled_options = HtmlRendererOptions::default();
        group.bench_function("enabled/hits/fresh", |b| {
            b.iter_batched(
                || enabled_options.clone(),
                |options| {
                    let mut renderer = HtmlRenderer::with_options_and_abbreviations(
                        options,
                        enabled_abbreviations.clone(),
                    );
                    black_box(renderer.render_borrowed(black_box(&document)));
                },
                BatchSize::SmallInput,
            );
        });
        let mut enabled_renderer = HtmlRenderer::with_options_and_abbreviations(
            enabled_options.clone(),
            enabled_abbreviations,
        );
        group.bench_function("enabled/hits/reused", |b| {
            b.iter(|| {
                let _ = black_box(enabled_renderer.render_borrowed(black_box(&document)));
            });
        });
        group.finish();
    }

    let raw_html_heavy = RAW_HTML_FRAGMENT.repeat(64);
    let allocator = Allocator::for_source_len(raw_html_heavy.len());
    let document = Parser::new(&allocator, &raw_html_heavy).parse().unwrap();
    {
        let mut group = c.benchmark_group("abbreviations/disabled_raw_html");
        group.throughput(Throughput::Bytes(raw_html_heavy.len() as u64));
        group.bench_function("fresh", |b| {
            b.iter(|| {
                let mut renderer = HtmlRenderer::new();
                black_box(renderer.render_borrowed(black_box(&document)));
            });
        });
        let mut raw_html_renderer = HtmlRenderer::new();
        group.bench_function("reused", |b| {
            b.iter(|| {
                black_box(raw_html_renderer.render_borrowed(black_box(&document)));
            });
        });
        group.finish();
    }

    let allocator = Allocator::for_source_len(NO_CANDIDATES.len());
    let document = Parser::new(&allocator, NO_CANDIDATES).parse().unwrap();
    {
        let mut group = c.benchmark_group("abbreviations/no_candidates");
        group.throughput(Throughput::Bytes(NO_CANDIDATES.len() as u64));
        let mut no_candidate_renderer = HtmlRenderer::with_options_and_abbreviations(
            HtmlRendererOptions::default(),
            AbbreviationOptions::default(),
        );
        group.bench_function("enabled/reused", |b| {
            b.iter(|| {
                let _ = black_box(no_candidate_renderer.render_borrowed(black_box(&document)));
            });
        });
        group.finish();
    }
}

criterion_group!(benches, bench_abbreviations);
criterion_main!(benches);
