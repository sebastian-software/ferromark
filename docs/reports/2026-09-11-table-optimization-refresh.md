# Native benchmark refresh after table-cell batching

**Date:** 2026-09-11. **Measured source:** `c5ca4ac9f18e7173ebf44a75dd26a660686f287c` plus
the [archived production patch](2026-09-11-table-optimization-refresh/native/ferromark.patch).
The full [source hashes and environment](2026-09-11-table-optimization-refresh/native/metadata.json)
identify the measured build; the working tree was not described as a clean commit.

The [table-cell optimization](2026-09-11-table-optimization-variants.md) is included.
This is a completely fresh five-parser run. None of the earlier publication's
samples or mixed table/strikethrough headline cases are reused.

## Does Ferromark beat pulldown everywhere?

No universal speed claim follows from a synthetic corpus. Within this run,
Ferromark uses less median time than pulldown-cmark in **36 of 42**
measured input/configuration pairs; pulldown uses less time in **6**.
Of the **9 repeated publication cases**, Ferromark is lower in
**7**. Counts are descriptive and not a weighted
score for real Markdown documents. The remaining diagnostic cases have only one
measurement process each, so small differences there deserve particular caution.

Pulldown has the lower median in:

- `commonmark/short-100b`: Ferromark 0.651 µs, pulldown 0.538 µs (20.9% more elapsed time).
- `commonmark/tiny`: Ferromark 0.390 µs, pulldown 0.280 µs (39.5% more elapsed time).
- `gfm_overlap/short-100b`: Ferromark 0.647 µs, pulldown 0.537 µs (20.5% more elapsed time).
- `gfm_overlap/tiny`: Ferromark 0.411 µs, pulldown 0.284 µs (44.7% more elapsed time).
- `tables/tables-plain`: Ferromark 30.786 µs, pulldown 29.690 µs (3.7% more elapsed time).
- `tables/tables-commonmark-inline`: Ferromark 38.130 µs, pulldown 36.657 µs (4.0% more elapsed time).

## Repeated publication results

The README and homepage display these nine cases. Pulldown follows Ferromark;
bold values identify the lowest unrounded measured time, including exact ties.

The third 10 KiB repetition slowed across all five parsers, producing unusually
large run-to-run variation. Ferromark remains the fastest in each of those three
runs, but the size of its lead varies. All samples are retained, and the table
uses the preselected median-of-three statistic without discarding that run.

<!-- native-results:start -->
ferromark has the lowest measured median for: CommonMark · 2 KiB, CommonMark · 5 KiB, CommonMark · 10 KiB, Tables: link column, Strikethrough only, CommonMark links and images, CommonMark entities and inline markup.
pulldown-cmark has the lowest measured median for: Tables: plain text, Tables: emphasis and strong.

These are results for the named inputs and contracts, not a universal parser ranking.

Latencies are medians of three run medians. Spread is the range of those
run medians divided by their median; it is not a confidence interval.
Output byte counts may differ because of accepted serialization or renderer differences.
The main README and homepage tables also report input throughput.

| Input / configuration | Parser | Time / document | Run medians (µs) | Spread | Output bytes |
| --- | --- | ---: | --- | ---: | ---: |
| CommonMark · 2 KiB | ferromark | **9.733 µs** | 9.704, 9.767, 9.733 | 0.6% | 3033 |
| CommonMark · 2 KiB | pulldown-cmark | 10.277 µs | 10.210, 10.315, 10.277 | 1.0% | 3033 |
| CommonMark · 2 KiB | Bun (native) | 15.085 µs | 15.037, 15.180, 15.085 | 0.9% | 3033 |
| CommonMark · 2 KiB | comrak | 22.619 µs | 22.408, 22.710, 22.619 | 1.3% | 3033 |
| CommonMark · 2 KiB | md4c (C) | 12.079 µs | 12.042, 12.194, 12.079 | 1.3% | 3033 |
| CommonMark · 5 KiB | ferromark | **24.586 µs** | 24.491, 24.620, 24.586 | 0.5% | 7735 |
| CommonMark · 5 KiB | pulldown-cmark | 27.585 µs | 27.560, 27.745, 27.585 | 0.7% | 7735 |
| CommonMark · 5 KiB | Bun (native) | 38.059 µs | 38.059, 38.287, 38.040 | 0.6% | 7735 |
| CommonMark · 5 KiB | comrak | 58.336 µs | 58.004, 58.485, 58.336 | 0.8% | 7735 |
| CommonMark · 5 KiB | md4c (C) | 30.089 µs | 29.992, 30.349, 30.089 | 1.2% | 7735 |
| CommonMark · 10 KiB | ferromark | **50.155 µs** | 49.984, 50.155, 81.762 | 63.4% | 15626 |
| CommonMark · 10 KiB | pulldown-cmark | 56.140 µs | 55.957, 56.140, 83.673 | 49.4% | 15626 |
| CommonMark · 10 KiB | Bun (native) | 77.033 µs | 76.734, 77.033, 115.149 | 49.9% | 15626 |
| CommonMark · 10 KiB | comrak | 120.391 µs | 119.839, 120.391, 160.255 | 33.6% | 15626 |
| CommonMark · 10 KiB | md4c (C) | 61.338 µs | 60.869, 61.338, 86.855 | 42.4% | 15626 |
| Tables: plain text | ferromark | 30.786 µs | 30.786, 30.630, 30.973 | 1.1% | 11120 |
| Tables: plain text | pulldown-cmark | **29.690 µs** | 29.897, 29.500, 29.690 | 1.3% | 10240 |
| Tables: plain text | Bun (native) | 40.974 µs | 40.974, 40.824, 41.076 | 0.6% | 10640 |
| Tables: plain text | comrak | 124.528 µs | 124.800, 123.207, 124.528 | 1.3% | 11120 |
| Tables: plain text | md4c (C) | 40.757 µs | 40.757, 40.596, 41.075 | 1.2% | 11120 |
| Tables: emphasis and strong | ferromark | 38.130 µs | 38.711, 38.130, 37.810 | 2.4% | 12160 |
| Tables: emphasis and strong | pulldown-cmark | **36.657 µs** | 37.170, 36.657, 36.588 | 1.6% | 11280 |
| Tables: emphasis and strong | Bun (native) | 48.546 µs | 48.969, 48.546, 48.188 | 1.6% | 11680 |
| Tables: emphasis and strong | comrak | 142.276 µs | 144.060, 142.276, 141.935 | 1.5% | 12160 |
| Tables: emphasis and strong | md4c (C) | 50.274 µs | 50.555, 50.274, 50.088 | 0.9% | 12160 |
| Tables: link column | ferromark | **43.202 µs** | 44.155, 43.202, 42.868 | 3.0% | 14200 |
| Tables: link column | pulldown-cmark | 44.422 µs | 45.004, 44.422, 44.284 | 1.6% | 13320 |
| Tables: link column | Bun (native) | 62.553 µs | 63.454, 62.553, 62.327 | 1.8% | 13720 |
| Tables: link column | comrak | 163.272 µs | 165.168, 162.870, 163.272 | 1.4% | 14200 |
| Tables: link column | md4c (C) | 67.930 µs | 68.686, 67.930, 67.926 | 1.1% | 14200 |
| Strikethrough only | ferromark | **25.476 µs** | 25.958, 25.476, 25.300 | 2.6% | 7300 |
| Strikethrough only | pulldown-cmark | 33.729 µs | 34.042, 33.729, 33.356 | 2.0% | 7300 |
| Strikethrough only | Bun (native) | 38.046 µs | 38.358, 38.046, 37.550 | 2.1% | 7300 |
| Strikethrough only | comrak | 72.504 µs | 74.170, 72.504, 72.101 | 2.9% | 7300 |
| Strikethrough only | md4c (C) | 28.135 µs | 28.406, 28.135, 27.956 | 1.6% | 7300 |
| CommonMark links and images | ferromark | **27.009 µs** | 27.009, 27.633, 26.969 | 2.5% | 7350 |
| CommonMark links and images | pulldown-cmark | 28.707 µs | 28.685, 29.553, 28.707 | 3.0% | 7350 |
| CommonMark links and images | Bun (native) | 38.819 µs | 38.730, 39.705, 38.819 | 2.5% | 7350 |
| CommonMark links and images | comrak | 57.471 µs | 57.169, 59.274, 57.471 | 3.7% | 7350 |
| CommonMark links and images | md4c (C) | 41.044 µs | 40.912, 41.931, 41.044 | 2.5% | 7210 |
| CommonMark entities and inline markup | ferromark | **56.948 µs** | 56.322, 62.095, 56.948 | 10.1% | 8400 |
| CommonMark entities and inline markup | pulldown-cmark | 62.530 µs | 61.930, 69.735, 62.530 | 12.5% | 7900 |
| CommonMark entities and inline markup | Bun (native) | 66.297 µs | 65.762, 71.892, 66.297 | 9.2% | 8400 |
| CommonMark entities and inline markup | comrak | 96.327 µs | 95.479, 107.176, 96.327 | 12.1% | 8400 |
| CommonMark entities and inline markup | md4c (C) | 58.364 µs | 57.752, 65.281, 58.364 | 12.9% | 8400 |
<!-- native-results:end -->

## Complete input/configuration matrix

This table includes the full diagnostic matrix. The publication cases use three
run medians; other cases use one. Time per complete document in microseconds,
lower is faster. Bold identifies the lowest median within that row. Table-shaped
input in a CommonMark-only diagnostic is not an enabled-table performance result.

| Case | Runs | ferromark | pulldown-cmark | Bun (native) | comrak | md4c (C) |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| commonmark/short-100b | 1 | 0.651 | **0.538** | 0.846 | 1.214 | 1.208 |
| commonmark/strikethrough | 1 | **27.891** | 34.672 | 46.899 | 84.719 | 30.845 |
| commonmark/tasks | 1 | **31.975** | 45.199 | 44.349 | 98.006 | 36.739 |
| commonmark/gfm-features | 1 | 37.852 | 38.788 | 46.558 | 93.527 | **33.318** |
| commonmark/tiny | 1 | 0.390 | **0.280** | 0.332 | 0.630 | 0.900 |
| commonmark/prose | 1 | **6.504** | 10.581 | 27.813 | 26.912 | 12.113 |
| commonmark/links | 3 | **27.009** | 28.707 | 38.819 | 57.471 | 41.044 |
| commonmark/entities | 3 | **56.948** | 62.530 | 66.297 | 96.327 | 58.364 |
| commonmark/publication-2k | 3 | **9.733** | 10.277 | 15.085 | 22.619 | 12.079 |
| commonmark/publication-5k | 3 | **24.586** | 27.585 | 38.059 | 58.336 | 30.089 |
| commonmark/publication-10k | 3 | **50.155** | 56.140 | 77.033 | 120.391 | 61.338 |
| commonmark/publication-50k | 1 | **491.620** | 495.232 | 764.254 | 1345.137 | 743.214 |
| commonmark/light-1k | 1 | **6.313** | 6.372 | 9.687 | 13.823 | 7.612 |
| commonmark/commonmark-5k | 1 | **16.863** | 19.715 | 35.430 | 45.905 | 19.765 |
| commonmark/commonmark-50k | 1 | **158.335** | 184.652 | 328.864 | 445.404 | 189.732 |
| commonmark/commonmark-1m | 1 | **3288.408** | 3878.404 | 6826.449 | 11318.978 | 4086.902 |
| commonmark/tables-5k | 1 | 12.857 | 13.860 | 25.145 | 37.616 | **12.553** |
| gfm_overlap/short-100b | 1 | 0.647 | **0.537** | 0.840 | 1.213 | 1.199 |
| gfm_overlap/strikethrough | 1 | **25.642** | 33.825 | 38.018 | 74.123 | 28.152 |
| gfm_overlap/tasks | 1 | **24.536** | 27.444 | 36.408 | 110.001 | 29.362 |
| gfm_overlap/gfm-features | 1 | **54.504** | 55.602 | 72.849 | 207.353 | 65.207 |
| gfm_overlap/tiny | 1 | 0.411 | **0.284** | 0.339 | 0.652 | 0.919 |
| gfm_overlap/prose | 1 | **6.769** | 11.096 | 29.061 | 27.760 | 12.664 |
| gfm_overlap/links | 1 | **27.464** | 28.762 | 38.960 | 59.280 | 41.477 |
| gfm_overlap/entities | 1 | **57.710** | 63.264 | 67.656 | 99.532 | 59.270 |
| gfm_overlap/publication-2k | 1 | **10.031** | 10.524 | 15.525 | 23.564 | 12.419 |
| gfm_overlap/publication-5k | 1 | **25.097** | 28.100 | 38.529 | 59.670 | 30.492 |
| gfm_overlap/publication-10k | 1 | **50.780** | 56.547 | 77.485 | 123.027 | 61.739 |
| gfm_overlap/publication-50k | 1 | **247.618** | 278.979 | 393.301 | 609.178 | 304.718 |
| gfm_overlap/light-1k | 1 | **5.263** | 5.462 | 7.916 | 11.866 | 6.574 |
| gfm_overlap/commonmark-5k | 1 | **18.775** | 21.530 | 37.946 | 56.909 | 22.950 |
| gfm_overlap/commonmark-50k | 1 | **172.512** | 201.004 | 349.508 | 533.929 | 214.027 |
| gfm_overlap/commonmark-1m | 1 | **3263.594** | 3883.056 | 6697.723 | 10860.663 | 3960.406 |
| gfm_overlap/tables-5k | 1 | **29.464** | 31.723 | 51.616 | 138.015 | 40.318 |
| tables/tables-plain | 3 | 30.786 | **29.690** | 40.974 | 124.528 | 40.757 |
| tables/tables-links | 3 | **43.202** | 44.422 | 62.553 | 163.272 | 67.930 |
| tables/tables-commonmark-inline | 3 | 38.130 | **36.657** | 48.546 | 142.276 | 50.274 |
| tables/light-1k | 1 | **5.299** | 5.490 | 7.958 | 11.907 | 6.610 |
| strikethrough/strikethrough | 3 | **25.476** | 33.729 | 38.046 | 72.504 | 28.135 |
| strikethrough/light-1k | 1 | **5.264** | 5.416 | 7.876 | 11.784 | 6.548 |
| task_lists/tasks | 1 | **24.026** | 26.919 | 35.986 | 106.610 | 28.959 |
| task_lists/light-1k | 1 | **5.300** | 5.495 | 7.978 | 11.965 | 6.637 |

## Workload and measurement contract

All five parsers render trusted input with fresh parser state and owned HTML output. The feature set is named per table; bare autolinks, tag filtering, heading IDs, and other extensions are disabled. All parsers use Bun's pinned mimalloc, including md4c's C allocation calls. Rust uses the same pinned nightly compiler and generic CPU target; Bun retains its native Highway support. No PGO is used.

These measurements are Apple Silicon results only; this comparison has not been re-measured on x86-64. The shared Bun-native support environment is part of the experiment, and the results do not measure the JavaScript runtime or Ferromark's secure-default rendering.

All 42 cases passed workload review before timing. Normalized HTML agrees
in 36 cases; original outputs, hashes, spec
checks, and accepted renderer differences remain in the verification evidence.
HTML normalization is never included in timed rendering.

Each process uses 80 alternating-order windows of at least 63 ms per parser/input
and three seconds of warmup per parser. The first process measures the complete
matrix; two more processes repeat the nine displayed cases with rotated parser
order. Each timed call creates parser state, produces owned HTML, and releases it.
The raw windows and three individual run medians remain available. Spread is not
a confidence interval. All measured parser sources, native dependencies, and
compiler settings remain frozen for the entire run.

The table rows cover plain cells, mixed plain/emphasis/strong cells, and a Markdown
link column. Each enables CommonMark plus tables; ordinary inline parsing is
always active. Table strikethrough is outside the published comparison. Standalone
strikethrough remains a separate feature case. The complete diagnostic matrix also
retains the broader existing GFM and CommonMark controls; it is not presented as
an additive feature-cost measurement.

The earlier stable/System-allocator paired experiment has different compiler,
CPU-target, allocator, and output-reuse settings. Its margins must not be copied
into this native/shared-mimalloc publication. Earlier figures remain in the
[historical report](2026-09-11-benchmark-refresh.md), with their own source revisions
and the md4c flag-correction evidence.

## Evidence and reproduction

- [Raw summaries](2026-09-11-table-optimization-refresh/native/summary.json),
  [metadata](2026-09-11-table-optimization-refresh/native/metadata.json),
  [workload and spec verification](2026-09-11-table-optimization-refresh/native/verification.json).
- [Locked dependencies](2026-09-11-table-optimization-refresh/native/Cargo.lock),
  [production patch](2026-09-11-table-optimization-refresh/native/ferromark.patch),
  and [measurement-time harness](2026-09-11-table-optimization-refresh/measured-harness).
  The harness snapshot precedes presentation-only generator changes made while
  timing ran. Measurement code, native adapters, inputs, and parser sources did
  not change during the run.
- Three `samples-*.jsonl` files retain all windows; `verify`, `spec`, `catalog`, and
  `options` retain original data. Files may be gzip-compressed without alteration.
- [Validation log](2026-09-11-table-optimization-refresh/validation.log) records
  publication guards, generated-data contracts, and homepage validation.

Follow the [native harness instructions](../../benchmarks/bun-comparison/README.md)
to prepare the pinned checkouts, then run:

```bash
python3 benchmarks/bun-comparison/run.py "$BUN_BENCH_DIR" /private/tmp/new-complete-benchmark-run
python3 benchmarks/bun-comparison/publish.py
python3 benchmarks/bun-comparison/publish.py --check
```

The generator chooses winners from unrounded medians and places pulldown-cmark
immediately after Ferromark. Historical publications remain reproducible through
their own case catalog; they cannot supply partial rows to the new publication.
