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

## 2. Preclassify table rows before splitting — rejected

A row-level backslash/backtick scan selected a single-byte pipe search for
ordinary rows. All-feature tests and all 90 exact-output comparisons passed,
including added escape/code tests across scan lengths. The target table/strike
case nevertheless changed by +0.85% to +1.72%, and the tables fixture showed
+0.09% / -1.17%. This does not justify the extra scan and branch. The experiment
is preserved in history; its implementation is reverted, with tests retained.

[Paired measurements](2026-09-10-gfm-profiling/optimizations/table-prescan-comparison.json).

## 3. Special-case a single text event — rejected

Bypassing general renderer setup for exactly one text event reduced the table
fixture's elapsed time by 3.16% / 3.65% and helped prose/tasks. However, the URL
fixture regressed by 2.61% / 2.64%, and full-GFM README rendering by 2.85% / 1.22%.
All-feature tests and exact-output checks passed. The tradeoff does not justify
retaining this branch; preserve and revert it before trying a text-event shortcut
inside the existing render loop, which also benefits mixed inline content.

[Paired measurements](2026-09-10-gfm-profiling/optimizations/single-text-comparison.json).
