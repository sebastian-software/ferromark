//! Diagnostic only. Timing and instrumented builds must remain separate.
use std::{hint::black_box, time::{Duration, Instant}};
use ferromark::{Options, RenderPolicy, Renderer, BlockParser};
use pulldown_cmark::{Parser, Options as POptions, html};
use serde_json::json;
#[cfg(feature = "counts")]
use ferromark_pulldown_comparison::{CountingAllocator, MeasurementWindow};
#[cfg(feature = "counts")]
#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator;

fn table(cols: usize, rows: usize, cell: &str) -> String {
    let row = format!("|{}\n", format!(" {cell} |").repeat(cols));
    format!("{}|{}\n{}\n", row, " --- |".repeat(cols), row.repeat(rows))
}
fn corpus(name: &str, root: &str) -> String {
    match name {
        "published" => "| Item | State |\n| --- | --- |\n| **table** | ~~pending~~ done |\n\n".repeat(80),
        "fixture" => std::fs::read_to_string(format!("{root}/benches/fixtures/tables-5k.md")).unwrap(),
        "control" => std::fs::read_to_string(format!("{root}/benches/fixtures/commonmark-5k.md")).unwrap(),
        "short" => table(4, 100, "data"),
        "long" => table(4, 100, &"data".repeat(32)),
        "emphasis" => table(4, 100, "**data**"),
        "escaped" => table(4, 100, r"da\|ta"),
        "cols8" => table(8, 100, "data"),
        "cols9" => table(9, 100, "data"),
        "cols16" => table(16, 100, "data"),
        "rows10" => table(4, 10, "data"),
        "rows1000" => table(4, 1000, "data"),
        "many" => table(4, 1, "data").repeat(50),
        "long-prose" => format!("{}\n\n", "data".repeat(32)).repeat(404),
        _ => panic!("unknown corpus"),
    }
}
fn options(config: &str) -> (Options, POptions) {
    let mut f = Options::commonmark();
    f.render_policy = RenderPolicy::Trusted;
    f.tables = true;
    let mut p = POptions::ENABLE_TABLES;
    if config == "overlap" { f.strikethrough = true; f.task_lists = true; p |= POptions::ENABLE_STRIKETHROUGH | POptions::ENABLE_TASKLISTS; }
    assert!(config == "tables" || config == "overlap");
    (f,p)
}
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let (mode, name, config, lane, root) = (&args[1], &args[2], &args[3], &args[4], &args[5]);
    let input = corpus(name, root);
    let (f,p) = options(config);
    if mode == "verify" {
        let fh = ferromark::to_html_with_options(&input, &f);
        let mut ph = String::new();
        html::push_html(&mut ph, Parser::new_ext(&input,p));
        let mut renderer = Renderer::with_options(f);
        let mut reuse = Vec::new();
        renderer.render_into(&input, &mut reuse);
        assert_eq!(fh.as_bytes(), reuse);
        ferromark::to_html_into_with_options(&input, &mut reuse, renderer.options());
        assert_eq!(fh.as_bytes(), reuse);
        println!("{}", json!({"input":input,"ferromark":fh,"pulldown":ph}));
        return;
    }
    let mut output = Vec::with_capacity(input.len()*2);
    let mut pout = String::with_capacity(input.len()*2);
    let mut renderer = Renderer::with_options(f.clone());
    let mut events = Vec::new();
    let mut render = || -> usize {
        match lane.as_str() {
            "ferro-owned" => { let out = ferromark::to_html_with_options(black_box(&input), &f); black_box(&out); out.len() },
            "pull-owned" => { let mut out = String::new(); html::push_html(&mut out, Parser::new_ext(black_box(&input),p)); black_box(&out); out.len() },
            "ferro-reuse" => { ferromark::to_html_into_with_options(black_box(&input), &mut output, &f); black_box(&output); output.len() },
            "pull-reuse" => { pout.clear(); html::push_html(&mut pout, Parser::new_ext(black_box(&input),p)); black_box(&pout); pout.len() },
            "ferro-retained" => { renderer.render_into(black_box(&input), &mut output); black_box(&output); output.len() },
            "ferro-block" => { events.clear(); BlockParser::new_with_options(black_box(input.as_bytes()), f.clone()).parse(&mut events); black_box(&events); events.len() },
            "pull-events" => { let mut n=0; for event in Parser::new_ext(black_box(&input),p) { black_box(event); n+=1; } n },
            _ => panic!("unknown lane"),
        }
    };
    for _ in 0..32 { black_box(render()); }
    if mode == "counts" {
        #[cfg(feature = "counts")]
        {
            ferromark::profiling::reset();
            let w = MeasurementWindow::start();
            let bytes = render();
            let allocations = w.finish();
            let c = ferromark::profiling::snapshot();
            println!("{}",json!({"lane":lane,"corpus":name,"config":config,"allocations":allocations,"output_bytes":bytes,"pipeline":{"block_events":c.block_events,"table_events":c.block_table_events,"inline_parses":c.inline_parses,"inline_fast_paths":c.inline_fast_paths,"inline_input_bytes":c.inline_input_bytes,"inline_events":c.inline_events,"paragraph_copied_bytes":c.paragraph_copied_bytes}}));
            return;
        }
        #[cfg(not(feature = "counts"))]
        panic!("counts feature required");
    }
    assert!(!cfg!(feature = "counts"), "never time instrumented code");
    let duration = Duration::from_millis(args.get(6).map_or(250, |s| s.parse().unwrap()));
    let start = Instant::now();
    let mut iterations=0;
    let mut bytes=0;
    loop {
        for _ in 0..16 { bytes=black_box(render()); iterations+=1; }
        if mode != "forever" && start.elapsed() >= duration { break; }
    }
    println!("{}",json!({"lane":lane,"corpus":name,"config":config,"input_bytes":input.len(),"output_bytes":bytes,"iterations":iterations,"elapsed_ns":start.elapsed().as_nanos()}));
}
