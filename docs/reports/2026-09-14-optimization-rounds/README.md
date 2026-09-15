# Optimization rounds: exact SIMD scans and less repeated work

**Decision: promote five optimizations locally, one per commit.** The combined
core improves complete processing on the frozen mixed corpus. This is not a
claim that every stage or input wins: longer follow-ups retain fresh-allocation
and render-only regressions below. All twelve registered experiments, rejected
patches, failed correctness variants, and raw measurements are archived.

Reference: `4de75d4` (the same core as `ae963d4`). Final measured core:
`adf891a`. Rust 1.95, Apple M1 Pro, generic AArch64, fat LTO. These are v2-versus-v2
measurements; Ferromark v1 was not remeasured.

## Complete processing across real inputs

The 57 frozen cases span 37–113,609 UTF-8 bytes: comments, project documents,
and Wikipedia-derived views. They retain the previous CommonMark/GFM comparison
options. Each case has equal weight in the geometric mean of its median paired
ratio. Three rounds × three pairs were measured; each window was 40 ms. Ratios
above 1 favor the candidate. Overlapping Wikipedia views are not independent
samples, and these descriptive ratios are not confidence intervals.

| Stage | Baseline / candidate | Throughput change |
| --- | ---: | ---: |
| New arena, owned HTML | 1.037× | +3.7% |
| Reused arena, borrowed HTML | 1.045× | +4.5% |
| Parse only, reused arena | 1.055× | +5.5% |
| Render prebuilt AST | 1.009× | +0.9% |

Reused complete processing measured above 1 in 56 of 57 cases, although many
small differences are within noise. Compiler Options is 0.995×. Examples:
Vue slots 39.79 → 35.46 µs (1.122×), the Chess article 194.60 → 175.52 µs
(1.091×), and the Volcano opening paragraph 4.23 → 3.74 µs (1.131×).
The 310-byte table comment is 1.013×; this does not establish that the earlier
gap against v1 has closed.

| Content group | Cases | Fresh | Reused | Parse | Render |
| --- | ---: | ---: | ---: | ---: | ---: |
| comments | 12 | 0.996× | 1.011× | 1.014× | 1.002× |
| encyclopedia | 12 | 1.111× | 1.122× | 1.158× | 0.995× |
| plain-prose | 4 | 1.011× | 1.005× | 1.012× | 1.001× |
| readme | 2 | 1.031× | 1.032× | 1.039× | 1.014× |
| reference | 4 | 1.022× | 1.015× | 1.014× | 1.010× |
| syntax-guard | 1 | 1.001× | 1.058× | 1.067× | 1.022× |
| technical-docs | 22 | 1.031× | 1.036× | 1.041× | 1.020× |

| Input bytes | Cases | Fresh | Reused | Parse | Render |
| --- | ---: | ---: | ---: | ---: | ---: |
| ≤512 | 12 | 0.999× | 1.020× | 1.026× | 1.004× |
| 513–2,048 | 9 | 1.054× | 1.066× | 1.080× | 1.014× |
| 2,049–16,384 | 20 | 1.041× | 1.046× | 1.055× | 1.017× |
| 16,385–65,536 | 13 | 1.049× | 1.044× | 1.056× | 1.005× |
| >65,536 | 3 | 1.075× | 1.071× | 1.093× | 0.976× |

A separate replay of the same 57 documents uses CommonMark plus renderer bare-URL
recognition: **1.048× complete processing and 1.015× rendering**. This profile is
explicitly different from GFM cases in the original matrix and is not pooled
with that matrix. [Original-profile rows](raw/final-broad/summary.csv) and
[autolink-profile rows](raw/final-default-autolink/summary.csv) include every case.

## Confirmed limits and longer controls

Six cases were selected after the broad run to revisit its weakest results.
The follow-up uses three rounds of five 100 ms pairs; the A/A control uses the
exact same baseline binary in both roles, three rounds of three 100 ms pairs.
No samples were removed.

| Case | Fresh | Complete reused | Render | A/A render |
| --- | ---: | ---: | ---: | ---: |
| comment-ack | 0.972× | 1.009× | 0.992× | 1.000× |
| comment-review | 0.997× | 1.008× | 0.999× | 1.002× |
| comment-unicode | 0.978× | 1.002× | 0.999× | 0.999× |
| typescript-handbook-compiler-options | 0.996× | 0.994× | 0.988× | 1.001× |
| wiki-chess-article-body | 1.096× | 1.086× | 0.948× | 1.008× |
| wiki-volcano-article-body | 1.107× | 1.125× | 0.971× | 1.015× |

The Chess render result means about **5.5% more render-only time**, despite 8.6%
more complete-processing throughput in the same follow-up. The Unicode comment
also needs about 2.2% more fresh-processing time. These are retained tradeoffs.
The tiny acknowledgment is especially noisy: its fresh A/A ratio is 0.979×, so
the 0.972× candidate ratio does not cleanly isolate a code effect. Conversely,
Chess render A/A is 1.008×, so its larger cross-binary loss is not dismissed as
ordinary role noise. See [all follow-up rows](raw/long-followup/summary.csv) and
[all A/A rows](raw/same-binary-control/summary.csv).

Promotion prioritizes verified complete processing across the mixed corpus and
extension workloads, with these isolated costs explicitly recorded. It does
not rely on the extreme synthetic speedups to claim a universal improvement.

## Targeted diagnostics in the combined build

These 93 authored diagnostics are excluded from the broad geometric means.
Final parser diagnostics used three rounds × three 30 ms pairs; renderer
diagnostics used three rounds × three 40 ms pairs. Selected results:

| Input / stage | Baseline / candidate |
| --- | ---: |
| All optional markers enabled, 16K plain text: parse | 1.517× |
| Sparse escaped-pipe table, 16K cell: parse | 3.829× |
| Formatted escaped-pipe table, 16K cell: parse | 1.049× |
| Long clean 2K URL: render | 3.038× |
| URL followed by 2,046 mixed closers: render | 127.089× |
| Repeated Unicode/CJK URLs: render | 0.996× |

The closer input uses the nominal 2,048 size rounded to complete three-bracket
runs; actual input bytes and SHA-256 are in the corpus. Its large result is the
combination of removing quadratic recounting and scanning the URL with SIMD,
not a typical Markdown-document speedup. [Parser rows](raw/final-parser-diagnostics/summary.csv)
and [renderer rows](raw/final-renderer-diagnostics/summary.csv) retain all controls.

## Experiment decisions and retained failures

All ratios below are baseline time / candidate time. Above 1 is faster. Screens
used one round of three 20–30 ms pairs and are exploratory, not confidence
intervals. The rank-map follow-up used three rounds of three 40 ms pairs. The
combined acceptance runs provide the final repeated evidence.

| ID | Attempt | Observation | Decision |
| --- | --- | --- | --- |
| 01A | Rebuild the earlier link probe with symbols; compare renderer function sizes and relocated instruction bytes. | All 42 matched renderer symbols have equal function lengths; relocated bytes differ. A partial PC-relative normalization is insufficient to establish identical instructions. | Diagnostic only; do not claim a precise cache/layout mechanism. |
| 01B | Disable LTO in the original worker, changing no source or corpus. | Chess/TypeScript render ratios move to 1.006/0.997. Repeating the original fat-LTO binaries gives 0.946/0.953. Chess full processing still gains 7.7% without LTO. | Renderer regression depends on build configuration; retain all observations. No project-wide LTO change. |
| 01C | Rebuild the link probe with the expanded worker, preserving the timed loops and optimizer barriers. | Chess/TypeScript render ratios become 1.027/1.001; Chess full processing 1.093. | Promote the original single-probe optimization after combined validation. The exact microarchitectural cause remains unproven. |
| 02A | Fuse optional markers using `memchr3` and one combined memo. | Sparse opt-7 parsing falls to 0.230. Sharing the memo makes the absent core-marker suffix get rescanned after every optional hit. | Reject. A fast search does not repair repeated work; preserve independent memoization or truly fuse classification. |
| 02B | One SIMD table containing every optional marker, followed by disabled-marker filtering. | All-enabled plain parsing reaches 1.508, but disabled-marker-heavy parsing falls to 0.079. Initial short tails also missed a one-byte `{`; the differential guard caught and corrected this. | Reject the retry strategy. Archive the corrected prototype and exact failure correction. |
| 02C | Eight exact option-specific nibble tables for core plus enabled markers. | All-enabled plain parsing reaches 1.514; worst screen is 0.984 on an unchanged TypeScript control. No disabled-marker retry loop. | Promoted after combined validation. NEON executed; x86 paths remain unmeasured. |
| 03A | Sparse list of removed-backslash positions, with binary-search remapping. | Valid-boundary tests pass, but independent review finds the old out-of-range clamp was lost. | Reject this initial correctness variant before timing. |
| 03B | Restore clamping in the sparse map. | Sparse 16K cells parse at 3.985, but formatted 16K cells at 0.746. | Reject for production. The per-span binary search is costly when a cell contains many inline nodes. |
| 03C | 64-byte removal bitsets plus prefix counts and `count_ones` for constant-time remapping. | Three-round ratios: sparse 16K parse 3.842, formatted 16K parse 1.055, dense 16K parse 1.011. | Promoted. Retains the memory reduction without the binary-search regression. |
| 04 | Lazy cached bracket counts for trailing URL punctuation. | A 2,048-closer diagnostic renders at 106.6×; work is bounded by three linear scans rather than a full recount per closer. | Promoted. Normal URLs and exact stopping behavior remain controls. |
| 05A | NEON skip over ASCII URL bytes until one of nine terminators or a high-bit byte. | A long 2K URL renders at 3.070×; Unicode URL rendering is 0.940. Initial Unicode continuation omitted a later ASCII terminator; differential tests caught and corrected it. | Revise. Preserve original prototype and failure correction. |
| 05B | Check the first byte before SIMD setup for an immediately non-ASCII IRI host. | Long 2K URL rendering remains 3.017×; Unicode screen improves to 0.976. Some unchanged render controls still move with executable layout. | Promoted after combined and real-document autolink validation; do not claim every Unicode case wins. |

The sparse and rank variants often reserve the same number of arena/system
bytes because `Allocator::for_source_len` preallocates at least 16 KB and otherwise
8× input length. This is not evidence that the map failed to shrink. A separate
untimed diagnostic measures occupied chunk bytes, including padding, while the
AST is alive. On the 16,044-byte sparse table, occupied arena bytes fall from
94,856 to 31,368 (66.9%); reserved capacity remains 131,008 bytes. Formatted tables
reduce occupied arena bytes by 7.3%. Global allocation counts and requested bytes
are archived separately and are not presented as reduced when unchanged.

## Correctness and instrumentation

The final combined source passes 638 Rust unit/integration/doc tests, the existing
snapshot and CommonMark/GFM baseline checks, format checking, strict workspace
Clippy, and all seven Criterion suite builds. No snapshot output or specification
failure baseline was changed. New guards cover all eight marker option masks,
all byte values, offsets, tails, UTF-8, scalar URL matching, over a million bracket
boundary comparisons, table span mapping, and defensive out-of-range clamping.
The marker and URL unsafe code retain portable fallbacks.

The measurement harness retains the previous timed loops, including
`black_box(&document)` in the parse lane. The expanded profile selection and
source snapshotting change the executable, so old-worker comparisons are kept
separate. Candidate sources, worker bytes, locks, binaries, toolchain, and build
configuration are hashed. Exact HTML, AST Debug (including spans), and children
are checked across candidates and stages before timing and around every live
batch; timing checksums are checked for every sample. Capacity differences are
retained as data, not grounds for excluding valid structural optimizations.

Harness review caught two inadequate initial diagnostic shapes before timing:
the proposed sparse table contained no escape, and the long-closer input repeated
short runs rather than creating a long run. Both were corrected and protected by
corpus-shape tests. The first source snapshot location nested a library workspace
inside the worker workspace, which Cargo rejected; snapshots now live beside the
workers. Failed zero-test filters were replaced with focused test invocations.
Eight harness guards test actual admission/protocol behavior, including corrupted
worker/binary rejection and independent AST/HTML/child-count mismatch rejection.

Timing ran sequentially without our compiler or agent test workloads. The host is
an Apple M1 Pro on AC power, Rust 1.95, generic AArch64, fat LTO unless explicitly
labeled otherwise. The sandboxed hardware/thermal probes are retained even when
they fail; a separately authorized CPU-name observation supplies the CPU model.
No CPU affinity or fixed clock was imposed. x86 SIMD and non-AArch64 fallbacks
were not executed or cross-compiled because only the AArch64 target is installed.


## Local commits and reproduction

| Commit | Change |
| --- | --- |
| `d7dec13` | perf(parser): skip unescaping clean link components |
| `1fbd0f0` | perf(parser): fuse enabled inline markers in exact SIMD tables |
| `eb8f45f` | perf(parser): compact table source spans with a constant-time rank map |
| `8e9373f` | perf(renderer): trim URL brackets in linear time |
| `adf891a` | perf(renderer): scan ASCII URL spans with NEON |

The promoted core was byte-compared with all 354 frozen core files from the
actual acceptance build. [Verification](raw/promoted-core-verification.json)
and [commit identities](raw/promoted-commits.json) are archived. Snapshot bodies,
fixture baselines, package publication settings, dependencies, and public APIs
are unchanged. Only a local repository is involved; nothing was pushed.

Use the [harness instructions](../../../benchmarks/optimization-rounds/README.md)
from the repository root. (Relative repository path:
`benchmarks/optimization-rounds/README.md`.) Exact per-suite commands, build
identities, per-round samples, output verification, stage capacities, allocation
counters, occupied arena bytes, and failed variants are under [raw](raw/) and
[patches](patches/). The frozen [150-case corpus](corpus.json.gz) is the initial
screen/diagnostic set; [207-case corpus](final-corpus.json.gz) adds the 57
autolink-profile replays. Original source attribution and licensing remain in
[the corpus documentation](../../../benchmarks/broad-comparison/README.md).
New authored diagnostics use this repository's MIT license.

[SHA256SUMS.json](SHA256SUMS.json) covers archived artifacts. The earlier source
and raw-corpus licenses remain unchanged; no corpus content was downloaded in
this round. All temporary source checkouts are isolated from Ferromark v1.
