# Performance Attempts Log

This log records performance experiments for md-fast. Each attempt is run on `commonmark50k/md-fast` unless noted. We keep changes only if the benchmark shows a measurable improvement.

## 2026-02-05

- Change: NEON scan for HTML escape (`escape_text_into`, `escape_full_into`) using vectorized search for escapable bytes.
- Command: `cargo bench --bench comparison -- "commonmark50k/md-fast"`
- Result: 276.2-277.8 MiB/s, change within noise threshold (no clear improvement).
- Decision: Reverted.

- Change: Fast-path for consecutive simple paragraph lines at top level (skip container matching and per-line block checks).
- Command: `cargo bench --bench comparison -- "commonmark50k/md-fast"`
- Result: 285.0-286.6 MiB/s, ~+2.9% throughput.
- Decision: Kept.

- Change: NEON scan for URL escaping in `url_escape_link_destination_raw` to bulk-copy safe chunks and handle specials.
- Command: `cargo bench --bench comparison -- "commonmark50k/md-fast"`
- Result: 283.7-287.2 MiB/s, change within noise threshold (no clear improvement).
- Decision: Reverted.

- Change: ASCII fast-path for link label normalization with NEON chunk copy when no uppercase/whitespace.
- Command: `cargo bench --bench comparison -- "commonmark50k/md-fast"`
- Result: 283.9-285.2 MiB/s, change within noise threshold (slightly slower).
- Decision: Reverted.

- Change: Escape fast-path using `memchr` to locate next escapable byte instead of table scan.
- Command: `cargo bench --bench comparison -- "commonmark50k/md-fast"`
- Result: 279.2-281.1 MiB/s, change within noise threshold (slower).
- Decision: Reverted.

- Change: Skip HTML entity decoding when no `&` is present (pre-check with `memchr`).
- Command: `cargo bench --bench comparison -- "commonmark50k/md-fast"`
- Result: 276.0-284.9 MiB/s, change within noise threshold (slower, noisy).
- Decision: Reverted.

- Change: NEON blank-line scan in block parser (`is_blank_line`) to fast-skip whitespace.
- Command: `cargo bench --bench comparison -- "commonmark50k/md-fast"`
- Result: 280.6-282.4 MiB/s, ~+4% throughput.
- Decision: Kept.

- Change: Inline SIMD scan widened to 32-byte chunks for mark detection.
- Command: `cargo bench --bench comparison -- "commonmark50k/md-fast"`
- Result: 273.8-277.2 MiB/s, regression (~-2%).
- Decision: Reverted.

- Change: SIMD ASCII fast-path for link-label normalization (no-whitespace detection + lowercasing).
- Command: `cargo bench --bench comparison -- "commonmark50k/md-fast"`
- Result: 276.6-279.5 MiB/s, change within noise threshold.
- Decision: Reverted.

- Change: ASCII fast-path for emphasis flag computation in mark collection.
- Command: `cargo bench --bench comparison -- "commonmark50k/md-fast"`
- Result: 274.4-275.7 MiB/s, change within noise threshold (slower).
- Decision: Reverted.

- Change: Skip link resolution when no `(` is present and no reference defs exist.
- Command: `cargo bench --bench comparison -- "commonmark50k/md-fast"`
- Result: 278.4-279.7 MiB/s, change within noise threshold.
- Decision: Reverted.

- Change: Precompute emit-point end-flag to simplify sorting in inline emission.
- Command: `cargo bench --bench comparison -- "commonmark50k/md-fast"`
- Result: 279.1-280.4 MiB/s, change within noise threshold.
- Decision: Reverted.

- Change: Avoid binary search on open brackets in `find_matching_close` by passing open index.
- Command: `cargo bench --bench comparison -- "commonmark50k/md-fast"`
- Result: 281.9-283.1 MiB/s, change within noise threshold.
- Decision: Reverted.

- Change: ASCII-only label normalization path (byte-wise lowercase + whitespace collapse).
- Command: `cargo bench --bench comparison -- "commonmark50k/md-fast"`
- Result: 280.9-282.0 MiB/s, no change detected.
- Decision: Reverted.

- Change: Hoist `has_outer_close` scan out of the per-open loop in inline link deactivation logic.
- Command: `cargo bench --bench comparison -- "commonmark50k/md-fast"`
- Result: 282.9-284.1 MiB/s, no change detected.
- Decision: Reverted.

- Change: Reuse label buffer + binary-start close lookup in `contains_ref_link_candidate`.
- Command: `cargo bench --bench comparison -- "commonmark50k/md-fast"`
- Result: 281.4-282.8 MiB/s, no change detected.
- Decision: Reverted.

- Change: Switch emit-point sorting to a custom comparator with pre-ranked end events.
- Command: `cargo bench --bench comparison -- "commonmark50k/md-fast"`
- Result: 277.0-278.6 MiB/s, regression (~-1.7%).
- Decision: Reverted.

- Change: Split `render_inline_event` into dedicated image/non-image paths to reduce per-event branching.
- Command: `cargo bench --bench comparison -- "commonmark50k/md-fast"`
- Result: 280.8-282.7 MiB/s, change within noise threshold.
- Decision: Reverted.

- Change: Short-input fast path in `escape_into_with_table` (avoid memchr setup for <=32 bytes).
- Command: `cargo bench --bench comparison -- "commonmark50k/md-fast"`
- Result: 274.1-276.1 MiB/s, regression (~-2.0%).
- Decision: Reverted.

- Change: Replace expensive list-item blank-line expression with `is_blank_line_scalar` in `match_containers`.
- Command: `cargo bench --bench comparison -- "commonmark50k/md-fast"`
- Result: first run 284.7-286.7 MiB/s (~+3.6%), rerun 263.1-282.2 MiB/s (no clear improvement, high variance).
- Decision: Reverted.

## 2026-02-06

- Change: Escape fast path rewrite with `memchr` segment scanning (`escape_text_into`, `escape_full_into`) plus skip entity decode in `url_escape_link_destination` when no `&`.
- Command: `target/release/deps/comparison-* --bench --measurement-time 20 --warm-up-time 3 --sample-size 80 '^commonmark50k/md-fast$'`
- Result: baseline `174.55 us` vs candidate `172.59 us` (about `+1.2%` throughput, significant).
- Decision: Kept.

- Change: Reuse buffers in inline link resolution (`resolve_links_into`) to avoid per-parse allocations.
- Command: `target/release/deps/comparison-* --bench --measurement-time 20 --warm-up-time 3 --sample-size 80 '^commonmark50k/md-fast$'`
- Result: baseline `172.83 us` vs candidate `167.67 us` (about `+3.1%` throughput, significant).
- Decision: Kept.

- Change: Merge bracket collection + emphasis-candidate detection into one marks pass.
- Command: `target/release/deps/comparison-* --bench --measurement-time 20 --warm-up-time 3 --sample-size 80 '^commonmark50k/md-fast$'`
- Result: baseline `167.25 us` vs candidate `166.71 us` (about `+0.4%` throughput, significant but small).
- Decision: Kept.

- Change: ASCII fast path for link-label normalization (`normalize_label_text_ascii`) with Unicode fallback.
- Command: `target/release/deps/comparison-* --bench --measurement-time 20 --warm-up-time 3 --sample-size 80 '^commonmark50k/md-fast$'`
- Result: baseline `166.90 us` vs candidate `165.94 us` (about `+0.56%` throughput, significant).
- Decision: Kept.

- Change: ASCII-neighbor fast path for emphasis flanking in `collect_marks` (`compute_emphasis_flags_with_context`).
- Command: `target/release/deps/comparison-* --bench --measurement-time 20 --warm-up-time 3 --sample-size 80 '^commonmark50k/md-fast$'`
- Result: baseline `165.50 us` vs candidate `165.09 us` (small gain, not statistically significant at `p=0.08`).
- Decision: Reverted.

- Change: Byte-dispatch guards in `parse_line_content_with_indent` to skip expensive `try_*` parser checks unless the first non-indent byte can start that construct.
- Command: `target/release/deps/comparison-* --bench --measurement-time 20 --warm-up-time 3 --sample-size 80 '^commonmark50k/md-fast$'`
- Result: baseline `166.36 us` vs candidate `159.75 us` (about `+4.67%` throughput, significant).
- Decision: Kept.

- Change: Replace retain+`any()` filters in `InlineParser::parse` (autolinks/code-spans, brackets/html-spans) with linear pointer-based scans.
- Command: `target/release/deps/comparison-* --bench --measurement-time 20 --warm-up-time 3 --sample-size 80 '^commonmark50k/md-fast$'`
- Result: baseline `158.52 us` vs candidate `160.84 us` (about `-1.1%` throughput, significant).
- Decision: Reverted.

- Change: Add paragraph/lazy-continuation fast paths in `parse_line` + `can_lazy_continue` to bypass block-start checks on simple text starts.
- Command: `target/release/deps/comparison-* --bench --measurement-time 20 --warm-up-time 3 --sample-size 80 '^commonmark50k/md-fast$'`
- Result: baseline `158.24 us` vs candidate `158.52 us` (no significant improvement, `p=0.18`).
- Decision: Reverted.

- Change: Optimize `parse_paragraph_line` cursor movement (use `advance/bump` instead of rebuilding via `Cursor::new_at`).
- Command: `target/release/deps/comparison-* --bench --measurement-time 20 --warm-up-time 3 --sample-size 80 '^commonmark50k/md-fast$'`
- Result: baseline `157.97 us` vs candidate `157.53 us` (about `+0.68%` throughput, significant).
- Decision: Kept.

- Change: Profile-Guided Optimization (PGO) experiment for bench binary (`-Cprofile-generate` training on `commonmark50k/md-fast`, then rebuild with `-Cprofile-use`).
- Command: non-PGO `target/release/deps/comparison-dc2365de0bc04f02 --bench --measurement-time 15 --warm-up-time 3 --sample-size 60 '^commonmark50k/md-fast$'`; PGO `target/release/deps/comparison-b3872a0f6a0e868d --bench --measurement-time 15 --warm-up-time 3 --sample-size 60 '^commonmark50k/md-fast$'`
- Result: non-PGO `157.20 us` vs PGO `132.97 us` (about `+18.3%` throughput, significant).
- Decision: Keep as build strategy (no source-code behavior change).

- Change: Fairness check: apply same PGO workflow to `pulldown-cmark` (train on `commonmark50k/pulldown-cmark`, rebuild with `-Cprofile-use`).
- Command: non-PGO `target/release/deps/comparison-dc2365de0bc04f02 --bench --measurement-time 15 --warm-up-time 3 --sample-size 60 '^commonmark50k/pulldown-cmark$'`; PGO `target/release/deps/comparison-539bd1b9193b9dd8 --bench --measurement-time 15 --warm-up-time 3 --sample-size 60 '^commonmark50k/pulldown-cmark$'`
- Result: non-PGO `180.87 us` vs PGO `138.06 us` (about `+31.0%` throughput, significant).
- Decision: Keep for tuned comparisons; report separately from non-PGO baseline.

- Change: Cross-profile sanity check (use parser A's PGO profile to run parser B).
- Command: `comparison-539bd1b9193b9dd8` on `commonmark50k/md-fast`; `comparison-b3872a0f6a0e868d` on `commonmark50k/pulldown-cmark`.
- Result: strongly regressive (`md-fast` to `184.28 us`, `pulldown-cmark` to `234.15 us`), showing profile specialization.
- Decision: Do not use cross-profile binaries for fair benchmarking.

- Change: Reuse more inline scratch buffers (`emphasis_matches`, HTML code/autolink range vectors) and pre-size parser-owned vectors in `InlineParser::new`.
- Command: baseline PGO build (commit `c4a4633`) `target/release/deps/comparison-5090cc332b72e3a6 --bench --measurement-time 20 --warm-up-time 3 --sample-size 80 '^commonmark50k/md-fast$'`; candidate PGO build (current) `target/release/deps/comparison-982be754c0375a28 --bench --measurement-time 20 --warm-up-time 3 --sample-size 80 '^commonmark50k/md-fast$'`
- Result: baseline `132.79 us` vs candidate `129.83 us` (about `+2.3%` throughput, significant).
- Decision: Kept.

- Change: Precompute inline emphasis boundary membership per mark (to avoid repeated boundary scans in `find_opener`), then hybrid correction (precompute only for large boundary sets).
- Command: PGO candidate v1 `target/release/deps/comparison-* --bench --measurement-time 20 --warm-up-time 3 --sample-size 80 '^commonmark50k/md-fast$'`; PGO candidate v2 (hybrid correction) same command.
- Result: v1 `130.41 us` (regression vs `129.83 us`), v2 `132.59 us` (larger regression).
- Decision: Reverted.
