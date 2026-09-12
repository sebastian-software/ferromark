# Native Rushdown comparison

Ferromark revision: `8a9ad15c1a899f4e7c125a9b74f209863ffb5356`. Rushdown source: `e5eb4e4446541ea0ed53111c1b37e779283ff57c` (release `0.18.0`).

Environment: Apple M1 Pro, macOS-26.6.2-arm64-arm-64bit-Mach-O.

Native Parser::parse to a fresh arena, then html::Renderer::render to a fresh String; immutable parser/renderer configuration reused; full default html-entities support.

CommonMark plus independently selected table, double-tilde strikethrough and task-list workloads. Normal native single-tilde behavior is retained in diagnostics. No cached AST, alternate parser, Node binding or MDX compilation is measured.

| Input | Bytes | Ferromark µs | Rushdown µs | Competitor / Ferromark | Run-median ranges µs (Ferromark / competitor) |
| --- | ---: | ---: | ---: | ---: | --- |
| `commonmark/5k` | 5,120 | 23.87 | 56.65 | 2.37× | 23.73–23.91 / 56.12–56.67 |
| `tables/tables-commonmark-inline` | 4,800 | 32.58 | 96.78 | 2.97× | 32.52–32.60 / 96.49–96.90 |
| `gfm_overlap/features` | 5,050 | 36.74 | 109.04 | 2.97× | 36.46–36.77 / 108.26–109.10 |

3 fresh process runs; 3000 ms warmup per engine/case; 80 alternating windows of at least 63 ms. Each window checks its native monotonic timer after 16 calls. Values are medians of run medians, with observed ranges rather than confidence intervals. Smaller times are better; selected workloads are not a general engine ranking.

Fresh parse/render state and owned HTML are produced each time. Rust/Zig destruction is timed; managed runtimes retain automatic GC. Startup, configuration, IPC, JSON, fixture loading and output review are outside timing. No Node.js bindings, WASM or per-document CLI launch is used. These system-allocator results are separate from the published Bun/mimalloc comparison.

## Workload and capability evidence

Admitted workloads: 18/18. Excluded: none.

Admitted renderer differences: none.

Normalized mismatches against the 652 stored CommonMark examples: Ferromark 0; Rushdown 0. This is diagnostic evidence, not a conformance certification.

Original HTML, effective options, all eight feature-switch combinations, single-tilde and trusted-URL probes, and spec outputs are retained. Admission follows ARCH-COMP-002 independently of HTML fidelity. Fixed-dialect engines are excluded when the required switches cannot be matched; no parser source is patched to remove native behavior.

For configurable engines, CommonMark disables extensions; table/strike/task lanes enable only their named syntax; GFM overlap enables those three together. Raw HTML and arbitrary URL schemes are preserved. Bare autolinks, tag filtering, footnotes, math, heading IDs, typography and MDX stay off. No full-GFM or MDX-throughput claim is made.

## Reproduction

See the engine adapter README. Metadata records upstream hashes, compiler versions, build commands, dependency locks, local source hashes and executable hashes. The archive retains raw evidence; report generation rechecks every timed window and recomputes summaries.

```sh
python3 benchmarks/native-pipeline-comparison/report.py docs/reports/2026-09-12-ox-corpus-optimizations/rushdown --check
```
