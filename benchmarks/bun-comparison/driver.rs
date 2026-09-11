//! Direct native render comparison. Copied into the pinned Bun workspace by prepare.py.
use serde_json::json;
use std::{
    hint::black_box,
    time::{Duration, Instant},
};

#[global_allocator]
static ALLOC: bun_alloc::Mimalloc = bun_alloc::Mimalloc;

const PARSERS: [&str; 5] = ["ferromark", "bun_md", "pulldown-cmark", "comrak", "md4c"];

unsafe extern "C" {
    fn md_html(
        input: *const u8,
        len: u32,
        output: extern "C" fn(*const u8, u32, *mut std::ffi::c_void),
        userdata: *mut std::ffi::c_void,
        parser_flags: u32,
        renderer_flags: u32,
    ) -> i32;
}

extern "C" fn md4c_output(data: *const u8, len: u32, userdata: *mut std::ffi::c_void) {
    if len == 0 {
        return;
    }
    // md_html invokes this synchronously with a live byte span and our unique output buffer.
    unsafe {
        (&mut *userdata.cast::<Vec<u8>>())
            .extend_from_slice(std::slice::from_raw_parts(data, len as usize));
    }
}

const CONFIGURATIONS: [(&str, u32); 5] = [
    ("commonmark", 0),
    ("gfm_overlap", 7),
    ("tables", 1),
    ("strikethrough", 2),
    ("task_lists", 4),
];

fn bun_flag(name: &str, flags: u32) -> bool {
    match name {
        "tables" => flags & 1 != 0,
        "strikethrough" => flags & 2 != 0,
        "tasklists" => flags & 4 != 0,
        _ => false,
    }
}

struct Renderers {
    ferro: ferromark::Options,
    bun: bun_md::root::Options,
    pulldown: pulldown_cmark::Options,
    comrak: comrak::Options<'static>,
    md4c_flags: u32,
}

impl Renderers {
    fn new(flags: u32) -> Self {
        let (tables, strike, tasks) = (flags & 1 != 0, flags & 2 != 0, flags & 4 != 0);
        let ferro = ferromark::options!(ferromark::Options::commonmark();
            render_policy: ferromark::RenderPolicy::Trusted,
            tables, strikethrough: strike, task_lists: tasks,
            heading_ids: false, callouts: false, disallowed_raw_html: false,
        );
        let mut bun = bun_md::root::Options::default();
        for (name, _, setter) in bun_md::root::Options::BOOL_FIELD_SETTERS {
            setter(&mut bun, bun_flag(name, flags));
        }
        let mut comrak = comrak::Options::default();
        comrak.extension.table = tables;
        comrak.extension.strikethrough = strike;
        comrak.extension.tasklist = tasks;
        comrak.render.r#unsafe = true;
        let mut pulldown = pulldown_cmark::Options::empty();
        pulldown.set(pulldown_cmark::Options::ENABLE_TABLES, tables);
        pulldown.set(pulldown_cmark::Options::ENABLE_STRIKETHROUGH, strike);
        pulldown.set(pulldown_cmark::Options::ENABLE_TASKLISTS, tasks);
        Self {
            ferro,
            bun,
            pulldown,
            comrak,
            md4c_flags: (if tables { 0x0100 } else { 0 })
                | (if strike { 0x0200 } else { 0 })
                | (if tasks { 0x2000 } else { 0 }),
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
            4 => {
                let mut out = Vec::new();
                // Input and output live throughout the synchronous C call; length is checked.
                let rc = unsafe {
                    md_html(
                        input.as_ptr(),
                        input.len().try_into().unwrap(),
                        md4c_output,
                        (&mut out as *mut Vec<u8>).cast(),
                        self.md4c_flags,
                        0,
                    )
                };
                assert_eq!(rc, 0, "md4c render failed");
                out
            }
            _ => unreachable!(),
        }
    }
}

fn corpora() -> Vec<(String, String)> {
    let repo = std::path::Path::new(env!("FERROMARK_SOURCE"));
    let mut result = vec![
        ("short-100b".into(), "A **short update** with a [link](/guide), `code`, and a second sentence for the reader. Stay informed.\n".into()),
        ("strikethrough".into(), "Some ~~old words~~ and **replacement words**.\n\n".repeat(100)),
        ("tasks".into(), "- [x] First task\n- [ ] Second task\n\n".repeat(100)),
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
    // CommonMark-only inputs for the headline, separate from the historical table-containing fixtures.
    let unit = "## Project notes\n\nA paragraph with **strong emphasis**, *emphasis*, and `inline code`.\n\nRead the [documentation](https://example.com/guide?a=1&b=2) for details &amp; examples.\n\n- First item\n- Second item with a [link](/relative)\n\n> A quoted paragraph with a useful explanation.\n\n```rust\nlet answer = 42;\n```\n\n";
    for (name, bytes) in [
        ("publication-2k", 2 * 1024),
        ("publication-5k", 5 * 1024),
        ("publication-10k", 10 * 1024),
        ("publication-50k", 50 * 1024),
        ("light-1k", 1024),
    ] {
        let mut text = unit.repeat(bytes / unit.len());
        text.push_str(&"x".repeat(bytes - text.len() - 1));
        text.push('\n');
        result.push((name.into(), text));
    }
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
    if mode == "catalog" {
        println!("{}", json!(corpora()));
        return;
    }
    if mode == "options" {
        for (lane, flags) in CONFIGURATIONS {
            let r = Renderers::new(flags);
            println!(
                "{}",
                json!({"lane":lane, "ferromark":format!("{:?}",r.ferro),
                "comrak":format!("{:?}",r.comrak), "pulldown_flags":r.pulldown.bits(),
                "md4c_parser_flags":r.md4c_flags, "md4c_renderer_flags":0,
                "bun": bun_md::root::Options::BOOL_FIELD_SETTERS.iter().map(|(name,_,_)|
                    ((*name).to_owned(),json!(bun_flag(name, flags))))
                    .collect::<serde_json::Map<_,_>>() })
            );
        }
        return;
    }
    if mode == "spec" {
        let spec: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(
                std::path::Path::new(env!("FERROMARK_SOURCE")).join("tests/spec.json"),
            )
            .unwrap(),
        )
        .unwrap();
        let r = Renderers::new(0);
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
    let repetitions = args.get(3).map_or(80, |s| s.parse::<usize>().unwrap());
    let window_ms = args.get(4).map_or(63, |s| s.parse::<u64>().unwrap());
    let warmup_ms = args.get(5).map_or(3000, |s| s.parse::<u64>().unwrap());
    let rotation = args.get(6).map_or(0, |s| s.parse::<usize>().unwrap());
    assert!(repetitions > 0 && window_ms > 0);
    for (lane, flags) in CONFIGURATIONS {
        let r = Renderers::new(flags);
        for (name, input) in corpora() {
            if flags == 1 && !["gfm-tables", "light-1k"].contains(&name.as_str()) {
                continue;
            }
            if flags == 2 && !["strikethrough", "light-1k"].contains(&name.as_str()) {
                continue;
            }
            if flags == 4 && !["tasks", "light-1k"].contains(&name.as_str()) {
                continue;
            }
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
            let mut samples: [Vec<f64>; PARSERS.len()] = std::array::from_fn(|_| vec![]);
            let output_bytes: Vec<_> = (0..PARSERS.len())
                .map(|p| r.render(p, &input).len())
                .collect();
            for p in 0..PARSERS.len() {
                let start = Instant::now();
                while start.elapsed() < Duration::from_millis(warmup_ms) {
                    black_box(r.render(p, black_box(&input)));
                }
            }
            for rep in 0..repetitions {
                // Alternate direction and rotate the start to avoid always favoring one parser.
                for offset in 0..PARSERS.len() {
                    let p = if rep % 2 == 0 {
                        (rep + rotation + offset) % PARSERS.len()
                    } else {
                        (rep + rotation + PARSERS.len() - offset) % PARSERS.len()
                    };
                    let start = Instant::now();
                    let mut n = 0u64;
                    loop {
                        for _ in 0..16 {
                            black_box(r.render(p, black_box(&input)));
                        }
                        n += 16;
                        if start.elapsed() >= Duration::from_millis(window_ms) {
                            break;
                        }
                    }
                    samples[p].push(start.elapsed().as_secs_f64() * 1e9 / n as f64);
                }
            }
            for (p, times) in samples.iter().enumerate() {
                println!(
                    "{}",
                    json!({"case":key,"parser":PARSERS[p],"bytes":input.len(),"output_bytes":output_bytes[p],"ns_per_render":times})
                );
            }
        }
    }
}
