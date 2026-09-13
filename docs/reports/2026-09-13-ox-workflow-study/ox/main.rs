mod adapter {
    use ox_content_allocator::Allocator;
    use ox_content_parser::{Parser, ParserOptions};
    use ox_content_renderer::{HtmlRenderer, HtmlRendererOptions};
    use std::cell::RefCell;

    pub struct Renderer {
        parser_options: ParserOptions,
        html_options: HtmlRendererOptions,
        html: RefCell<HtmlRenderer>,
        fresh: bool,
    }
    impl Renderer {
        pub fn new(flags: u32) -> Self {
            assert!(flags <= 7);
            let parser_options = ParserOptions {
                tables: flags & 1 != 0,
                strikethrough: flags & 2 != 0,
                task_lists: flags & 4 != 0,
                ..Default::default()
            };
            let html_options = HtmlRendererOptions {
                autolink_urls: false,
                autolink_target_blank: false,
                link_target_blank: false,
                ..Default::default()
            };
            let html = RefCell::new(HtmlRenderer::with_options(html_options.clone()));
            Self {
                parser_options,
                html_options,
                html,
                fresh: std::env::args().nth(1).unwrap() == "ox-fresh",
            }
        }
        pub fn render(&self, input: &str) -> String {
            let arena = Allocator::new();
            let document = Parser::with_options(&arena, input, self.parser_options.clone())
                .parse()
                .unwrap();
            // render() resets per-document state and moves the owned output out.
            // render_borrowed(), AST reuse and incremental render APIs are not used.
            if self.fresh {
                HtmlRenderer::with_options(self.html_options.clone()).render(&document)
            } else {
                self.html.borrow_mut().render(&document)
            }
        }
        pub fn options(&self) -> String {
            format!(
                "{:?}; {:?}; native generated heading IDs retained; fresh arena and owned output; renderer scratch reused",
                self.parser_options, self.html_options
            )
        }
    }
}

// Shared complete-collection protocol, appended to the existing native adapters.
// Preloaded input/configuration and JSON are outside the timer. Owned output,
// its retention container, parsing and destruction are inside each workload.
fn main() {
    use std::io::{BufRead, Write};
    let args: Vec<String> = std::env::args().collect();
    let engine = &args[1];
    let corpus: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&args[2]).unwrap()).unwrap();
    let group = &args[3];
    let documents = corpus[group].as_array().unwrap();
    let inputs: Vec<&str> = documents
        .iter()
        .map(|d| d["input"].as_str().unwrap())
        .collect();
    let renderer = adapter::Renderer::new(if engine == "cmark" { 0 } else { 7 });
    let retain = args[4] == "retain";
    let run = || {
        let mut kept = Vec::new();
        let mut bytes = 0u64;
        for input in &inputs {
            let html = std::hint::black_box(renderer.render(std::hint::black_box(input)));
            let value: &[u8] = html.as_ref();
            bytes += value.len() as u64;
            if retain {
                kept.push(html);
            }
        }
        std::hint::black_box(&kept);
        drop(kept);
        std::hint::black_box(bytes)
    };
    let mut out = std::io::BufWriter::new(std::io::stdout().lock());
    for line in std::io::stdin().lock().lines() {
        let request: serde_json::Value = serde_json::from_str(&line.unwrap()).unwrap();
        let result = match request["action"].as_str().unwrap() {
            "verify" => {
                let outputs: Vec<_> = documents.iter().map(|d| {
                    let html = renderer.render(d["input"].as_str().unwrap());
                    let bytes: &[u8] = html.as_ref();
                    serde_json::json!({"id": d["id"], "html": std::str::from_utf8(bytes).unwrap(), "metadata": null})
                }).collect();
                serde_json::json!({"engine": engine, "group": group, "outputs": outputs, "options": renderer.options(), "retain": retain})
            }
            "time" => {
                let duration =
                    std::time::Duration::from_millis(request["milliseconds"].as_u64().unwrap());
                let start = std::time::Instant::now();
                let mut iterations = 0u64;
                let mut output_bytes = 0u64;
                loop {
                    for _ in 0..4 {
                        output_bytes += run();
                        iterations += 1;
                    }
                    if start.elapsed() >= duration {
                        break;
                    }
                }
                let elapsed_ns = start.elapsed().as_nanos() as u64;
                serde_json::json!({"iterations": iterations, "elapsed_ns": elapsed_ns, "output_units": output_bytes,
                    "ns_per_workload": elapsed_ns as f64 / iterations as f64})
            }
            _ => panic!("unknown action"),
        };
        writeln!(out, "{result}").unwrap();
        out.flush().unwrap();
    }
}
