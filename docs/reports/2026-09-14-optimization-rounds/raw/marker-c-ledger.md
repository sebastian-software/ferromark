# Optional inline marker fusion experiments

This ledger records the marker-scan prototypes for the SIMD optimization round.
The accepted baseline is commit `4de75d4` in this detached worktree. Timings are
intentionally left to the parent round harness; this worktree only runs
correctness/build checks.

## Feedback loop

The differential seam is `InlineMarkerScan::next` over a byte slice and cursor.
The scalar oracle finds the first core marker (`* _ ` [ ! ~ \\ < &` and line
ending) or enabled optional marker (`{`, `^`, `$`) at or after `from`. The test
matrix covers all eight option masks, every cursor offset, short/tail lengths,
all byte values, UTF-8 bytes, no-match inputs, and earliest-marker precedence.

## Ranked hypotheses

1. **Fused optional classification can remove repeated traversals.** If a
   single optional candidate search is used for all enabled extension markers,
   sparse MDX/math/superscript inputs should preserve results and reduce scan
   work relative to three independent `memchr` calls.
2. **The existing nibble SIMD classifier can cheaply include all optionals.**
   If its all-marker tables are used only for nonzero option masks, optional
   sparse paths should have one wide pass; disabled option bytes must be
   filtered before returning to preserve parser behavior.
3. **Filtering disabled candidates may erase the SIMD win.** If a disabled
   optional byte is frequent, repeated retry scans may cost more than the
   baseline. Targeted probes and the parent benchmark must measure this.
4. **A portable `memchr3` optional selector may beat dynamic SIMD dispatch on
   short extension blocks.** If true, a short-input threshold should avoid
   vector setup while retaining the existing zero-option fast path.

## Variants

### Variant A — fused optional `memchr3`

Use one `memchr3` search for the enabled subset (with `memchr`/`memchr2` for
one/two bytes), then take the minimum with the existing core SIMD scan. This is
the conservative candidate and has no disabled-marker false positives.

### Variant B — all-option nibble SIMD with retry filtering

Extend the nibble tables with `{`, `^`, and `$`, scan all active candidates in
the existing architecture-specific wide loops, and retry after a candidate
whose option is disabled. The zero-option path remains exactly the baseline.

## Disposition

Variants and discarded patches are saved under
`/private/tmp/ferromark-v2-rounds-2/artifacts/markers-*.patch` before reset or
replacement. Variant A is saved as
`markers-variant-a-fused-memchr3.patch`; Variant B is saved as
`markers-variant-b-fused-all-simd.patch` and is the current worktree candidate.

Variant B initially missed optional markers in sub-vector tails because the
shared SIMD helper fell back to the core-only scalar classifier for inputs
shorter than its vector width. That failed the one-byte `{` differential case;
the final candidate routes short tails to an all-marker scalar fallback and
passes the complete matrix.

Parent timing feedback after the first screen: Variant B's disabled-heavy
inputs were 0.079–0.10× baseline (roughly 10–12× slower) because retry filtering
rescanned every disabled optional byte; its full all-enabled plain case improved
about 1.5×. Variant A's sparse option-7 parse was 0.23× baseline because one
combined memo replaced the separate core memo and repeatedly rescanned the
core suffix. Variant C addresses both mechanisms by selecting one exact
core-plus-enabled-options table pair once per lookup and never retrying a
disabled candidate; the zero-option path remains the original core scan.

### Variant C — exact option-specific nibble tables

Variant C shares the existing NEON/SSSE3/AVX2 table-driven loops, with eight
compile-time table pairs selected from the three option bits. Short tails use
an option-aware scalar fallback. This is intentionally a prototype until the
parent reruns the 72-case timing harness.

Correctness checks completed: `cargo fmt --all --check`; 47 parser library
tests; and `cargo clippy -p ferromark_parser --all-targets --all-features
--locked -- -D warnings`. No benchmark was run here. The parent should measure
plain CommonMark, MDX, math, and superscript workloads and compare both patch
artifacts before promotion.
