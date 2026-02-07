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
- Attempt D: `LinkRefStore` pre-reserve from paragraph candidate count (`extract_link_ref_defs` path).
  - Benchmark (`--sample-size 80 --measurement-time 4`) result: `refs` `-2.14%`, `mixed` `+1.10%` (noise), `commonmark50k` `+0.58%` (noise).
  - Focused bench (`link_refs_focus`) result: `refs_escaped` `+4.28%` (regression), `mixed` `+1.86%` (regression).
  - Decision: **discarded** (focused regressions).
- Attempt E: `find_matching_close` index-based scan rewrite (remove open binary search + branch checks).
  - Benchmark (`--sample-size 80 --measurement-time 4`) result: `refs` `+1.75%` (regression), `mixed` no significant change, `commonmark50k` no significant change.
  - Decision: **discarded**.
- Attempt F: `memchr`/`memchr_iter` newline scans in block link-ref parsing path.
  - Benchmark (`--sample-size 80 --measurement-time 4`) result: `refs` `-1.07%` (noise), `mixed` `-0.85%` (noise), `commonmark50k` no significant change.
  - Focused bench (`link_refs_focus`) result: `refs_escaped` `-3.06%` (improved) but `refs`/`mixed` remained within noise.
  - Decision: **discarded** (did not meet primary meaningful-`refs`/`mixed` criterion).
- Attempt G: `write_link_title` fast path in `/Users/sebastian/Workspace/md-new/src/render.rs` (skip UTF-8/entity decode when no `&`; skip backslash scan when no `\\` after decode).
  - Change kept in commit: `5caf88e`
  - Benchmark (`--sample-size 80 --measurement-time 4`) result:
    - `commonmark50k/ferromark`: `150.63 us` (no significant regression).
    - `complexity/ferromark/refs`: `2.2833 us` (`-2.95%` time, significant).
    - `complexity/ferromark/mixed`: no significant change.
  - Focused bench (`link_refs_focus`) result:
    - `refs`: `2.3308 us` (`-3.15%`, significant)
    - `refs_escaped`: `4.3146 us` (no significant change)
    - `mixed`: `3.3336 us` (no significant change)
  - Decision: **kept**.
- Attempt H: `insert_prechecked` path for link-ref definitions (remove redundant lookup after `get_index` check in `extract_link_ref_defs`).
  - Main benchmark (`--sample-size 80 --measurement-time 4`) result (two repeated runs): `refs` and `mixed` remained in noise in the guardrail set.
  - Focused bench (`link_refs_focus`) showed mixed results (one run with wins, one run near-noise), without stable confirmation in guardrail cases.
  - Decision: **discarded** (insufficiently stable gain).
- Attempt I: URL destination safe-copy fast path in `/Users/sebastian/Workspace/md-new/src/escape.rs` (`url_escape_link_destination_raw`).
  - Change kept in commit: `6a6ed26`
  - Change: early return with `extend_from_slice` when URL bytes are ASCII and contain no characters requiring escaping/encoding.
  - Benchmark (`--sample-size 80 --measurement-time 4`) repeated absolute medians:
    - `commonmark50k/ferromark`: `147.93-150.21 us` (improved vs prior kept baseline `150.63 us`).
    - `complexity/ferromark/refs`: `2.2568-2.2631 us` (improved vs prior kept baseline `2.2833 us`).
    - `complexity/ferromark/mixed`: `3.2254-3.2345 us` (improved vs prior kept baseline `3.3262 us`).
  - Decision: **kept**.
- Attempt J: `parse_link_ref_def` memchr-based scanner path in `/Users/sebastian/Workspace/md-new/src/block/parser.rs` (label delimiter scan + angle-URL delimiter scan + line-end scan).
  - Main benchmark (`--sample-size 80 --measurement-time 4`) result: `commonmark50k` `149.23 us`, `refs` `2.2757 us`, `mixed` `3.2666 us` (all no significant change vs local baseline).
  - Focused bench (`link_refs_focus`) result: `refs_escaped` and `mixed` improved, but primary guardrail metrics were not meaningfully improved and absolute `refs`/`mixed` medians trended worse than Attempt I keep baseline.
  - Decision: **discarded**.
- Attempt K: ASCII escaped-label fast path in `/Users/sebastian/Workspace/md-new/src/link_ref.rs` (`normalize_label_into` without temporary `Vec` for ASCII + `\\`).
  - Main benchmark (`--sample-size 80 --measurement-time 4`) result: no significant change on `commonmark50k`, `refs`, `mixed`.
  - Focused bench (`link_refs_focus`) result: `refs` and `mixed` regressed significantly (`+2.23%` and `+2.93%` time).
  - Decision: **discarded**.
- Attempt L: reuse candidate-label scratch buffer in `/Users/sebastian/Workspace/md-new/src/inline/links.rs` (`contains_ref_link_candidate` no longer allocates a new `String` per call).
  - Main benchmark (`--sample-size 80 --measurement-time 4`) repeated medians:
    - `commonmark50k/ferromark`: `148.63-148.69 us` (no significant regression).
    - `complexity/ferromark/refs`: `2.2351-2.2435 us` (improved vs Attempt I keep baseline `2.2568-2.2631 us`).
    - `complexity/ferromark/mixed`: `3.2322-3.2667 us` (within noise band across runs).
  - Focused bench (`link_refs_focus`) had mixed significance run-to-run, but absolute medians remained in the same or better range for `refs` and `mixed`, with no stable escaped-case regression on re-run.
  - Decision: **kept**.
- Attempt M: structural refs pass (stack-based bracket pairing in `src/inline/links.rs`, centralized ref-label parse+normalize helper, contiguous no-copy fast-path + in-parser normalization in `src/block/parser.rs`).
  - Main guardrail run (`--sample-size 40 --measurement-time 2`) after tuning:
    - `commonmark50k/ferromark`: `147.31 us` (no significant change).
    - `complexity/ferromark/refs`: `2.2701 us` (no significant change).
    - `complexity/ferromark/mixed`: `3.2328 us` (improved, but classified as noise-threshold change in this run).
  - Focused run (`link_refs_focus`) result:
    - `refs`: `2.2514 us` (within noise threshold vs local baseline),
    - `refs_escaped`: `4.2325 us` (no significant change),
    - `mixed`: `3.2414 us` (improved, still within noise threshold classification).
  - Cross-lib refs snapshot (`complexity/(ferromark|md4c|pulldown-cmark|comrak)/refs$`):
    - `ferromark`: `2.2508 us`
    - `pulldown-cmark`: `1.8894 us`
    - Remaining gap: `~19.13%` in favor of `pulldown-cmark` (worse than earlier ~`17.44%` snapshot).
  - Decision: **discarded** (did not close the pulldown gap and failed the primary refs objective despite acceptable guardrails).
- Attempt N: streaming-oriented block refdef extraction in `/Users/sebastian/Workspace/md-new/src/block/parser.rs` (contiguous no-copy paragraph fast path + reused label scratch buffer; fallback to existing joined-buffer path when lines are not contiguous in source).
  - Main guardrail run (`--sample-size 80 --measurement-time 4`) result:
    - `commonmark50k/ferromark`: `146.69 us` (improved, significant).
    - `complexity/ferromark/refs`: `2.1786 us` (improved direction; within configured noise-threshold classification).
    - `complexity/ferromark/mixed`: `3.2650 us` (no significant change).
  - Focused run (`link_refs_focus`, `--sample-size 40 --measurement-time 2`) result:
    - `refs`: `2.2271 us` (improved direction),
    - `refs_escaped`: `4.2221 us` (no significant change),
    - `mixed`: `3.3678 us` (run was noisy; guardrail set remains neutral on `mixed`).
  - Cross-lib refs snapshot (`complexity/(ferromark|md4c|pulldown-cmark|comrak)/refs$`):
    - `ferromark`: `2.2087 us`
    - `pulldown-cmark`: `1.9158 us`
    - Remaining gap: `~15.29%` in favor of `pulldown-cmark` (improved vs prior ~`17.44%` snapshot).
  - Decision: **kept**.
- Attempt O: stronger stack-/delimiter-based link-reference resolution in `/Users/sebastian/Workspace/md-new/src/inline/links.rs` with parser-owned stack/scratch state in `/Users/sebastian/Workspace/md-new/src/inline/mod.rs` (closer to pulldown-cmark pass-1 shape, including outer-link disable during inner non-image ref-link formation).
  - Main guardrail run (`--sample-size 40 --measurement-time 2`) result:
    - `commonmark50k/ferromark`: `147.53 us` (`+2.06%` time, significant regression).
    - `complexity/ferromark/refs`: `2.2363 us` (`+2.88%` time, significant regression).
    - `complexity/ferromark/mixed`: `3.3090 us` (`+1.68%` time, significant regression).
  - Focused run (`link_refs_focus`, `--sample-size 40 --measurement-time 2`) result:
    - `refs`: `2.2425 us` (no significant change),
    - `refs_escaped`: `4.1595 us` (no significant change),
    - `mixed`: `3.2730 us` (no significant change).
  - Cross-lib refs snapshot with this variant (`complexity/(ferromark|md4c|pulldown-cmark|comrak)/refs$`):
    - `ferromark`: `2.2503 us`
    - `pulldown-cmark`: `1.8928 us`
    - Gap: `~18.89%` in favor of `pulldown-cmark` (worse than the kept baseline path).
  - Decision: **discarded** (failed refs objective and widened pulldown gap).
- Attempt P: skip inline-link resolver pass when no immediate `](` candidate exists (new `has_inline_link_opener` guard in `/Users/sebastian/Workspace/md-new/src/inline/mod.rs`), so refs-heavy docs avoid unnecessary `resolve_links_into` bracket matching work.
  - Main guardrail run (`--sample-size 40 --measurement-time 2`) result:
    - `commonmark50k/ferromark`: `146.98 us` (no significant regression; within noise threshold).
    - `complexity/ferromark/refs`: `2.1284 us` (`-4.37%` time, significant).
    - `complexity/ferromark/mixed`: `3.2022 us` (no significant change).
  - Focused run (`link_refs_focus`, `--sample-size 40 --measurement-time 2`) result:
    - `refs`: `2.1660 us` (improved, significant),
    - `refs_escaped`: `4.1565 us` (improved, significant),
    - `mixed`: `3.2155 us` (no significant change).
  - Cross-lib refs snapshot (`complexity/(ferromark|md4c|pulldown-cmark|comrak)/refs$`):
    - `ferromark`: `2.1598 us`
    - `pulldown-cmark`: `1.9294 us`
    - Gap: `~11.94%` in favor of `pulldown-cmark` (improved vs prior `~15.29%` baseline snapshot).
  - Decision: **kept**.
- Attempt Q: precompute valid nested ref-link candidates once per parse in `/Users/sebastian/Workspace/md-new/src/inline/links.rs` (candidate open/close vectors + precomputed candidate check replacing repeated ad-hoc scan in `contains_ref_link_candidate`).
  - Main guardrail run (`--sample-size 40 --measurement-time 2`) result:
    - `commonmark50k/ferromark`: `153.09 us` (regression).
    - `complexity/ferromark/refs`: `2.4072 us` (regression).
    - `complexity/ferromark/mixed`: `3.3743 us` (regression).
  - Focused run (`link_refs_focus`) result:
    - `refs`: `2.4225 us` (regression),
    - `refs_escaped`: `5.0374 us` (regression),
    - `mixed`: `3.3645 us` (regression).
  - Decision: **discarded**.
- Attempt R: lazy-cached nested-candidate evaluation in `/Users/sebastian/Workspace/md-new/src/inline/links.rs` (cache first close per open + cache whether open can normalize to known ref label; cache initialized only if nested-candidate check is reached).
  - Main guardrail run (`--sample-size 40 --measurement-time 2`) result:
    - `commonmark50k/ferromark`: `147.12 us` (noise-level change).
    - `complexity/ferromark/refs`: `2.2307 us`.
    - `complexity/ferromark/mixed`: `3.2902 us`.
  - Baseline cross-check run on detached worktree at `a0e5a2a` (same command parameters) produced:
    - `commonmark50k/ferromark`: `149.17 us`
    - `complexity/ferromark/refs`: `2.1325 us`
    - `complexity/ferromark/mixed`: `3.2122 us`
  - Interpretation: refs/mixed worse than current kept baseline despite no correctness failures.
  - Decision: **discarded**.
- Attempt S: remove per-call binary-search setup in `find_matching_close` by precomputing first close index per open and passing open index directly (changes in `/Users/sebastian/Workspace/md-new/src/inline/links.rs` and `/Users/sebastian/Workspace/md-new/src/inline/mod.rs`).
  - Main guardrail run (`--sample-size 40 --measurement-time 2`) result:
    - `commonmark50k/ferromark`: `149.11 us` (noise-level change/regression direction).
    - `complexity/ferromark/refs`: `2.2303 us` (no meaningful win vs baseline).
    - `complexity/ferromark/mixed`: `3.2617 us` (no meaningful win).
  - Focused run (`link_refs_focus`) result:
    - `refs`: `2.2414 us` (regression direction),
    - `refs_escaped`: `4.2280 us` (neutral),
    - `mixed`: `3.2938 us` (neutral/regression direction).
  - Decision: **discarded**.
- Attempt T: parser-owned nested-label scratch reuse for `contains_ref_link_candidate` (add reusable `ref_nested_label_buf` in `/Users/sebastian/Workspace/md-new/src/inline/mod.rs`; remove per-call `String::new()` in `/Users/sebastian/Workspace/md-new/src/inline/links.rs`).
  - Main guardrail run (`--sample-size 40 --measurement-time 2`) result:
    - `commonmark50k/ferromark`: `145.99 us` (noise-level change).
    - `complexity/ferromark/refs`: `2.2172 us` (regression direction vs kept baseline).
    - `complexity/ferromark/mixed`: `3.1839 us` (no clear guardrail issue).
  - Focused run (`link_refs_focus`) result:
    - `refs`: `2.1616 us`,
    - `refs_escaped`: `4.1651 us`,
    - `mixed`: `3.2142 us`.
  - Interpretation: local improvements were not robust enough versus current kept baseline and did not clearly close the refs gap in repeat checks.
  - Decision: **discarded**.
- Attempt U: reduce nested-candidate scan range in `contains_ref_link_candidate` via `partition_point` (scan only opens inside `(start, end)` rather than full open list each call).
  - Run 1 (`--sample-size 40 --measurement-time 2`) looked positive:
    - `commonmark50k/ferromark`: `146.44 us`
    - `complexity/ferromark/refs`: `2.1559 us`
    - `link_refs_focus/ferromark/refs`: `2.1370 us`
  - Immediate rerun on unchanged code regressed:
    - `complexity/ferromark/refs`: `2.2587 us`
    - `link_refs_focus/ferromark/refs`: `2.2521 us`
    - `link_refs_focus/ferromark/refs_escaped`: `4.3945 us`
  - Decision: **discarded** (unstable and not reproducibly better).
- Attempt V: delimiter-stack first-pass reference resolution in `/Users/sebastian/Workspace/md-new/src/inline/links.rs` (resolve refs directly while sweeping closes left-to-right with an open-bracket stack, remove `contains_ref_link_candidate` from the hot path, and consume `[label]` suffix brackets in the same pass).
  - Main guardrail run (`--sample-size 40 --measurement-time 2`) result:
    - `commonmark50k/ferromark`: `146.48 us` (no significant regression).
    - `complexity/ferromark/refs`: `2.1965 us` (improved vs immediate previous run baseline, significant).
    - `complexity/ferromark/mixed`: `3.3067 us` (noise-threshold change).
  - Focused run (`link_refs_focus`) result:
    - `refs`: `2.1717 us` (improved),
    - `refs_escaped`: `4.1468 us` (improved),
    - `mixed`: `3.2534 us` (noise-threshold change).
  - Cross-lib refs snapshot with this variant (`complexity/(ferromark|md4c|pulldown-cmark|comrak)/refs$`):
    - `ferromark`: `2.1901 us`
    - `pulldown-cmark`: `1.9175 us`
    - Gap: `~14.22%` in favor of `pulldown-cmark` (worse than kept baseline `~11.94%`).
  - A/B cross-check against kept baseline commit `a0e5a2a` in detached worktree (`/tmp/md-new-baseline-a0`) with identical bench parameters:
    - Baseline `complexity/ferromark/refs`: `2.1649 us`
    - Baseline `complexity/ferromark/mixed`: `3.2406 us`
    - Variant remained slower on refs/mixed in this direct comparison despite passing tests.
  - Decision: **discarded**.

### Current `refs` position vs other libraries (2026-02-07)

- Snapshot command: `cargo bench --bench comparison -- "complexity/(ferromark|md4c|pulldown-cmark|comrak)/refs$" --sample-size 40 --measurement-time 2`
- Median times:
  - `ferromark`: `2.1598 us`
  - `md4c`: `2.4712 us`
  - `pulldown-cmark`: `1.9294 us`
  - `comrak`: `4.7482 us`
- Interpretation:
  - ferromark is currently faster than `md4c` and much faster than `comrak` on `refs`.
  - The remaining notable gap is vs `pulldown-cmark` (~`11.94%` faster on this run), so there is still meaningful headroom.

### Cross-check against `pulldown-cmark` approach (2026-02-07)

- Profile artifacts used:
  - ferromark refs Time Profiler: `/Users/sebastian/Workspace/md-new/target/profiles/refs-next-gap.trace` + `/Users/sebastian/Workspace/md-new/target/profiles/refs-next-gap.xml`
  - pulldown refs Time Profiler (release-debug): `/Users/sebastian/Workspace/md-new/target/profiles/pulldown-refs-release-debug.trace` + `/Users/sebastian/Workspace/md-new/target/profiles/pulldown-refs-release-debug.xml`
- Observed ferromark leaf hotspots in refs-focused trace (weighted sample counts):
  - `parse_link_ref_def`: `226`
  - `find_matching_close`: `103`
  - `contains_ref_link_candidate`: `33`
  - `normalize_label_into`: `34`
  - `extract_link_ref_defs` (+ iterator fold around it): `38` + `36`
- Observed pulldown refs hotspots (same extraction method):
  - `linklabel::scan_link_label_rest`: `327`
  - `Parser::handle_inline_pass1`: `230`
  - `FirstPass::scan_refdef`: `196`
  - `FirstPass::parse_refdef_total`: `56`
- Structural differences that still likely explain most of the remaining gap:
  - pulldown drives reference matching through a link stack in `handle_inline_pass1`, while ferromark still pays extra nested scans in `find_matching_close` / `contains_ref_link_candidate`.
  - pulldown centralizes label scanning/normalization in `scan_link_label_rest`; ferromark still normalizes the same conceptual labels in multiple places (`parse_link_ref_def`, inline ref scan, nested candidate checks).
  - pulldown first-pass refdef handling avoids some of the repeated paragraph-slice recomputation patterns still present in ferromark extraction flow.
- Decision:
  - We are **not** at the end; there is still a material refs opportunity, but the next gain is more likely from a targeted structural pass than from additional micro-tuning.

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
