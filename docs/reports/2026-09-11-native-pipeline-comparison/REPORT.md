# Native Goldmark and Sätteri comparison

Measured Ferromark revision: `e352ecab4c443942f550e4d6eb0929ddb59c6a4e`.

Environment: Apple M1 Pro, macOS-26.6.2-arm64-arm-64bit-Mach-O. Each pair has an independent Ferromark baseline.

| Native pair | Input | Bytes | Ferromark µs | Competitor µs | Competitor / Ferromark | Run-median range, Ferromark / competitor µs |
| --- | --- | ---: | ---: | ---: | ---: | --- |
| goldmark-v1 | `commonmark/5k` | 5,120 | 25.53 | 169.74 | 6.65× | 24.95–25.96 / 168.64–171.41 |
| goldmark-v1 | `tables/tables-commonmark-inline` | 4,800 | 34.42 | 388.43 | 11.28× | 34.37–34.68 / 386.44–400.18 |
| goldmark-v1 | `gfm_overlap/features` | 5,050 | 38.16 | 446.46 | 11.70× | 37.91–38.33 / 439.38–455.33 |
| goldmark-v2 | `commonmark/5k` | 5,120 | 25.38 | 160.75 | 6.33× | 25.35–25.41 / 159.92–162.85 |
| goldmark-v2 | `tables/tables-commonmark-inline` | 4,800 | 34.36 | 397.43 | 11.57× | 34.32–34.41 / 394.35–401.50 |
| goldmark-v2 | `gfm_overlap/features` | 5,050 | 38.23 | 460.00 | 12.03× | 37.97–38.24 / 454.81–461.94 |
| satteri | `commonmark/5k` | 5,120 | 25.44 | 39.90 | 1.57× | 25.21–25.62 / 39.37–39.94 |
| satteri | `tables/tables-commonmark-inline` | 4,800 | 34.18 | 65.19 | 1.91× | 34.10–34.49 / 64.83–66.51 |
| satteri | `gfm_overlap/features` | 5,050 | 37.97 | 77.93 | 2.05× | 37.77–38.21 / 77.87–78.19 |

Smaller times are better. These are selected workload observations, not a general engine ranking. The ranges describe observed run medians, not confidence intervals.

## Protocol and native work

3 process runs per pair; 3000 ms warmup and 80 alternating-order windows of at least 63 ms per engine/case. Each worker times batches of 16 fresh parse/render calls using its native monotonic clock. Reported values are medians of process-run medians. Startup, fixture loading, IPC, JSON, validation, and memory-statistics reads are outside timing.

Rust uses its system allocator and destroys the arena/AST and owned HTML output inside each call. Goldmark uses fresh ASTs and bytes.Buffer outputs, reuses immutable parser/renderer configuration, and keeps normal automatic GC enabled: GOGC=100, GOMAXPROCS=1, no explicit GOMEMLIMIT or forced collections. Go GC during a window is timed; GC may also progress between windows. This measures steady-state render-window latency, not total process CPU or a forced full-collection cost per document. Allocation/GC counters are retained in every Go sample.

Sätteri runs its public parse-to-MDAST arena stage with position tracking, then mdast_to_html with its normal fused/fallback renderer. Only syntax flags differ from the convenience API's broad defaults; no parser, AST, position, or rendering stage is removed. Its native MDX capability is compiled in but disabled by the Markdown lane's runtime flags. No Node.js bindings, JavaScript plugins, WASM, cgo, or per-document CLI calls are used.

CommonMark disables extensions. Table lanes enable tables alone; gfm_overlap enables tables, strikethrough, and task lists. Trusted HTML and URL schemes are preserved. Bare autolinks, tag filtering, footnotes, math, and typography remain off. This overlap is not full GFM. Single-tilde dialect differences remain visible in the probes and are not patched away.

## Output and capability diagnostics

- **goldmark-v1:** 18/18 admitted workloads; 0 normalized mismatches against the 652 stored CommonMark examples. Excluded: none. Admitted renderer differences: none.
  Recorded Go GC cycles during sampled windows: 18,513.
- **goldmark-v2:** 18/18 admitted workloads; 0 normalized mismatches against the 652 stored CommonMark examples. Excluded: none. Admitted renderer differences: none.
  Recorded Go GC cycles during sampled windows: 21,285.
- **satteri:** 18/18 admitted workloads; 0 normalized mismatches against the 652 stored CommonMark examples. Excluded: none. Admitted renderer differences: task_lists/features, gfm_overlap/features.

Full original outputs, effective options, all eight feature-switch probes, and specification diagnostics are archived. Workload admission uses the shared ARCH-COMP-002 review; it is distinct from HTML fidelity and specification conformance.

## MDX boundary

The archive includes native results for Markdown-only MDX, root JSX, inline expressions, container JSX, and invalid JavaScript. Ferromark emits a JSX module after segmentation; Sätteri compiles MDX to JavaScript with OXC. These are different output stages and validation contracts, so no MDX throughput ratio is reported. The invalid-JavaScript probe and adapter test demonstrate the boundary without executing JavaScript.

## Reproduction

The adjacent metadata pins upstream revisions, dependency checksums, compiler versions, build flags, binary hashes, and local input hashes. The native harness README gives the build/run commands. The source patch records any uncommitted production changes at measurement time. These system-allocator measurements are separate from the published Bun/mimalloc figures.

Regenerate or verify this report from the repository root:

```sh
python3 benchmarks/native-pipeline-comparison/report.py docs/reports/2026-09-11-native-pipeline-comparison --check
```
