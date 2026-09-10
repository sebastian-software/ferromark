# ARCH-EXP-018: NEON position and match-cache experiments

**Date:** 2026-09-06  
**Decision:** Rejected; no production implementation retained.

## Question

Would direct NEON first-hit localization or caching the remaining matches in a
vector block improve complete Markdown-to-HTML rendering?

## What was tested

Six independent variants started from merged PR #283, commit
`dc7716be5728f233353c8cec789597cd0e33c85a`:

- Three first-hit variants: horizontal minimum, packed positions, and packed
  positions restricted to small byte sets.
- Three match-cache variants: eager caching, a no-hit guard before mask
  conversion, and lazy cache construction after the first search.

## Result and decision

Some delimiter-heavy inputs improved, but mixed Markdown did not improve enough
to justify the complexity. Longer counterchecks confirmed regressions on links,
tables parsed as CommonMark, and inputs with a late first delimiter. None of the
six variants was retained; the public byte-search API and production code stayed
unchanged. Output comparisons covered 78 input/preset/lifecycle combinations;
the cache prototypes also tested all 65,536 possible 16-byte hit masks.

## Scope and revisit conditions

Measurements used Apple M1 Pro, Rust 1.97.1 / LLVM 22.1.6, Apple M1/NEON flags,
fat LTO, one codegen unit, and no PGO. These findings do not establish x86-64
performance or rule out other algorithms. No instruction-level profile isolated
the relative costs of mask conversion, bookkeeping, inlining, or code layout.

Revisit only when new profiles identify this work as a material bottleneck,
a different platform warrants measurement, or a substantially different
algorithm changes the hypothesis. Avoid repeating the same six implementations.

## Original evidence

[PR #284](https://github.com/sebastian-software/ferromark/pull/284), head
`09e38936d31870b3f9d9a0712a77813f93d21d32`, contains the full report, archived
patches, raw measurements, test logs, and hashes. They are intentionally not
copied into the active codebase; this note preserves the finding and its limits.
