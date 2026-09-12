# ARCH-EXP-021: Ox corpus line scans and render ranges

**Date:** 2026-09-12

**Hypothesis:** the profiles in the [Ox corpus investigation](../reports/2026-09-12-ox-corpus-profiling/REPORT.md)
identify avoidable byte-at-a-time line scans in HTML-heavy documents and repeated
renderer dispatch for contiguous code/HTML source ranges. Inline event sorting
is another measured cost, but changing it requires independent evidence.

**Method:** freeze the merged production source, compare exact rendering and
reused document state before timing, screen each change separately, and confirm
the combination using repeated alternating native process pairs. Keep all real
corpus files in the output guard, including known pre-existing differences from
Ox. Repeat public comparisons independently with their established protocols.

**Decision:** use memchr for line ends in the block cursor and line lookahead.
Coalesce adjacent code or HTML events only while consuming them in the private
renderer, after the existing table-cell shortcut. The public block and MDX event
streams retain their original line ranges. Contiguity checks prevent including
container prefixes, removed indentation, or virtual spaces. Existing block
rendering still handles custom fence callbacks and document state. Filtered
trusted HTML retains its original dispatch boundaries; raw and escaped policies
can safely consume contiguous ranges together.

**Rejected:** a comparator that defers end-event classification until position
ties, a packed position/precedence key, and inserting code-span points in source
order did not provide a sufficiently broad, stable improvement. The initial
range shortcut ran before the table-cell shortcut and added avoidable work to
tables; the retained order preserves that shortcut. No AST, arena, added buffer,
or event-layout change is introduced.

**Evidence and limits:** the [implementation report](../reports/2026-09-12-ox-corpus-optimizations/REPORT.md)
retains the experimental patches, exact-output guards, raw timed windows,
source and executable identities, allocation checks, and the refreshed public
comparison archives. Gains are concentrated in the profiled HTML/code workloads;
inline-heavy prose remains a separate optimization problem. These measurements
are Apple M1 Pro results, not an x86 speed claim or a universal engine ranking.
MDX compilation and Node wrappers remain outside the competitor timing contract.
