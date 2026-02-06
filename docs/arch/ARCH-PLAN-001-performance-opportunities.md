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
