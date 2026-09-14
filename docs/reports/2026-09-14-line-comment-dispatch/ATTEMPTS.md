# Line-comment dispatch attempts

Baseline for both cumulative patches: `78d7e21`. Frozen workers use the same
runtime options and pinned dependencies. Only parser implementation and the
C2 regression fixture differ. No SIMD or new profile API is introduced.

## C1 — reject impossible physical prefixes

**Hypothesis:** physical-line eligibility checks repeat previous-newline and
indentation work even for ordinary text starting with an ASCII letter.

**Change:** after the option and authoritative mapped-line checks, reject any
root-source starting byte other than space or slash.
[Patch](patches/C1-prefix-rejection.patch), [build](builds/C1/build.json).

**Screen:** 11 cases × parse/reuse, two process rounds, three 20 ms pairs each.
Exact HTML and AST equality precedes timing. Plain parser ratios at the three
size targets are 1.031×, 1.016×, and 1.013×. Active ratios are 0.994×, 0.983×,
and 0.998×. The 4 KiB active result is a possible loss; this small screen alone
does not justify promotion. [All results](screen-C1/summary.csv).

**Decision:** retain as part of C2, whose broader validation measures the
combined change. C1 is not promoted independently, and the C2 gains cannot be
attributed to C1 alone.

## C2 — reuse the existing block/paragraph discriminator

**Hypothesis:** block dispatch and paragraph continuation already scan the
line's indentation. Calling comment recognition before those scans adds
redundant work on nearly every ordinary line.

**Change:** build on C1 and call the eligibility helper only when the existing
first non-space/tab byte is `/`. Keep physical-origin eligibility and code/HTML
context boundaries. Establish a fixture for dedented comments and surviving
inline source positions before changing these call sites; it passes both
before and after the change. [Patch](patches/C2-fused-dispatch.patch),
[build](builds/C2/build.json).

**Result:** 37 cases × four stages, three rounds of three 40 ms pairs. Plain
parser time decreases by 4.2–6.3%. Active comment cases are effectively
unchanged at the smaller sizes and save about 1.0% at the largest size.
The disabled plain controls also improve, so the lower feature tax is not
merely shifted to the off path. Mixed Docs and strict CommonMark reused
geometric means are 1.014× and 1.015×. Small individual losses and render-only
controls remain in the [full results](final-C2/summary.csv).

Same-binary off/on remeasurement lowers the 4,125-byte parser tax from +3.6%
to +1.0%; complete reused overhead changes from +2.6% to +0.5%. See the
[report](README.md) for variability and input-specific limits.

**Decision:** retain the combined C2 implementation after all semantic and
workspace checks pass. No syntax or defaults change.

## Context-blind global deletion — considered, not implemented

Deleting every physical `//` line before Markdown parsing would also delete
literal lines inside code and raw HTML. Reproducing those context decisions
would require additional parsing, while compacting the input would also need
source-position mapping. No code or timing was produced for this idea. The
retained change instead integrates recognition into existing parser scans.
