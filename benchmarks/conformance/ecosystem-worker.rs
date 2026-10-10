//! Untimed public CommonMark/GFM configurations for the pinned Rust competitors.
use std::io::{self, BufRead, Write};
fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    assert!(args.len() >= 4 && args[2] == "fresh");
    assert!(matches!(args[1].as_str(), "commonmark" | "gfm"));
    let gfm = args[1] == "gfm";
    let mut comrak = comrak::Options::default();
    comrak.extension.table = gfm;
    comrak.extension.strikethrough = gfm;
    comrak.extension.tasklist = gfm;
    comrak.extension.autolink = gfm;
    comrak.extension.tagfilter = gfm;
    comrak.render.r#unsafe = true;
    let mut markdown = if gfm {
        markdown::Options::gfm()
    } else {
        markdown::Options::default()
    };
    markdown.parse.constructs.gfm_footnote_definition = false;
    markdown.parse.constructs.gfm_label_start_footnote = false;
    markdown.compile.allow_dangerous_html = true;
    markdown.compile.allow_dangerous_protocol = true;
    let inputs: Vec<String> = args[3..]
        .iter()
        .map(|p| std::fs::read_to_string(p).expect("input"))
        .collect();
    let mut out = io::BufWriter::new(io::stdout().lock());
    for line in io::stdin().lock().lines() {
        match line.unwrap().as_str() {
            "quit" => break,
            "verify" => {
                for (index, source) in inputs.iter().enumerate() {
                    let html = match args[0].as_str() {
                        "comrak" => comrak::markdown_to_html(source, &comrak),
                        "markdown-rs" => {
                            markdown::to_html_with_options(source, &markdown).expect("render")
                        }
                        _ => panic!("unknown engine"),
                    };
                    write!(out, "html {index} ").unwrap();
                    for byte in html.bytes() {
                        write!(out, "{byte:02x}").unwrap();
                    }
                    writeln!(out).unwrap();
                    out.flush().unwrap();
                }
                writeln!(out, "done").unwrap();
                out.flush().unwrap();
            }
            _ => panic!("unknown command"),
        }
    }
}
