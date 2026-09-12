# ARCH-EXP-020: Ox-inspired scanning and allocation

**Date:** 2026-09-12

**Hypothesis:** the native comparison with Ox Content exposes separable costs
in special-byte scanning, normalized text copies, code-info allocations and
heading-ID registry growth. These can improve without changing the streaming
architecture or Markdown semantics.

**Method:** screen thirteen independent implementations and three combinations
against frozen main, compare exact output and document-state transitions before
timing, then rebuild the retained production source and confirm it across three
process pairs. Extend the timing and equivalence corpus with MDX segment changes.
Measure allocation counts separately, and repeat the native comparison with Ox.

**Decision:** retain exact NEON nibble classification and direct matching-lane
location; borrow contiguous paragraph and heading source ranges; borrow fence
info without backslash escapes; reserve the heading registry using a count
collected during block parsing; and directly emit a single parsed text event.
Full block/inline parsing, entity handling, heading deduplication and renderer
reset behavior remain in place. This extends the narrow whole-document borrowing
shortcut from ARCH-EXP-019 to normalized content in ordinary document state.

The classifier shares bits only between identical nibble membership rows and
falls back to byte comparisons when the set requires more than eight row groups.
Short NEON inputs retain scalar lookup; the packed lane-location path requires
little-endian byte order. SSE2 and scalar-only implementations are unchanged.
Borrowed text materializes when source fragments cease to be contiguous and
clears its range at paragraph/heading completion, including across MDX sources.

**Rejected:** flat heading collision chains, a generated-slug cache, SWAR escape
scanning, doubling initial output capacity, and carrying the first inline mark
position into mark collection. A separate heading-count event pass was replaced
with counting in the existing parser pass. These alternatives either regressed
other workloads or did not justify their complexity/capacity cost.

**Evidence and limits:** the [experiment report](../reports/2026-09-12-ox-inspired-optimizations/REPORT.md)
archives every prototype, raw window, source hash, lockfile and validation log.
The largest improvements concern long scans; lists, rich inline content and MDX
benefit less. Empty-input relative differences remain visible in the complete
table. Allocation counts are cumulative requests, not peak memory. Performance
was measured on an Apple M1 Pro with Rust's default `apple-m1` target; x86 and scalar paths were
compile-checked only. README/homepage publication figures remain independently
sourced from the regular comparison suite.
