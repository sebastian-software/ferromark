use serde::Deserialize;
use serde_json::{Value, json};
use std::hint::black_box;
use std::io::{self, BufRead, Write};

mod heap;
mod normalize;

#[global_allocator]
static GLOBAL: heap::Counting = heap::Counting;

#[derive(Deserialize)]
struct Case {
    case: String,
    input: String,
    flags: u32,
}

#[cfg(not(feature = "oxide"))]
fn options(flags: u32) -> ferro::Options {
    let mut o = ferro::Options::commonmark();
    o.tables = flags & 1 != 0;
    o.strikethrough = flags & 2 != 0;
    o.task_lists = flags & 4 != 0;
    o.heading_ids = true;
    o.render_policy = ferro::RenderPolicy::Trusted;
    o
}

fn main() {
    let args: Vec<_> = std::env::args().collect();
    heap::self_test();
    if args.get(1).map(String::as_str) == Some("--self-test") {
        println!("allocator accounting self-test passed");
        return;
    }
    let cases: Vec<Case> = serde_json::from_slice(&std::fs::read(&args[1]).unwrap()).unwrap();
    let mut out = io::BufWriter::new(io::stdout().lock());
    for line in io::stdin().lock().lines() {
        let q: Value = serde_json::from_str(&line.unwrap()).unwrap();
        let case = &cases[q["index"].as_u64().unwrap() as usize];
        assert!(case.flags < 16 && case.flags & 8 != 0);

        #[cfg(not(feature = "oxide"))]
        let opts = options(case.flags);
        #[cfg(not(feature = "oxide"))]
        let render = || ferro::to_html_with_options(black_box(&case.input), &opts);

        #[cfg(feature = "oxide")]
        let parser_options = ox_content_parser::ParserOptions {
            tables: case.flags & 1 != 0,
            strikethrough: case.flags & 2 != 0,
            task_lists: case.flags & 4 != 0,
            ..Default::default()
        };
        #[cfg(feature = "oxide")]
        let html_options = ox_content_renderer::HtmlRendererOptions {
            autolink_urls: false,
            autolink_target_blank: false,
            link_target_blank: false,
            ..Default::default()
        };
        #[cfg(feature = "oxide")]
        let presize = q["presize"].as_bool().unwrap_or(false);
        #[cfg(feature = "oxide")]
        let render = || {
            let source = black_box(&case.input);
            let arena = if presize {
                ox_content_allocator::Allocator::for_source_len(source.len())
            } else {
                ox_content_allocator::Allocator::new()
            };
            let doc = ox_content_parser::Parser::with_options(
                &arena, source, parser_options.clone(),
            ).parse().unwrap();
            ox_content_renderer::HtmlRenderer::with_options(html_options.clone()).render(&doc)
        };

        // Complete first-use initialization before measuring a fresh operation.
        // No renderer, parser scratch or arena is retained between operations.
        drop(black_box(render()));
        let mut samples = Vec::with_capacity(10);
        for _ in 0..10 {
            let base = heap::begin();
            let html = black_box(render());
            let stats = heap::end(base);
            assert_eq!(stats.live_after_render, html.capacity() as u64);
            drop(html);
            assert_eq!(heap::remaining(base), 0);
            samples.push(stats);
        }
        assert!(samples.windows(2).all(|pair| pair[0] == pair[1]));
        let html = render();
        let row = json!({
            "case": case.case,
            "input_bytes": case.input.len(),
            "html_bytes": html.len(),
            "html": html,
            "normalized": normalize::normalize_html(&html),
            "samples": samples,
        });
        writeln!(out, "{row}").unwrap();
        out.flush().unwrap();
    }
}
