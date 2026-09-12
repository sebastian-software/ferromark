# ARCH-EXP-023: HTML block continuations and recognition

**Date:** 2026-09-12

**Hypothesis:** after ARCH-EXP-022, HTML-heavy documents still spend substantial
CPU time dispatching physical lines through the general block parser. Root HTML
continuations need fewer state checks. HTML block-tag recognition and end-marker
search are additional costs that can be measured independently.

**Method:** freeze merged main, collect two native profiles per selected document,
screen independent changes, and compare exact HTML plus public block, inline and
MDX streams against the frozen source. Confirm the formatted production source
with three fresh alternating process pairs. Keep all corpus inputs in output
checks, including existing differences from Ox, and measure allocator requests
separately from time.

**Decision:** consume root HTML continuations in a dedicated loop inside
HTML-block recognition after the opener has been consumed. Ordinary Markdown
lines do not check the new path. Preserve one public raw-source event per physical
line. Keep container matching and the first line on the existing
path, and delegate blank-line termination to its existing state transitions.
Preserve definition-list line bookkeeping. Group the unchanged block-tag set by
length and first letter before case-insensitive comparison. Search for candidate
end-marker bytes with `memchr`, then check the original marker within that line.

**Rejected:** a loop that repeatedly calls the original HTML-line handler saves
less work; a newline iterator over the remaining input does not beat direct line
scanning. Deferring the initial line lookahead provides little additional value
in its isolated screen. The general substring-search prototype helps long lines
but adds overhead on some short-block workloads; the retained short-marker search
avoids that setup.

**Compatibility:** public events, source ranges, container boundaries, raw-HTML
policies and filters, source-only comments, EOF behavior, and reused renderer
state remain unchanged. No AST, arena, scratch buffer or heap reservation is
added. The performance patch preserves existing parsing behavior rather than
silently fixing pre-existing output differences.

**Evidence and limits:** the [experiment report](../reports/2026-09-12-html-block-parser/REPORT.md)
retains source identities, exploratory patches, output oracles, raw timing windows,
profiles and allocation observations. Results are from an Apple M1 Pro; they do
not establish an x86 improvement, a general engine ranking, or lower RSS.
