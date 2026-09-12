use serde::Deserialize;
use serde_json::{Value, json};
use std::{
    hint::black_box,
    io::{self, BufRead, Write},
    time::{Duration, Instant},
};
mod normalize;
#[derive(Deserialize)]
struct Case {
    case: String,
    input: String,
    profile: String,
}

#[cfg(not(feature = "oxide"))]
mod engine {
    use super::*;
    pub type Options = ferro::Options;
    pub fn options(profile: &str) -> Options {
        let mut o = ferro::Options::commonmark();
        o.tables = true;
        o.strikethrough = true;
        o.task_lists = true;
        o.heading_ids = true;
        o.render_policy = ferro::RenderPolicy::Trusted;
        if profile == "upstream" {
            o.autolink_literals = true;
            o.footnotes = true;
        }
        o
    }
    #[inline(never)]
    pub fn render(input: &str, o: &Options, _presize: bool, _upstream: bool) -> String {
        ferro::to_html_with_options(black_box(input), o)
    }
}
#[cfg(feature = "oxide")]
mod engine {
    use super::*;
    use ox_content_allocator::Allocator;
    use ox_content_parser::{Parser, ParserOptions};
    use ox_content_renderer::{HtmlRenderer, HtmlRendererOptions};
    pub type Options = (ParserOptions, HtmlRendererOptions);
    pub fn options(profile: &str) -> Options {
        let p = if profile == "upstream" {
            ParserOptions::gfm()
        } else {
            ParserOptions {
                tables: true,
                strikethrough: true,
                task_lists: true,
                ..Default::default()
            }
        };
        (
            p,
            HtmlRendererOptions {
                autolink_urls: false,
                autolink_target_blank: false,
                link_target_blank: false,
                ..Default::default()
            },
        )
    }
    #[inline(never)]
    pub fn render(input: &str, o: &Options, presize: bool, upstream: bool) -> String {
        let arena = if presize {
            Allocator::for_source_len(input.len())
        } else {
            Allocator::new()
        };
        let doc = Parser::with_options(&arena, black_box(input), o.0.clone())
            .parse()
            .unwrap();
        if upstream {
            HtmlRenderer::new().render(&doc)
        } else {
            HtmlRenderer::with_options(o.1.clone()).render(&doc)
        }
    }
}
fn main() {
    let args: Vec<_> = std::env::args().collect();
    let cases: Vec<Case> = serde_json::from_slice(&std::fs::read(&args[1]).unwrap()).unwrap();
    let opts: Vec<_> = cases.iter().map(|c| engine::options(&c.profile)).collect();
    let presize = args.get(2).is_some_and(|s| s == "presize");
    let mut out = io::BufWriter::new(io::stdout().lock());
    for line in io::stdin().lock().lines() {
        let q: Value = serde_json::from_str(&line.unwrap()).unwrap();
        let i = q["index"].as_u64().unwrap() as usize;
        let c = &cases[i];
        let o = &opts[i];
        let render = || engine::render(&c.input, o, presize, c.profile == "upstream");
        let row = match q["op"].as_str().unwrap() {
            "verify" => {
                let html = render();
                json!({"html": html, "normalized": normalize::normalize_html(&html)})
            }
            #[cfg(feature = "counters")]
            "counters" => {
                ferro::profiling::reset();
                drop(black_box(render()));
                json!({"snapshot": format!("{:?}", ferro::profiling::snapshot())})
            }
            "window" => {
                let duration = Duration::from_millis(q["ms"].as_u64().unwrap());
                let start = Instant::now();
                let mut count = 0;
                loop {
                    for _ in 0..8 {
                        drop(black_box(render()));
                    }
                    count += 8;
                    if start.elapsed() >= duration {
                        break;
                    }
                }
                json!({"case": c.case, "count": count, "elapsed_ns": start.elapsed().as_nanos() as u64})
            }
            _ => panic!("unknown operation"),
        };
        writeln!(out, "{row}").unwrap();
        out.flush().unwrap();
    }
}
