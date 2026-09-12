# Ox corpus optimizations and refreshed native comparisons

Date: September 12, 2026.

The profiles from the [previous investigation](../2026-09-12-ox-corpus-profiling/REPORT.md)
led to two retained changes: line-end searches through memchr, and
rendering larger contiguous code/HTML ranges through the existing block renderer.
The latter runs after the table-cell shortcut. Public block and MDX events,
custom fence callbacks, HTML policies, and parser options retain their contracts.

The [generated experiment results](experiments/RESULTS.md) show the measured
before/after changes against merged main `a1c308cba63f7bb038699df8560daa8959159b4e`.
The report includes every screened variant, repeated confirmations, a fresh
comparison against Ox on its corpus, and a separate allocation cross-check.
The strongest improvements concern HTML-heavy references and code-heavy guides.
They do not establish that Ferromark is the fastest engine on every document.

## What was retained and rejected

- Replace the block cursor's scalar line-end loop and line lookahead with memchr.
- Coalesce contiguous code ranges while privately consuming block events. The
  existing renderer still collects code for custom fence callbacks or escapes it.
- Coalesce contiguous raw or escaped HTML ranges. Trusted filtered HTML retains
  its previous event boundaries so tag filtering sees precisely the same slices.
- Keep the table-cell shortcut first; the initial placement imposed unnecessary
  checks on that existing fast path.
- Reject the deferred end-event comparator, packed sort key, and reordered code
  points: their screens did not justify a production sorting change.

No AST, arena, additional scratch buffer, or public event-layout change is needed.
[ARCH-EXP-021](../../arch/ARCH-EXP-021-ox-corpus-line-scans-and-render-ranges.md)
records the decision and its boundaries.

## Correctness and experimental method

Each candidate first passes 2,695 exact-output guards: 2,046 specification,
extension, MDX, and renderer-reuse cases plus all 649 cases from the previous
corpus investigation. The latter include 638 original files, eight project
concatenations, and three upstream parser fixtures. Known differences from Ox
remain in the guard; equal Ferromark output does not imply those differences
have been fixed. Corpus concatenations with unequal normalized HTML remain
performance diagnostics and cannot support a fair engine ranking.

Independent screens use five alternating 30 ms windows per implementation and
input. The chosen combination receives three fresh process-pair confirmations
with nine alternating 75 ms windows. The formatted production source is independently rebuilt and receives the same
three-pair confirmation. The 69 timing inputs include the original
45 extension/lifecycle/MDX guards and 24 selected corpus cases. These short
experiments select changes; they do not feed the homepage tables.

The standalone experiments use the aarch64-apple-darwin default CPU target
(`apple-m1`) with release builds, opt-level 3,
fat LTO, one codegen unit, panic abort, and the system allocator. The corpus
comparison uses Ox's growing arena as the primary result and retains presized
arena samples separately. Allocation counting runs separately from timing and
measures requested live heap bytes, not RSS. Neither AST shape nor a single
fixture establishes a universal memory advantage.

[Reproduction instructions](experiments/REPRODUCE.md) retain source identities,
lockfiles, transformations, exact guard inputs, and raw windows. Original
corpus attribution and license texts are in the [previous archive](../2026-09-12-ox-corpus-profiling/licenses/ATTRIBUTION.md):
Vue and TypeScript documentation use CC BY 4.0; Vite uses MIT; Rust Book uses
MIT/Apache-2.0; Ox fixtures use MIT. Synthetic/specification guard attribution
also follows the [previous optimization archive](../2026-09-12-ox-inspired-optimizations/REPORT.md).

Rust 1.97.1's target specification confirms `apple-m1` as its default. Earlier
standalone reports described this as generic; that label was incorrect, although
both engines used the same target. This report corrects the profiling descriptor
and retains all original samples. The public harnesses below explicitly set
`-C target-cpu=generic`.

## Fresh public five-parser results

Every displayed input receives three new process runs, each with three seconds
of warmup and 80 alternating windows of at least 63 ms per parser. All nine
published inputs are remeasured for all five parsers. Other catalog inputs and
all 652 CommonMark examples receive output verification. Old timing samples
are not substituted into the new tables. This source also includes the earlier
merged scanning/allocation optimizations; only the separate paired experiment
isolates the incremental changes in this PR.

This controlled environment uses the pinned Bun nightly toolchain and mimalloc
for every Rust parser and md4c C allocation. The exact sources, compiler,
options, raw output, and all windows are retained in [the native archive](bun/metadata.json).
Sources are unchanged during each measurement. Differences near one percent
are practical ties; run spread is descriptive, not a confidence interval.

<!-- native-results:start -->
ferromark has the lowest measured median for: CommonMark · 2 KiB, CommonMark · 5 KiB, CommonMark · 10 KiB, Tables: plain text, Tables: emphasis and strong, Tables: link column, Strikethrough only, CommonMark links and images, CommonMark entities and inline markup.

These are results for the named inputs and contracts, not a universal parser ranking.

Latencies are medians of three run medians. Spread is the range of those
run medians divided by their median; it is not a confidence interval.
Output byte counts may differ because of accepted serialization or renderer differences.
The main README and homepage tables also report input throughput.

| Input / configuration | Parser | Time / document | Run medians (µs) | Spread | Output bytes |
| --- | --- | ---: | --- | ---: | ---: |
| CommonMark · 2 KiB | ferromark | **8.916 µs** | 8.916, 8.923, 8.907 | 0.2% | 3033 |
| CommonMark · 2 KiB | pulldown-cmark | 10.016 µs | 10.017, 10.012, 10.016 | 0.1% | 3033 |
| CommonMark · 2 KiB | Bun (native) | 14.818 µs | 14.815, 14.818, 14.840 | 0.2% | 3033 |
| CommonMark · 2 KiB | comrak | 22.192 µs | 22.208, 22.178, 22.192 | 0.1% | 3033 |
| CommonMark · 2 KiB | md4c (C) | 11.881 µs | 11.901, 11.870, 11.881 | 0.3% | 3033 |
| CommonMark · 5 KiB | ferromark | **22.873 µs** | 22.873, 22.882, 22.788 | 0.4% | 7735 |
| CommonMark · 5 KiB | pulldown-cmark | 27.197 µs | 27.163, 27.281, 27.197 | 0.4% | 7735 |
| CommonMark · 5 KiB | Bun (native) | 37.540 µs | 37.540, 37.571, 37.477 | 0.2% | 7735 |
| CommonMark · 5 KiB | comrak | 57.516 µs | 57.590, 57.516, 57.130 | 0.8% | 7735 |
| CommonMark · 5 KiB | md4c (C) | 29.722 µs | 29.801, 29.722, 29.692 | 0.4% | 7735 |
| CommonMark · 10 KiB | ferromark | **46.654 µs** | 46.654, 46.722, 46.635 | 0.2% | 15626 |
| CommonMark · 10 KiB | pulldown-cmark | 55.101 µs | 54.955, 55.179, 55.101 | 0.4% | 15626 |
| CommonMark · 10 KiB | Bun (native) | 75.974 µs | 75.974, 75.911, 76.055 | 0.2% | 15626 |
| CommonMark · 10 KiB | comrak | 118.434 µs | 118.434, 118.604, 118.147 | 0.4% | 15626 |
| CommonMark · 10 KiB | md4c (C) | 60.397 µs | 60.407, 60.274, 60.397 | 0.2% | 15626 |
| Tables: plain text | ferromark | **25.519 µs** | 25.500, 25.549, 25.519 | 0.2% | 11120 |
| Tables: plain text | pulldown-cmark | 29.092 µs | 29.092, 29.151, 29.063 | 0.3% | 10240 |
| Tables: plain text | Bun (native) | 40.186 µs | 40.225, 40.186, 40.180 | 0.1% | 10640 |
| Tables: plain text | comrak | 120.894 µs | 120.965, 120.894, 120.732 | 0.2% | 11120 |
| Tables: plain text | md4c (C) | 39.702 µs | 39.785, 39.702, 39.670 | 0.3% | 11120 |
| Tables: emphasis and strong | ferromark | **32.489 µs** | 32.489, 32.510, 32.451 | 0.2% | 12160 |
| Tables: emphasis and strong | pulldown-cmark | 36.305 µs | 36.255, 36.406, 36.305 | 0.4% | 11280 |
| Tables: emphasis and strong | Bun (native) | 47.652 µs | 47.702, 47.651, 47.652 | 0.1% | 11680 |
| Tables: emphasis and strong | comrak | 139.884 µs | 139.884, 139.910, 139.573 | 0.2% | 12160 |
| Tables: emphasis and strong | md4c (C) | 49.289 µs | 49.285, 49.289, 49.314 | 0.1% | 12160 |
| Tables: link column | ferromark | **37.305 µs** | 37.467, 37.283, 37.305 | 0.5% | 14200 |
| Tables: link column | pulldown-cmark | 43.909 µs | 43.909, 43.900, 43.957 | 0.1% | 13320 |
| Tables: link column | Bun (native) | 61.547 µs | 61.722, 61.397, 61.547 | 0.5% | 13720 |
| Tables: link column | comrak | 160.332 µs | 161.466, 160.332, 159.635 | 1.1% | 14200 |
| Tables: link column | md4c (C) | 68.000 µs | 68.350, 68.000, 67.978 | 0.5% | 14200 |
| Strikethrough only | ferromark | **21.049 µs** | 21.092, 21.049, 20.951 | 0.7% | 7300 |
| Strikethrough only | pulldown-cmark | 33.116 µs | 33.362, 33.116, 32.985 | 1.1% | 7300 |
| Strikethrough only | Bun (native) | 37.124 µs | 37.124, 37.155, 36.892 | 0.7% | 7300 |
| Strikethrough only | comrak | 72.216 µs | 72.319, 72.216, 71.020 | 1.8% | 7300 |
| Strikethrough only | md4c (C) | 27.614 µs | 27.664, 27.614, 27.539 | 0.5% | 7300 |
| CommonMark links and images | ferromark | **24.523 µs** | 24.485, 24.536, 24.523 | 0.2% | 7350 |
| CommonMark links and images | pulldown-cmark | 28.296 µs | 28.314, 28.296, 28.266 | 0.2% | 7350 |
| CommonMark links and images | Bun (native) | 38.010 µs | 37.994, 38.020, 38.010 | 0.1% | 7350 |
| CommonMark links and images | comrak | 56.867 µs | 56.869, 56.867, 56.787 | 0.1% | 7350 |
| CommonMark links and images | md4c (C) | 40.873 µs | 40.873, 40.895, 40.833 | 0.2% | 7210 |
| CommonMark entities and inline markup | ferromark | **51.172 µs** | 51.163, 51.222, 51.172 | 0.1% | 8400 |
| CommonMark entities and inline markup | pulldown-cmark | 61.919 µs | 61.592, 61.919, 62.005 | 0.7% | 7900 |
| CommonMark entities and inline markup | Bun (native) | 64.071 µs | 64.071, 64.140, 64.013 | 0.2% | 8400 |
| CommonMark entities and inline markup | comrak | 95.415 µs | 95.778, 95.415, 94.497 | 1.3% | 8400 |
| CommonMark entities and inline markup | md4c (C) | 57.396 µs | 57.396, 57.340, 57.773 | 0.8% | 8400 |
<!-- native-results:end -->

## Additional native engines

All six additional engines receive new paired measurements of the final
Ferromark implementation: [Goldmark and Sätteri](native-pipeline/REPORT.md),
[Rushdown](rushdown/REPORT.md), [Markdig](markdig/REPORT.md),
[markdown-rs](markdown-rs/REPORT.md), and [Ox Content](ox-content/REPORT.md).
They use system allocators or normal managed-runtime GC, so their overview
remains separate from the Bun/mimalloc table. Node wrappers, WASM, and MDX
compilation remain outside the measured operation.

The public overview shows Ferromark once per document, using the median of its
independently measured reference medians. Each candidate retains its measured
time; its ratio uses that common reference. All original paired baselines and
run variation remain in the archives. The short Ox document is not the other
engines' 5 KiB document: mandatory heading IDs keep that case excluded.

The fixed native adapter uses Ox release revision `5c97078779cf099245a78aacca8fa7cc5e993316`;
the corpus investigation uses `026d1859d1c35e5fb1ea65e7e855b428a918b9bb`.
Both identify themselves as 3.2.0. Their revisions, renderer lifecycle and syntax
contracts are explicit in their respective archives; timings are not mixed.
The [cmark/cmark-gfm report](../2026-09-11-native-cmark-comparison.md) remains a
separately dated historical comparison with its original baseline.

The README source, themed README, homepage data, and relevant harness READMEs
are refreshed together. The Node package README contains no timing table.

## Validation

The required locked all-feature Rust tests, all-target Clippy checks with warnings
as errors, byte-search crate checks, formatting, and warning-free Rustdoc pass.
All 122 repository contract tests pass; the migration example compilation uses
the existing dependency cache in offline mode. The native publication/runner,
Bun, cmark, and CI benchmark Python contracts also pass. Both public datasets
reconstruct exactly from their raw measurement archives.

The pinned mdtheme 0.4.0 release archive is SHA-256 verified against `mise.lock`.
It generates and checks the themed README; the registry family check also passes.
The homepage passes type checking, build-time data checks, and verification of
all six prerendered pages. A Chromium check visits every native table selection,
asserts the single Ferromark row and all displayed values/winners, checks the
benchmark guide, and verifies mobile overflow and absence of runtime errors.

A fresh reconstruction from the compact experiment archive builds the baseline
and production drivers and passes all 2,695 guards for each. Validation logs and
commands are in [the experiment archive](experiments/validation/). GitHub CI is
reported on the accompanying PR rather than frozen into these measurement data.

The raw report archives remain in Git and are excluded from the published Cargo
package. `cargo package --allow-dirty --offline --locked` builds the packaged
crate successfully, and its file list contains no report archives. This is a
packaging-only adjustment after the measurements; the measured parser sources
are unchanged. Required local package checks also pass: frozen Node install,
security audit, native build, 23 package tests, typecheck, lint, pack verification,
and a clean consumer install. No dependency versions or release versions change.
