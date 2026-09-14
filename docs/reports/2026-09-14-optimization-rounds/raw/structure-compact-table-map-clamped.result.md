# Structure clamped-map correction

Corrected patch: `structure-compact-table-map-clamped.patch`

The sparse table-cell map now explicitly stores `generated_len` alongside the
removed-backslash positions. Boundary remapping clamps every incoming index to
that generated length before applying the sparse correction, preserving the
old dense map's out-of-range behavior. Valid boundaries, adjacent escaped
pipes, and Unicode source cells continue to use the strict-before-removed
boundary rule.

Added an independent dense-oracle test covering valid and out-of-range
boundaries for Unicode cells. Validation passed:

- `cargo fmt --all`
- `cargo test -p ferromark_parser table_cell_source --locked` — 5 focused
  tests passed
- `cargo clippy -p ferromark_parser --all-targets --all-features --locked -- -D warnings`

No timing was run.
