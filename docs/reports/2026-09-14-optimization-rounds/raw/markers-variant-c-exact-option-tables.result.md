# Marker Variant C result

Worktree: `/private/tmp/ferromark-v2-rounds-2/marker-c` (detached at `4de75d4`).

Patch: `markers-variant-c-exact-option-tables.patch`

Variant C replaces Variant B's all-marker plus retry loop with eight
compile-time core-plus-enabled nibble table pairs. The existing NEON,
SSSE3, and AVX2 loops receive selected table references; short tails use an
option-aware scalar fallback. Disabled `{`, `^`, or `$` bytes are never emitted
by the SIMD classifier, while option mask zero keeps the original core-only
fast path.

Validation passed:

- `cargo fmt --all --check`
- `cargo test -p ferromark_parser parser::inline::marker_scan::tests --locked`
  — all 3 differential tests passed, including all 8 masks, every byte value,
  offsets, tails, UTF-8, and earliest-marker behavior
- `cargo clippy -p ferromark_parser --all-targets --all-features --locked -- -D warnings`

Parent timing feedback recorded in the worktree ledger: Variant B disabled
heavy cases were 0.079–0.10× baseline from retry rescans, while Variant A
sparse option-7 parse was 0.23× baseline from replacing the separate core memo.
Variant C is ready for the parent 72-case timing harness; no benchmark was run
in this worker.
