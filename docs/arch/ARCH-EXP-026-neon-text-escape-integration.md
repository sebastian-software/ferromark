# ARCH-EXP-026: NEON text-escape integration

**Status:** Not adopted
**Date:** 2026-09-12

Follow-up: [ARCH-EXP-027](ARCH-EXP-027-linear-html-escaping.md) adopts a different
bounded integration for both text and attributes. The five patches evaluated
here remain unadopted.

## Decision

Retain the merged escape implementation after testing five integrations of the
shared four-byte NEON scanner. A favorable mean or a large synthetic improvement
is insufficient when ordinary controls repeatedly become slower and there is no
measured workload-frequency justification. This applies a stricter acceptance
policy than the preceding screen's over-2% regression threshold.

The direct replacement improves several Vue and Rust documents but repeatedly
penalizes table, MDX and separate-HTML controls. Outlining long searches, changing
only subsequent searches, outlining whole long-output loops and forcing the bulk
dispatcher to inline each retain or introduce counterexamples. No library source
change from these experiments is adopted.

## Finding worth preserving

The existing long text search combines a `<>&` search with a bounded quote
search. On quote-heavy input without `<>&`, each new quote causes the first
search to traverse the remaining suffix again. This is quadratic scanning at
fixed quote density. A single four-byte scan removes that rescan pattern and
strongly improves large quoted-code controls; the complete integration still
needs to avoid the measured penalties elsewhere.

The [report](../reports/2026-09-12-neon-text-escape/REPORT.md) archives the stress
reproducer, five integration patches, complete screens, longer counterexample
follow-ups, exact-output guards and reconstruction tools. The experiments only
change AArch64 with NEON. Their measurements on Apple M1 Pro do not establish
x86 performance, workload frequencies or a new engine ranking. No new allocation
or memory-consumption claim is made.
