# Native markdown-rs comparison

Ferromark revision: `8a9ad15c1a899f4e7c125a9b74f209863ffb5356`. markdown-rs source: `1506572f9b406431402928f3a8b3df0b4ae2d8f5` (release `1.0.0`).

Environment: Apple M1 Pro, macOS-26.6.2-arm64-arm-64bit-Mach-O.

Native markdown::to_html_with_options: fresh Markdown parsing to events and compilation to owned HTML; normal full entity support; no intermediate MDAST serialization.

Uses the stable release, not an alpha despite GitHub's release ordering. CommonMark and independently selected table/strikethrough/task constructs are measured. The public single-tilde switch is disabled to match Ferromark. MDX, math, footnotes, frontmatter, bare autolinks and tag filtering remain off. MDX parsing support is not presented as equivalent to full compilation.

| Input | Bytes | Ferromark µs | markdown-rs µs | Competitor / Ferromark | Run-median ranges µs (Ferromark / competitor) |
| --- | ---: | ---: | ---: | ---: | --- |
| `commonmark/5k` | 5,120 | 23.91 | 638.15 | 26.69× | 23.79–23.93 / 637.49–649.08 |
| `tables/tables-commonmark-inline` | 4,800 | 32.51 | 1810.61 | 55.69× | 32.50–32.65 / 1809.79–1816.26 |
| `gfm_overlap/features` | 5,050 | 36.78 | 1518.19 | 41.28× | 36.78–36.80 / 1517.73–1519.27 |

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
python3 benchmarks/native-pipeline-comparison/report.py docs/reports/2026-09-12-ox-corpus-optimizations/markdown-rs --check
```
