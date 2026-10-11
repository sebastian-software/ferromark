use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use std::hint::black_box;

macro_rules! measure_core {
    ($group:expr, $version:ident, $source:expr, $gfm:expr) => {{
        let options = if $gfm {
            $version::ParserOptions::gfm()
        } else {
            $version::ParserOptions::default()
        };
        $group.bench_function(concat!(stringify!($version), "/parse"), |b| {
            b.iter(|| {
                let arena = $version::Allocator::for_source_len($source.len());
                black_box(
                    $version::Parser::with_options(&arena, black_box($source), options.clone())
                        .parse()
                        .unwrap(),
                );
            })
        });
        $group.bench_function(concat!(stringify!($version), "/compile_html"), |b| {
            b.iter(|| {
                let arena = $version::Allocator::for_source_len($source.len());
                let document =
                    $version::Parser::with_options(&arena, black_box($source), options.clone())
                        .parse()
                        .unwrap();
                black_box($version::HtmlRenderer::new().render(&document));
            })
        });
        let arena = $version::Allocator::for_source_len($source.len());
        let document = $version::Parser::with_options(&arena, $source, options)
            .parse()
            .unwrap();
        $group.bench_function(concat!(stringify!($version), "/render_html"), |b| {
            b.iter(|| {
                black_box($version::HtmlRenderer::new().render(black_box(&document)));
            })
        });
    }};
}

fn html_before(source: &str, gfm: bool) -> String {
    let arena = before::Allocator::for_source_len(source.len());
    let options = if gfm {
        before::ParserOptions::gfm()
    } else {
        before::ParserOptions::default()
    };
    before::HtmlRenderer::new().render(
        &before::Parser::with_options(&arena, source, options)
            .parse()
            .unwrap(),
    )
}
fn html_after(source: &str, gfm: bool) -> String {
    let arena = after::Allocator::for_source_len(source.len());
    let options = if gfm {
        after::ParserOptions::gfm()
    } else {
        after::ParserOptions::default()
    };
    after::HtmlRenderer::new().render(
        &after::Parser::with_options(&arena, source, options)
            .parse()
            .unwrap(),
    )
}

fn ordinary_paths(c: &mut Criterion) {
    let prose = "# Heading\n\nOrdinary prose with **strong** and *emphasis*, `inline code`, [link](/guide), and Unicode café 😄.\n\n".repeat(256);
    let bare = format!(
        "{} https://example.com/guide?q=one then end.\n\n",
        "plain prose ".repeat(32)
    )
    .repeat(64);
    let dense =
        "https://example.com/a www.example.org/x person@example.com then text.\n\n".repeat(256);
    let decoded =
        "before &amp; \\* café https://example.com/a?q=1&amp;x=2 then www.example.org/é.\n\n"
            .repeat(256);
    for (name, source, gfm) in [
        ("prose", prose, false),
        ("gfm_bare", bare, true),
        ("gfm_dense", dense, true),
        ("gfm_decoded", decoded, true),
    ] {
        assert_eq!(
            html_before(&source, gfm),
            html_after(&source, gfm),
            "render equality: {name}"
        );
        let mut group = c.benchmark_group(name);
        group.throughput(Throughput::Bytes(source.len() as u64));
        measure_core!(group, before, source.as_str(), gfm);
        measure_core!(group, after, source.as_str(), gfm);
        group.finish();
    }
    let source = include_str!("../fixtures/text-extraction.mdx").repeat(64);
    let before_arena = before::Allocator::for_source_len(source.len());
    let before_options = before::ParserOptions {
        mdx: true,
        mdx_compatible: true,
        ..before::ParserOptions::gfm()
    };
    let before_document =
        before::Parser::with_options(&before_arena, &source, before_options.clone())
            .parse()
            .unwrap();
    let after_arena = after::Allocator::for_source_len(source.len());
    let after_options = after::ParserOptions {
        mdx: true,
        mdx_compatible: true,
        ..after::ParserOptions::gfm()
    };
    let after_document = after::Parser::with_options(&after_arena, &source, after_options.clone())
        .parse()
        .unwrap();
    assert_eq!(
        before::JsxRenderer::new()
            .render(&before_document, &source)
            .body,
        after::JsxRenderer::new()
            .render(&after_document, &source)
            .body
    );
    let mut group = c.benchmark_group("mdx");
    group.throughput(Throughput::Bytes(source.len() as u64));
    group.bench_function("before/compile_jsx", |b| {
        b.iter(|| {
            let arena = before::Allocator::for_source_len(source.len());
            let document =
                before::Parser::with_options(&arena, black_box(&source), before_options.clone())
                    .parse()
                    .unwrap();
            black_box(before::JsxRenderer::new().render(&document, &source));
        })
    });
    group.bench_function("after/compile_jsx", |b| {
        b.iter(|| {
            let arena = after::Allocator::for_source_len(source.len());
            let document =
                after::Parser::with_options(&arena, black_box(&source), after_options.clone())
                    .parse()
                    .unwrap();
            black_box(after::JsxRenderer::new().render(&document, &source));
        })
    });
    group.bench_function("before/render_jsx", |b| {
        b.iter(|| {
            black_box(before::JsxRenderer::new().render(black_box(&before_document), &source))
        })
    });
    group.bench_function("after/render_jsx", |b| {
        b.iter(|| black_box(after::JsxRenderer::new().render(black_box(&after_document), &source)))
    });
    group.finish();
}
criterion_group!(benches, ordinary_paths);
criterion_main!(benches);
