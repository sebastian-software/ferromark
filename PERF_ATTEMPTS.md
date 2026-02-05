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
