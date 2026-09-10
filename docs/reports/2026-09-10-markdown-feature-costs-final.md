# Final Markdown feature costs and light documents — 2026-09-10

Final production: `41d5201c804e760f400a2d25e2ebe06394ec6f04`, including the
measured lightweight-document and feature optimizations. Apple M1 Pro, macOS 26.6.2, Rust 1.97.1 / LLVM 22.1.6,
release-debug, fat LTO, one codegen unit, apple-m1/NEON, System allocator.

The catalog has 140 scenarios and 292 input/configuration pairs. Five >=60 ms
windows per variant and API lifecycle, 16 warmup renders, alternating off/on
order. Allocation counters run in a separate profiling-feature executable.
A retained Renderer also retains output capacity; the owned lane constructs
fresh parser state and returns/drops a fresh String inside timing. These API
lifecycles answer different questions. CPU samples use the fresh-parser,
reused-output profile harness and are not phase-accurate timing percentages.

## Reading the results

- **Activation:** same plain input and byte-identical HTML, with one option off/on.
  This isolates detection/setup overhead without using the feature.
- **Syntax:** same input, feature off/on, intentionally different output. This is
  the total cost of interpreting that syntax, not avoidable overhead. Enabling
  comments, metadata removal, task lists, or code parsing can remove work.
- **Core constructs:** Markdown versus equally many ASCII `a` bytes. CommonMark
  constructs cannot all be switched off. These are workload comparisons with
  different semantics, structure, and output volume, not additive feature prices.

All cases use Untrusted rendering except the disallowed-HTML filter, which needs
Trusted rendering to exercise its contract; that policy is fixed across its pair.
Merged cells and width hints enable tables in both variants. Front matter is only
recognized at the document start. Footnote/reference labels are unique per unit.
The catalog covers all 21 boolean Markdown options and link-base rewriting;
MDX and custom code-renderer callbacks are outside this Markdown study.

Values below describe the final optimized implementation. See the
[optimization log](2026-09-10-markdown-feature-optimizations.md) for paired
before/after comparisons and the original baseline study for earlier costs. Small differences around 3% should be treated as noise
unless a targeted longer run confirms them. These synthetic workloads describe
cost on one machine; they do not establish universal feature rankings.

## Option activation without matching syntax

Percent change in elapsed time; negative is faster. All activation pairs have
identical HTML. The inputs contain 17 bytes, about 1 KiB, or about 16 KiB of prose.

| Option | 17 B fresh owned | 1 KiB retained Renderer | 16 KiB retained Renderer |
| --- | ---: | ---: | ---: |
| allow_html | -0.3% | -0.4% | +0.2% |
| allow_link_refs | +0.1% | +1.2% | -0.8% |
| tables | +0.3% | -0.1% | +0.2% |
| merged_table_cells | -0.4% | +0.1% | +3.3% |
| table_column_widths | +0.8% | -0.3% | -0.8% |
| strikethrough | +0.1% | -0.4% | -0.7% |
| highlight | +0.3% | +2.7% | -3.2% |
| superscript | -0.6% | +2.9% | -3.6% |
| subscript | -0.1% | -0.3% | +1.2% |
| task_lists | -0.5% | -0.1% | -0.0% |
| autolink_literals | +1.4% | +8.2% | +3.1% |
| disallowed_raw_html | -0.1% | -0.1% | +1.2% |
| footnotes | +0.6% | -0.5% | -4.6% |
| inline_footnotes | -0.3% | +1.9% | -4.7% |
| front_matter | +0.4% | -0.7% | +0.5% |
| heading_ids | -0.0% | -0.4% | -5.4% |
| math | +0.4% | -0.0% | -2.1% |
| callouts | +0.3% | +0.1% | +1.6% |
| definition_lists | +0.2% | +1.9% | -0.8% |
| line_comments | -0.2% | +0.1% | +0.7% |
| indented_code_blocks | +0.3% | +0.2% | -0.2% |
| link_base_path | +6.3% | +2.0% | -6.6% |

## Features actually used

Small is one syntax unit; medium repeats it 16 times (except front matter).
Units differ in size and syntax density. The per-KiB column normalizes input
bytes, not output size or semantic work; the off/on percentage is a different
comparison and must not be used as a universal ranking.

| Feature | Small fresh owned (µs) | Medium retained (µs/KiB) | Medium off→on time | Medium retained allocation calls |
| --- | ---: | ---: | ---: | ---: |
| allow_html | 0.482 | 4.76 | -28.2% | 0 |
| allow_link_refs | 1.167 | 10.54 | +79.4% | 55 |
| tables | 0.777 | 8.39 | +47.1% | 0 |
| merged_table_cells | 0.734 | 8.08 | +1.0% | 0 |
| table_column_widths | 0.843 | 9.16 | +12.5% | 0 |
| strikethrough | 0.645 | 3.46 | +30.1% | 0 |
| highlight | 0.644 | 3.62 | +142.6% | 0 |
| superscript | 0.640 | 4.35 | +128.7% | 0 |
| subscript | 0.627 | 3.76 | +29.8% | 0 |
| task_lists | 0.711 | 6.96 | -25.0% | 2 |
| autolink_literals | 0.943 | 5.85 | +510.5% | 0 |
| disallowed_raw_html | 0.487 | 4.19 | +7.8% | 0 |
| footnotes | 1.533 | 11.30 | +59.8% | 74 |
| inline_footnotes | 1.407 | 11.67 | +148.7% | 19 |
| front_matter | 0.355 | 1.24 | -7.8% | 0 |
| heading_ids | 0.554 | 4.45 | +28.7% | 0 |
| math | 0.590 | 3.25 | +26.0% | 0 |
| callouts | 0.517 | 3.70 | -42.4% | 0 |
| definition_lists | 0.557 | 5.90 | +70.5% | 16 |
| line_comments | 0.354 | 1.80 | -44.7% | 0 |
| indented_code_blocks | 0.405 | 2.76 | -36.3% | 0 |
| link_base_path | 0.991 | 6.74 | +11.9% | 1 |

## CommonMark workload comparison

Medium repeats each syntax unit 32 times. The plain control is a single ASCII
run with exactly the same byte count. Differences include block count and output
expansion. Use these figures to choose profiling targets, not to sum feature costs.

| Construct | Small fresh owned (µs) | Medium retained (µs/KiB) | Equal-byte plain retained (µs/KiB) |
| --- | ---: | ---: | ---: |
| plain | 0.345 | 1.19 | 0.23 |
| headings | 0.482 | 6.04 | 0.29 |
| emphasis | 0.865 | 5.78 | 0.25 |
| inline_code | 0.605 | 3.68 | 0.25 |
| fenced_code | 0.483 | 4.35 | 0.26 |
| links | 0.944 | 5.72 | 0.24 |
| images | 0.816 | 3.70 | 0.24 |
| entities | 0.867 | 8.63 | 0.23 |
| unordered_lists | 0.772 | 8.75 | 0.26 |
| nested_lists | 0.821 | 9.06 | 0.25 |
| blockquotes | 0.633 | 4.23 | 0.24 |
| soft_breaks | 0.559 | 3.50 | 0.23 |

## Fixed per-call cost

| Input / preset | Fresh owned (µs) | Retained Renderer (µs) | Owned allocation calls |
| --- | ---: | ---: | ---: |
| empty / commonmark | 0.221 | 0.046 | 6 |
| empty / default | 0.220 | 0.045 | 6 |
| tiny / commonmark | 0.372 | 0.090 | 9 |
| tiny / default | 0.371 | 0.089 | 9 |
| plain-1k / commonmark | 1.511 | 1.230 | 8 |
| plain-1k / default | 1.512 | 1.232 | 8 |
| light-1k / commonmark | 7.250 | 6.352 | 27 |
| light-1k / default | 7.945 | 6.945 | 30 |
| plain-16k / commonmark | 20.292 | 20.233 | 8 |
| plain-16k / default | 20.325 | 18.972 | 8 |
| readme / commonmark | 117.552 | 110.073 | 73 |
| readme / default | 125.051 | 112.606 | 74 |

## Reproduction and evidence

The `2026-09-10-markdown-feature-costs/` directory preserves the full catalog,
final timing windows and allocation counts, plus four baseline CPU profiles. `scripts/markdown-feature-catalog.py` recreates the
catalog; `examples/feature_cost_probe.rs` measures it. For example:

```sh
python3 scripts/markdown-feature-catalog.py target/feature-costs/catalog.json
cargo run --locked --profile release-debug --example feature_cost_probe -- measure target/feature-costs/catalog.json
cargo run --locked --profile release-debug --features profiling --example feature_cost_probe -- counts target/feature-costs/catalog.json
```

Always preserve an uninstrumented binary before building allocation counters.
`compare-feature-probes.py` requires exact HTML for all 292 pairs before timing
before/after binaries. Requested bytes are cumulative, not peak/live memory.
The report tables are generated by `scripts/summarize-markdown-features.py`.
