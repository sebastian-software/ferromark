# Native Markdig comparison

Ferromark revision: `5ee645dea61c8fbf36ac36a6f5cb1bf1c4795e4c`. Markdig source: `fc705234fa211d179ee1d5e7656b51ab99f70ca9` (release `1.3.2`).

Environment: Apple M1 Pro, macOS-26.6.2-arm64-arm-64bit-Mach-O.

Native .NET Markdown.ToHtml with immutable pipeline configuration; complete parse/render to a fresh owned HTML string; normal workstation concurrent GC and tiered JIT/PGO enabled.

This is a warmed .NET engine-core comparison, not AOT and not a Node wrapper. GC within a render window is timed; GC may also progress between windows. Collection/allocation counters are retained outside timing. Startup, fixture decoding and JIT warmup are excluded; timings measure steady-state render-window latency, not total process CPU.

| Input | Bytes | Ferromark µs | Markdig µs | Competitor / Ferromark | Run-median ranges µs (Ferromark / competitor) |
| --- | ---: | ---: | ---: | ---: | --- |
| `commonmark/5k` | 5,120 | 25.01 | 63.76 | 2.55× | 24.91–25.57 / 62.76–64.22 |
| `tables/tables-commonmark-inline` | 4,800 | 32.97 | 246.87 | 7.49× | 32.93–33.10 / 243.91–250.74 |
| `gfm_overlap/features` | 5,050 | 37.13 | 248.48 | 6.69× | 37.12–37.28 / 245.58–250.98 |

3 fresh process runs; 3000 ms warmup per engine/case; 80 alternating windows of at least 63 ms. Each window checks its native monotonic timer after 16 calls. Values are medians of run medians, with observed ranges rather than confidence intervals. Smaller times are better; selected workloads are not a general engine ranking.

Fresh parse/render state and owned HTML are produced each time. Rust/Zig destruction is timed; managed runtimes retain automatic GC. Startup, configuration, IPC, JSON, fixture loading and output review are outside timing. No Node.js bindings, WASM or per-document CLI launch is used. These system-allocator results are separate from the published Bun/mimalloc comparison.

## Workload and capability evidence

Admitted workloads: 18/18. Excluded: none.

Admitted renderer differences: task_lists/features, gfm_overlap/features.

Normalized mismatches against the 652 stored CommonMark examples: Ferromark 0; Markdig 0. This is diagnostic evidence, not a conformance certification.

Original HTML, effective options, all eight feature-switch combinations, single-tilde and trusted-URL probes, and spec outputs are retained. Admission follows ARCH-COMP-002 independently of HTML fidelity. Fixed-dialect engines are excluded when the required switches cannot be matched; no parser source is patched to remove native behavior.

For configurable engines, CommonMark disables extensions; table/strike/task lanes enable only their named syntax; GFM overlap enables those three together. Raw HTML and arbitrary URL schemes are preserved. Bare autolinks, tag filtering, footnotes, math, heading IDs, typography and MDX stay off. No full-GFM or MDX-throughput claim is made.

## Reproduction

See the engine adapter README. Metadata records upstream hashes, compiler versions, build commands, dependency locks, local source hashes and executable hashes. The archive retains raw evidence; report generation rechecks every timed window and recomputes summaries.

```sh
python3 benchmarks/native-pipeline-comparison/report.py docs/reports/2026-09-11-native-markdig --check
```
