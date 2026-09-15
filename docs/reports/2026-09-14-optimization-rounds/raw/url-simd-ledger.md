# URL span SIMD scan experiment

Baseline: detached commit `4de75d4`. Scope is only
`crates/ferromark_renderer/src/html/autolink.rs::scan_url_end`; punctuation
trimming remains untouched. Timings are intentionally deferred to the parent
round harness.

## Feedback loop

The scalar `scan_url_end` loop is the oracle. The differential tests exercise
every byte value at every start offset and short/tail length, plus valid UTF-8
with mixed ASCII/CJK content. The expected result is the first ASCII URL
terminator (`space`, tab, LF, CR, `<`, `>`, double quote, apostrophe, backtick),
the first CJK/fullwidth terminator, or the string end. A start inside a UTF-8
sequence must still preserve the existing safe fallback behavior.

## Ranked hypotheses

1. **Wide ASCII classification can skip prose-sized URL spans.** If 16/32-byte
   chunks classify only the nine ASCII terminators and high-bit bytes, long
   ASCII URL runs should scan with fewer branches while preserving the scalar
   Unicode path.
2. **High-bit bytes must terminate the wide phase, not be treated as ASCII.**
   If the chunk mask includes any byte with bit 7 set, scalar `char` decoding
   should resume at that byte and retain CJK punctuation handling.
3. **Short tails may erase the benefit.** If a remaining suffix is shorter than
   the chosen vector width, a scalar tail should avoid unsafe overreads and
   preserve exact end offsets.

## Variant

Use a portable scalar chunk classifier as the first bounded prototype. It folds
each 8-byte chunk's terminator/high-bit predicates into a mask, then falls back
to the existing byte/Unicode loop at the first candidate. This gives a clean
oracle comparison before introducing architecture-specific intrinsics.

The patch is archived at
`/private/tmp/ferromark-v2-rounds-2/artifacts/url-simd-variant-a.patch` before
any replacement. The first focused test run exposed a draft defect: the
Unicode continuation loop had omitted the ASCII terminator check, so a
backtick after CJK (`"https://example.test/é中/path`tail"`) was swallowed. The
failure was in `preserves_mixed_utf8_and_cjk_boundaries`; restoring the scalar
ASCII check fixed it. The focused URL tests then passed (3/3). A final clippy
run was started and reached one test-only sign-cast lint, which was corrected
without another build after the parent requested CPU pause. No timing claim is
made here.
