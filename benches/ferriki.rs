//! Small self-contained lifecycle benchmark for the optional Ferriki adapter.
//! It uses custom in-memory assets, so it does not measure catalog or CDN I/O.

use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use ferromark::ferriki::Highlighter;
use ferromark::{Allocator, FerrikiHighlightHooks, HtmlRenderer, Parser};

const GRAMMAR: &str = r#"{
  "name": "rust", "scopeName": "source.rust",
  "patterns": [{"match": "\\b(let|fn)\\b", "name": "keyword.control.rust"}]
}"#;
const THEME: &str = r##"{
  "name": "test-dark", "type": "dark",
  "colors": {"editor.foreground": "#eaf0ff", "editor.background": "#101820"},
  "tokenColors": [{"scope": "keyword.control.rust", "settings": {"foreground": "#ff0000"}}]
}"##;
const SOURCE: &str =
    "```rust\nfn main() {\n    let value = \"<hello>\";\n    println!(\"{value}\");\n}\n```";

fn new_highlighter() -> Highlighter {
    let mut highlighter = Highlighter::builder().build().unwrap();
    highlighter.register_language_json(GRAMMAR).unwrap();
    highlighter.register_theme_json(THEME).unwrap();
    highlighter
}

fn bench_ferriki_lifecycle(c: &mut Criterion) {
    let allocator = Allocator::new();
    let document = Parser::new(&allocator, SOURCE).parse().unwrap();
    let mut group = c.benchmark_group("ferriki_lifecycle_custom_assets");

    group.bench_function("highlighter_build", |b| {
        b.iter(|| black_box(new_highlighter()));
    });
    group.bench_function("build_and_first_render", |b| {
        b.iter(|| {
            let mut highlighter = new_highlighter();
            let mut renderer = HtmlRenderer::new();
            let mut hooks = FerrikiHighlightHooks::new(&mut highlighter, "test-dark");
            black_box(renderer.render_with_hooks(black_box(&document), &mut hooks));
        });
    });

    let mut highlighter = new_highlighter();
    let mut renderer = HtmlRenderer::new();
    group.bench_function("repeated_render", |b| {
        b.iter(|| {
            let mut hooks = FerrikiHighlightHooks::new(&mut highlighter, "test-dark");
            black_box(renderer.render_with_hooks(black_box(&document), &mut hooks));
        });
    });
    group.finish();
}

criterion_group!(benches, bench_ferriki_lifecycle);
criterion_main!(benches);
