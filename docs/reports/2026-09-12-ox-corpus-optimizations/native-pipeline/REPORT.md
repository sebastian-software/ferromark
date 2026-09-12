# Native Goldmark and Sätteri comparison

Measured Ferromark revision: `8a9ad15c1a899f4e7c125a9b74f209863ffb5356`.

Environment: Apple M1 Pro, macOS-26.6.2-arm64-arm-64bit-Mach-O. Each pair has an independent Ferromark baseline.

| Native pair | Input | Bytes | Ferromark µs | Competitor µs | Competitor / Ferromark | Run-median range, Ferromark / competitor µs |
| --- | --- | ---: | ---: | ---: | ---: | --- |
| Goldmark | `commonmark/5k` | 5,120 | 23.91 | 152.67 | 6.38× | 23.83–23.92 / 151.63–152.99 |
| Goldmark | `tables/tables-commonmark-inline` | 4,800 | 32.89 | 371.26 | 11.29× | 32.58–33.22 / 367.67–372.72 |
| Goldmark | `gfm_overlap/features` | 5,050 | 36.70 | 442.66 | 12.06× | 36.48–36.71 / 441.92–443.64 |
| Sätteri | `commonmark/5k` | 5,120 | 23.88 | 39.03 | 1.63× | 23.71–23.92 / 38.78–39.06 |
| Sätteri | `tables/tables-commonmark-inline` | 4,800 | 32.79 | 64.07 | 1.95× | 32.73–32.81 / 63.66–64.28 |
| Sätteri | `gfm_overlap/features` | 5,050 | 36.72 | 77.04 | 2.10× | 36.66–36.78 / 76.94–77.09 |

Smaller times are better. These are selected workload observations, not a general engine ranking. The ranges describe observed run medians, not confidence intervals.

## Protocol and native work

3 process runs per pair; 3000 ms warmup and 80 alternating-order windows of at least 63 ms per engine/case. Each worker times batches of 16 fresh parse/render calls using its native monotonic clock. Reported values are medians of process-run medians. Startup, fixture loading, IPC, JSON, validation, and memory-statistics reads are outside timing.

Rust uses its system allocator and destroys the arena/AST and owned HTML output inside each call. Goldmark uses fresh ASTs and bytes.Buffer outputs, reuses immutable parser/renderer configuration, and keeps normal automatic GC enabled: GOGC=100, GOMAXPROCS=1, no explicit GOMEMLIMIT or forced collections. Go GC during a window is timed; GC may also progress between windows. This measures steady-state render-window latency, not total process CPU or a forced full-collection cost per document. Allocation/GC counters are retained in every Go sample.

Sätteri runs its public parse-to-MDAST arena stage with position tracking, then mdast_to_html with its normal fused/fallback renderer. Only syntax flags differ from the convenience API's broad defaults; no parser, AST, position, or rendering stage is removed. Its native MDX capability is compiled in but disabled by the Markdown lane's runtime flags. No Node.js bindings, JavaScript plugins, WASM, cgo, or per-document CLI calls are used.

CommonMark disables extensions. Table lanes enable tables alone; gfm_overlap enables tables, strikethrough, and task lists. Trusted HTML and URL schemes are preserved. Bare autolinks, tag filtering, footnotes, math, and typography remain off. This overlap is not full GFM. Single-tilde dialect differences remain visible in the probes and are not patched away.

## Output and capability diagnostics

- **Goldmark:** 18/18 admitted workloads; 0 normalized mismatches against the 652 stored CommonMark examples. Excluded: none. Admitted renderer differences: none.
  Recorded Go GC cycles during sampled windows: 21,511.
- **Sätteri:** 18/18 admitted workloads; 0 normalized mismatches against the 652 stored CommonMark examples. Excluded: none. Admitted renderer differences: task_lists/features, gfm_overlap/features.

Full original outputs, effective options, all eight feature-switch probes, and specification diagnostics are archived. Workload admission uses the shared ARCH-COMP-002 review; it is distinct from HTML fidelity and specification conformance.

## MDX boundary

The archive includes native results for Markdown-only MDX, root JSX, inline expressions, container JSX, and invalid JavaScript. Ferromark emits a JSX module after segmentation; Sätteri compiles MDX to JavaScript with OXC. These are different output stages and validation contracts, so no MDX throughput ratio is reported. The invalid-JavaScript probe and adapter test demonstrate the boundary without executing JavaScript.

## Reproduction

The adjacent metadata pins upstream revisions, dependency checksums, compiler versions, build flags, binary hashes, and local input hashes. The native harness README gives the build/run commands. The source patch records any uncommitted production changes at measurement time. These system-allocator measurements are separate from the published Bun/mimalloc figures.

Regenerate or verify this report from the repository root:

```sh
python3 benchmarks/native-pipeline-comparison/report.py docs/reports/2026-09-12-ox-corpus-optimizations/native-pipeline --check
```
