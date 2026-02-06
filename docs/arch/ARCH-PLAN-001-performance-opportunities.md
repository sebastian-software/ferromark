# ARCH-PLAN-001: ferromark Performance Opportunities

## Goal

Increase end-to-end Markdown-to-HTML throughput while preserving full CommonMark compliance.

## Evidence source

This plan is calibrated against `/Users/sebastian/Workspace/md-new/PERF_ATTEMPTS.md` so we avoid re-running low-yield micro-optimizations.

## Baseline and guardrails

- Benchmark command: `cargo bench --bench comparison`
- Primary datasets: `REFS`, `MIXED`, `COMMONMARK_5K`, `COMMONMARK_20K`, `COMMONMARK_50K`
- Correctness gate: `cargo test`
- Rule: no regression in spec compliance for optimization merges.
- Rule: skip already-attempted micro-tweaks unless new profiling data points to a new hotspot.

## Priority workstreams

### P0: Link reference cost reduction (highest expected ROI)

Context:
- Link reference extraction and resolution currently performs several avoidable allocations and repeated scans.
- This is the main reason the "Link reference processing cost" row is not a clear ferromark advantage.

Tasks:
1. Remove double hash lookup in `LinkRefStore::insert`.
   - Current path in `/Users/sebastian/Workspace/md-new/src/link_ref.rs` does `contains_key` followed by `insert`.
   - Use single-lookup `entry` flow.
2. Avoid paragraph-wide copy before definition parsing.
   - Current extraction path builds a temporary paragraph buffer in `/Users/sebastian/Workspace/md-new/src/block/parser.rs`.
   - Parse directly from existing ranges or through a lightweight paragraph cursor.
3. Replace early `Vec<u8>` copies in link reference parser with ranges.
   - Current `ParsedLinkRefDef` owns `Vec<u8>` fields in `/Users/sebastian/Workspace/md-new/src/block/parser.rs`.
   - Return ranges and materialize bytes only when definition is accepted.
4. Remove repeated nested scanning in reference candidate checks.
   - `contains_ref_link_candidate` in `/Users/sebastian/Workspace/md-new/src/inline/links.rs` currently re-scans opens/closes and re-normalizes labels.
   - Precompute candidate ranges once per inline parse pass.

Success criteria:
- `REFS` throughput +15% or better.
- `MIXED` throughput +5% or better.
- No throughput regression on `COMMONMARK_50K`.

### P1: Label normalization micro-optimizations

Context:
- Label normalization is called in both block extraction and inline resolution.
- PERF_ATTEMPTS already includes multiple ASCII/SIMD label-normalization variants; most were noise/regressions, one ASCII fast path was already kept.

Tasks:
1. Focus on reducing normalization call count (algorithmic), not on further byte-level micro-tuning.
2. Reuse parser-owned scratch state only where profiling shows allocation pressure and where previous regressions are avoided.
3. Add dedicated benchmark cases with heavy escaped/entity labels to confirm real wins on `REFS`-like workloads.

Success criteria:
- Reference-heavy microbench +10% normalization throughput.
- No measurable regression on non-reference documents.

### P1: Reference resolution algorithm simplification

Context:
- Current matching logic combines binary searches and repeated range checks.
- PERF_ATTEMPTS already tried several local tweaks in `contains_ref_link_candidate` and open/close lookup paths with no gain.

Tasks:
1. Prioritize structural simplification (single-pass or precomputed bracket pairing) over local lookup tweaks.
2. Reduce or remove fallback nested-candidate rescans by improving up-front candidate modeling.
3. Keep occupied range checks sorted and measurable with profile-driven validation.

Success criteria:
- Reduced CPU samples in `/Users/sebastian/Workspace/md-new/src/inline/links.rs`.
- Improved worst-case bracket-heavy benchmark behavior.

### P2: Measurement and regression automation

Tasks:
1. Add a focused bench group for link reference extraction/resolution only.
2. Save benchmark snapshots before/after each workstream.
3. Add a simple perf regression check in CI for key fixtures.

Success criteria:
- Repeatable before/after evidence per optimization PR.
- Early detection of throughput regressions.

### P3: Platform and compiler-level throughput tuning

Context:
- PGO has already shown strong wins and is now an established benchmark strategy.
- Several NEON/SIMD escape/scan experiments were already tested and mostly reverted due to no clear gain.

Tasks:
1. Keep non-PGO and PGO benchmark tracks documented and reported separately for fair comparisons.
2. Evaluate `simdutf8` only if profiling attributes meaningful time to UTF-8 validation.
3. Test selective loop unrolling only in profiled hot loops with strict A/B checks.

Success criteria:
- Measurable speedup on at least one primary dataset without regressions.
- Feature-gated or build-time toggles where platform-specific code is added.

## Already attempted: do not prioritize again (unless new evidence)

From `/Users/sebastian/Workspace/md-new/PERF_ATTEMPTS.md`:

- NEON escape scans and NEON URL-escape scans: no clear improvement (reverted).
- Multiple SIMD/ASCII label-normalization micro variants: mostly no gain or regression.
- `contains_ref_link_candidate` micro-tweaks (scratch string reuse, binary close lookup, nested-candidate precheck): no gain/regression.
- Lookup-local changes like passing open index to avoid one binary search: no clear improvement.

These should stay out of short-term roadmap unless a new profiler run shows changed hotspot distribution.

## Suggested implementation order

1. P0 task 1 (single-lookup insert)
2. P0 task 2 (remove paragraph-wide copy)
3. P0 task 3 (range-based parse output)
4. P0 task 4 (precompute ref candidates with structural redesign, not prior micro-tweaks)
5. P1 workstreams
6. P2 automation
7. P3 platform/compiler tuning

## Execution results (2026-02-06)

### Benchmark protocol used for all A/B decisions

- Correctness gate before each decision: `cargo test` (all green in all runs).
- Perf command: `cargo bench --bench comparison -- "(complexity/ferromark/(refs|mixed)|commonmark50k/ferromark)" --sample-size 60 --measurement-time 2`
- Decision rule: keep only if no `COMMONMARK_50K` regression and meaningful `REFS`/`MIXED` gain.

### Baseline snapshot

- `commonmark50k/ferromark`: `154.33 us`
- `complexity/ferromark/refs`: `2.4448 us`
- `complexity/ferromark/mixed`: `3.3006 us`

### P0 task-by-task outcomes

1. P0.1 single-lookup insert in `LinkRefStore::insert` (`entry` API)
- Result vs baseline: `refs` `+2.51%` (regression), `mixed` `+0.80%` (noise), `commonmark50k` `-0.75%` (noise).
- Decision: **discarded** (regresses primary `refs` target).

2. P0.3 range-based parsed definition output + deferred materialization
- Change kept in commit: `19acffc`
- File: `/Users/sebastian/Workspace/md-new/src/block/parser.rs`
- Result vs baseline: `refs` `-5.72%` time (`+6.06%` throughput), `mixed` `-1.39%` time (small win), `commonmark50k` no significant change.
- Decision: **kept + committed**.

3. P0.2 remove paragraph-wide copy before parsing definitions (incremental parse buffer)
- Result vs baseline: `refs` `+5.01%` (regression), `mixed` `+0.59%` (noise), `commonmark50k` `+1.19%` (noise/regression direction).
- Decision: **discarded**.

4. P0.4 precompute nested reference-link candidates once per inline pass
- Result vs baseline: `refs` `+6.91%` (regression), `mixed` `+3.02%` (regression), `commonmark50k` `+3.68%` (regression).
- Decision: **discarded**.

5. Post-profile `refs` pass (Time Profiler guided)
- Profiling evidence (`xctrace` Time Profiler on `^complexity/ferromark/refs$`):
  - Dominant samples were allocator-heavy (`_xzm_xzone_malloc_tiny`, `_xzm_free`, `_malloc_zone_malloc`).
  - ferromark hotspots included `extract_link_ref_defs`, `parse_link_ref_def`, and `normalize_label_into` in `/Users/sebastian/Workspace/md-new/src/block/parser.rs` and `/Users/sebastian/Workspace/md-new/src/link_ref.rs`.
- Attempt A: parser-owned reuse of paragraph parse buffer + parser-owned label scratch buffer.
  - Benchmark (`--sample-size 80 --measurement-time 4`) result: `refs` `-0.69%` (within noise), `mixed` `+0.48%` (within noise), `commonmark50k` no change.
  - Decision: **discarded** (no meaningful gain).
- Attempt B: parser-owned reuse of paragraph parse buffer only (kept label scratch local to avoid `String` clone in accepted-definition path).
  - Change kept in commit: `90b9fb2`
  - File: `/Users/sebastian/Workspace/md-new/src/block/parser.rs`
  - Benchmark (`--sample-size 80 --measurement-time 4`) result:
    - `commonmark50k/ferromark`: `153.67 us` (no significant change).
    - `complexity/ferromark/refs`: `2.3317 us` (`-3.46%` time, significant).
    - `complexity/ferromark/mixed`: `3.2546 us` (`-1.56%` time, significant).
  - Focused bench confirmation (`link_refs_focus`):
    - `refs`: `2.3595 us`
    - `refs_escaped`: `4.2747 us`
    - `mixed`: `3.2789 us`
  - Decision: **kept** (clear `refs` win, no `commonmark50k` regression).
- Attempt C: removed pre-sizing pass (`total_len`) and `reserve()` in `extract_link_ref_defs`.
  - Benchmark (`--sample-size 80 --measurement-time 4`) result: `refs` `+1.15%` (regression direction), `mixed` no significant change, `commonmark50k` no significant change.
  - Decision: **discarded**.

### Current `refs` position vs other libraries (2026-02-06)

- Snapshot command: `cargo bench --bench comparison -- "complexity/(ferromark|md4c|pulldown-cmark|comrak)/refs$" --sample-size 40 --measurement-time 2`
- Median times:
  - `ferromark`: `2.3726 us`
  - `md4c`: `2.4468 us`
  - `pulldown-cmark`: `1.8857 us`
  - `comrak`: `4.6969 us`
- Interpretation:
  - ferromark is currently faster than `md4c` and much faster than `comrak` on `refs`.
  - The remaining notable gap is vs `pulldown-cmark` (~`20-26%` faster depending on run), so there is still meaningful headroom.

### P1 and P2 progress

1. P1.1/P1.2 (normalization call-count/scratch reuse):
- Repeated micro-tuning was not retried because `/Users/sebastian/Workspace/md-new/PERF_ATTEMPTS.md` already records no-gain/regression variants and this run did not introduce new profiler evidence to justify revisiting them.

2. P1.3 + P2.1 (new focused benchmark coverage):
- Implemented new bench group in `/Users/sebastian/Workspace/md-new/benches/comparison.rs`: `link_refs_focus`.
- Added cases: `refs`, `refs_escaped` (escaped/entity-heavy), `mixed`.
- Sample measurements from `cargo bench --bench comparison -- "link_refs_focus/ferromark/(refs|refs_escaped|mixed)" --sample-size 40 --measurement-time 2`:
  - `refs`: `2.4212 us`
  - `refs_escaped`: `4.4347 us`
  - `mixed`: `3.3616 us`
- Decision: **kept** (measurement quality improvement).

3. P2.2 (before/after snapshots):
- Completed in this section (baseline and per-attempt deltas recorded).

4. P2.3 (simple CI perf regression check):
- Deferred: repository currently has no checked-in CI workflow in `.github/workflows`, so no non-speculative target pipeline was available to wire safely in this pass.

### P3 status

- Deferred intentionally in this pass:
  - No new profiler evidence pointed to UTF-8 validation as a hotspot (`simdutf8` check not justified yet).
  - Loop-unrolling/platform tuning remains behind a profile-first gate to avoid repeating prior no-gain SIMD churn.
  - PGO/non-PGO split reporting should be done together with CI/perf harness wiring.
