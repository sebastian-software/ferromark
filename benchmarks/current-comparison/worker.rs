use std::hint::black_box;
use std::io::{self, BufRead, Write};
use std::time::{Duration, Instant};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    Fresh,
    Owned,
    Reuse,
}

impl Mode {
    fn parse(value: &str) -> Self {
        match value {
            "fresh" => Self::Fresh,
            "owned" => Self::Owned,
            "reuse" => Self::Reuse,
            _ => panic!("unknown lifecycle"),
        }
    }
}

#[cfg(not(feature = "v2"))]
mod engine {
    use super::Mode;
    pub struct Engine {
        options: ferromark::Options,
        renderer: ferromark::Renderer,
        output: Vec<u8>,
        mode: Mode,
    }

    impl Engine {
        pub fn new(profile: &str, mode: &str, _max_source: usize) -> Self {
            let mut options = match profile {
                "commonmark" => ferromark::Options::commonmark(),
                "gfm" => ferromark::Options::gfm(),
                _ => panic!("unknown profile"),
            };
            options.heading_ids = true;
            options.render_policy = ferromark::RenderPolicy::Trusted;
            Self {
                renderer: ferromark::Renderer::with_options(options.clone()),
                options,
                output: Vec::new(),
                mode: Mode::parse(mode),
            }
        }

        pub fn render<T>(&mut self, source: &str, consume: impl FnOnce(&[u8]) -> T) -> T {
            match self.mode {
                Mode::Fresh => {
                    let html = ferromark::to_html_with_options(source, &self.options);
                    consume(html.as_bytes())
                }
                Mode::Owned => {
                    let html = self.renderer.render(source);
                    consume(html.as_bytes())
                }
                Mode::Reuse => {
                    self.renderer.render_into(source, &mut self.output);
                    consume(&self.output)
                }
            }
        }
    }
}

#[cfg(feature = "v2")]
mod engine {
    use super::Mode;
    pub struct Engine {
        options: ferromark::ParserOptions,
        html_options: ferromark::HtmlRendererOptions,
        allocator: ferromark::Allocator,
        renderer: ferromark::HtmlRenderer,
        mode: Mode,
    }

    impl Engine {
        pub fn new(profile: &str, mode: &str, max_source: usize) -> Self {
            let mut options = match profile {
                "commonmark" => ferromark::ParserOptions::default(),
                "gfm" => ferromark::ParserOptions::gfm(),
                _ => panic!("unknown profile"),
            };
            options.footnotes = false;
            let html_options = ferromark::HtmlRendererOptions {
                xhtml: true,
                hard_break: "<br />\n".into(),
                autolink_urls: false,
                autolink_target_blank: false,
                link_target_blank: false,
                disallow_raw_html: profile == "gfm",
                ..ferromark::HtmlRendererOptions::new()
            };
            Self {
                renderer: ferromark::HtmlRenderer::with_options(html_options.clone()),
                allocator: ferromark::Allocator::for_source_len(max_source),
                options,
                html_options,
                mode: Mode::parse(mode),
            }
        }

        pub fn render<T>(&mut self, source: &str, consume: impl FnOnce(&[u8]) -> T) -> T {
            if self.mode == Mode::Fresh {
                let allocator = ferromark::Allocator::for_source_len(source.len());
                let document =
                    ferromark::Parser::with_options(&allocator, source, self.options.clone())
                        .parse()
                        .expect("parse input");
                let mut renderer = ferromark::HtmlRenderer::with_options(self.html_options.clone());
                let html = renderer.render(&document);
                consume(html.as_bytes())
            } else {
                let result = {
                    let document = ferromark::Parser::with_options(
                        &self.allocator,
                        source,
                        self.options.clone(),
                    )
                    .parse()
                    .expect("parse input");
                    if self.mode == Mode::Owned {
                        let html = self.renderer.render(&document);
                        consume(html.as_bytes())
                    } else {
                        consume(self.renderer.render_borrowed(&document).as_bytes())
                    }
                };
                self.allocator.reset();
                result
            }
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    assert!(args.len() >= 3, "worker PROFILE MODE INPUT...");
    let inputs: Vec<String> =
        args[2..].iter().map(|path| std::fs::read_to_string(path).expect("read input")).collect();
    let max_source = inputs.iter().map(String::len).max().unwrap();
    let mut engine = engine::Engine::new(&args[0], &args[1], max_source);
    let mut stdout = io::BufWriter::new(io::stdout().lock());
    for line in io::stdin().lock().lines() {
        let line = line.expect("read command");
        if line == "verify" {
            for (index, source) in inputs.iter().enumerate() {
                let html = engine.render(source, |bytes| bytes.to_vec());
                write!(stdout, "html {index} ").unwrap();
                for byte in html {
                    write!(stdout, "{byte:02x}").unwrap();
                }
                writeln!(stdout).unwrap();
            }
            writeln!(stdout, "done").unwrap();
        } else if let Some(ns) = line.strip_prefix("bench ") {
            let budget = Duration::from_nanos(ns.parse().expect("nanosecond budget"));
            let start = Instant::now();
            let mut iterations = 0u64;
            let mut checksum = 0usize;
            loop {
                for _ in 0..32 {
                    for source in &inputs {
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
            let elapsed = start.elapsed().as_nanos();
            writeln!(stdout, "timing {iterations} {elapsed} {}", black_box(checksum)).unwrap();
        } else if line == "quit" {
            break;
        } else {
            panic!("unknown command");
        }
        stdout.flush().unwrap();
    }
}
