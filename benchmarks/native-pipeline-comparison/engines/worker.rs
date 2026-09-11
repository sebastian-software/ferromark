// Shared native Rust worker. All parsing, fresh output allocation, and drop are timed.
use serde::Deserialize;
use serde_json::{json, Value};
use std::{hint::black_box, io::{self, BufRead, Write}, time::{Duration, Instant}};

#[derive(Deserialize)]
struct Case { case: String, input: String, flags: u32 }

fn main() {
    let args: Vec<_> = std::env::args().collect();
    assert_eq!(args[1], adapter::NAME);
    let cases: Vec<Case> = serde_json::from_slice(&std::fs::read(&args[2]).unwrap()).unwrap();
    let renderers: Vec<_> = cases.iter().map(|c| adapter::Renderer::new(c.flags)).collect();
    let mut output = io::BufWriter::new(io::stdout().lock());
    for line in io::stdin().lock().lines() {
        let request: Value = serde_json::from_str(&line.unwrap()).unwrap();
        let index = request["index"].as_u64().unwrap() as usize;
        let case = &cases[index];
        let renderer = &renderers[index];
        let row = match request["op"].as_str().unwrap() {
            "verify" => json!({"case":case.case,"engine":adapter::NAME,"flags":case.flags,
                "bytes":case.input.len(),"html":renderer.render(&case.input),"options":renderer.options()}),
            "window" => {
                let ms = request["ms"].as_u64().unwrap();
                assert!(ms > 0);
                let start = Instant::now();
                let mut count = 0u64;
                loop {
                    for _ in 0..16 { drop(black_box(renderer.render(black_box(&case.input)))); }
                    count += 16;
                    if start.elapsed() >= Duration::from_millis(ms) { break; }
                }
                let elapsed = start.elapsed().as_nanos() as u64;
                json!({"case":case.case,"engine":adapter::NAME,"count":count,"elapsed_ns":elapsed,
                    "ns_per_render":elapsed as f64/count as f64})
            }
            _ => panic!("unknown operation"),
        };
        writeln!(output,"{row}").unwrap();
        output.flush().unwrap();
    }
}
