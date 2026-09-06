//! Direct native render comparison. Copied into the pinned Bun workspace by prepare.py.
use serde_json::json;
use std::{
    hint::black_box,
    time::{Duration, Instant},
};

#[global_allocator]
static ALLOC: bun_alloc::Mimalloc = bun_alloc::Mimalloc;

const PARSERS: [&str; 4] = ["ferromark", "bun_md", "pulldown-cmark", "comrak"];

struct Renderers {
    ferro: ferromark::Options,
    bun: bun_md::root::Options,
    pulldown: pulldown_cmark::Options,
    comrak: comrak::Options<'static>,
}

impl Renderers {
    fn new(gfm: bool) -> Self {
        let ferro = ferromark::options!(ferromark::Options::default();
            render_policy: ferromark::RenderPolicy::Trusted,
            tables: gfm, strikethrough: gfm, task_lists: gfm,
            heading_ids: false, callouts: false, disallowed_raw_html: false,
        );
        let mut bun = bun_md::root::Options::default();
        for (name, _, setter) in bun_md::root::Options::BOOL_FIELD_SETTERS {
            setter(
                &mut bun,
                gfm && ["tables", "strikethrough", "tasklists"].contains(name),
            );
        }
        let mut comrak = comrak::Options::default();
        comrak.extension.table = gfm;
        comrak.extension.strikethrough = gfm;
        comrak.extension.tasklist = gfm;
        comrak.render.r#unsafe = true;
        let pulldown = if gfm {
            pulldown_cmark::Options::ENABLE_TABLES
                | pulldown_cmark::Options::ENABLE_STRIKETHROUGH
                | pulldown_cmark::Options::ENABLE_TASKLISTS
        } else {
            pulldown_cmark::Options::empty()
        };
        Self {
            ferro,
            bun,
            pulldown,
            comrak,
        }
    }

    fn render(&self, parser: usize, input: &str) -> Vec<u8> {
        match parser {
            0 => ferromark::to_html_with_options(input, &self.ferro).into_bytes(),
            1 => bun_md::root::render_to_html_with_options(input.as_bytes(), self.bun)
                .expect("Bun render failed")
                .into_vec(),
            2 => {
                let mut out = String::new();
                pulldown_cmark::html::push_html(
                    &mut out,
                    pulldown_cmark::Parser::new_ext(input, self.pulldown),
                );
                out.into_bytes()
            }
            3 => comrak::markdown_to_html(input, &self.comrak).into_bytes(),
            _ => unreachable!(),
        }
    }
}

fn corpora() -> Vec<(String, String)> {
    let repo = std::path::Path::new(env!("FERROMARK_SOURCE"));
    let mut result = vec![
        (
            "gfm-tables".into(),
            "| Item | State |\n| --- | --- |\n| **table** | ~~pending~~ done |\n\n".repeat(80),
        ),
        (
            "gfm-features".into(),
            "| Item | State |\n| --- | --- |\n| **table** | ~~pending~~ done |\n\n- [x] First task\n- [ ] Second task\n\n".repeat(50),
        ),
        ("tiny".into(), "Hello, **world**!\n".into()),
        (
            "prose".into(),
            "Ordinary prose with several words and a little punctuation.\n\n".repeat(90),
        ),
        (
            "links".into(),
            "A [link](https://example.com/a?b=1&c=2) and ![picture](image.png).\n\n".repeat(70),
        ),
        (
            "entities".into(),
            "Text &amp; &lt; &quot; &#169; café **bold** and `code`.\n\n".repeat(100),
        ),
    ];
    for name in [
        "commonmark-5k",
        "commonmark-50k",
        "commonmark-1m",
        "tables-5k",
    ] {
        result.push((
            name.into(),
            std::fs::read_to_string(repo.join("benches/fixtures").join(format!("{name}.md")))
                .unwrap(),
        ));
    }
    result
}

fn main() {
    bun_core::StackCheck::configure_thread();
    let args: Vec<_> = std::env::args().collect();
    let mode = args.get(1).map(String::as_str).unwrap_or("verify");
    if mode == "spec" {
        let spec: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(
                std::path::Path::new(env!("FERROMARK_SOURCE")).join("tests/spec.json"),
            )
            .unwrap(),
        )
        .unwrap();
        let r = Renderers::new(false);
        for item in spec.as_array().unwrap() {
            let input = item["markdown"].as_str().unwrap();
            println!(
                "{}",
                json!({"example": item["example"], "expected": item["html"],
                "outputs": PARSERS.iter().enumerate().map(|(p,name)|
                    ((*name).to_owned(), json!(String::from_utf8(r.render(p,input)).unwrap()))).collect::<serde_json::Map<_,_>>() })
            );
        }
        return;
    }
    assert!(
        mode == "verify" || mode == "bench",
        "expected verify, spec, or bench"
    );
    let allowed: Vec<String> = if mode == "bench" {
        serde_json::from_str(
            &std::fs::read_to_string(args.get(2).expect("bench requires verified allowlist"))
                .unwrap(),
        )
        .unwrap()
    } else {
        vec![]
    };
    let repetitions = 5;
    for (lane, gfm) in [("commonmark", false), ("gfm_overlap", true)] {
        let r = Renderers::new(gfm);
        for (name, input) in corpora() {
            let key = format!("{lane}/{name}");
            if mode == "verify" {
                println!(
                    "{}",
                    json!({"case":key,"bytes":input.len(),"outputs":
                    PARSERS.iter().enumerate().map(|(p,name)| ((*name).to_owned(), json!(String::from_utf8(r.render(p,&input)).unwrap())))
                    .collect::<serde_json::Map<_,_>>() })
                );
                continue;
            }
            if !allowed.contains(&key) {
                continue;
            }
            let mut samples: [Vec<f64>; 4] = std::array::from_fn(|_| vec![]);
            for p in 0..4 {
                let start = Instant::now();
                while start.elapsed() < Duration::from_millis(50) {
                    black_box(r.render(p, black_box(&input)));
                }
            }
            for rep in 0..repetitions {
                // Alternate direction and rotate the start to avoid always favoring one parser.
                for offset in 0..4 {
                    let p = if rep % 2 == 0 {
                        (rep + offset) % 4
                    } else {
                        (rep + 4 - offset) % 4
                    };
                    let start = Instant::now();
                    let mut n = 0u64;
                    loop {
                        for _ in 0..16 {
                            black_box(r.render(p, black_box(&input)));
                        }
                        n += 16;
                        if start.elapsed() >= Duration::from_millis(150) {
                            break;
                        }
                    }
                    samples[p].push(start.elapsed().as_secs_f64() * 1e9 / n as f64);
                }
            }
            for (p, times) in samples.iter().enumerate() {
                println!(
                    "{}",
                    json!({"case":key,"parser":PARSERS[p],"bytes":input.len(),"ns_per_render":times})
                );
            }
        }
    }
}
