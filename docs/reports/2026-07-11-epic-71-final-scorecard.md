# Epic #71 final scorecard

Epic #71 completed an evidence-gated portable optimization campaign. Production
changes were merged only after the issue-specific throughput and correctness
gates passed; rejected experiments remain available through closed draft PRs.

## Merged source wins

- Heading-ID state is now absent when IDs are disabled, removing two repeated
  heading allocations.
- Rare inline buffers are lazy, reducing 5 KB allocation pressure without a
  20/50 KB regression.
- Inline resolvers skip work that the mark summary proves absent; mixed 50 KB
  improved in all three repeated comparisons.
- Container matching stops at the first non-whitespace byte. List-heavy input
  improved by 10.614%, 10.406%, and 10.424% across alternating runs.

## Rejected directions

- The shared secure-URL scan did not meet its mixed/50 KB gate; its exact
  [rejection report](https://github.com/sebastian-software/ferromark/blob/codex/issue-62-secure-url-scan/docs/reports/2026-07-11-issue-62-secure-url-scan.json)
  remains on closed draft PR #73.
- Borrowed contiguous paragraphs cut copies by 79-82% but repeatedly regressed
  20/50 KB and simple prose throughput; see the
  [exact report](https://github.com/sebastian-software/ferromark/blob/codex/issue-66-contiguous-paragraphs/docs/reports/2026-07-11-issue-66-contiguous-paragraphs.json)
  preserved by closed draft PR #77.
- A direct resolved-inline HTML sink would duplicate renderer policy/state or
  weaken the transform boundary; event materialization was not isolated as the
  dominant universal bottleneck.

## Deployment result

Portable remains the source-performance baseline. On the balanced PGO training
set, the separate PGO artifact improved secure Extended CommonMark 50 KB by
4.893% and improved every checked focused corpus; it increases the diagnostic
binary by 5.1%. Apple-M1 (+1.196%) and native (+0.475%) do not justify separate
default artifacts. Ship PGO only as an explicit, reproducible deployment lane.

Detailed baseline, allocation, event, copied-byte, source-experiment, and PGO
evidence live in the dated JSON reports beside this scorecard.
