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

fn main() {
    use std::hint::black_box;
    use std::io::{BufRead, Write};
    let mode = std::env::args().nth(1).unwrap();
    let corpus: serde_json::Value =
        serde_json::from_slice(&std::fs::read(std::env::args().nth(2).unwrap()).unwrap()).unwrap();
    let renderer = adapter::Renderer::new(7);
    let mut output = std::io::BufWriter::new(std::io::stdout().lock());
    for line in std::io::stdin().lock().lines() {
        let request: serde_json::Value = serde_json::from_str(&line.unwrap()).unwrap();
        let result = match request["action"].as_str().unwrap() {
            "render" => {
                let html = renderer.render(black_box(request["input"].as_str().unwrap()));
                serde_json::json!({"html": html})
            }
            "sequence" => {
                let inputs: Vec<String> = request["inputs"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| v.as_str().unwrap().to_owned())
                    .collect();
                let mut buffer =
                    String::with_capacity(inputs.iter().map(String::len).max().unwrap().max(1));
                let original_address = buffer.as_ptr() as usize;
                let mut answers = Vec::new();
                for input in &inputs {
                    let source = if request["storage"].as_str().unwrap() == "same-address" {
                        buffer.clear();
                        buffer.push_str(input);
                        assert_eq!(buffer.as_ptr() as usize, original_address);
                        buffer.as_str()
                    } else {
                        input.as_str()
                    };
                    let html = black_box(renderer.render(black_box(source)));
                    answers
                        .push(serde_json::json!({"html":html,"address":source.as_ptr() as usize}));
                }
                serde_json::json!({"answers":answers})
            }
            "pool" => {
                let policy = request["policy"].as_str().unwrap();
                let size = if policy == "repeated" { 1 } else { 32 };
                let mut pool = Vec::new();
                for index in 0..size {
                    let nonce = if policy == "changed-content" {
                        index
                    } else {
                        0
                    };
                    let set: Vec<String> = corpus["documentation"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|d| {
                            format!(
                                "{}\n\nAudit nonce {nonce:04}.\n",
                                d["input"].as_str().unwrap()
                            )
                        })
                        .collect();
                    pool.push(set);
                }
                let mut expected_units = Vec::new();
                for set in &pool {
                    expected_units.push(
                        set.iter()
                            .map(|input| renderer.render(input).len() as u64)
                            .sum::<u64>(),
                    );
                }
                let start = std::time::Instant::now();
                let duration =
                    std::time::Duration::from_millis(request["milliseconds"].as_u64().unwrap());
                let mut iterations = 0usize;
                let mut units = 0u64;
                let mut expected = 0u64;
                loop {
                    for _ in 0..4 {
                        let index = iterations % size;
                        for input in &pool[index] {
                            let html = black_box(renderer.render(black_box(input)));
                            units += html.len() as u64;
                        }
                        expected += expected_units[index];
                        iterations += 1;
                    }
                    if start.elapsed() >= duration {
                        break;
                    }
                }
                let elapsed_ns = start.elapsed().as_nanos() as u64;
                assert_eq!(units, expected);
                serde_json::json!({"iterations":iterations,"elapsed_ns":elapsed_ns,"ns_per_collection":elapsed_ns as f64/iterations as f64,"output_bytes":units,"pool_size":size,"mode":mode})
            }
            _ => panic!("Unknown audit action"),
        };
        writeln!(output, "{result}").unwrap();
        output.flush().unwrap();
    }
}
