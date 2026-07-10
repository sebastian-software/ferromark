use criterion::{Criterion, black_box, criterion_group, criterion_main};
use ferromark::{
    FencedCodeBlock, FencedCodeRenderer, Options, TrustedHtml, to_html, to_html_with_renderer,
};

struct FallbackRenderer;

impl FencedCodeRenderer for FallbackRenderer {
    fn render(&mut self, block: FencedCodeBlock<'_>) -> Option<TrustedHtml> {
        black_box(block);
        None
    }
}

struct EscapingRenderer;

impl FencedCodeRenderer for EscapingRenderer {
    fn render(&mut self, block: FencedCodeBlock<'_>) -> Option<TrustedHtml> {
        let mut escaped = Vec::with_capacity(block.code.len());
        ferromark::escape::escape_text_into(&mut escaped, block.code.as_bytes());
        let escaped = String::from_utf8(escaped).expect("HTML escaping preserves UTF-8");
        Some(TrustedHtml::from_trusted(format!(
            "<pre class=\"highlighted\"><code>{escaped}</code></pre>\n"
        )))
    }
}

fn fenced_code_renderer_benches(c: &mut Criterion) {
    let prose = include_str!("fixtures/commonmark-50k.md");
    let code_heavy = (0..256)
        .map(|index| format!("```rust\nfn item_{index}() -> usize {{ {index} }}\n```\n\n"))
        .collect::<String>();
    let options = Options::default();

    c.bench_function("renderer/default_path_commonmark_50k", |b| {
        b.iter(|| to_html(black_box(prose)));
    });
    c.bench_function("renderer/code_heavy_default", |b| {
        b.iter(|| to_html(black_box(&code_heavy)));
    });
    c.bench_function("renderer/code_heavy_fallback", |b| {
        let mut renderer = FallbackRenderer;
        b.iter(|| {
            to_html_with_renderer(black_box(&code_heavy), &options, black_box(&mut renderer))
        });
    });
    c.bench_function("renderer/code_heavy_custom", |b| {
        let mut renderer = EscapingRenderer;
        b.iter(|| {
            to_html_with_renderer(black_box(&code_heavy), &options, black_box(&mut renderer))
        });
    });
}

criterion_group!(benches, fenced_code_renderer_benches);
criterion_main!(benches);
