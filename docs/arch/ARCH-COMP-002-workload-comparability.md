# ARCH-COMP-002: Compare Markdown work, report HTML fidelity separately

**Status:** Accepted
**Date:** 2026-09-11

## Decision

Performance comparisons require matched feature settings and comparable completed
Markdown work. They do not require identical HTML or a perfect correctness score.
The original five-parser HTML-equivalence gate was stricter than the benchmark's
purpose. This decision supersedes the exclusion policy in the September 11
output-parity audit; its captured observations remain historical evidence.

The harness records HTML hashes and normalized differences independently of
workload eligibility. Known renderer differences and small implementation bugs
are accepted with explicit disclosure:

- Ordinary HTML flow whitespace is normalized, preserving inline word boundaries
  and literal/code text.
- The pinned Bun table-alignment bug is admitted: it still recognizes and renders
  the same tables, rows, cells, and content, despite choosing wrong alignment.
- Task checkbox placement inside or immediately before the first paragraph,
  renderer task classes, and spacing beside a checkbox are admitted. The same
  list structure, task labels, checkbox count, and checked states must remain.

The native Sätteri follow-up also reviews its `contains-task-list` class on
`ul`/`ol` as task presentation. The projection ignores exactly that class on
those list containers; unknown classes, list type, content, checkbox count,
and checked states remain significant. Positive and negative contracts cover
this extension of the reviewed renderer conventions.

The workload projection is only a review mechanism outside the timer. Parsers
render their own original HTML during measurements; normalization does not add
work to any candidate's timed path. Output byte counts are retained. CSS/layout
fidelity is not established by an admitted workload.

A parser that leaves the tested feature literal, omits content, or stops because
of a resource limit performs materially different work and remains ineligible.
Unrecognized differences trigger review rather than automatic acceptance. This
rule applies equally to every parser, including Ferromark; majority agreement is
not proof of correctness. The initial reviewed exceptions cover table alignment
and task presentation, not arbitrary attribute or content changes.

Both `run.py` and `publish.py` use the same workload review. Effective options,
raw-output hashes, sample counts, output sizes, and source revisions remain
publication requirements. Renderer differences cannot excuse a misconfigured
feature flag such as the earlier md4c task/wiki mix-up.

## Task-list standard

[GFM section 5.3](https://github.github.com/gfm/#task-list-items-extension-)
defines task markers and semantic checkbox states. It explicitly leaves checkbox
interaction unspecified. The observed CSS classes and paragraph-placement
choices are benchmark-compatible renderer contracts, without asserting that all
possible rendering choices are normatively equivalent.

## Evidence and publication

The [earlier audit](../reports/2026-09-11-output-parity-audit.md) contains the
small table and task reproductions. With its corrected md4c adapter, the new
workload policy admits 40/42 archived cases: six renderer-different cases join
the 34 matching-HTML cases. The two remaining historical exclusions reach
Ferromark's former reference budget.

[ADR-0014](ADR-0014-reference-resolution-budget.md) increases that budget.
A fresh native screening run exercises all 42 cases with the new production
limit. Screening is validation of coverage, not evidence for public speed claims;
public figures retain their recorded source revisions and full sampling protocol.

The [screening evidence](../reports/2026-09-11-benchmark-refresh/workload-screening/metadata.json)
records 42 admitted cases: 36 matching-HTML cases and six reviewed renderer
differences. All 210 parser/case combinations were exercised, with five short
sample windows each. The [verification](../reports/2026-09-11-benchmark-refresh/workload-screening/verification.json)
records both the unchanged HTML diagnostics and workload decisions. It also
preserves spec diagnostics; a timing admission is not a correctness certification.

## Publishing independent native pairs

The archives retain the separately measured Ferromark baseline for each native
competitor. The public overview shows Ferromark once per document: the median of
those reference medians, with candidate times preserved and relative speeds
calculated against the overview reference. This descriptive aggregation is
disclosed beside the tables; the original paired ratios remain in the reports. System-allocator pairs and the shared-mimalloc five-parser
experiment remain distinct datasets. Their absolute times are not combined into
one engine ranking, and a missing workload is never filled from another input
or run. Heading-ID exclusions and specification diagnostics stay visible beside
the native results, including cases where a competitor takes less time.

The native publisher verifies archived reports, checksums and raw timing windows
before producing homepage data and the additional README section. The existing
five-parser publisher assembles the benchmark section in `README.md.src`; mdtheme
alone composes the generated root README. This preserves both measurement
provenance and generated-file ownership.
