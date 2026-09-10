//! Reproducible GFM optimization probe; timing and allocation builds are separate.
//! Arguments: mode case preset lane window_ms rounds. Use `all` for a filter.
use ferromark::{Options, RenderPolicy, Renderer};
use std::{
    hint::black_box,
    time::{Duration, Instant},
};

#[cfg(feature = "profiling")]
#[path = "support/allocation.rs"]
mod allocation;

#[cfg(feature = "profiling")]
#[global_allocator]
static ALLOCATOR: allocation::Counter = allocation::Counter;

fn options(name: &str) -> Options {
    let mut o = Options::commonmark();
    match name {
        "commonmark" => {}
        "tables" => o.tables = true,
        "autolinks" => o.autolink_literals = true,
        "strike" => o.strikethrough = true,
        "tasks" => o.task_lists = true,
        "filter" => o.disallowed_raw_html = true,
        "overlap" => {
            o.tables = true;
            o.strikethrough = true;
            o.task_lists = true;
        }
        "gfm" => o = Options::gfm(),
        "gfm-no-autolinks" => {
            o = Options::gfm();
            o.autolink_literals = false;
        }
        "gfm-no-tables" => {
            o = Options::gfm();
            o.tables = false;
        }
        _ => panic!("unknown preset"),
    }
    o
}
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mode = args.get(1).map_or("measure", String::as_str);
    let case_filter = args.get(2).map_or("all", String::as_str);
    let preset_filter = args.get(3).map_or("all", String::as_str);
    let lane_filter = args.get(4).map_or("all", String::as_str);
    let window_ms = args.get(5).map_or(Ok(150), |s| s.parse::<u64>()).unwrap();
    let requested_rounds = args.get(6).map_or(Ok(5), |s| s.parse::<usize>()).unwrap();
    let corpus: Vec<_> = corpora()
        .into_iter()
        .filter(|(name, _)| case_filter == "all" || *name == case_filter)
        .collect();
    assert!(!corpus.is_empty(), "unknown case");
    if mode == "fixtures" {
        for (name, input) in &corpus {
            std::fs::write(format!("target/gfm-profile/{name}.md"), input).unwrap();
        }
        return;
    }
    let presets = [
        "commonmark",
        "tables",
        "autolinks",
        "strike",
        "tasks",
        "filter",
        "overlap",
        "gfm",
        "gfm-no-autolinks",
        "gfm-no-tables",
    ];
    let mut results = Vec::new();
    let rounds = if mode == "measure" {
        requested_rounds
    } else {
        1
    };
    for round in 0..rounds {
        for (case, input) in &corpus {
            let baseline = ferromark::to_html_with_options(input, &Options::commonmark());
            let mut order: Vec<_> = (0..presets.len()).collect();
            order.rotate_left(round % presets.len());
            if round % 2 == 1 {
                order.reverse();
            }
            for index in order {
                let preset = presets[index];
                if preset_filter != "all" && preset != preset_filter {
                    continue;
                }
                let opts = options(preset);
                let expected = ferromark::to_html_with_options(input, &opts);
                if mode == "html" {
                    results.push(serde_json::json!({"case":case,"preset":preset,"html":expected}));
                    continue;
                }
                for lane in ["owned", "buffer", "renderer"] {
                    if lane_filter != "all" && lane != lane_filter {
                        continue;
                    }
                    let mut renderer = Renderer::with_options(opts.clone());
                    let mut output = Vec::with_capacity(input.len() * 2);
                    renderer.render_into(input, &mut output);
                    assert_eq!(output, expected.as_bytes());
                    ferromark::to_html_into_with_options(input, &mut output, &opts);
                    assert_eq!(output, expected.as_bytes());
                    let mut render = || match lane {
                        "owned" => {
                            black_box(ferromark::to_html_with_options(
                                black_box(input),
                                black_box(&opts),
                            ));
                        }
                        "buffer" => {
                            ferromark::to_html_into_with_options(
                                black_box(input),
                                &mut output,
                                black_box(&opts),
                            );
                            black_box(&output);
                        }
                        _ => {
                            renderer.render_into(black_box(input), &mut output);
                            black_box(&output);
                        }
                    };
                    for _ in 0..16 {
                        render();
                    }
                    if mode == "measure" {
                        let start = Instant::now();
                        let mut n = 0u64;
                        loop {
                            for _ in 0..16 {
                                render();
                            }
                            n += 16;
                            if start.elapsed() >= Duration::from_millis(window_ms) {
                                break;
                            }
                        }
                        results.push(serde_json::json!({"case":case,"preset":preset,"lane":lane,"round":round,"input_bytes":input.len(),"output_bytes":expected.len(),"same_as_commonmark":expected==baseline,"ns":start.elapsed().as_secs_f64()*1e9/n as f64}));
                    } else {
                        #[cfg(feature = "profiling")]
                        {
                            use std::sync::atomic::Ordering::Relaxed;
                            ferromark::profiling::reset();
                            allocation::CALLS.store(0, Relaxed);
                            allocation::BYTES.store(0, Relaxed);
                            allocation::ENABLED.store(true, Relaxed);
                            render();
                            allocation::ENABLED.store(false, Relaxed);
                            let snap = ferromark::profiling::snapshot();
                            results.push(serde_json::json!({"case":case,"preset":preset,"lane":lane,"calls":allocation::CALLS.load(Relaxed),"requested_bytes":allocation::BYTES.load(Relaxed),"inline_parses":snap.inline_parses,"inline_bytes":snap.inline_input_bytes,"fast_paths":snap.inline_fast_paths,"marks":snap.inline_marks,"inline_events":snap.inline_events,"block_events":snap.block_events,"table_events":snap.block_table_events,"paragraph_copied_bytes":snap.paragraph_copied_bytes}));
                        }
                        #[cfg(not(feature = "profiling"))]
                        panic!("counts require --features profiling");
                    }
                }
            }
        }
        eprintln!("round {}/{} complete", round + 1, rounds);
    }
    assert!(!results.is_empty(), "unknown preset or lane");
    println!("{}", serde_json::to_string_pretty(&results).unwrap());
    // Keep the trust policy explicit in this probe: every preset is untrusted.
    assert_eq!(options("gfm").render_policy, RenderPolicy::Untrusted);
}
fn corpora() -> Vec<(&'static str, String)> {
    vec![
      ("plain","Ordinary words with no special syntax and enough text for several vector loads.\n\n".repeat(128)),
      ("commonmark-5k",include_str!("../benches/fixtures/commonmark-5k.md").into()),
      ("readme",include_str!("../README.md").into()),
      ("commonmark-50k",include_str!("../benches/fixtures/commonmark-50k.md").into()),
      ("gfm-overlap-tables","| Item | State |\n| --- | --- |\n| **table** | ~~pending~~ done |\n\n".repeat(80)),
      ("tables-5k",include_str!("../benches/fixtures/tables-5k.md").into()),
      ("autolinks","Visit https://example.com/guide and www.example.org or mail person@example.com.\n\n".repeat(96)),
      ("tasks","- [x] First task\n- [ ] Second **task**\n- Ordinary item\n\n".repeat(96)),
      ("mixed-gfm",("## Notes\n\nVisit https://example.com/guide and [the guide](/guide).\n\n| Item | State |\n| --- | --- |\n| **table** | ~~pending~~ done |\n\n- [x] First task\n- [ ] Second task\n\n").repeat(48)),
    ]
}
