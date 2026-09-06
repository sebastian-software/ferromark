//! Paired-executable screening for core Markdown optimizations.
//! Usage: core_performance [window-ms] [case-filter] [preset-filter]
//! Each lane checks identical output before timing. Keep baseline and candidate
//! executables and alternate their order; these windows are not Criterion CIs.
use ferromark::{Options, Renderer};
use std::{
    hint::black_box,
    time::{Duration, Instant},
};

fn main() {
    let args: Vec<_> = std::env::args().collect();
    let window = Duration::from_millis(args.get(1).map_or(50, |s| s.parse().unwrap()));
    let filter = args.get(2).map_or("", String::as_str);
    let preset_filter = args.get(3).map_or("", String::as_str);
    let cases = [
        ("tiny", "Hello, **world**!".into()),
        ("prose", "Ordinary words with no special syntax and enough text for several vector loads. ".repeat(64)),
        ("emphasis", "Some **strong** and *emphasized* words with `inline code`.\n\n".repeat(96)),
        ("links", "Visit [the guide](https://example.com/guide) and <https://example.org>.\n\n".repeat(96)),
        ("entities", "Words &amp; &#0; &ngE; &NotEqualTilde; &unknown; and \"quotes\".\n\n".repeat(64)),
        ("late_special", format!("{}**end**\n", "ordinary words ".repeat(256))),
        ("references", "[the guide][guide] and ![image][picture]\n\n[guide]: /guide \"Title\"\n[picture]: /image.png\n".repeat(64)),
        ("lists", "- first item\n- second **item**\n  - nested item\n\n".repeat(96)),
        ("commonmark_5k", include_str!("../benches/fixtures/commonmark-5k.md").into()),
        ("commonmark_20k", include_str!("../benches/fixtures/commonmark-20k.md").into()),
        ("commonmark_50k", include_str!("../benches/fixtures/commonmark-50k.md").into()),
        ("commonmark_1m", include_str!("../benches/fixtures/commonmark-1m.md").into()),
        ("tables", include_str!("../benches/fixtures/tables-5k.md").into()),
    ];
    let mut results = Vec::new();
    for (name, input) in cases {
        if !name.contains(filter) {
            continue;
        }
        for (preset, options) in [
            ("commonmark", Options::commonmark()),
            ("default", Options::default()),
            ("gfm", Options::gfm()),
        ] {
            if !preset.contains(preset_filter) {
                continue;
            }
            let expected = ferromark::to_html_with_options(&input, &options);
            let mut renderer = Renderer::with_options(options.clone());
            let mut out = Vec::with_capacity(input.len() * 2);
            renderer.render_into(&input, &mut out);
            assert_eq!(out, expected.as_bytes());
            for lane in ["fresh", "reused"] {
                let mut samples = Vec::new();
                for _ in 0..3 {
                    let start = Instant::now();
                    let mut n = 0u64;
                    loop {
                        // Amortize clock overhead on small documents.
                        for _ in 0..16 {
                            if lane == "fresh" {
                                black_box(ferromark::to_html_with_options(
                                    black_box(&input),
                                    black_box(&options),
                                ));
                            } else {
                                renderer.render_into(black_box(&input), &mut out);
                                black_box(&out);
                            }
                        }
                        n += 16;
                        if start.elapsed() >= window {
                            break;
                        }
                    }
                    samples.push(start.elapsed().as_nanos() as f64 / n as f64);
                }
                samples.sort_by(f64::total_cmp);
                results.push(
                    serde_json::json!({"case": name, "preset": preset, "lane": lane,
                    "input_bytes": input.len(), "ns": samples[1], "samples_ns": samples,
                    "html": expected}),
                );
            }
        }
    }
    println!("{}", serde_json::to_string(&results).unwrap());
}
