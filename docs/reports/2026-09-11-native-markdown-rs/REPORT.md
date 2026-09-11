# Native markdown-rs comparison

Ferromark revision: `5ee645dea61c8fbf36ac36a6f5cb1bf1c4795e4c`. markdown-rs source: `1506572f9b406431402928f3a8b3df0b4ae2d8f5` (release `1.0.0`).

Environment: Apple M1 Pro, macOS-26.6.2-arm64-arm-64bit-Mach-O.

Native markdown::to_html_with_options: fresh Markdown parsing to events and compilation to owned HTML; normal full entity support; no intermediate MDAST serialization.

Uses the stable release, not an alpha despite GitHub's release ordering. CommonMark and independently selected table/strikethrough/task constructs are measured. The public single-tilde switch is disabled to match Ferromark. MDX, math, footnotes, frontmatter, bare autolinks and tag filtering remain off. MDX parsing support is not presented as equivalent to full compilation.

| Input | Bytes | Ferromark µs | markdown-rs µs | Competitor / Ferromark | Run-median ranges µs (Ferromark / competitor) |
| --- | ---: | ---: | ---: | ---: | --- |
| `commonmark/5k` | 5,120 | 25.19 | 652.56 | 25.91× | 25.16–25.26 / 651.95–652.69 |
| `tables/tables-commonmark-inline` | 4,800 | 33.37 | 1825.71 | 54.71× | 33.27–33.42 / 1821.63–1830.01 |
| `gfm_overlap/features` | 5,050 | 37.43 | 1541.90 | 41.20× | 37.34–37.47 / 1533.39–1543.71 |

3 fresh process runs; 3000 ms warmup per engine/case; 80 alternating windows of at least 63 ms. Each window checks its native monotonic timer after 16 calls. Values are medians of run medians, with observed ranges rather than confidence intervals. Smaller times are better; selected workloads are not a general engine ranking.

Fresh parse/render state and owned HTML are produced each time. Rust/Zig destruction is timed; managed runtimes retain automatic GC. Startup, configuration, IPC, JSON, fixture loading and output review are outside timing. No Node.js bindings, WASM or per-document CLI launch is used. These system-allocator results are separate from the published Bun/mimalloc comparison.

## Workload and capability evidence

Admitted workloads: 18/18. Excluded: none.

Admitted renderer differences: none.

Normalized mismatches against the 652 stored CommonMark examples: Ferromark 0; markdown-rs 0. This is diagnostic evidence, not a conformance certification.

Original HTML, effective options, all eight feature-switch combinations, single-tilde and trusted-URL probes, and spec outputs are retained. Admission follows ARCH-COMP-002 independently of HTML fidelity. Fixed-dialect engines are excluded when the required switches cannot be matched; no parser source is patched to remove native behavior.

For configurable engines, CommonMark disables extensions; table/strike/task lanes enable only their named syntax; GFM overlap enables those three together. Raw HTML and arbitrary URL schemes are preserved. Bare autolinks, tag filtering, footnotes, math, heading IDs, typography and MDX stay off. No full-GFM or MDX-throughput claim is made.

## Reproduction

See the engine adapter README. Metadata records upstream hashes, compiler versions, build commands, dependency locks, local source hashes and executable hashes. The archive retains raw evidence; report generation rechecks every timed window and recomputes summaries.

```sh
python3 benchmarks/native-pipeline-comparison/report.py docs/reports/2026-09-11-native-markdown-rs --check
```
