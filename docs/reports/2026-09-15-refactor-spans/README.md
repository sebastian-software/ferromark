# Retained refactor: shared full-tree source-span traversal

Baseline: `55f3faa`. The candidate patch replaces the constant-offset and JSX
walkers with mapping policies behind the existing `SpanMap` traversal. Three
full-tree implementations become one, removing more than 300 repeated lines.
Inline/block distinctions, MDX attribute spans, table captions/rows/cells, list
items, root document extent, and mapping order remain unchanged. The compact
restricted table-cell traversal remains separate: its node-domain policy is not
interchangeable with a full-tree traversal.

All 826 workspace tests, Clippy with warnings denied, formatting, and workspace
benchmark builds pass. The 57-document broad corpus and eight authored diagnostics
preserve exact HTML, debug AST (including spans), and root-child counts across
both revisions, fresh/reused lifecycles, and all measurement rounds.

[Generated timing tables](tables.md) retain the results: broad workload +0.79%
fresh and +0.12% reused; targeted workload −0.69% fresh and +0.19% reused. Broad
fresh rounds are +0.60% and +0.98%; reused rounds are −0.06% and +0.31%. The small
fresh cost is visible and accepted for eliminating duplicate traversal; this is
not a claimed speedup. There is no large repeatable targeted regression.

Broad timing uses two rounds, three alternating pairs, 30 ms windows; targeted
timing uses three rounds, five pairs, 40 ms windows. Each job warms up for 20 ms.
Builds and tests completed before timing. Both workers use Rust 1.95, generic CPU,
fat LTO, the default allocator and unchanged locked registry dependencies.
[Build metadata](span-broad/builds.json), raw samples, exact outputs, input
provenance, host observations and options are retained under each run directory.
Workstation results do not establish performance on other architectures.

Apply [candidate.patch](candidate.patch) to the baseline to reconstruct the core.
Use [the harness](../../../benchmarks/refactoring/README.md) to reproduce. The
[harness hashes](harness-sha256.json) identify the recorded scripts. The broad
corpus keeps its [source licenses](../../../benchmarks/broad-comparison/licenses/ATTRIBUTION.md)
and [Wikipedia attribution](../../../benchmarks/broad-comparison/WIKIPEDIA-ATTRIBUTION.md).
