# Markdown feature costs and light documents — 2026-09-11

Production revision: `8adecc1d017e406eccbd582eb08992054af0411b`. Apple M1 Pro, macOS 26.6.2, Rust 1.97.1 / LLVM 22.1.6, release-debug, fat LTO, one codegen unit, apple-m1/NEON, System allocator.
Frozen catalog replay; no new parser optimization or CPU profile in this run.
The catalog contains 140 scenarios and 292 input/configuration pairs. Five
windows of at least 60 ms per variant and lifecycle, 16 warmup renders, with
alternating off/on order. Allocation counters run in a separate executable.
Fresh owned output includes parser construction and String destruction;
retained Renderer keeps parser buffers and output capacity between calls.

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

Values below describe this replay. Small differences around 3% should be treated as noise
unless a targeted longer run confirms them. These synthetic workloads describe
cost on one machine; they do not establish universal feature rankings.

## Option activation without matching syntax

Percent change in elapsed time; negative is faster. All activation pairs have
identical HTML. The inputs contain 17 bytes, about 1 KiB, or about 16 KiB of prose.

| Option | 17 B fresh owned | 1 KiB retained Renderer | 16 KiB retained Renderer |
| --- | ---: | ---: | ---: |
| allow_html | +0.1% | -0.0% | -0.2% |
| allow_link_refs | +0.0% | +1.5% | +0.3% |
| tables | +0.0% | +0.2% | -1.7% |
| merged_table_cells | +0.3% | +0.4% | +0.0% |
| table_column_widths | -0.4% | -0.0% | +0.3% |
| strikethrough | -0.4% | +0.2% | -0.5% |
| highlight | +0.5% | +4.9% | -1.5% |
| superscript | +0.1% | +3.4% | -1.4% |
| subscript | +0.4% | +0.6% | -0.7% |
| task_lists | -0.7% | -0.3% | +0.7% |
| autolink_literals | +1.3% | +9.1% | +3.6% |
| disallowed_raw_html | +0.2% | -0.1% | -0.1% |
| footnotes | +0.6% | -0.0% | -4.3% |
| inline_footnotes | +0.0% | +3.1% | -3.0% |
| front_matter | +0.0% | +0.3% | +1.4% |
| heading_ids | -0.0% | +0.1% | -5.5% |
| math | -0.3% | +0.1% | +1.7% |
| callouts | +0.3% | +0.1% | -1.2% |
| definition_lists | +0.3% | +2.0% | -1.0% |
| line_comments | +0.5% | -0.5% | -0.3% |
| indented_code_blocks | +0.4% | +0.5% | -1.7% |
| link_base_path | +6.2% | +2.0% | -6.6% |

## Features actually used

Small is one syntax unit; medium repeats it 16 times (except front matter).
Units differ in size and syntax density. The per-KiB column normalizes input
bytes, not output size or semantic work; the off/on percentage is a different
comparison and must not be used as a universal ranking.

| Feature | Small fresh owned (µs) | Medium retained (µs/KiB) | Medium off→on time | Medium retained allocation calls |
| --- | ---: | ---: | ---: | ---: |
| allow_html | 0.481 | 4.69 | -27.3% | 0 |
| allow_link_refs | 1.152 | 10.36 | +81.0% | 55 |
| tables | 0.762 | 8.27 | +47.3% | 0 |
| merged_table_cells | 0.722 | 8.01 | +1.7% | 0 |
| table_column_widths | 0.839 | 9.08 | +12.3% | 0 |
| strikethrough | 0.638 | 3.45 | +34.8% | 0 |
| highlight | 0.638 | 3.60 | +141.4% | 0 |
| superscript | 0.636 | 4.30 | +127.5% | 0 |
| subscript | 0.623 | 3.73 | +29.3% | 0 |
| task_lists | 0.714 | 6.90 | -25.4% | 2 |
| autolink_literals | 0.947 | 5.84 | +512.7% | 0 |
| disallowed_raw_html | 0.486 | 4.15 | +7.0% | 0 |
| footnotes | 1.525 | 11.22 | +59.8% | 74 |
| inline_footnotes | 1.395 | 11.65 | +151.7% | 19 |
| front_matter | 0.354 | 1.22 | -7.5% | 0 |
| heading_ids | 0.559 | 4.39 | +28.4% | 0 |
| math | 0.584 | 3.17 | +24.6% | 0 |
| callouts | 0.515 | 3.66 | -42.3% | 0 |
| definition_lists | 0.552 | 5.87 | +71.1% | 16 |
| line_comments | 0.353 | 1.77 | -42.0% | 0 |
| indented_code_blocks | 0.411 | 2.73 | -36.2% | 0 |
| link_base_path | 0.981 | 6.63 | +11.2% | 1 |

## CommonMark workload comparison

Medium repeats each syntax unit 32 times. The plain control is a single ASCII
run with exactly the same byte count. Differences include block count and output
expansion. Use these figures to choose profiling targets, not to sum feature costs.

| Construct | Small fresh owned (µs) | Medium retained (µs/KiB) | Equal-byte plain retained (µs/KiB) |
| --- | ---: | ---: | ---: |
| plain | 0.343 | 1.18 | 0.23 |
| headings | 0.481 | 5.90 | 0.29 |
| emphasis | 0.855 | 5.74 | 0.25 |
| inline_code | 0.603 | 3.49 | 0.25 |
| fenced_code | 0.479 | 4.25 | 0.26 |
| links | 0.938 | 5.63 | 0.23 |
| images | 0.807 | 3.66 | 0.23 |
| entities | 0.874 | 8.47 | 0.23 |
| unordered_lists | 0.767 | 8.63 | 0.25 |
| nested_lists | 0.817 | 8.98 | 0.25 |
| blockquotes | 0.632 | 4.18 | 0.24 |
| soft_breaks | 0.552 | 3.40 | 0.23 |

## Fixed per-call cost

| Input / preset | Fresh owned (µs) | Retained Renderer (µs) | Owned allocation calls |
| --- | ---: | ---: | ---: |
| empty / commonmark | 0.223 | 0.046 | 6 |
| empty / default | 0.221 | 0.045 | 6 |
| tiny / commonmark | 0.372 | 0.089 | 9 |
| tiny / default | 0.371 | 0.089 | 9 |
| plain-1k / commonmark | 1.514 | 1.217 | 8 |
| plain-1k / default | 1.514 | 1.220 | 8 |
| light-1k / commonmark | 7.185 | 6.267 | 27 |
| light-1k / default | 7.813 | 6.885 | 30 |
| plain-16k / commonmark | 20.711 | 19.790 | 8 |
| plain-16k / default | 20.302 | 18.756 | 8 |
| readme / commonmark | 117.057 | 107.224 | 73 |
| readme / default | 120.644 | 113.708 | 74 |

## Reproduction and evidence

The [2026-09-11-benchmark-refresh/supplemental/](2026-09-11-benchmark-refresh/supplemental/)
directory retains the frozen catalog, separate timing/counter outputs, exact
commands, source and executable hashes, and compiler details. Timing executables
were preserved before the profiling-feature counter build. Raw JSON may be gzip
compressed; decompress it before passing the catalog to the Rust probe.

```bash
feature-cost-probe measure catalog.json all all 60 5
feature-count-probe counts catalog.json all all
python3 scripts/summarize-markdown-features.py docs/reports/2026-09-11-benchmark-refresh/supplemental docs/reports/2026-09-11-markdown-feature-costs.md \
  --snapshot final --revision 8adecc1d017e406eccbd582eb08992054af0411b \
  --replay-metadata docs/reports/2026-09-11-benchmark-refresh/supplemental/metadata.json
```

The executable paths and build commands are recorded in the replay metadata.
Requested bytes are cumulative, not peak/live memory. These are short internal
feature diagnostics, not the longer repeated five-parser publication protocol.
Historical profiles and before/after experiments remain in the
[optimization report](2026-09-10-markdown-feature-optimizations.md).
