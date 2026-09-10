# Markdown feature costs and light documents — 2026-09-10

Baseline production: `99fbf0107c8a08851ba48d6173da99003c5ca22f`, including the
previous GFM optimizations. Apple M1 Pro, macOS 26.6.2, Rust 1.97.1 / LLVM 22.1.6,
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

Values below describe this baseline. Follow-up optimizations and measurements
are recorded separately. Small differences around 3% should be treated as noise
unless a targeted longer run confirms them. These synthetic workloads describe
cost on one machine; they do not establish universal feature rankings.

## Option activation without matching syntax

Percent change in elapsed time; negative is faster. All activation pairs have
identical HTML. The inputs contain 17 bytes, about 1 KiB, or about 16 KiB of prose.

| Option | 17 B fresh owned | 1 KiB retained Renderer | 16 KiB retained Renderer |
| --- | ---: | ---: | ---: |
| allow_html | -0.1% | -0.4% | +1.6% |
| allow_link_refs | -0.1% | +1.3% | +0.6% |
| tables | +0.3% | -0.0% | -0.9% |
| merged_table_cells | +0.0% | +0.9% | -0.3% |
| table_column_widths | -0.5% | +0.2% | -1.8% |
| strikethrough | -0.1% | +0.4% | +0.1% |
| highlight | +0.4% | -2.0% | -4.6% |
| superscript | +0.4% | -3.9% | -6.3% |
| subscript | -0.1% | -0.5% | -1.1% |
| task_lists | +0.2% | +1.0% | -1.0% |
| autolink_literals | +1.4% | +6.7% | +5.1% |
| disallowed_raw_html | -0.4% | +0.5% | -0.0% |
| footnotes | -0.0% | +0.0% | -0.1% |
| inline_footnotes | -0.6% | +0.5% | -1.9% |
| front_matter | +0.6% | -0.1% | +1.1% |
| heading_ids | +6.3% | +0.6% | -1.2% |
| math | +0.6% | -0.6% | -0.0% |
| callouts | +0.2% | -0.5% | +3.5% |
| definition_lists | -0.0% | +2.7% | +2.3% |
| line_comments | +0.2% | +0.1% | -1.9% |
| indented_code_blocks | +0.3% | -0.1% | +1.7% |
| link_base_path | +2.4% | -0.0% | -2.8% |

## Features actually used

Small is one syntax unit; medium repeats it 16 times (except front matter).
Units differ in size and syntax density. The per-KiB column normalizes input
bytes, not output size or semantic work; the off/on percentage is a different
comparison and must not be used as a universal ranking.

| Feature | Small fresh owned (µs) | Medium retained (µs/KiB) | Medium off→on time | Medium retained allocation calls |
| --- | ---: | ---: | ---: | ---: |
| allow_html | 1.170 | 4.74 | -29.5% | 0 |
| allow_link_refs | 1.501 | 10.38 | +77.7% | 55 |
| tables | 1.459 | 8.22 | +44.2% | 0 |
| merged_table_cells | 1.417 | 7.98 | +0.8% | 0 |
| table_column_widths | 1.517 | 8.98 | +11.8% | 0 |
| strikethrough | 1.197 | 3.51 | +42.5% | 0 |
| highlight | 1.213 | 4.24 | +180.7% | 16 |
| superscript | 1.217 | 4.89 | +189.3% | 16 |
| subscript | 1.202 | 4.52 | +57.5% | 16 |
| task_lists | 1.398 | 6.79 | -27.1% | 2 |
| autolink_literals | 1.513 | 5.72 | +524.0% | 0 |
| disallowed_raw_html | 1.170 | 4.11 | +6.0% | 0 |
| footnotes | 2.102 | 12.22 | +73.5% | 106 |
| inline_footnotes | 2.106 | 15.54 | +226.3% | 131 |
| front_matter | 1.039 | 1.25 | -5.7% | 0 |
| heading_ids | 1.238 | 4.33 | +26.8% | 0 |
| math | 1.173 | 3.66 | +53.9% | 16 |
| callouts | 1.204 | 3.66 | -44.7% | 0 |
| definition_lists | 1.247 | 5.88 | +62.8% | 16 |
| line_comments | 1.027 | 1.76 | -45.2% | 0 |
| indented_code_blocks | 1.092 | 2.67 | -44.3% | 0 |
| link_base_path | 1.401 | 6.65 | +11.4% | 1 |

## CommonMark workload comparison

Medium repeats each syntax unit 32 times. The plain control is a single ASCII
run with exactly the same byte count. Differences include block count and output
expansion. Use these figures to choose profiling targets, not to sum feature costs.

| Construct | Small fresh owned (µs) | Medium retained (µs/KiB) | Equal-byte plain retained (µs/KiB) |
| --- | ---: | ---: | ---: |
| plain | 1.028 | 1.22 | 0.23 |
| headings | 1.164 | 5.86 | 0.28 |
| emphasis | 1.407 | 5.53 | 0.24 |
| inline_code | 1.160 | 3.50 | 0.24 |
| fenced_code | 1.155 | 4.19 | 0.26 |
| links | 1.382 | 5.67 | 0.23 |
| images | 1.250 | 3.75 | 0.23 |
| entities | 1.578 | 8.48 | 0.23 |
| unordered_lists | 1.454 | 8.79 | 0.25 |
| nested_lists | 1.494 | 9.08 | 0.25 |
| blockquotes | 1.218 | 4.51 | 0.24 |
| soft_breaks | 1.132 | 3.61 | 0.23 |

## Fixed per-call cost

| Input / preset | Fresh owned (µs) | Retained Renderer (µs) | Owned allocation calls |
| --- | ---: | ---: | ---: |
| empty / commonmark | 0.906 | 0.046 | 37 |
| empty / default | 0.953 | 0.044 | 40 |
| tiny / commonmark | 1.055 | 0.089 | 40 |
| tiny / default | 1.113 | 0.088 | 43 |
| plain-1k / commonmark | 2.122 | 1.262 | 39 |
| plain-1k / default | 2.173 | 1.251 | 42 |
| light-1k / commonmark | 7.774 | 6.314 | 53 |
| light-1k / default | 8.372 | 6.884 | 56 |
| plain-16k / commonmark | 19.983 | 19.872 | 39 |
| plain-16k / default | 20.107 | 19.966 | 42 |
| readme / commonmark | 117.214 | 108.679 | 85 |
| readme / default | 123.660 | 111.725 | 87 |

## Reproduction and evidence

The adjacent directory preserves the full catalog, all timing windows, allocation
counts, and four CPU profiles. `scripts/markdown-feature-catalog.py` recreates the
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
