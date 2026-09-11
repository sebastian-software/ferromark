# Native Ox Content comparison

Ferromark revision: `e7c79ab88d905d8fb7a3c6ce8fdc5f6ced91cc09`. Ox Content source: `5c97078779cf099245a78aacca8fa7cc5e993316` (release `3.2.0`).

Environment: Apple M1 Pro, macOS-26.6.2-arm64-arm-64bit-Mach-O.

Native Parser::parse in a fresh allocator arena, then HtmlRenderer::render to moved owned HTML; immutable configuration and the normal renderer scratch storage are reused, but no arena, AST or output buffer is reused.

Parser syntax switches are independent. Renderer autolinking and external-link target rewriting are disabled, while native generated heading IDs are retained. Workloads differing through heading IDs remain excluded by the existing output review. MDX is disabled at runtime; no NAPI, MDAST serialization, incremental update, borrowed HTML output or site generation is timed.

| Input | Bytes | Ferromark µs | Ox Content µs | Competitor / Ferromark | Run-median ranges µs (Ferromark / competitor) |
| --- | ---: | ---: | ---: | ---: | --- |
| `commonmark/short` | 18 | 0.49 | 0.29 | 0.59× | 0.49–0.49 / 0.29–0.29 |
| `tables/tables-commonmark-inline` | 4,800 | 32.90 | 37.25 | 1.13× | 32.79–32.99 / 37.04–37.33 |
| `gfm_overlap/features` | 5,050 | 37.04 | 35.21 | 0.95× | 36.66–37.14 / 34.79–35.32 |

3 fresh process runs; 3000 ms warmup per engine/case; 80 alternating windows of at least 63 ms. Each window checks its native monotonic timer after 16 calls. Values are medians of run medians, with observed ranges rather than confidence intervals. Smaller times are better; selected workloads are not a general engine ranking.

Fresh parse/render state and owned HTML are produced each time. Rust/Zig destruction is timed; managed runtimes retain automatic GC. Startup, configuration, IPC, JSON, fixture loading and output review are outside timing. No Node.js bindings, WASM or per-document CLI launch is used. These system-allocator results are separate from the published Bun/mimalloc comparison.

## Workload and capability evidence

Admitted workloads: 10/18. Excluded: commonmark/2k, commonmark/5k, commonmark/10k, commonmark/50k, tables/neutral, strikethrough/neutral, task_lists/neutral, gfm_overlap/neutral.

Admitted renderer differences: task_lists/features.

Normalized mismatches against the 652 stored CommonMark examples: Ferromark 0; Ox Content 52. This is diagnostic evidence, not a conformance certification.

Original HTML, effective options, all eight feature-switch combinations, single-tilde and trusted-URL probes, and spec outputs are retained. Admission follows ARCH-COMP-002 independently of HTML fidelity. Fixed-dialect engines are excluded when the required switches cannot be matched; no parser source is patched to remove native behavior.

CommonMark parser options disable extensions; table/strike/task lanes enable only their named syntax, and GFM overlap enables those three together. Raw HTML and arbitrary URL schemes are preserved. Bare autolinks, tag filtering, footnotes, math, typography, heading attributes and MDX are off. The native HTML renderer still generates heading IDs; affected workloads are excluded without changing the shared output projection. No full-GFM or MDX-throughput claim is made.

## Reproduction

See the engine adapter README. Metadata records upstream hashes, compiler versions, build commands, dependency locks, local source hashes and executable hashes. The archive retains raw evidence; report generation rechecks every timed window and recomputes summaries.

```sh
python3 benchmarks/native-pipeline-comparison/report.py docs/reports/2026-09-11-native-ox-content --check
```
