# Benchmark refresh: five native parsers and feature costs

**Follow-up:** performance admission now follows the
[workload contract](../arch/ARCH-COMP-002-workload-comparability.md), accepting the
known Bun alignment and task-renderer differences. HTML agreement remains a
separate diagnostic. [ADR-0014](../arch/ADR-0014-reference-resolution-budget.md)
increases the production reference budget. Public figures below retain their
recorded measurement revisions; the later full-matrix screening validates
coverage and is not substituted for publication timing samples.

**Date:** 2026-09-11

**Measured Ferromark revision:** `8adecc1d017e406eccbd582eb08992054af0411b`

**Scope:** measurement and documentation; no new production parser optimization.

## Questions and document sizes

This refresh asks how five native parsers compare when they render equivalent
output under a named feature set, and how Ferromark's individual features and
API lifecycles affect its own costs. Those are separate measurement questions.

The public size matrix uses **2, 5, and 10 KiB**. Each input repeats a CommonMark
unit containing headings, emphasis, strong text, inline code, links with a query
string/entity, lists, a quote, and fenced Rust code, then pads to the exact byte
count. The extension-free configuration is explicit. These synthetic documents
control size and syntax density; they do not establish a population's typical
Markdown size or reproduce the diversity of real documents.

The choice responds to the intended focus on short documentation and comments
that actually use Markdown. A 100-byte input mostly measures per-call overhead;
50 KiB is a useful long-document case, but a poor sole headline for this focus.
As a local sanity check before the refresh, five existing guide pages ranged
from 1,426 to 6,239 bytes, with a median of 1,616 bytes; the repository README
was 48,414 bytes. This convenience sample is not a representative usage survey.
Tiny, 50 KiB, and 1 MiB controls remain in the detailed diagnostics. Inputs that
fail the shared output gate are preserved without a cross-parser timing ratio.

The feature matrix separately covers tables, double-tilde strikethrough, a
shared GFM subset, links/images, and entities/inline markup. Compare parsers
within a row; row-to-row differences are not additive prices for syntax.

## Native comparison contract

All five implementations are linked into one native executable:

| Parser | Pin | API/task |
| --- | --- | --- |
| Ferromark | `8adecc1` | Trusted CommonMark plus the selected extension flags; fresh parser and owned output. |
| Bun Markdown | `76e9dcc6ad272a4fb1ee4a4dbbde4809201b71d1` | Native Rust parser with its original C++ Highway searches and C mimalloc support. |
| pulldown-cmark | `0.13.4` | Fresh event parser followed by HTML serialization. |
| Comrak | `0.54.0` | Fresh owned-output convenience API, with unsafe/raw HTML rendering enabled. |
| md4c | `65c6c9d72cebd9a731aaa5597414ce04d9ea5de3` | Original C parser and HTML renderer, callback appending to a fresh output vector. |

Options are constructed outside timing. Every timed call includes fresh parser
setup, parsing, rendering, output allocation, and destruction. Each output buffer
starts empty. The parsers retain their own growth and allocation strategies.
No cached AST is reused. Raw HTML is trusted/preserved throughout; this does not
measure Ferromark's secure-default policy or a matched sanitization pipeline.

| Configuration | Enabled extensions |
| --- | --- |
| `commonmark` | None. |
| `tables` | Tables only. |
| `strikethrough` | Double-tilde strikethrough only. |
| `task_lists` | Task lists only; used for verification and eligible neutral controls. |
| `gfm_overlap` | Tables, strikethrough, and task lists. |

Bare autolinks, tag filtering, heading IDs, and other extensions are disabled.
The GFM overlap is **not full GFM**. Exact effective option values are archived
in `options.jsonl.gz`, so similarly named preset flags are not the parity claim.
The tables-only and GFM-subset headline rows use the same input containing both
tables and strikethrough: the first leaves strikethrough literal, while the
second renders it and enables unused task-list parsing.

The [Bun provenance](../../benchmarks/bun-comparison/PROVENANCE.md) traces md4c
through Zig and Rust ports. Building this parser retains native code; it is not
a pure-Rust or JavaScript-runtime benchmark. The old stable/System-allocator
four-parser comparison is superseded for publication. Its [original table data
and adapter disclosures](2026-09-11-benchmark-refresh/historical-tables.json)
are preserved unchanged from the preceding commit. A lower or higher throughput
in the new table cannot be read as a production regression or improvement over
that old table: the corpus, compiler, allocator, policy, and lifecycle changed.

## Environment and timing

Apple M1 Pro, macOS 26.6.2, AC power, no concurrent benchmark or build processes.
The native comparison uses pinned `nightly-2026-07-20`, rustc 1.99.0-nightly
(`9f36de775`), LLVM 22.1.8, generic CPU target, optimization level 3, fat LTO,
one codegen unit, panic abort, and no PGO. Original Bun Highway support remains
available. C/C++ uses Apple Clang 21.0.0; cross-language LTO is not enabled.

All five parsers share Bun's pinned mimalloc
`6a64e1ba7f5b2130d4efccb67ec87fd0003f0f6a`. A benchmark-only header redirects
C-md4c's malloc/calloc/realloc/free calls to that same allocator. This isolates
an allocator difference, but also makes the shared Bun-native environment part
of the experiment. It does not describe normal Ferromark or C-md4c builds.
Highway is pinned to `2607d3b5b0113992fe84d3848859eae13b3b52c1`.

Every eligible case runs once with 80 alternating-order windows of at least
63 ms, totaling at least 5.04 seconds of measurement per parser/input after
three seconds of warmup. The timer checks after batches of 16 renders. The eight
preselected public cases run in two additional processes with rotated starting
parser order. Their published estimate is the **median of three run medians**.
The full raw windows remain available; run spread is the range of run medians
divided by their median, not a statistical confidence interval.

The long public protocol is separate from the short internal feature/GFM probes
and conventional Criterion suites below. Do not combine their numbers into one
ranking. All current claims are scoped to this Apple Silicon machine; no native
x86-64 comparison was performed. No fresh CPU samples were taken in this refresh;
the earlier profiles retain their historical context.

## Historical HTML gate and follow-up review

Before timing, all five parsers render every benchmark case and all 652 stored
CommonMark examples. Exact outputs and hashes are archived. A limited HTML
comparator permits entity spelling, void-tag slashes, attribute order, boolean
attribute serialization, equivalent table-alignment attributes/styles, and
ordinary HTML flow whitespace at known block boundaries (with literal/code text preserved). It preserves visible text, URLs, code
whitespace, IDs, classes, checkbox values, and substantive markup differences.
This is neither a browser DOM equivalence proof nor official conformance testing.

The original run admitted **32 of 42** cases. A subsequent
[output audit](2026-09-11-output-parity-audit.md) found overly strict whitespace
handling and an incorrect md4c task-list flag in our adapter. Fresh verification
with the corrected flag and revised flow-whitespace comparison admits **34 of 42**.
The generated [grouping evidence](2026-09-11-benchmark-refresh/parity-groups.json)
shows which parsers agree, independently of Ferromark as a reference.

| Cause among the original ten exclusions | Cases | Outcome |
| --- | --- | --- |
| Whitespace before nested lists | CommonMark 5k and 50k | All five now agree; no new timings collected for these two cases. |
| Bun borrows the last table's alignment | GFM overlap 5k, 50k, tables-5k | Other four agree; a minimized two-table example reproduces it. |
| Ferromark's reference-resolution work limit | CommonMark and GFM overlap 1m | Other four agree; Ferromark reports the limit and preserves remaining reference syntax literally. |
| Task-list renderer conventions | GFM overlap tasks, mixed features, task-only tasks | Same tasks and states after correcting md4c; paragraph placement and classes differ. |

The [audit](2026-09-11-output-parity-audit.md) distinguishes comparable task
workloads from identical HTML contracts. Whitespace alone no longer excludes a
case. The subsequent workload policy accepts the alignment discrepancy for timing
while retaining it in the HTML diagnostics; missing link work remains ineligible.

**Measurement correction:** original `gfm_overlap/*` and `task_lists/*` timings
used the wrong md4c option and are historical, not matched-option comparisons.
The one affected public row (`gfm_overlap/gfm-tables`) was remeasured for all five
parsers with the same three-run protocol. Its complete
[replacement run](2026-09-11-benchmark-refresh/native-corrected/metadata.json)
and [publication source map](2026-09-11-benchmark-refresh/publication-sources.json)
remain separate from the immutable original run. The other seven public cases
are unaffected. All public rows below are generated from their valid source;
output eligibility does not imply fresh timing coverage of every admitted case.

Under the original comparator, stored-spec exact/normalized matches were Ferromark 652/652,
Comrak 652/652, pulldown-cmark 630/652, Bun 649/649, and md4c 541/644 out of 652.
The mismatch example numbers and all output are retained in `verification.json`
and `spec.jsonl.gz`. These counts describe this comparator and these pinned
adapters; they are not universal feature or compliance scores.

## Measured native results

<!-- native-results:start -->
ferromark has the lowest measured median for: CommonMark · 2 KiB, CommonMark · 5 KiB, CommonMark · 10 KiB, Strikethrough only, CommonMark links and images, CommonMark entities and inline markup.
pulldown-cmark has the lowest measured median for: Tables only, GFM subset: tables + strikethrough.

These are results for the named inputs and contracts, not a universal parser ranking.

Latencies are medians of three run medians. Spread is the range of those
run medians divided by their median; it is not a confidence interval.
Output byte counts may differ because of accepted serialization or renderer differences.
The main README and homepage tables also report input throughput.

| Input / configuration | Parser | Time / document | Run medians (µs) | Spread | Output bytes |
| --- | --- | ---: | --- | ---: | ---: |
| CommonMark · 2 KiB | ferromark | 9.784 µs | 9.801, 9.784, 9.764 | 0.4% | 3033 |
| CommonMark · 2 KiB | Bun (native) | 14.764 µs | 14.782, 14.719, 14.764 | 0.4% | 3033 |
| CommonMark · 2 KiB | pulldown-cmark | 10.130 µs | 10.144, 10.126, 10.130 | 0.2% | 3033 |
| CommonMark · 2 KiB | comrak | 22.264 µs | 22.231, 22.325, 22.264 | 0.4% | 3033 |
| CommonMark · 2 KiB | md4c (C) | 12.091 µs | 12.091, 12.087, 12.103 | 0.1% | 3033 |
| CommonMark · 5 KiB | ferromark | 24.886 µs | 24.921, 24.846, 24.886 | 0.3% | 7735 |
| CommonMark · 5 KiB | Bun (native) | 37.232 µs | 37.331, 37.163, 37.232 | 0.5% | 7735 |
| CommonMark · 5 KiB | pulldown-cmark | 27.590 µs | 27.590, 27.478, 27.660 | 0.7% | 7735 |
| CommonMark · 5 KiB | comrak | 57.654 µs | 57.611, 57.766, 57.654 | 0.3% | 7735 |
| CommonMark · 5 KiB | md4c (C) | 29.823 µs | 29.823, 29.758, 29.846 | 0.3% | 7735 |
| CommonMark · 10 KiB | ferromark | 50.629 µs | 50.629, 50.652, 50.563 | 0.2% | 15626 |
| CommonMark · 10 KiB | Bun (native) | 75.236 µs | 75.428, 75.137, 75.236 | 0.4% | 15626 |
| CommonMark · 10 KiB | pulldown-cmark | 55.587 µs | 55.593, 55.539, 55.587 | 0.1% | 15626 |
| CommonMark · 10 KiB | comrak | 118.706 µs | 118.706, 118.690, 118.750 | 0.0% | 15626 |
| CommonMark · 10 KiB | md4c (C) | 60.192 µs | 60.039, 60.192, 60.273 | 0.4% | 15626 |
| Tables only | ferromark | 46.178 µs | 47.011, 46.136, 46.178 | 1.9% | 12800 |
| Tables only | Bun (native) | 48.093 µs | 48.456, 48.093, 47.911 | 1.1% | 12320 |
| Tables only | pulldown-cmark | 35.228 µs | 36.051, 35.228, 35.129 | 2.6% | 11920 |
| Tables only | comrak | 135.257 µs | 135.915, 135.257, 134.999 | 0.7% | 12800 |
| Tables only | md4c (C) | 48.208 µs | 48.244, 48.208, 48.125 | 0.2% | 12800 |
| Strikethrough only | ferromark | 25.492 µs | 25.809, 25.436, 25.492 | 1.5% | 7300 |
| Strikethrough only | Bun (native) | 36.596 µs | 36.733, 36.419, 36.596 | 0.9% | 7300 |
| Strikethrough only | pulldown-cmark | 33.794 µs | 34.013, 33.794, 33.575 | 1.3% | 7300 |
| Strikethrough only | comrak | 72.102 µs | 72.144, 71.906, 72.102 | 0.3% | 7300 |
| Strikethrough only | md4c (C) | 28.158 µs | 28.224, 28.045, 28.158 | 0.6% | 7300 |
| GFM subset: tables + strikethrough | ferromark | 48.186 µs | 48.021, 48.222, 48.186 | 0.4% | 13360 |
| GFM subset: tables + strikethrough | Bun (native) | 54.136 µs | 54.132, 54.136, 54.181 | 0.1% | 12880 |
| GFM subset: tables + strikethrough | pulldown-cmark | 43.635 µs | 43.551, 43.635, 43.677 | 0.3% | 12480 |
| GFM subset: tables + strikethrough | comrak | 151.253 µs | 151.253, 151.046, 151.290 | 0.2% | 13360 |
| GFM subset: tables + strikethrough | md4c (C) | 56.118 µs | 56.074, 56.138, 56.118 | 0.1% | 13360 |
| CommonMark links and images | ferromark | 26.667 µs | 26.800, 26.632, 26.667 | 0.6% | 7350 |
| CommonMark links and images | Bun (native) | 38.253 µs | 38.374, 38.212, 38.253 | 0.4% | 7350 |
| CommonMark links and images | pulldown-cmark | 28.600 µs | 28.714, 28.600, 28.562 | 0.5% | 7350 |
| CommonMark links and images | comrak | 57.406 µs | 57.293, 57.412, 57.406 | 0.2% | 7350 |
| CommonMark links and images | md4c (C) | 41.000 µs | 41.018, 40.903, 41.000 | 0.3% | 7210 |
| CommonMark entities and inline markup | ferromark | 56.133 µs | 56.317, 56.109, 56.133 | 0.4% | 8400 |
| CommonMark entities and inline markup | Bun (native) | 64.853 µs | 64.856, 64.804, 64.853 | 0.1% | 8400 |
| CommonMark entities and inline markup | pulldown-cmark | 63.527 µs | 63.693, 63.527, 63.443 | 0.4% | 7900 |
| CommonMark entities and inline markup | comrak | 95.446 µs | 95.446, 95.295, 95.510 | 0.2% | 8400 |
| CommonMark entities and inline markup | md4c (C) | 57.269 µs | 57.373, 57.255, 57.269 | 0.2% | 8400 |
<!-- native-results:end -->

## Feature and lifecycle costs

The separate feature replay uses the **frozen 140-scenario, 292-pair catalog**
from September 10, including its original README content. This avoids changing
the workload simply because the README gains new results. It covers all 21
boolean Markdown options, link-base rewriting, core CommonMark constructs,
light documents, and fresh versus retained Renderer lifecycles.

The [generated feature tables](2026-09-11-markdown-feature-costs.md) distinguish
activation on byte-identical HTML from syntax actually interpreted and rendered.
Short timing windows are five runs of at least 60 ms, with alternating off/on
order and 16 warmup renders. The GFM probe separately replays every corpus,
preset, and owned/buffer/Renderer lifecycle in five windows of at least 75 ms.
Both probes verify lifecycle output before timing.

Timings use preserved, uninstrumented release-debug executables. Allocation
counters use a separate profiling-feature executable; allocation calls and
cumulatively requested bytes are not peak or live memory. These use the stable
Rust 1.97.1, Apple M1/NEON, System-allocator environment. They answer internal
API/workload questions and do not feed the native five-parser tables.

<!-- supplemental-results:start -->
The replay completed 416 retained Criterion cases, 2920 feature
timing windows, 584 allocation-counter observations, and 1350 GFM timing
windows. Feature and GFM lifecycle output checks passed before measuring.

| Suite | Measured cases | Protocol / scope |
| --- | ---: | --- |
| Ferromark parser, options, footnotes, custom code renderer, MDX | 64 | One Criterion run; 80 samples, 5 s measurement, 3 s warmup. Footnotes overrides sample count to 10 and pathological parsing to 20. |
| Byte search, external crate and embedded module | 216 | Built-in 30 samples, 300 ms measurement, 100 ms warmup; primitive diagnostics. |
| pulldown comparison and profiling matrices | 41 | One Criterion run, 80 samples, 5 s measurement, 3 s warmup; selected feature intersections, with separate secure-default lanes. |
| Legacy four-parser suite | 95 | One shorter diagnostic run, 30 samples, 1 s measurement, 1 s warmup; historical unequal policies/lifecycles, excluded from public rankings. |

Criterion estimates and individual samples are archived in `criterion-summary.json`
and `criterion-raw.jsonl.gz`. These are fresh snapshots, not paired production
before/after comparisons. The native tables alone use the three-run publication
protocol. The root suites inherit the root optimized bench profile; the isolated
comparison manifests retain their own Cargo bench profiles. Exact commands,
configurations, compiler details, and source hashes are in the metadata.

### GFM preset and lifecycle replay

Medians of five short timing windows, in microseconds per document. The last
column indicates whether enabling GFM leaves the CommonMark output byte-identical.
Changed output includes additional semantic work; it is not pure option overhead.

| Input | Bytes | CommonMark fresh owned | GFM fresh owned | GFM retained Renderer | Identical HTML |
| --- | ---: | ---: | ---: | ---: | --- |
| plain | 10368 | 9.465 | 10.950 | 9.806 | yes |
| commonmark-5k | 5632 | 19.490 | 22.791 | 20.582 | no |
| readme | 48414 | 134.045 | 142.502 | 130.254 | no |
| commonmark-50k | 52342 | 173.345 | 195.543 | 188.392 | no |
| gfm-overlap-tables | 5200 | 32.193 | 49.210 | 44.996 | no |
| tables-5k | 4552 | 13.051 | 36.535 | 33.916 | no |
| autolinks | 7776 | 7.201 | 41.618 | 39.129 | no |
| tasks | 5376 | 53.272 | 48.119 | 42.681 | no |
| mixed-gfm | 8112 | 53.024 | 68.105 | 64.042 | no |
<!-- supplemental-results:end -->

In this replay, nested/unordered lists and entities are among the most expensive
core synthetic workloads per input KiB, while plain text is cheapest. Input
structure, byte count, and HTML expansion differ, so these are profiling targets,
not costs that can be added to predict an arbitrary document. Literal-autolink
detection again has a visible activation cost on neutral prose. Small activation
deltas need longer confirmation before motivating an implementation change.
The task-list fixture is faster under GFM than under CommonMark in this replay,
while its HTML changes. Enabling additional syntax does not necessarily increase
elapsed time; presets specify semantics rather than performance tiers.

The [earlier CPU profiles](2026-09-10-gfm-profiling.md) and
[optimization decisions](2026-09-10-markdown-feature-optimizations.md) explain
previous hypotheses and accepted tradeoffs. This rerun does not by itself
establish a before/after gain or identify a new causal bottleneck.

## Evidence and reproduction

The [native evidence directory](2026-09-11-benchmark-refresh/native/) contains
source and binary hashes, compiler metadata, the resolved Cargo lockfile, all
inputs, exact output, options, verification results, raw windows, and summaries.
Large raw files are gzip-compressed without losing individual observations.
The [supplemental directory](2026-09-11-benchmark-refresh/supplemental/) records
probe outputs and hashes, frozen feature/GFM inputs, every command and exit code,
and Criterion estimates and samples. A hash manifest covers the saved evidence.

See the [native harness instructions](../../benchmarks/bun-comparison/README.md)
for pinned source setup, native support builds, and lockfile replay. The initial
Bun lockfile resolution updated the local Ferromark package from 0.7 to 0.8;
the recorded resolved lockfile was then rebuilt successfully with `--locked`
before verification and timing. Production parser sources remain unchanged.
The later output audit deliberately corrected the adapter and whitespace
comparator, retaining the original run and its verification as historical evidence.

```bash
python3 benchmarks/bun-comparison/run.py "$BUN_BENCH_DIR" /private/tmp/new-benchmark-run
python3 benchmarks/bun-comparison/publish.py docs/reports/2026-09-11-benchmark-refresh/native
python3 benchmarks/bun-comparison/publish.py --check
```

The publisher recalculates run medians from raw samples, rechecks original
outputs, rejects screening/incomplete data, and generates both README and
homepage JSON. CI checks the generated values without building Bun. Normal
library and package builds acquire no Bun, Highway, mimalloc, or C-md4c dependency.

## Validation

- `cargo test --locked --all-features`: 979 passed; three existing ignored tests.
- `cargo test -p ferro-byte-search --locked`: five tests passed, including its doctest.
- `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` and
  `cargo fmt --all --check` passed.
- All 117 Node repository contract tests and 12 native verification/publication
  tests passed. The early-exit profiling fixture now waits for its test process
  to finish before advancing the simulated startup pause; a fixed 100 ms delay
  had been shorter than process startup on this macOS host. Production profiling
  scripts and the measured parser implementation are unchanged.
- The README structure, measured-number generator, archived evidence hashes,
  workflow pins, generated family blocks, and script formatting passed.
- Homepage type checking, production build, six-page prerender verification,
  and the production dependency audit at the high-severity threshold passed.
- The historical final feature report still regenerates byte-for-byte with its
  original inputs and revision; the replay option does not relabel old evidence.
