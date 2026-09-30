//! Matched native Markdown-to-HTML pair; both engines use the system allocator.
use ferromark as v2;
use std::hint::black_box;
use std::io::{self, BufRead, Write};
use std::time::{Duration, Instant};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    Fresh,
    Reuse,
}

trait Engine {
    fn verify_input(&self, _source: &str) {}
    fn render<T>(&mut self, source: &str, consume: impl FnOnce(&[u8]) -> T) -> T;
}

struct Ferromark {
    options: v2::ParserOptions,
    arena: v2::Allocator,
    renderer: v2::HtmlRenderer,
    mode: Mode,
}
impl Engine for Ferromark {
    fn render<T>(&mut self, source: &str, consume: impl FnOnce(&[u8]) -> T) -> T {
        if self.mode == Mode::Fresh {
            let arena = v2::Allocator::for_source_len(source.len());
            let doc = v2::Parser::with_options(&arena, source, self.options.clone())
                .parse()
                .expect("parse");
            let html =
                v2::HtmlRenderer::with_options(v2::HtmlRendererOptions::commonmark()).render(&doc);
            consume(html.as_bytes())
        } else {
            let doc = v2::Parser::with_options(&self.arena, source, self.options.clone())
                .parse()
                .expect("parse");
            let result = consume(self.renderer.render_borrowed(&doc).as_bytes());
            self.arena.reset();
            result
        }
    }
}
struct MarkdownRs(markdown::Options);
impl Engine for MarkdownRs {
    fn render<T>(&mut self, source: &str, consume: impl FnOnce(&[u8]) -> T) -> T {
        // No public retained parser/output API; reuse retains only configuration.
        let html = markdown::to_html_with_options(source, &self.0).expect("parse");
        consume(html.as_bytes())
    }
}
#[allow(
    clippy::disallowed_types,
    reason = "Owned CLI arguments and file contents cross the worker I/O boundary."
)]
fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    assert!(args.len() >= 4, "worker ENGINE PROFILE MODE INPUT...");
    let gfm = match args[1].as_str() {
        "commonmark" => false,
        "gfm" | "gfm-shared" => true,
        _ => panic!("profile"),
    };
    let mode = match args[2].as_str() {
        "fresh" => Mode::Fresh,
        "reuse" => Mode::Reuse,
        _ => panic!("mode"),
    };
    let inputs: Vec<String> = args[3..]
        .iter()
        .map(|p| std::fs::read_to_string(p).expect("input"))
        .collect();
    match args[0].as_str() {
        "v2" => serve(
            Ferromark {
                options: v2::ParserOptions {
                    tables: gfm,
                    strikethrough: gfm,
                    task_lists: gfm,
                    ..v2::ParserOptions::commonmark()
                },
                arena: v2::Allocator::for_source_len(inputs.iter().map(String::len).max().unwrap()),
                renderer: v2::HtmlRenderer::with_options(v2::HtmlRendererOptions::commonmark()),
                mode,
            },
            &inputs,
        ),
        "markdown-rs" => serve(
            MarkdownRs(markdown::Options {
                parse: markdown::ParseOptions {
                    constructs: markdown::Constructs {
                        gfm_table: gfm,
                        gfm_strikethrough: gfm,
                        gfm_task_list_item: gfm,
                        ..markdown::Constructs::default()
                    },
                    ..markdown::ParseOptions::default()
                },
                compile: markdown::CompileOptions {
                    allow_dangerous_html: true,
                    allow_dangerous_protocol: true,
                    ..markdown::CompileOptions::default()
                },
            }),
            &inputs,
        ),
        _ => panic!("engine"),
    }
}
#[allow(
    clippy::disallowed_types,
    reason = "Input strings are owned by the worker I/O boundary."
)]
fn serve(mut engine: impl Engine, inputs: &[String]) {
    let mut stdout = io::BufWriter::new(io::stdout().lock());
    for line in io::stdin().lock().lines() {
        let line = line.expect("command");
        if line == "verify" {
            for (index, source) in inputs.iter().enumerate() {
                engine.verify_input(source);
                let html = engine.render(source, |bytes| bytes.to_vec());
                write!(stdout, "html {index} ").unwrap();
                for byte in html {
                    write!(stdout, "{byte:02x}").unwrap();
                }
                writeln!(stdout).unwrap();
            }
            writeln!(stdout, "done").unwrap();
        } else if let Some(ns) = line.strip_prefix("bench ") {
            let budget = Duration::from_nanos(ns.parse().expect("budget"));
            let start = Instant::now();
            let mut iterations = 0u64;
            let mut checksum = 0usize;
            loop {
                for _ in 0..32 {
                    for source in inputs {
                        checksum = checksum.wrapping_add(
                            engine.render(black_box(source), |html| black_box(html).len()),
                        );
                    }
                }
                iterations += 32;
                if start.elapsed() >= budget {
                    break;
                }
            }
            writeln!(
                stdout,
                "timing {iterations} {} {}",
                start.elapsed().as_nanos(),
                black_box(checksum)
            )
            .unwrap();
        } else if line == "quit" {
            break;
        } else {
            panic!("unknown command");
        }
        stdout.flush().unwrap();
    }
}
