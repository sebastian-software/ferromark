# Ferromark main vs. the initial v2 core

Measured on 2026-09-13T21:14:59Z on Apple M1 Pro.

The rotating CommonMark mix is effectively tied. The table-heavy GFM mix favors current main; v2 wins several plain-text, fenced-code, and escaping diagnostics. This is a workload-dependent baseline, not a universal ranking.

## Rotating document groups

Times are microseconds per document. Ratios are paired main-time/v2-time medians: above 1 means v2 is faster. Each group visits every document once per traversal; inputs have distinct contents and allocations.

| Group | Lifecycle | Main µs | v2 µs | v2 speed ratio | Paired range |
| --- | --- | ---: | ---: | ---: | --- |
| rotating-commonmark (11 docs) | fresh | 52.595 | 51.773 | 1.017× | 1.012–1.031× |
| rotating-commonmark (11 docs) | owned | 51.053 | 50.864 | 1.008× | 0.992–1.021× |
| rotating-commonmark (11 docs) | reuse | 50.160 | 50.768 | 0.989× | 0.980–0.998× |
| rotating-gfm (4 docs) | fresh | 25.712 | 31.554 | 0.813× | 0.809–0.826× |
| rotating-gfm (4 docs) | owned | 24.434 | 30.978 | 0.788× | 0.781–0.794× |
| rotating-gfm (4 docs) | reuse | 23.765 | 30.590 | 0.774× | 0.766–0.786× |

## Individual cases

Each cell is main µs / v2 µs, followed by the paired v2 speed ratio. The README has different heading IDs and is diagnostic only; it is excluded from both rotating groups.

| Case | Input bytes | Output check | Fresh | Owned output | Full reuse |
| --- | ---: | --- | --- | --- | --- |
| tiny | 17 | exact | 0.520 / 0.525 (0.99×) | 0.224 / 0.222 (1.01×) | 0.187 / 0.145 (1.28×) |
| short-authored-prose | 140 | exact | 1.560 / 1.077 (1.45×) | 0.931 / 0.728 (1.28×) | 0.820 / 0.695 (1.18×) |
| plain-text-10k | 10,000 | exact | 9.093 / 5.937 (1.53×) | 8.817 / 5.559 (1.58×) | 8.437 / 5.476 (1.54×) |
| link-heavy | 10,000 | exact | 46.067 / 43.239 (1.06×) | 45.442 / 42.435 (1.06×) | 44.763 / 42.549 (1.05×) |
| nested-lists | 10,000 | serialization-equivalent | 83.218 / 131.106 (0.63×) | 81.608 / 130.568 (0.63×) | 81.087 / 131.109 (0.62×) |
| fenced-code | 10,000 | exact | 30.068 / 19.483 (1.54×) | 29.362 / 19.114 (1.54×) | 28.777 / 19.099 (1.51×) |
| escape-heavy | 10,000 | exact | 60.535 / 26.551 (2.28×) | 59.771 / 25.959 (2.29×) | 59.604 / 26.121 (2.29×) |
| gfm-task-autolink | 120 | serialization-equivalent | 1.276 / 1.193 (1.06×) | 0.885 / 0.832 (1.06×) | 0.736 / 0.746 (0.99×) |
| reference-heavy | 12,250 | exact | 86.799 / 98.168 (0.88×) | 85.623 / 97.070 (0.88×) | 84.953 / 97.678 (0.87×) |
| commonmark-5k | 5,632 | serialization-equivalent | 18.648 / 16.409 (1.14×) | 16.867 / 15.837 (1.06×) | 16.209 / 15.802 (1.03×) |
| commonmark-20k | 20,484 | serialization-equivalent | 67.154 / 63.597 (1.05×) | 64.283 / 62.956 (1.03×) | 62.926 / 62.992 (1.00×) |
| commonmark-50k | 52,342 | serialization-equivalent | 163.270 / 153.887 (1.07×) | 160.457 / 153.157 (1.04×) | 156.437 / 153.146 (1.02×) |
| tables-plain | 4,560 | exact | 25.629 / 34.875 (0.73×) | 24.752 / 34.728 (0.71×) | 24.477 / 34.603 (0.71×) |
| tables-commonmark-inline | 4,800 | exact | 32.801 / 41.327 (0.79×) | 31.040 / 40.446 (0.77×) | 30.836 / 40.332 (0.76×) |
| tables-links | 6,360 | exact | 41.451 / 47.309 (0.88×) | 39.549 / 47.002 (0.84×) | 38.475 / 45.747 (0.84×) |
| real-readme | 39,068 | different | 104.669 / 89.899 (1.16×) | 102.265 / 90.298 (1.13×) | 96.082 / 89.983 (1.07×) |

## Validation and boundaries

- Output admission: 10 byte-identical cases, 5 equivalent HTML serializations, 1 diagnostic mismatch. All lifecycle outputs agree within each engine.
- The README differs substantively in nine heading IDs (for example, `nodejs` versus `node-js`). These affect anchors and are not normalized away. Other observed differences are harmless entity/whitespace/checkbox serialization.
- Both engines use trusted CommonMark/GFM with matched heading IDs, links, tagfilter, and hard-break options; optional footnotes and MDX are outside this comparison. This does not compare default security policies.
- Fresh mode includes v2's owned renderer-option cloning and both libraries' actual allocation strategies. Reuse mode retains scratch/output; the parser/prepass and AST construction still run on every document.
- Owned strings are freed, and v2 documents/arenas are dropped or reset, inside the timer. I/O, normalization, verification, and process startup are excluded.
- 3 independent process rounds × 3 paired windows, at least 75 ms each; 486 pairs total. Orders alternate and jobs are deterministically shuffled. Per-round medians and every raw window remain in the data.
- Rust 1.95.0 / LLVM 22.1.2, generic AArch64, system allocator, opt-level 3, fat LTO, one codegen unit, panic abort. Each engine retains its own locked dependency versions/checksums.
- Local Apple Silicon results, with AC attached. Load/power observations are archived; thermal telemetry was unavailable. Small single-digit differences should not be treated as portable wins.
- Inputs include synthetic repeated diagnostics and existing synthetic fixtures. The one real README remains separately visible. This selection is not a production traffic distribution.

## Next optimization candidates

The measured gaps prioritize nested-list handling, GFM tables, and reference-heavy documents for investigation. Current Ferromark is a concrete donor candidate in those areas. These timings identify workloads, not proven internal bottlenecks; profile before choosing the code to port.

## Reproduce and inspect

- Main: `a6e9906f7b4a01355d336f209fd12534796419df` (Ferromark 0.9.0).
- v2 core: `d1481ca7687e94d30e56f473d3044a175dc6c9a6` (2.0.0-dev.0).
- [Harness and exact methodology](../../../benchmarks/current-comparison/README.md).
- [Summary CSV](summary.csv), [summary JSON](summary.json), and [run configuration](run.json).
- [Frozen inputs/provenance](corpus.json.gz), [original HTML outputs](verification.json.gz), and [raw paired samples](samples.json.gz).
- [Build/binary/lock hashes](build.json) and [harness hashes](harness-hashes.json).

The benchmark harness and this report are local additions. Neither measured parser implementation was modified.
