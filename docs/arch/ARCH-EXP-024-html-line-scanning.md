# ARCH-EXP-024: Bounded HTML line scanning

**Date:** 2026-09-12

**Hypothesis:** after ARCH-EXP-023, short root HTML lines still pay substantial
newline-search overhead. Prefix-only HTML classification also searches for line
ends before it needs them. Indentation and repeated parser-state work are
independent candidate costs.

**Method:** freeze merged PR #310, use an isolated non-inlined continuation-loop
probe for source attribution, screen individual changes, and rebuild the formatted
production source for repeated alternating process pairs. Preserve exact public
events, rendering, metadata and reused state against the frozen source. Measure
allocator requests separately from time.

**Decision:** on NEON-enabled AArch64, search the first 128 bytes of root HTML
continuations with the existing byte-set scanner, then use memchr for any remaining
input. Keep the searcher constant and specialize its single-byte NEON comparison
to one equality. Other targets retain the existing newline search. Types 1-6 of
HTML-block recognition inspect only their opening prefix and tag boundary; defer
the complete physical-line lookup until type 7 needs it.

**Rejected:** local indentation, delayed column updates, separate block-kind loops
and a locally held block kind did not justify their added state machinery. A
portable word check and bounded memchr calls provided less useful gains. Searching
the entire remainder with the byte-set loop penalized long lines. Constructing
the searcher at the call site also added avoidable work. The bounded constant
searcher retains the long-line fallback.

**Compatibility:** the public block and MDX event streams retain each physical
line and its original source range. CR bytes, partial final lines, blank-line
termination, containers, raw-HTML policies and reused document state keep their
existing behavior. The change adds no unsafe load, allocation, arena or reservation.
The shared byte-search crate continues to compile the same implementation as the
embedded parser module, as required by ADR-0013.

**Evidence and limits:** the [experiment report](../reports/2026-09-12-html-line-scanning/REPORT.md)
retains exploratory patches, source identities, exact-output oracles, raw timing
windows, profiles and allocation observations. Measurements use an Apple M1 Pro;
they do not establish an x86 speedup or a universal ranking against Ox.
