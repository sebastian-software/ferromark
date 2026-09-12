# ARCH-EXP-027: Linear text and attribute escaping

**Status:** Adopted
**Date:** 2026-09-12

## Decision

Bound both searches used to locate HTML escapes, including attribute quotes.
Search a shared 128-byte prefix inline, then use an outlined continuation.
AArch64 with NEON uses the existing byte-set scanner for continuation inputs of
at least 256 bytes. Shorter tails use one bounded memchr window. Other targets
search disjoint, geometrically growing memchr windows.

The prior long path repeatedly searched the entire remaining suffix for `<>&`
before checking quotes. Dense quotes without `<>&` therefore made total work
quadratic. Keeping both searches inside a window bounds lookahead beyond the next
escape; doubling window sizes amortizes setup on unmatched runs. Each output
iteration advances past the escape, making total work linear. Saturating growth
and slicing by the remaining length keep the arithmetic bounded.

The byte set and replacements do not change. Attribute escaping still includes
single quotes while text escaping does not. Existing `ByteSet` and memchr
backends remain responsible for searching; there is no new unsafe code,
allocation policy, output-capacity reservation or parser feature.

## Validation and alternatives

The [report](../reports/2026-09-12-linear-html-escaping/REPORT.md) preserves failing
work-count reproducers, exact source snapshots, repeated native measurements,
A/A controls, output guards and reconstruction tools. Release builds contain no
work counters. Boundary tests compare all byte values with a scalar oracle.

This follows [ARCH-EXP-026](ARCH-EXP-026-neon-text-escape-integration.md), whose
five exact patches remain rejected. Global replacement, unconditional long
NEON scanning, dispatch thresholds, cached cursors and whole-output dispatch all
receive counterexample screens in the two reports. The adopted integration keeps
short bounded tails on memchr and applies the joint scan to longer NEON tails.

No measured loss is averaged away using assumed workload frequencies. Tiny paired
differences are assessed alongside identical-binary variation and longer
counterchecks; the record does not prove that every input on every machine has
identical or improved performance. Published engine rankings and memory claims
are outside this before/after experiment. Throughput evidence is from AArch64;
other targets require their own native performance measurements.
