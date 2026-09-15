# Retained refactor: shared anchor output policy

Baseline: `b26c829`. One private helper now owns anchor-opening output for normal
and hook-enabled rendering: URL rewriting, sanitization, escaping, external-link
attributes and titles. Both paths retain their existing child traversal, `in_link`
state management and closing tag. The normal path does not dispatch hooks. The
candidate introduces no new allocation or public interface.

## Validation

All **828 workspace tests**, Clippy with warnings denied, formatting, and workspace
benchmark builds pass. The relevant Python harness suites pass (12 tests). Node
native build, **28 tests**, typecheck, package verification and clean installation
from the local tarballs pass. Clean installation used `npm_config_offline=true`;
the initial online attempt stalled on unavailable optional-package metadata and
was interrupted. No code change was needed for the successful offline run.

The [coverage summary](coverage-summary.json) records **93.99% lines**
(12,337 / 13,126; 789 missed), up from 93.55% before the three refactors. Scope is
unchanged: workspace/all features excluding the N-API crate, exercised separately
by Node tests. No new exclusions were added. Much of this increase comes from
removing duplicated code; it does not represent equivalent additional behavioral
coverage. Region coverage is 93.57%; branch coverage was not measured.

## Results

[Generated timing tables](tables.md) show:

| Workload | Fresh | Reused |
| --- | ---: | ---: |
| Default, broad | −0.09% | +0.25% |
| No-op hooks, broad | +0.32% | −0.11% |
| Default, ten diagnostics | +0.92% | +0.27% |
| No-op hooks, ten diagnostics | +0.12% | +0.57% |

The broad suite is effectively stable in this study. Small diagnostic costs remain:
fresh JSX +1.53% default / +1.22% hooks; reused tables +1.06% / +1.27%; plain links
+0.71% reused default / +1.13% hooks. Configured links stay within +0.31%. These
costs are recorded and accepted for one policy owner; the refactor is not a claimed
renderer speedup. Synthetic inputs do not represent every application workload.
The [cross-path check](hook-default-equivalence.json) confirms identical HTML,
AST and child counts between normal/no-op hooks for all 57 + 10 inputs.

## Combined three-step check

A separate direct comparison from `55f3faa` to the final candidate measures all
three retained refactors together: **+0.56% fresh / +0.38% reused** over the broad
suite. Fresh rounds are +0.20%, +0.66%, +0.56%; reused rounds −0.23%, +0.38%, +0.51%.
The same aggregate-only three-round method and exact-output gates apply. These
are direct measurements, not a multiplication of results from different runs.
See [combined raw results](refactor-combined-broad/summary.json). The small cost is
accepted for removing 529 net production lines and redundant grammar/AST policy.

## Measurement method

Default and no-op-hook workers are compared separately before/after. The generated
hook worker changes only HTML entry points; both hook builds use identical worker
bytes. It inserts no runtime toggle into the default measurement. Source hashes,
binary hashes, compiler settings, registry lock checks and raw evidence are retained
in each run directory. Normal and no-op-hook outputs are also cross-compared.
Existing hook tests additionally exercise replacement, skipping, wrapping and
highlighter fallback. The timing study does not model user callback costs.

Broad timings use the frozen 57 documents, three rounds, seven alternating pairs
and 50 ms windows. They time the full rotating profile batches; each document is
still checked for exact HTML, debug AST including spans, and root-child counts.
Aggregate-only timing cannot establish individual broad-document regressions.
The ten authored diagnostics time each case plus profile batches: three rounds,
five pairs, 40 ms windows. Configured links cover sanitization, Markdown route
conversion, titles, base URLs and external-link attributes. Each job warms up for
20 ms. Builds, checks and coverage collection completed before timing.

All builds use Rust 1.95, generic CPU, fat LTO, default allocator and unchanged
locked registry dependencies. The default baseline is the frozen step-2 build,
whose only differences from `b26c829` are documented comments; the hook baseline
uses that commit directly. Workstation measurements do not prove other platforms'
costs. The [decision](../../decisions/2026-09-15-refactoring.md) records the scope.

Apply [candidate.patch](candidate.patch) to the baseline and follow the
[harness instructions](../../../benchmarks/refactoring/README.md) to reproduce.
The broad corpus retains its [licenses](../../../benchmarks/broad-comparison/licenses/ATTRIBUTION.md)
and [Wikipedia attribution](../../../benchmarks/broad-comparison/WIKIPEDIA-ATTRIBUTION.md).
