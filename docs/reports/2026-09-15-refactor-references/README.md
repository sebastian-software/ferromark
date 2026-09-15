# Retained refactor: one block grammar for link definitions

Baseline: `a185287`. The candidate replaces option mutation with a private parse
phase and removes the flat reference scanner. A cheap shape filter avoids a
collection pass when no link definition is possible; enabled footnote-only inputs
keep their separate raw-line scan. Definition discovery uses the same block parser
as the final document, omitting ordinary inline content. Temporary block allocations
are released after the map is copied into the document arena.

All 827 workspace tests at the validation checkpoint passed, together with Clippy
with warnings denied, formatting, and workspace benchmark builds. The subsequently
added quote-fence regression also passed in the six-test container-reference target.
No specification fixtures, conformance snapshots or coverage exclusions changed.

## Correctness

The 57-document broad corpus and nine authored diagnostics retain exact HTML,
debug AST including source spans, and root-child counts across fresh/reused
lifecycles, before and after timing. One deliberate semantic correction is outside
those corpora: an unclosed quoted fence no longer hides a later root reference.
The previous edge expectation was wrong under CommonMark example 128. The
[decision](../../decisions/2026-09-15-refactoring.md#intended-semantic-correction)
explains the correction and new exact HTML tests. Footnote scope is unchanged.

## Measurements

[Generated tables](tables.md): broad aggregate **+0.33% fresh / −0.29% reused**;
nine-case aggregate **−2.09% / −1.36%**. The dense root-reference probe improves
**11.08% / 11.19%**, and footnote-only input improves **7.12% / 7.63%**. These are
synthetic workload results, not universal speedup claims. Fenced decoys remain a
possible unnecessary structural pass; their measured change is +0.49% / +0.93%.

Broad runs use three rounds, seven alternating pairs and 50 ms windows, timing
only the rotating profile batches while checking every document for equality.
They cannot establish the absence of individual outliers among the 57 documents.
Targeted runs time each diagnostic plus a rotating batch: three rounds, five pairs,
40 ms windows. Each job warms up for 20 ms. Builds and tests finished before timing.
Both workers use Rust 1.95, generic CPU, fat LTO, the default allocator, and unchanged
locked registry dependencies. Raw samples, outputs, options, host observations,
[build metadata](structural-broad/builds.json), and corpus provenance are archived.

The earlier phase-only and forced-structural prototypes motivated this candidate;
only the final simplified implementation supplies the retained measurements here.
Apply [candidate.patch](candidate.patch) to the baseline and follow the
[harness instructions](../../../benchmarks/refactoring/README.md) to reproduce.
The measured build precedes comment-only wording corrections recorded in the patch.
Broad input [licenses](../../../benchmarks/broad-comparison/licenses/ATTRIBUTION.md)
and [Wikipedia attribution](../../../benchmarks/broad-comparison/WIKIPEDIA-ATTRIBUTION.md)
remain applicable. Workstation results do not establish other architectures' costs.
