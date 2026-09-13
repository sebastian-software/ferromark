# Practical documentation workflows across the native engine field

## Documentation collection results

Twelve actual documentation files, 53,656 input bytes, with trusted
CommonMark plus tables, strikethrough, and tasks. Every timed row completes the entire
collection. Native defaults for allocation, GC, and output representation
remain in place; Markdig returns UTF-16 strings, the other workers UTF-8.

| Engine | Release each: time | Keep all: time | Release each: peak process RSS | Keep all: peak process RSS |
| --- | ---: | ---: | ---: | ---: |
| Ferromark | 137.4 µs | 138.0 µs | 3.0 MiB | 3.1 MiB |
| pulldown-cmark | 173.5 µs | 173.9 µs | 3.1 MiB | 3.2 MiB |
| Comrak | 530.8 µs | 528.3 µs | 3.5 MiB | 3.5 MiB |
| md4c | 195.7 µs | 196.4 µs | 2.6 MiB | 2.7 MiB |
| cmark | Not comparable (9/12 complete documents) | — | — | — |
| cmark-gfm | 454.1 µs | 456.0 µs | 2.6 MiB | 2.8 MiB |
| Goldmark | 606.4 µs | 606.6 µs | 14.7 MiB | 14.4 MiB |
| Sätteri | 288.3 µs | 287.8 µs | 3.8 MiB | 3.8 MiB |
| Rushdown | 384.6 µs | 385.3 µs | 3.3 MiB | 3.3 MiB |
| Markdig | 330.2 µs | 329.5 µs | 70.3 MiB | 70.3 MiB |
| markdown-rs | 3829.7 µs | 3815.0 µs | 5.7 MiB | 5.5 MiB |
| Ox Content | **87.8 µs** | **87.4 µs** | **2.5 MiB** | **2.6 MiB** |

Ox Content had the lowest collection time with immediate release in this run: 87.8 µs. Ferromark took 137.4 µs. This result applies to the complete archived workload, not every Markdown application.

Bun's native support uses its pinned nightly compiler and shared mimalloc.
This separate environment has its own freshly measured Ferromark baseline:

| Engine | Release each: time | Keep all: time | Release each: peak process RSS | Keep all: peak process RSS |
| --- | ---: | ---: | ---: | ---: |
| Ferromark (Bun support) | **128.1 µs** | **129.1 µs** | 3.1 MiB | 3.1 MiB |
| Bun (native) | 331.8 µs | 332.5 µs | **3.0 MiB** | **3.0 MiB** |

Bold marks the lowest unrounded observation in each column and environment.

**Process RSS is a different memory measurement from the Rust heap table.**
It includes the runtime/JIT, stacks, input, allocator/GC reserves, and worker
infrastructure. Each cell is the median of three whole-process peaks during
startup, warmup, repeated collection work, and shutdown. It is not incremental
parser memory, a single-request peak, or a concurrent-service capacity estimate.

Timing still surrounds only completed Markdown work. GC in those windows is
included; OS peak-RSS accounting needs no instrumented allocator. Collection
timings and the earlier Rust heap measurements are separate fresh runs.

Ox Content's extra generated heading IDs may be admitted as additional output;
content, heading levels, links, tables, and checkbox states must remain intact.
cmark's core-only dialect cannot complete the GFM collection. No input is
removed to obtain a timing row. Full secure-preview and metadata adapters are
currently measured for Ferromark, pulldown-cmark, and Comrak only; the wider
HTML-only collection comparison does not establish those additional contracts.

[All engine versions, reviewed output differences, variation, and raw evidence](#engine-versions-and-compiler-settings).

## Engine versions and compiler settings

| Engine | Measured version / revision |
| --- | --- |
| Ferromark | 0.9.0 |
| pulldown-cmark | 0.13.4 |
| Comrak | 0.54.0 |
| md4c | 65c6c9d72ceb |
| cmark | 0.31.1 · bb3678d7a73c |
| cmark-gfm | 0.29.0.gfm.13 · 587a12bb54d9 |
| Goldmark | v2.0.2 |
| Sätteri | 0.2.13 (native crate); fork 0.6.3 |
| Rushdown | 0.18.0 |
| Markdig | 1.3.2 |
| markdown-rs | 1.0.0 |
| Ox Content | 3.2.0 |
| Ferromark (Bun support) | 0.9.0 |
| Bun (native) | 76e9dcc6ad27 |

Full lockfiles and compiler details are in `build.json.gz`. Rust compiler:
`rustc 1.97.1 (8bab26f4f 2026-07-14)`. Bun compiler:
`rustc 1.99.0-nightly (9f36de775 2026-07-19)`.
`go version go1.27.1 darwin/arm64`; .NET SDK 10.0.401;
`Apple clang version 21.0.0 (clang-2100.3.34.2)`.

## Provenance and reproduction

Measured source: `9ad64811e8152ed88d20967f9222f48e64642966`. Host: Apple M1 Pro.

See [the field harness](../../../benchmarks/workflows/field/README.md).
`build.json.gz` records exact Cargo, Go, and NuGet locks, upstream revisions,
build commands, compiler settings, native-support hashes, and executable hashes.
`corpus.json` preserves the exact earlier inputs and their source provenance.
`outputs.json.gz` retains every engine's original HTML and effective options
for both lifetimes, including engines ineligible for timing. `admission.json`
records every document, with no timing-based selection or corpus trimming.

Three fresh process rounds warm each variant for 3 seconds, then rotate
80 windows of at least 63 ms. Four whole collections run between clock
checks. `windows.json.gz` and `warmups.json` retain all observations and
native output lengths; `rss.json` stores the OS-reported peak and process
CPU usage for each worker. `observations.json` records host activity and
power. `checksums.json` protects every evidence input. All tables are
recomputed from these records by `field/publish.py --check`.

## Round variation and process-memory range

| Engine / lifetime | Round medians (µs) | Round spread | Peak RSS range |
| --- | --- | ---: | ---: |
| Ferromark / stream | 137.4, 136.5, 137.4 | 0.7% | 3.0–3.2 MiB |
| Ferromark / retain | 138.0, 137.4, 138.6 | 0.9% | 3.0–3.3 MiB |
| pulldown-cmark / stream | 173.5, 173.4, 173.6 | 0.1% | 3.1–3.1 MiB |
| pulldown-cmark / retain | 173.9, 173.9, 173.4 | 0.3% | 3.0–3.2 MiB |
| Comrak / stream | 525.8, 530.8, 532.8 | 1.3% | 3.4–3.5 MiB |
| Comrak / retain | 526.6, 528.9, 528.3 | 0.4% | 3.5–3.5 MiB |
| md4c / stream | 196.1, 195.0, 195.7 | 0.5% | 2.6–2.7 MiB |
| md4c / retain | 196.4, 195.2, 196.4 | 0.6% | 2.6–2.7 MiB |
| cmark-gfm / stream | 455.6, 454.1, 453.4 | 0.5% | 2.6–2.7 MiB |
| cmark-gfm / retain | 456.0, 455.3, 457.2 | 0.4% | 2.7–2.8 MiB |
| Goldmark / stream | 600.4, 606.9, 606.4 | 1.1% | 14.5–14.8 MiB |
| Goldmark / retain | 603.4, 610.1, 606.6 | 1.1% | 14.1–15.1 MiB |
| Sätteri / stream | 288.3, 287.9, 288.5 | 0.2% | 3.7–3.8 MiB |
| Sätteri / retain | 287.7, 287.8, 288.5 | 0.3% | 3.8–3.8 MiB |
| Rushdown / stream | 384.8, 384.6, 383.1 | 0.4% | 3.2–3.4 MiB |
| Rushdown / retain | 385.3, 384.0, 386.7 | 0.7% | 3.3–3.4 MiB |
| Markdig / stream | 330.2, 324.6, 330.9 | 1.9% | 69.7–70.9 MiB |
| Markdig / retain | 334.5, 326.5, 329.5 | 2.4% | 70.1–70.5 MiB |
| markdown-rs / stream | 3839.8, 3806.4, 3829.7 | 0.9% | 5.7–5.9 MiB |
| markdown-rs / retain | 3829.6, 3808.2, 3815.0 | 0.6% | 5.5–5.8 MiB |
| Ox Content / stream | 87.8, 87.1, 88.4 | 1.5% | 2.5–2.5 MiB |
| Ox Content / retain | 87.4, 86.7, 87.4 | 0.8% | 2.6–2.6 MiB |
| Ferromark (Bun support) / stream | 128.1, 128.1, 128.9 | 0.7% | 3.1–3.1 MiB |
| Ferromark (Bun support) / retain | 129.1, 128.3, 129.2 | 0.7% | 3.1–3.1 MiB |
| Bun (native) / stream | 331.8, 330.0, 332.9 | 0.9% | 3.0–3.0 MiB |
| Bun (native) / retain | 332.5, 330.2, 332.9 | 0.8% | 3.0–3.0 MiB |

This is one Apple Silicon workstation, not a cross-machine ranking. Background
desktop and OS work is recorded; the CPU was not pinned. .NET uses normal
workstation concurrent GC and tiered JIT/PGO. Go uses normal automatic GC
with cgo and static PGO disabled. Rust and C use generic release builds.
The RSS totals include startup and harness overhead and must not be treated
as intrinsic parser allocation sizes. The exact heap accounting in the
[separate Rust workflow report](../2026-09-13-workflow-comparisons/REPORT.md) answers that narrower question.
