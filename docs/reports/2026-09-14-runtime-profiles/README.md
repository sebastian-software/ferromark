# Runtime feature costs and profile candidates

The study finds meaningful unused-feature overhead in definition-list detection,
parser GFM autolinks, line-comment handling, and renderer URL detection. Other
features mostly charge for actual matching syntax. Four controlled parser-profile
ablations saved 9–41% of complete retained processing time with identical HTML
and AST; these are four authored probes, not corpus-wide guarantees.

Read the [interpretation and configuration recipes](../../runtime-profiles.md)
and the [source-level cost map](../../runtime-feature-costs.md) alongside these
tables:

- [All 37 feature toggles, plain and active syntax at three sizes](FEATURES.md).
- [Candidate profiles on mixed documents and the initial ablations](PROFILES.md).
- [Longer confirmation and parser/renderer autolink interaction](CONFIRMATION.md).
- [Active-probe time per byte and scaling](SCALING.md).

## Frozen execution

Core revision: `c57ef45` (the full revision is in [build.json](build.json)).
The study changes benchmark/documentation files only. One native worker contains
all configurations, selected at runtime; there is no separate optimized binary
per profile. Rust 1.95, optimization level 3, fat LTO, one codegen unit,
`target-cpu=generic`, and unchanged registry versions/checksums are recorded in
the build manifest. [Worker source](worker.rs), [worker lockfile](worker-Cargo.lock),
and [build log](build.log) preserve the measured harness.

The host was arm64 macOS 26.6.2 on AC power. CPU-model and thermal probes were
unavailable in this sandbox; their failed results are retained rather than
replaced by inferred hardware information. Load averages and timestamps are in
each run manifest. This was a shared workstation with no core/frequency pinning.
Small differences are not reliable rankings, and absolute times are host-specific.

The main sweep covered 294 cases: 222 single-feature probes, three unclosed
frontmatter probes, 65 mixed-document/profile pairs, and four same-output
ablations. There were 1,096 timed case/stage jobs across two process rounds,
three alternating pairs per round, and 20 ms minimum windows: 6,576 paired
observations. All four lifecycles were verified even where redundant timing
stages were omitted.

The confirmation selected 33 cases after the main sweep and remeasured 116
case/stage jobs across three process rounds, three pairs each, and 50 ms windows.
This is a targeted recheck, not a second independently selected corpus. A separate
six-case interaction run used the same longer schedule. These yielded another
1,044 and 216 paired observations respectively. Each worker received 5 ms warmup.

The `commonmark` self-comparisons and ignored `highlight`/`soft_break` options
serve as noise controls. Some individual confirmation windows moved by several
percent; per-round medians and complete ranges remain visible. Large main
findings and all four ablation directions reproduced in the longer runs.

## Data and verification

| Dataset | Main sweep | Longer confirmation | Autolink interaction |
| --- | --- | --- | --- |
| Exact frozen inputs/configurations/origins | [corpus](corpus.json.gz) | [corpus](confirmation/corpus.json.gz) | [corpus](interactions/corpus.json.gz) |
| Complete HTML and AST verification | [verification](verification.json.gz) | [verification](confirmation/verification.json.gz) | [verification](interactions/verification.json.gz) |
| Raw paired windows/checksums | [samples](samples.json.gz) | [samples](confirmation/samples.json.gz) | [samples](interactions/samples.json.gz) |
| Absolute times, ratios, ranges, round medians | [JSON](summary.json), [CSV](summary.csv) | [JSON](confirmation/summary.json), [CSV](confirmation/summary.csv) | [JSON](interactions/summary.json), [CSV](interactions/summary.csv) |
| Host, timing settings, harness hashes | [run](run.json) | [run](confirmation/run.json) | [run](interactions/run.json) |

Verification requires each configuration to produce identical HTML/AST across
fresh, retained, parse-only verification, and render-only lifecycles. Active
probes must demonstrate the feature in their HTML; explicitly ignored settings
must preserve exact output. Same-output ablations and interaction probes compare
both HTML and AST, including metadata. Checksums and post-timing output checks
guard every timed job. This establishes internal measurement consistency, not
an independent conformance oracle for every extension.

Feature-on/off comparisons with different output are cost observations. Enabling
a construct can reduce parsing work or change output size; a negative ratio is
not automatically an optimization. Synthetic active probes repeat short blocks
and labels. Their feature density and footnote/heading duplication are not
representative of every publication. Five profile recipes are separately measured
on 13 mixed documents. MDX is measured on authored MDX inputs rather than treating
arbitrary framework documentation as MDX.

All source origins and licenses travel with the frozen corpus. See the
[project-document attribution](../../../benchmarks/broad-comparison/licenses/ATTRIBUTION.md)
and [Wikipedia attribution](../../../benchmarks/broad-comparison/WIKIPEDIA-ATTRIBUTION.md).
HTML and debug AST outputs are mechanical transformations of the attributed inputs.

Reproduce using the [runtime-profile harness](../../../benchmarks/runtime-profiles/README.md).
The original six-engine benchmark timings are unaffected by this separate study.

Validation: 763 workspace tests across 54 suites, four measurement-contract tests,
workspace and standalone-worker Clippy, formatting, and all workspace benchmark
builds pass. The article recipe compiles and runs through the public facade.
Documentation links and rendered table structures were checked. Archived corpus,
worker, and lockfile hashes match their manifests; see [checks.json](checks.json).
