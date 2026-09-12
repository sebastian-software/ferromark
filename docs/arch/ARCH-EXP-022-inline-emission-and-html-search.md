# ARCH-EXP-022: Inline emission and HTML candidate search

**Date:** 2026-09-12

**Hypothesis:** after ARCH-EXP-021, inline event construction and ordering remain
visible costs in the Rust Book and Vue corpus. Fewer code-span points and smaller
point payloads can reduce this work. Already ordered newline/escape points might
also avoid a global sort. Fresh profiles additionally identify scalar searching
for inline HTML candidates as a substantial cost in Vue Suspense.

**Method:** freeze merged main, sample the native rendering operation twice per
selected input, and screen independent changes against exact rendering and public
inline/MDX events. Compare combinations with alternating native timing windows;
measure requested live heap bytes separately. Rebuild and verify the formatted
production source before recording final measurements.

**Decision:** combine code opener/content points only when the paragraph has no
resolved links, reference links, math, footnote references, or inline footnotes.
Keep the separate closing point and preserve the original three-point sequence
in the remaining cases. Store indices into existing immutable resolution records
instead of copying link, autolink, math, and inline-note payloads into the private
sort buffer. Share the existing code/math padding rule. Find inline HTML candidate
openers with `memchr`, retaining escape checks, protected ranges, and the existing
HTML recognizer.

**Rejected:** replacing all three code points with one changes existing output.
Unconditionally combining the opener and content also changes some public event
streams in mixed syntax. The explicit fallback preserves these interactions.
Merging the mark family through two iterator variants, and bounded insertion of
marks into the sorted construct family, did not justify their runtime costs on
the controls. A link-only payload reduction provides less memory benefit than
indexing all rich payloads.

**Compatibility:** public block, inline, and MDX event layouts are unchanged.
Resolution records remain immutable while their indices are consumed. No new
scratch buffer or allocator is introduced, and the global ordering and suppression
rules remain in effect. These optimizations preserve existing parser behavior;
they do not resolve the previously observed differences from Ox.

**Evidence and limits:** the [experiment report](../reports/2026-09-12-ox-inline-emission/REPORT.md)
retains profiles, prototype patches, exact-output checks, timing windows, and
allocation observations. The memory measurement covers allocator requests, not
RSS; savings depend on the size of the inline scratch buffer rather than document
size alone. CPU measurements are from an Apple M1 Pro and do not establish an x86
speedup or a universal ranking against another engine.
