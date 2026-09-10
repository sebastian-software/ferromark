# GFM optimization experiments — 2026-09-10

Follow-up to the [baseline profile](2026-09-10-gfm-profiling.md). Each candidate
is measured against its immediate predecessor using retained executables of
the same uninstrumented `release-debug` probe. The paired driver checks exact
HTML across 90 configurations before collecting seven alternating 150 ms
windows per case and API lifecycle. Negative changes mean less elapsed time.
These remain local Apple M1 Pro results, not universal throughput claims.

## 1. Reuse strikethrough opener scratch — retained

The inline parser now retains the resolver's opener buffer. The resolver clears
its contents on every invocation, preventing unmatched openers from crossing
cell or document boundaries; allocation stays lazy when strikethrough is absent.

- Warmed Renderer allocations on the repeated table/strike input: 80 → 0;
  requested bytes: 2,560 → 0.
- Shared table/strike subset: -5.04% with fresh parser/reused output, -5.75%
  with retained Renderer.
- Full GFM on that input: -6.76% / -5.59%; mixed GFM: -5.82% / -3.84%.
- Other measured cases stayed within approximately 1.7% in either direction.
- Validation: all-feature Rust tests passed, including a new reuse regression
  covering unmatched openers, multiple cells/documents, code spans, and links.
  All 90 probe outputs matched the baseline exactly.

[Timing windows](2026-09-10-gfm-profiling/optimizations/strike-comparison.json)
and [allocation counts](2026-09-10-gfm-profiling/optimizations/strike-counts.json)
are retained with the experiment.
