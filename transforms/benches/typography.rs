use criterion::{Criterion, criterion_group, criterion_main};
use ferromark::{Allocator, HtmlRenderer, HtmlRendererOptions, Parser, ParserOptions};
use ferromark_transforms::{
    TransformContext, TransformPass, TypographyLanguage, TypographyOptions, TypographyPass,
};
use std::hint::black_box;

fn render_with_typography(source: &str) -> String {
    let allocator = Allocator::new();
    let renderer_options = HtmlRendererOptions::default();
    let mut document = Parser::with_options(&allocator, source, ParserOptions::default())
        .parse()
        .expect("benchmark input should parse");
    let context = TransformContext::new(&allocator, source, &renderer_options);
    let mut typography = TypographyPass::new(TypographyOptions::new(TypographyLanguage::English));
    typography
        .apply(&mut document, &context)
        .expect("typography should complete");

    HtmlRenderer::with_options(renderer_options).render(&document)
}

fn benchmark_typography(c: &mut Criterion) {
    let prose = concat!(
        "She said \"Hello, world\" -- it's a 12 km walk...\n",
        "The next sentence keeps ordinary typography work active. "
    )
    .repeat(48);
    let angle_text = "x < y and y > z; use a < symbol and a > symbol in prose. ".repeat(48);

    for (name, source) in [
        ("typography/default/prose-with-quotes", prose.as_str()),
        (
            "typography/default/prose-with-angle-text",
            angle_text.as_str(),
        ),
    ] {
        c.bench_function(name, |b| {
            b.iter(|| black_box(render_with_typography(black_box(source))));
        });
    }
}

criterion_group!(benches, benchmark_typography);
criterion_main!(benches);
