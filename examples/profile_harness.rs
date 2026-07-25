//! Profiling harness: renders a fixture repeatedly under a named option preset.
//!
//! Usage: profile_harness <fixture-path> <preset> <iterations>
//!
//! Presets: minimal | commonmark | gfm | default | all | gfm-deflists |
//!          gfm-merged | gfm-widths | gfm-footnotes
//!
//! Designed for use under `valgrind --tool=callgrind` or `perf record`:
//!
//! ```bash
//! cargo build --profile release-debug --example profile_harness
//! valgrind --tool=callgrind \
//!   target/release-debug/examples/profile_harness benches/fixtures/commonmark-5k.md gfm 200
//! ```

use std::hint::black_box;

use ferromark::{Options, RenderPolicy};

fn all_extensions() -> Options {
    Options {
        render_policy: RenderPolicy::Untrusted,
        allow_html: true,
        allow_link_refs: true,
        tables: true,
        merged_table_cells: true,
        table_column_widths: true,
        strikethrough: true,
        highlight: true,
        superscript: true,
        subscript: true,
        task_lists: true,
        autolink_literals: true,
        disallowed_raw_html: true,
        footnotes: true,
        inline_footnotes: true,
        front_matter: true,
        heading_ids: true,
        math: true,
        callouts: true,
        definition_lists: true,
        line_comments: true,
        indented_code_blocks: true,
    }
}

fn preset(name: &str) -> Options {
    match name {
        "minimal" => Options::minimal(),
        "commonmark" => Options::commonmark(),
        "gfm" => Options::gfm(),
        "default" => Options::default(),
        "all" => all_extensions(),
        "gfm-deflists" => Options {
            definition_lists: true,
            ..Options::gfm()
        },
        "gfm-merged" => Options {
            merged_table_cells: true,
            ..Options::gfm()
        },
        "gfm-widths" => Options {
            table_column_widths: true,
            ..Options::gfm()
        },
        "gfm-footnotes" => Options {
            footnotes: true,
            inline_footnotes: true,
            ..Options::gfm()
        },
        "cm+tables" => Options {
            tables: true,
            ..Options::commonmark()
        },
        "cm+autolink" => Options {
            autolink_literals: true,
            ..Options::commonmark()
        },
        "cm+strike" => Options {
            strikethrough: true,
            ..Options::commonmark()
        },
        "cm+tasklists" => Options {
            task_lists: true,
            ..Options::commonmark()
        },
        "cm+disallowed" => Options {
            disallowed_raw_html: true,
            ..Options::commonmark()
        },
        "cm+headingids" => Options {
            heading_ids: true,
            ..Options::commonmark()
        },
        "cm+math" => Options {
            math: true,
            ..Options::commonmark()
        },
        "cm+highlight" => Options {
            highlight: true,
            ..Options::commonmark()
        },
        "cm+supsub" => Options {
            superscript: true,
            subscript: true,
            ..Options::commonmark()
        },
        "cm+callouts" => Options {
            callouts: true,
            ..Options::commonmark()
        },
        other => {
            eprintln!("unknown preset: {other}");
            std::process::exit(2);
        }
    }
}

fn main() {
    let mut args = std::env::args().skip(1);
    let (Some(path), Some(preset_name), Some(iters)) = (args.next(), args.next(), args.next())
    else {
        eprintln!("usage: profile_harness <fixture-path> <preset> <iterations>");
        std::process::exit(2);
    };
    let iterations: usize = iters.parse().expect("iterations must be a number");
    let input = std::fs::read_to_string(&path).expect("fixture must be readable");
    let options = preset(&preset_name);

    let mut out = Vec::with_capacity(input.len() * 2);
    let start = std::time::Instant::now();
    for _ in 0..iterations {
        out.clear();
        ferromark::to_html_into_with_options(black_box(&input), &mut out, &options);
        black_box(&out);
    }
    let elapsed = start.elapsed();
    let bytes = input.len() as f64 * iterations as f64;
    let mib_s = bytes / elapsed.as_secs_f64() / (1024.0 * 1024.0);
    eprintln!(
        "{path} preset={preset_name} iters={iterations} elapsed={elapsed:?} throughput={mib_s:.1} MiB/s out_len={}",
        out.len()
    );
}
