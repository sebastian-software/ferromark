use std::hint::black_box;
use std::time::Instant;
use ferromark::{Allocator, HtmlRenderer, HtmlRendererOptions, Parser, ParserOptions};

fn options(profile: &str) -> ParserOptions {
    match profile {
        "commonmark" | "autolink" => ParserOptions::default(),
        "gfm" => { let mut o = ParserOptions::gfm(); o.footnotes = false; o }
        _ => panic!("{profile}"),
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

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let corpus = &args[1];
    let mode = args[2].as_str(); // reuse | parse | render | fresh
    let seconds: f64 = args[3].parse().unwrap();
    let filter = args.get(4).cloned().unwrap_or_default();
    let v: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(corpus).unwrap()).unwrap();
    let cases: Vec<(String, String, String)> = v["cases"].as_array().unwrap().iter()
        .filter(|c| c["suite"] == "broad")
        .filter(|c| filter.is_empty() || c["name"].as_str().unwrap().contains(&filter))
        .map(|c| (c["name"].as_str().unwrap().to_string(), c["profile"].as_str().unwrap().to_string(), c["input"].as_str().unwrap().to_string()))
        .collect();
    eprintln!("{} cases", cases.len());
    let prepared: Vec<_> = cases.iter().map(|(n, p, s)| (n.clone(), options(p), html_options(p), s.as_str(), (400_000 / s.len().max(1)).max(1))).collect();
    if mode == "verify" {
        use std::hash::{Hash, Hasher};
        let mut all = std::collections::hash_map::DefaultHasher::new();
        for (n, o, h, s, _) in &prepared {
            let a = Allocator::new();
            let d = Parser::with_options(&a, s, o.clone()).parse().unwrap();
            let html = HtmlRenderer::with_options(h.clone()).render(&d);
            let mut one = std::collections::hash_map::DefaultHasher::new();
            html.hash(&mut one); format!("{d:?}").hash(&mut one);
            let v = one.finish(); v.hash(&mut all);
            if std::env::var("VERBOSE").is_ok() { println!("{v:016x} {n}"); }
        }
        println!("{:016x}", all.finish());
        return;
    }
    let start = Instant::now();
    let mut alloc = Allocator::for_source_len(200_000);
        let mut sweeps = 0u64;
    // render mode: pre-parse into leaked arena
    let rendered: Vec<_> = if mode == "render" {
        let a: &'static Allocator = Box::leak(Box::new(Allocator::for_source_len(4_000_000)));
        prepared.iter().map(|(_, o, h, s, k)| {
            let s: &'static str = Box::leak(s.to_string().into_boxed_str());
            (Parser::with_options(a, s, o.clone()).parse().unwrap(), HtmlRenderer::with_options(h.clone()), *k)
        }).collect()
    } else { Vec::new() };
    let mut rendered = rendered;
    while start.elapsed().as_secs_f64() < seconds {
        match mode {
            "render" => for (doc, r, k) in rendered.iter_mut() { for _ in 0..*k { black_box(r.render_borrowed(black_box(doc))); } },
            _ => for (_, o, h, s, k) in &prepared {
                let mut r = HtmlRenderer::with_options(h.clone());
                for _ in 0..*k {
                    match mode {
                        "fresh" => { let a = Allocator::for_source_len(s.len()); let d = Parser::with_options(&a, s, o.clone()).parse().unwrap(); black_box(r.render(&d)); }
                        "parse" => { let d = Parser::with_options(&alloc, s, o.clone()).parse().unwrap(); black_box(&d); drop(d); alloc.reset(); }
                        _ => { let d = Parser::with_options(&alloc, s, o.clone()).parse().unwrap(); black_box(r.render_borrowed(&d)); drop(d); alloc.reset(); }
                    }
                }
            },
        }
        sweeps += 1;
    }
    eprintln!("{sweeps} sweeps in {:.2}s", start.elapsed().as_secs_f64());
}
