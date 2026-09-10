//! Data-driven Markdown feature timing/allocation probe.
//! Arguments: mode catalog.json case_filter lane window_ms rounds.
use ferromark::{Options, RenderPolicy, Renderer};
use serde::Deserialize;
use serde_json::{Value, json};
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

#[derive(Deserialize)]
struct Case {
    id: String,
    group: String,
    feature: String,
    size: String,
    variants: Vec<Variant>,
}
#[derive(Deserialize)]
struct Variant {
    label: String,
    input: String,
    options: Value,
}

fn options(config: &Value) -> Options {
    let mut options = match config["preset"].as_str().unwrap_or("commonmark") {
        "commonmark" => Options::commonmark(),
        "minimal" => Options::minimal(),
        "gfm" => Options::gfm(),
        "default" => Options::default(),
        _ => panic!("unknown preset"),
    };
    for (key, value) in config.as_object().unwrap() {
        match key.as_str() {
            "preset" => {}
            "trusted" => {
                options.render_policy = if value.as_bool().unwrap() {
                    RenderPolicy::Trusted
                } else {
                    RenderPolicy::Untrusted
                }
            }
            "link_base_path" => options.link_base_path = value.as_str().map(Into::into),
            key => {
                let flag = match key {
                    "allow_html" => &mut options.allow_html,
                    "allow_link_refs" => &mut options.allow_link_refs,
                    "tables" => &mut options.tables,
                    "merged_table_cells" => &mut options.merged_table_cells,
                    "table_column_widths" => &mut options.table_column_widths,
                    "strikethrough" => &mut options.strikethrough,
                    "highlight" => &mut options.highlight,
                    "superscript" => &mut options.superscript,
                    "subscript" => &mut options.subscript,
                    "task_lists" => &mut options.task_lists,
                    "autolink_literals" => &mut options.autolink_literals,
                    "disallowed_raw_html" => &mut options.disallowed_raw_html,
                    "footnotes" => &mut options.footnotes,
                    "inline_footnotes" => &mut options.inline_footnotes,
                    "front_matter" => &mut options.front_matter,
                    "heading_ids" => &mut options.heading_ids,
                    "math" => &mut options.math,
                    "callouts" => &mut options.callouts,
                    "definition_lists" => &mut options.definition_lists,
                    "line_comments" => &mut options.line_comments,
                    "indented_code_blocks" => &mut options.indented_code_blocks,
                    _ => panic!("unknown option: {key}"),
                };
                *flag = value.as_bool().unwrap();
            }
        }
    }
    options
}

fn main() {
    let args: Vec<_> = std::env::args().collect();
    let mode = args.get(1).map_or("measure", String::as_str);
    assert!(matches!(mode, "measure" | "counts" | "html"));
    let catalog = std::fs::read_to_string(args.get(2).expect("catalog path")).unwrap();
    let filter = args.get(3).map_or("all", String::as_str);
    let lane_filter = args.get(4).map_or("all", String::as_str);
    let window = args.get(5).map_or(60, |s| s.parse::<u64>().unwrap());
    let rounds = if mode == "measure" {
        args.get(6).map_or(5, |s| s.parse::<usize>().unwrap())
    } else {
        1
    };
    let cases: Vec<Case> = serde_json::from_str(&catalog).unwrap();
    let cases: Vec<_> = cases
        .iter()
        .filter(|c| filter == "all" || c.id.contains(filter))
        .collect();
    let mut results = Vec::new();
    for round in 0..rounds {
        for case in &cases {
            let expected_control = ferromark::to_html_with_options(
                &case.variants[0].input,
                &options(&case.variants[0].options),
            );
            let order: Vec<_> = if round % 2 == 1 {
                (0..case.variants.len()).rev().collect()
            } else {
                (0..case.variants.len()).collect()
            };
            for index in order {
                let variant = &case.variants[index];
                let input = variant.input.as_str();
                let opts = options(&variant.options);
                let expected = ferromark::to_html_with_options(input, &opts);
                let mut row = json!({"id":case.id,"group":case.group,"feature":case.feature,"size":case.size,"variant":variant.label,"input_bytes":input.len(),"output_bytes":expected.len(),"same_html_as_control":expected==expected_control,"round":round});
                if mode == "html" {
                    row["html"] = json!(expected);
                    results.push(row);
                    continue;
                }
                for lane in ["owned", "renderer"] {
                    if lane_filter != "all" && lane_filter != lane {
                        continue;
                    }
                    let mut renderer = Renderer::with_options(opts.clone());
                    let mut out = Vec::with_capacity(input.len() * 2);
                    renderer.render_into(input, &mut out);
                    assert_eq!(out, expected.as_bytes());
                    let mut render = || {
                        if lane == "owned" {
                            black_box(ferromark::to_html_with_options(
                                black_box(input),
                                black_box(&opts),
                            ));
                        } else {
                            renderer.render_into(black_box(input), &mut out);
                            black_box(&out);
                        }
                    };
                    for _ in 0..16 {
                        render();
                    }
                    row["lane"] = json!(lane);
                    if mode == "measure" {
                        let start = Instant::now();
                        let mut n = 0u64;
                        loop {
                            for _ in 0..16 {
                                render();
                            }
                            n += 16;
                            if start.elapsed() >= Duration::from_millis(window) {
                                break;
                            }
                        }
                        row["ns"] = json!(start.elapsed().as_secs_f64() * 1e9 / n as f64);
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
                            row["allocation_calls"] = json!(allocation::CALLS.load(Relaxed));
                            row["requested_bytes"] = json!(allocation::BYTES.load(Relaxed));
                            row["inline_parses"] = json!(snap.inline_parses);
                            row["inline_fast_paths"] = json!(snap.inline_fast_paths);
                            row["paragraph_copied_bytes"] = json!(snap.paragraph_copied_bytes);
                        }
                        #[cfg(not(feature = "profiling"))]
                        panic!("counts require profiling feature");
                    }
                    results.push(row.clone());
                }
            }
        }
        eprintln!("round {}/{} complete", round + 1, rounds);
    }
    assert!(!results.is_empty(), "no cases matched");
    println!("{}", serde_json::to_string_pretty(&results).unwrap());
}
