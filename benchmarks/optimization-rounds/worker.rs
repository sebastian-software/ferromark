use std::hint::black_box;
use std::io::{self, BufRead, Write};
use std::time::{Duration, Instant};

use ferromark::ast::Document;
use ferromark::{Allocator, HtmlRenderer, HtmlRendererOptions, Parser, ParserOptions};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Stage {
    Fresh,
    Reuse,
    Parse,
    Render,
}

impl Stage {
    fn parse(value: &str) -> Self {
        match value {
            "fresh" => Self::Fresh,
            "reuse" => Self::Reuse,
            "parse" => Self::Parse,
            "render" => Self::Render,
            _ => panic!("unknown stage: {value}"),
        }
    }
}

struct Verified {
    html: String,
    ast: String,
    arena_capacity_bytes: usize,
    children: usize,
}

struct Engine {
    options: ParserOptions,
    html_options: HtmlRendererOptions,
    stage: Stage,
    sources: Vec<&'static str>,
    retained_allocator: Option<Allocator>,
    renderer: HtmlRenderer,
    render_documents: Vec<Document<'static>>,
    render_arena_capacity_bytes: usize,
}

impl Engine {
    fn options(profile: &str) -> ParserOptions {
        match profile {
            "commonmark" | "autolink" => ParserOptions::default(),
            "gfm" => ParserOptions::gfm(),
            "extensions" => ParserOptions {
                footnotes: true,
                superscript: true,
                subscript: true,
                math: true,
                definition_lists: true,
                heading_attributes: true,
                wiki_links: true,
                cjk_emphasis: true,
                ..ParserOptions::gfm()
            },
            "mdx" => ParserOptions::mdx(),
            profile if profile.starts_with("opt-") => {
                let bits: u8 = profile[4..].parse().expect("option bits");
                assert!(bits < 8);
                ParserOptions {
                    mdx: bits & 1 != 0,
                    math: bits & 2 != 0,
                    superscript: bits & 4 != 0,
                    ..ParserOptions::default()
                }
            }
            _ => panic!("unknown profile: {profile}"),
        }
    }

    fn html_options(profile: &str) -> HtmlRendererOptions {
        HtmlRendererOptions {
            xhtml: true,
            hard_break: "<br />\n".into(),
            autolink_urls: profile == "autolink",
            autolink_target_blank: false,
            link_target_blank: false,
            disallow_raw_html: profile == "gfm",
            ..HtmlRendererOptions::new()
        }
    }

    fn new(profile: &str, stage: &str, sources: Vec<&'static str>) -> Self {
        let stage = Stage::parse(stage);
        let mut options = Self::options(profile);
        if profile == "gfm" {
            // Match the current-comparison contract: GFM syntax is enabled,
            // while the benchmark's HTML output does not include footnotes.
            options.footnotes = false;
        }
        let html_options = Self::html_options(profile);
        let max_source = sources.iter().map(|source| source.len()).max().unwrap_or(0);
        let mut render_documents = Vec::new();
        let mut render_arena_capacity_bytes = 0;
        if stage == Stage::Render {
            // The render-only lane deliberately keeps its parsed documents alive
            // for the worker lifetime. Leaking this process-local arena avoids a
            // self-referential Engine while preserving the parse-once contract.
            let allocator = Box::leak(Box::new(Allocator::for_source_len(max_source)));
            for source in &sources {
                render_documents.push(
                    Parser::with_options(allocator, source, options.clone())
                        .parse()
                        .expect("parse input"),
                );
            }
            render_arena_capacity_bytes = allocator.allocated_bytes();
        }
        let retained_allocator = match stage {
            Stage::Reuse | Stage::Parse => Some(Allocator::for_source_len(max_source)),
            _ => None,
        };
        Self {
            options,
            html_options: html_options.clone(),
            stage,
            sources,
            retained_allocator,
            renderer: HtmlRenderer::with_options(html_options),
            render_documents,
            render_arena_capacity_bytes,
        }
    }

    fn render_fresh(&self, source: &str) -> Verified {
        let allocator = Allocator::for_source_len(source.len());
        let document = Parser::with_options(&allocator, source, self.options.clone())
            .parse()
            .expect("parse input");
        let ast = format!("{document:?}");
        let children = document.children.len();
        let arena_capacity_bytes = allocator.allocated_bytes();
        let mut renderer = HtmlRenderer::with_options(self.html_options.clone());
        let html = renderer.render(&document);
        Verified { html, ast, arena_capacity_bytes, children }
    }

    fn render_retained(&mut self, source: &str, output: bool) -> Verified {
        let allocator = self.retained_allocator.as_mut().expect("retained allocator");
        let (html, ast, arena_capacity_bytes, children) = {
            let document = Parser::with_options(allocator, source, self.options.clone())
                .parse()
                .expect("parse input");
            let ast = output.then(|| format!("{document:?}"));
            let children = document.children.len();
            let arena_capacity_bytes = allocator.allocated_bytes();
            let html = if output {
                self.renderer.render(&document)
            } else {
                self.renderer.render_borrowed(&document).to_owned()
            };
            (html, ast, arena_capacity_bytes, children)
        };
        allocator.reset();
        Verified { html, ast: ast.unwrap_or_default(), arena_capacity_bytes, children }
    }

    fn verify_one(&mut self, index: usize) -> Verified {
        match self.stage {
            Stage::Fresh => self.render_fresh(self.sources[index]),
            Stage::Reuse => self.render_retained(self.sources[index], true),
            Stage::Parse => self.render_retained(self.sources[index], true),
            Stage::Render => {
                let document = &self.render_documents[index];
                let ast = format!("{document:?}");
                let children = document.children.len();
                let html = self.renderer.render_borrowed(document).to_owned();
                Verified {
                    html,
                    ast,
                    arena_capacity_bytes: self.render_arena_capacity_bytes,
                    children,
                }
            }
        }
    }

    fn bench_fresh(&self, source: &str) -> usize {
        let allocator = Allocator::for_source_len(source.len());
        let document = Parser::with_options(&allocator, black_box(source), self.options.clone())
            .parse()
            .expect("parse input");
        let mut renderer = HtmlRenderer::with_options(self.html_options.clone());
        let html = renderer.render(&document);
        black_box(html.as_bytes()).len()
    }

    fn bench_reuse(&mut self, source: &str) -> usize {
        let allocator = self.retained_allocator.as_mut().expect("retained allocator");
        let result = {
            let document = Parser::with_options(allocator, black_box(source), self.options.clone())
                .parse()
                .expect("parse input");
            let html = self.renderer.render_borrowed(&document);
            black_box(html.as_bytes()).len()
        };
        allocator.reset();
        result
    }

    fn bench_parse(&mut self, source: &str) -> usize {
        let allocator = self.retained_allocator.as_mut().expect("retained allocator");
        let children = {
            let document = Parser::with_options(allocator, black_box(source), self.options.clone())
                .parse()
                .expect("parse input");
            black_box(&document);
            black_box(document.children.len())
        };
        allocator.reset();
        children
    }

    fn bench_render(&mut self, index: usize) -> usize {
        let html = self.renderer.render_borrowed(&self.render_documents[index]);
        black_box(html.as_bytes()).len()
    }

    fn bench_one(&mut self, index: usize) -> usize {
        match self.stage {
            Stage::Fresh => self.bench_fresh(self.sources[index]),
            Stage::Reuse => self.bench_reuse(self.sources[index]),
            Stage::Parse => self.bench_parse(self.sources[index]),
            Stage::Render => self.bench_render(index),
        }
    }

    fn bench(&mut self, budget_ns: u64) -> (u64, u128, usize) {
        let budget = Duration::from_nanos(budget_ns);
        let start = Instant::now();
        let mut iterations = 0u64;
        let mut checksum = 0usize;
        loop {
            for _ in 0..32 {
                for index in 0..self.sources.len() {
                    checksum = checksum.wrapping_add(self.bench_one(index));
                }
            }
            iterations += 32;
            if start.elapsed() >= budget {
                break;
            }
        }
        (iterations, start.elapsed().as_nanos(), black_box(checksum))
    }
}

fn hex(bytes: &[u8]) -> String {
    let mut result = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        result.push_str(&format!("{byte:02x}"));
    }
    result
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    assert!(args.len() >= 3, "worker PROFILE STAGE INPUT...");
    let sources: Vec<&'static str> = args[2..]
        .iter()
        .map(|path| -> &'static str {
            let bytes = std::fs::read(path).expect("read input");
            Box::leak(String::from_utf8(bytes).expect("UTF-8 input").into_boxed_str())
        })
        .collect();
    let mut engine = Engine::new(&args[0], &args[1], sources);
    let mut stdout = io::BufWriter::new(io::stdout().lock());
    for line in io::stdin().lock().lines() {
        let line = line.expect("read command");
        if line == "verify" {
            for index in 0..engine.sources.len() {
                let result = engine.verify_one(index);
                writeln!(
                    stdout,
                    "result {index} {} {} {} {}",
                    hex(result.html.as_bytes()),
                    hex(result.ast.as_bytes()),
                    result.arena_capacity_bytes,
                    result.children
                )
                .unwrap();
            }
            writeln!(stdout, "done").unwrap();
        } else if let Some(ns) = line.strip_prefix("bench ") {
            let (iterations, elapsed, checksum) = engine.bench(ns.parse().expect("nanoseconds"));
            writeln!(stdout, "timing {iterations} {elapsed} {checksum}").unwrap();
        } else if line == "quit" {
            break;
        } else {
            panic!("unknown command");
        }
        stdout.flush().unwrap();
    }
}
