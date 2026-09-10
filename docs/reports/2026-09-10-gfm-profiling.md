# GFM profiling before optimization — 2026-09-10

Subsequent implementations, rejected experiments, and final measurements are
recorded in the [optimization log](2026-09-10-gfm-optimizations.md).

The measurements identify three separate costs: repeated table-cell processing,
a temporary strikethrough allocation, and autolink detection on text without
links. No production implementation was changed. The next bounded experiment
should reuse the strikethrough opener scratch buffer, followed by investigation
of per-cell work. These are hypotheses to test, not demonstrated speedups.

## Scope and method

Baseline: `9c766f11a07cfe65af441d1a147cbfe5703d60b1`. Apple M1 Pro,
macOS 26.6.2, Rust 1.97.1 / LLVM 22.1.6, `release-debug` (optimized, fat LTO,
one codegen unit), repository `apple-m1`/NEON target settings, System allocator.
All syntax presets retain `RenderPolicy::Untrusted`.

The README's “CommonMark 5 KB” label names a fixture, not the CommonMark preset:
its benchmark already enables tables, strikethrough, and task lists. The earlier
[native Bun comparison](2026-09-05-bun-comparison.md) also used this shared GFM
subset, with autolinks disabled. Its table case favored pulldown-cmark, but did
not establish a general full-GFM ranking. That run used a different compiler,
allocator, CPU configuration, security policy, and output lifecycle. This report
isolates ferromark's own costs; it does not rerank competitors or establish the
cause of their relative advantage.

Three measurements were kept separate:

- Uninstrumented timings: seven inputs, ten syntax configurations, three API
  lifecycles; five windows of at least 75 ms each, 16 warmup renders per window,
  clock checked every 16 renders, rotated/reversed configuration order.
- Allocation and pipeline counts: one warmed render per combination in a separate
  build with the `profiling` feature and a counting System allocator. Requested
  bytes are cumulative allocation/reallocation sizes, not peak or live memory.
- CPU sampling: nine four-second macOS `sample` captures of the uninstrumented
  existing `profile_harness`, with fresh parser state and reused output.

Owned-output, reused-output, and retained-Renderer APIs produced byte-identical
HTML within every configuration. Cross-configuration equality was checked
separately for full GFM versus GFM without autolinks.

## Timing results

Microseconds per render, median of five window means; lower is better.
This table uses **fresh parser state and reused output**. “Overlap” enables
only tables, strikethrough, and task lists. Full GFM additionally enables
literal autolinks and the disallowed-HTML filter.

| Input | CommonMark | Overlap | Full GFM | GFM without autolinks |
| --- | ---: | ---: | ---: | ---: |
| Plain prose | 9.59 | 9.60 | 10.73 | 9.58 |
| CommonMark 50 KB fixture | 171.57 | 181.73 | 191.93 | 181.04 |
| Earlier comparison's table/strike input | 33.36 | 48.00 | 50.90 | 48.04 |
| Tables 5 KB fixture | 13.45 | 35.07 | 37.78 | 35.33 |
| Literal URLs, www links, and email | 7.46 | 7.47 | 42.20 | 7.43 |
| Task lists | 52.35 | 46.06 | 48.65 | 45.79 |
| Mixed GFM | 53.72 | 58.33 | 67.47 | 58.63 |

CommonMark renders GFM syntax differently, so the table/URL ratios are **not**
estimates of avoidable overhead. Full GFM and GFM without autolinks produce
identical HTML for the first four rows and the task-list row. For those inputs,
autolink detection adds approximately 6–12% to elapsed time. The URL and mixed
rows include actual additional link rendering and cannot support that inference.

## Findings and next experiments

1. **Strikethrough has a specific avoidable allocation candidate.** On the earlier
   table/strike input, the warmed Renderer allocates zero times with tables alone,
   but 80 times / 2,560 requested bytes with strikethrough enabled. Enabling
   strikethrough without tables also produces those 80 allocations. The local
   `openers: Vec<usize>` in `resolve_strikethrough_into` explains one 32-byte
   allocation per affected inline input. CPU stacks show allocation and free
   underneath this resolver. Test retaining that scratch buffer alongside the
   existing retained match buffer, with correct resets between cells/documents.
   The allocation count is not a prediction of the resulting time improvement.

2. **Tables warrant structural profiling, not a blanket allocation rewrite.**
   The tables fixture performs 402 inline parses, 330 taking the plain-text fast
   path. Top-of-stack samples attribute 373/3,308 (11.3%) to `split_table_cells`
   and 201/3,308 (6.1%) to `CellState::add_text`; inline parsing and rendering also
   consume substantial samples. The warmed Renderer allocates zero times on this
   fixture. Ordinary cells already borrow source ranges. Investigate repeated
   scans, per-cell setup, and event handoffs before changing data ownership or
   introducing another SIMD backend.

3. **Negative autolink detection is a separate full-GFM cost.** Plain prose takes
   10.73 µs with full GFM versus 9.58 µs without autolinks, with identical output.
   The 50 KB fixture shows 191.93 versus 181.04 µs. Sampled inline-parser source
   locations include the autolink candidate gate. Investigate reducing repeated
   candidate checks or carrying already-established negative results. The
   existing three-pass detector deliberately avoids a combined search that
   previously regressed on x86 AVX2; do not assume fewer passes means faster.
   This cost cannot explain the earlier overlap comparison, where autolinks were
   disabled.

4. **Actual autolinks also spend time rendering destinations.** On the URL input,
   `url_escape_link_destination_raw` accounts for 606/3,323 top-of-stack samples
   (18.2%). A later experiment can investigate repeated destination scans while
   preserving escaping and URL security behavior. Disabling link semantics or
   changing the trust policy would not demonstrate an optimization.

Task lists are not the first target on this evidence: full GFM is faster than
CommonMark on the task fixture. Task processing removes markers before inline
parsing, and plain-text fast paths increase from 96 to 192 out of 288 calls.
This is consistent with reduced inline work; it is not a general task-list ranking.

## Evidence, limits, and retention

[Compact evidence](2026-09-10-gfm-profiling/summary.json) preserves all five
reused-output timing samples for each configuration, selected warmed allocation
counts, CPU sample counts, output-equivalence results, fixture recipes/hashes,
and source identity. The [archived probe and full raw profiles](2026-09-10-gfm-profiling/raw/) preserve
the original experiment, including fixture contents and unrounded timing data.
They are evidence snapshots, not production benchmark infrastructure.
The checked-in `examples/profile_harness.rs` supports follow-up CPU sampling,
for example with `benches/fixtures/tables-5k.md gfm 0 --forever`.

These are exploratory measurements on one ARM64 machine, with short timing
windows and synthetic extension-heavy inputs. Sampling percentages are symbol
attributions affected by inlining, not precise independent phase timings or
recoverable speedups. CPU profiles use fresh parser state; warmed allocation
counts deliberately use a retained Renderer to expose residual allocation.
The HTML filter was toggled, but raw-HTML-specific workloads were not profiled.
Validate any implementation separately with longer paired before/after runs,
real-world GFM documents, unchanged HTML, and x86-64 coverage before publication.

This is a profiling report rather than an ADR: no architectural decision has yet
been made. Keep the measured evidence and priorities; record an ADR if a later
experiment justifies changing the parser architecture.
