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
