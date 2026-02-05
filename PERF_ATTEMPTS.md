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
