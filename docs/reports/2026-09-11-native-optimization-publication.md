# Native benchmark publication after table and short-document optimization

**Date:** 2026-09-11. **Measured source:** `58ba42bcbd0a2f90db6ba91a3500e6f8ed031e0d` plus
the [archived production patch](2026-09-11-native-optimization-publication/native/ferromark.patch).
The full [source hashes and environment](2026-09-11-native-optimization-publication/native/metadata.json)
identify the measured build; the working tree was not described as a clean commit.

The [retained table and short-document optimizations](2026-09-11-native-hotspot-optimization.md) are included, alongside the earlier table-cell batching.
This is a completely fresh five-parser run. None of the earlier publication's
samples or mixed table/strikethrough headline cases are reused.

## Does Ferromark beat pulldown everywhere?

No universal speed claim follows from a synthetic corpus. Within this run,
Ferromark uses less median time than pulldown-cmark in **38 of 42**
measured input/configuration pairs; pulldown uses less time in **4**.
Of the **9 repeated publication cases**, Ferromark is lower in
**9**. Counts are descriptive and not a weighted
score for real Markdown documents. The remaining diagnostic cases have only one
measurement process each, so small differences there deserve particular caution.

Pulldown has the lower median in:

- `commonmark/short-100b`: Ferromark 0.523 µs, pulldown 0.521 µs (0.3% more elapsed time).
- `commonmark/tiny`: Ferromark 0.286 µs, pulldown 0.281 µs (1.9% more elapsed time).
- `gfm_overlap/short-100b`: Ferromark 0.535 µs, pulldown 0.530 µs (0.9% more elapsed time).
- `gfm_overlap/tiny`: Ferromark 0.291 µs, pulldown 0.285 µs (2.0% more elapsed time).

## Repeated publication results

The README and homepage display these nine cases. Pulldown follows Ferromark;
bold values identify the lowest unrounded measured time, including exact ties.

All samples are retained. The tables use the preselected median-of-three
statistic; individual run medians and their spread are shown for review.
Differences near one percent should be treated as practical ties.

<!-- native-results:start -->
ferromark has the lowest measured median for: CommonMark · 2 KiB, CommonMark · 5 KiB, CommonMark · 10 KiB, Tables: plain text, Tables: emphasis and strong, Tables: link column, Strikethrough only, CommonMark links and images, CommonMark entities and inline markup.

These are results for the named inputs and contracts, not a universal parser ranking.

Latencies are medians of three run medians. Spread is the range of those
run medians divided by their median; it is not a confidence interval.
Output byte counts may differ because of accepted serialization or renderer differences.
The main README and homepage tables also report input throughput.

| Input / configuration | Parser | Time / document | Run medians (µs) | Spread | Output bytes |
| --- | --- | ---: | --- | ---: | ---: |
| CommonMark · 2 KiB | ferromark | **9.343 µs** | 9.364, 9.341, 9.343 | 0.2% | 3033 |
| CommonMark · 2 KiB | pulldown-cmark | 10.235 µs | 10.167, 10.235, 10.247 | 0.8% | 3033 |
| CommonMark · 2 KiB | Bun (native) | 15.092 µs | 15.077, 15.092, 15.132 | 0.4% | 3033 |
| CommonMark · 2 KiB | comrak | 23.209 µs | 23.241, 23.209, 23.179 | 0.3% | 3033 |
| CommonMark · 2 KiB | md4c (C) | 12.064 µs | 12.064, 12.033, 12.087 | 0.4% | 3033 |
| CommonMark · 5 KiB | ferromark | **23.702 µs** | 23.797, 23.658, 23.702 | 0.6% | 7735 |
| CommonMark · 5 KiB | pulldown-cmark | 27.761 µs | 27.765, 27.761, 27.676 | 0.3% | 7735 |
| CommonMark · 5 KiB | Bun (native) | 38.063 µs | 38.201, 38.008, 38.063 | 0.5% | 7735 |
| CommonMark · 5 KiB | comrak | 60.006 µs | 60.538, 60.006, 59.855 | 1.1% | 7735 |
| CommonMark · 5 KiB | md4c (C) | 29.981 µs | 30.071, 29.942, 29.981 | 0.4% | 7735 |
| CommonMark · 10 KiB | ferromark | **48.134 µs** | 48.134, 48.110, 48.237 | 0.3% | 15626 |
| CommonMark · 10 KiB | pulldown-cmark | 55.991 µs | 55.794, 55.991, 56.042 | 0.4% | 15626 |
| CommonMark · 10 KiB | Bun (native) | 76.926 µs | 76.926, 76.880, 77.232 | 0.5% | 15626 |
| CommonMark · 10 KiB | comrak | 124.042 µs | 124.042, 123.897, 124.052 | 0.1% | 15626 |
| CommonMark · 10 KiB | md4c (C) | 60.749 µs | 60.651, 60.749, 60.882 | 0.4% | 15626 |
| Tables: plain text | ferromark | **27.041 µs** | 27.185, 26.999, 27.041 | 0.7% | 11120 |
| Tables: plain text | pulldown-cmark | 29.770 µs | 30.314, 29.770, 29.732 | 2.0% | 10240 |
| Tables: plain text | Bun (native) | 41.330 µs | 41.919, 41.307, 41.330 | 1.5% | 10640 |
| Tables: plain text | comrak | 125.882 µs | 129.400, 125.882, 125.255 | 3.3% | 11120 |
| Tables: plain text | md4c (C) | 40.765 µs | 41.350, 40.765, 40.654 | 1.7% | 11120 |
| Tables: emphasis and strong | ferromark | **33.943 µs** | 34.052, 33.943, 33.886 | 0.5% | 12160 |
| Tables: emphasis and strong | pulldown-cmark | 36.953 µs | 37.527, 36.953, 36.706 | 2.2% | 11280 |
| Tables: emphasis and strong | Bun (native) | 48.958 µs | 49.371, 48.958, 48.865 | 1.0% | 11680 |
| Tables: emphasis and strong | comrak | 145.088 µs | 146.825, 145.088, 144.270 | 1.8% | 12160 |
| Tables: emphasis and strong | md4c (C) | 50.563 µs | 50.959, 50.563, 50.287 | 1.3% | 12160 |
| Tables: link column | ferromark | **38.854 µs** | 39.125, 38.713, 38.854 | 1.1% | 14200 |
| Tables: link column | pulldown-cmark | 44.639 µs | 45.023, 44.639, 44.630 | 0.9% | 13320 |
| Tables: link column | Bun (native) | 62.992 µs | 63.792, 62.734, 62.992 | 1.7% | 13720 |
| Tables: link column | comrak | 165.829 µs | 167.713, 165.414, 165.829 | 1.4% | 14200 |
| Tables: link column | md4c (C) | 68.207 µs | 68.742, 67.866, 68.207 | 1.3% | 14200 |
| Strikethrough only | ferromark | **22.837 µs** | 22.989, 22.837, 22.800 | 0.8% | 7300 |
| Strikethrough only | pulldown-cmark | 33.947 µs | 33.892, 33.970, 33.947 | 0.2% | 7300 |
| Strikethrough only | Bun (native) | 38.083 µs | 38.226, 38.083, 38.005 | 0.6% | 7300 |
| Strikethrough only | comrak | 75.156 µs | 75.156, 74.273, 75.777 | 2.0% | 7300 |
| Strikethrough only | md4c (C) | 28.142 µs | 28.237, 28.142, 28.107 | 0.5% | 7300 |
| CommonMark links and images | ferromark | **24.974 µs** | 24.974, 24.890, 25.022 | 0.5% | 7350 |
| CommonMark links and images | pulldown-cmark | 28.614 µs | 28.614, 28.584, 28.845 | 0.9% | 7350 |
| CommonMark links and images | Bun (native) | 38.640 µs | 38.640, 38.477, 38.834 | 0.9% | 7350 |
| CommonMark links and images | comrak | 58.349 µs | 58.210, 58.349, 58.558 | 0.6% | 7350 |
| CommonMark links and images | md4c (C) | 41.225 µs | 41.225, 41.179, 41.247 | 0.2% | 7210 |
| CommonMark entities and inline markup | ferromark | **52.624 µs** | 52.310, 54.666, 52.624 | 4.5% | 8400 |
| CommonMark entities and inline markup | pulldown-cmark | 63.644 µs | 62.690, 65.690, 63.644 | 4.7% | 7900 |
| CommonMark entities and inline markup | Bun (native) | 66.153 µs | 65.860, 67.759, 66.153 | 2.9% | 8400 |
| CommonMark entities and inline markup | comrak | 99.031 µs | 98.301, 101.303, 99.031 | 3.0% | 8400 |
| CommonMark entities and inline markup | md4c (C) | 58.013 µs | 57.581, 60.237, 58.013 | 4.6% | 8400 |
<!-- native-results:end -->

## Complete input/configuration matrix

This table includes the full diagnostic matrix. The publication cases use three
run medians; other cases use one. Time per complete document in microseconds,
lower is faster. Bold identifies the lowest median within that row. Table-shaped
input in a CommonMark-only diagnostic is not an enabled-table performance result.

| Case | Runs | ferromark | pulldown-cmark | Bun (native) | comrak | md4c (C) |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| commonmark/short-100b | 1 | 0.523 | **0.521** | 0.816 | 1.196 | 1.131 |
| commonmark/strikethrough | 1 | **18.567** | 21.794 | 31.604 | 51.251 | 20.441 |
| commonmark/tasks | 1 | **31.604** | 43.076 | 43.890 | 99.733 | 35.681 |
| commonmark/gfm-features | 1 | 35.867 | 37.456 | 45.973 | 94.939 | **32.884** |
| commonmark/tiny | 1 | 0.286 | **0.281** | 0.330 | 0.636 | 0.870 |
| commonmark/prose | 1 | **6.578** | 10.673 | 28.199 | 28.354 | 12.243 |
| commonmark/links | 3 | **24.974** | 28.614 | 38.640 | 58.349 | 41.225 |
| commonmark/entities | 3 | **52.624** | 63.644 | 66.153 | 99.031 | 58.013 |
| commonmark/publication-2k | 3 | **9.343** | 10.235 | 15.092 | 23.209 | 12.064 |
| commonmark/publication-5k | 3 | **23.702** | 27.761 | 38.063 | 60.006 | 29.981 |
| commonmark/publication-10k | 3 | **48.134** | 55.991 | 76.926 | 124.042 | 60.749 |
| commonmark/publication-50k | 1 | **240.472** | 281.133 | 396.877 | 629.067 | 305.036 |
| commonmark/light-1k | 1 | **4.934** | 5.318 | 7.775 | 12.014 | 6.458 |
| commonmark/commonmark-5k | 1 | **16.250** | 19.454 | 35.026 | 46.727 | 19.687 |
| commonmark/commonmark-50k | 1 | **155.900** | 185.262 | 328.180 | 460.824 | 191.850 |
| commonmark/commonmark-1m | 1 | **3210.411** | 3833.158 | 6647.126 | 10666.909 | 3963.478 |
| commonmark/tables-5k | 1 | **11.739** | 13.242 | 24.040 | 35.567 | 12.126 |
| gfm_overlap/short-100b | 1 | 0.535 | **0.530** | 0.836 | 1.240 | 1.159 |
| gfm_overlap/strikethrough | 1 | **22.786** | 33.893 | 37.874 | 75.142 | 28.005 |
| gfm_overlap/tasks | 1 | **23.126** | 26.621 | 35.841 | 107.767 | 27.550 |
| gfm_overlap/gfm-features | 1 | **37.719** | 42.292 | 54.510 | 155.597 | 51.539 |
| gfm_overlap/tiny | 1 | 0.291 | **0.285** | 0.336 | 0.663 | 0.890 |
| gfm_overlap/prose | 1 | **6.632** | 10.799 | 28.365 | 28.612 | 12.439 |
| gfm_overlap/links | 1 | **25.454** | 28.920 | 39.259 | 60.807 | 41.840 |
| gfm_overlap/entities | 1 | **52.641** | 63.552 | 66.302 | 100.075 | 58.087 |
| gfm_overlap/publication-2k | 1 | **9.543** | 10.461 | 15.360 | 23.636 | 12.234 |
| gfm_overlap/publication-5k | 1 | **23.998** | 28.057 | 38.187 | 61.023 | 30.106 |
| gfm_overlap/publication-10k | 1 | **48.456** | 55.876 | 76.802 | 125.457 | 60.913 |
| gfm_overlap/publication-50k | 1 | **240.437** | 277.424 | 396.201 | 633.212 | 304.466 |
| gfm_overlap/light-1k | 1 | **5.056** | 5.483 | 7.963 | 12.427 | 6.604 |
| gfm_overlap/commonmark-5k | 1 | **18.094** | 21.418 | 37.913 | 58.338 | 22.933 |
| gfm_overlap/commonmark-50k | 1 | **165.046** | 197.192 | 344.747 | 539.076 | 212.275 |
| gfm_overlap/commonmark-1m | 1 | **3234.955** | 3856.059 | 6763.155 | 11514.891 | 3993.818 |
| gfm_overlap/tables-5k | 1 | **26.400** | 31.738 | 52.208 | 141.145 | 40.584 |
| tables/tables-plain | 3 | **27.041** | 29.770 | 41.330 | 125.882 | 40.765 |
| tables/tables-links | 3 | **38.854** | 44.639 | 62.992 | 165.829 | 68.207 |
| tables/tables-commonmark-inline | 3 | **33.943** | 36.953 | 48.958 | 145.088 | 50.563 |
| tables/light-1k | 1 | **6.024** | 6.565 | 9.142 | 14.284 | 8.161 |
| strikethrough/strikethrough | 3 | **22.837** | 33.947 | 38.083 | 75.156 | 28.142 |
| strikethrough/light-1k | 1 | **5.012** | 5.451 | 7.902 | 12.252 | 6.537 |
| task_lists/tasks | 1 | **24.078** | 27.553 | 36.759 | 111.840 | 28.450 |
| task_lists/light-1k | 1 | **5.004** | 5.404 | 7.874 | 12.249 | 6.529 |

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
[previous publication](2026-09-11-table-optimization-refresh.md) and the
[historical report](2026-09-11-benchmark-refresh.md), with their own source revisions
and the md4c flag-correction evidence.

## Evidence and reproduction

- [Raw summaries](2026-09-11-native-optimization-publication/native/summary.json),
  [metadata](2026-09-11-native-optimization-publication/native/metadata.json),
  [workload and spec verification](2026-09-11-native-optimization-publication/native/verification.json).
- [Locked dependencies](2026-09-11-native-optimization-publication/native/Cargo.lock),
  [production patch](2026-09-11-native-optimization-publication/native/ferromark.patch),
  and [measurement-time harness](2026-09-11-native-optimization-publication/measured-harness).
  The harness snapshot records the measurement and publication scripts used for
  this run. Native adapters, inputs, and parser sources did not change during
  measurement.
- Three `samples-*.jsonl` files retain all windows; `verify`, `spec`, `catalog`, and
  `options` retain original data. Files may be gzip-compressed without alteration.
- [Validation log](2026-09-11-native-optimization-publication/validation.log) records
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
