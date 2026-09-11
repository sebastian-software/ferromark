# Benchmark comparability and documentation

**Status:** Native comparison and publication refresh implemented; targeted profiling and additional application contracts remain follow-ups.
**Date:** 2026-09-11

## Objective

Explain what each measurement buys the caller: supported syntax, rendering
policy, output, and allocation lifecycle. Profile the remaining costs before
choosing another optimization. This follows [ADR-0010](../arch/ADR-0010-explicit-markdown-options-and-dialect-presets.md):
syntax presets are contracts, not performance tiers.

## Delivery status

- **Already merged in [PR #283](https://github.com/sebastian-software/ferromark/pull/283):**
  the native Bun comparison, its md4c-to-Zig-to-Rust provenance, limited output
  equivalence gate, historical measurements, and shared byte-search work.
- **Implemented in [PR #293](https://github.com/sebastian-software/ferromark/pull/293):**
  Ferromark GFM and feature-cost profiling, measured allocation/scan
  optimizations, before/after evidence, and this comparison audit and plan.
  README and homepage now explain the native Bun and granular feature studies,
  distinguish the measurement questions, and disclose the historical comparison's
  unequal policies/lifecycles. Its labels are corrected; measured figures are unchanged.
- **Implemented in this follow-up:** the shared native harness now includes C-md4c,
  explicit per-feature configurations, fresh owned output for all five parsers,
  a complete per-fixture output gate, and repeated publication measurements.
  README/homepage use generated 2/5/10-KiB and feature-set tables, with Bun in
  every public comparison. The [refresh report](../reports/2026-09-11-benchmark-refresh.md)
  retains conditions, variability, exclusions, and the separate feature replay.
- **Still open:** a matched untrusted-input pipeline, cross-parser retained-buffer
  and retained-state comparisons, native x86 results, and broader real-document
  sampling. Task-list cases with different output remain excluded. The targeted
  CPU investigations at the end of this plan are not completed by a timing rerun.

Bun is part of the main five-parser publication environment. Its original
nightly/native-support conditions now apply to all five adapters; these results
remain separate from stable/System-allocator diagnostics.

## Findings before the documentation update

The audit uses the committed adapters and their locked comparison versions:
pulldown-cmark 0.13.4, Comrak 0.54.0, and md4c revision
`65c6c9d72cebd9a731aaa5597414ce04d9ea5de3`. These are not claims about every
version or possible configuration of those libraries.

The README/homepage wording and labels identified below have since been
corrected in #293. The native adapter, parity-gate, and publication-environment work is now
implemented in the refresh; the table preserves the audit that motivated it.

| Finding | Evidence | Consequence |
| --- | --- | --- |
| The headline “CommonMark” inputs contain tables, with table/strikethrough/task-list parsing enabled. | [Four-parser harness](../../benchmarks/md4c-comparison/benches/comparison.rs), [README](../../README.md) | Describe the existing run as mixed Markdown with tables; use extension-free fixtures for a future pure CommonMark comparison. |
| Rendering policies differ: Ferromark uses Untrusted, Comrak its default HTML omission, while pulldown-cmark and md4c pass through raw HTML. | The four-parser adapter functions | Equal extension flags do not establish equal output or security work. Verify the actual corpus; a policy difference need not affect every document. |
| The headline group reuses output for three parsers but calls Comrak's owned-output convenience function. | `bench_commonmark_group`; Comrak's [format_html API](https://docs.rs/comrak/0.54.0/comrak/fn.format_html.html) | Add a Comrak adapter using a fresh arena/parser and a retained output String before the next buffer-reuse comparison. |
| The homepage says “Same GFM settings for all parsers.” | [Home route](../../homepage/app/routes/home.tsx) | Replace this with the exact shared extension subset, rendering policy, and lifecycle. |
| The two-parser harness has semantic spot checks, not a complete output-equivalence gate for its timing corpus. | [Parity tests](../../benchmarks/pulldown-comparison/tests/parity_semantics.rs) | Treat “overlap” as the chosen feature intersection until per-fixture equivalence is established. |
| The two-parser directory pins Rust 1.93.0, below the root crate's Rust 1.94 minimum. | [Toolchain](../../benchmarks/pulldown-comparison/rust-toolchain.toml), [manifest](../../Cargo.toml) | Update the publication environment before replaying it; record the new toolchain and preserve historical run metadata. |

Do not silently reinterpret historical numbers as results from corrected
adapters. Keep their original conditions visible until a new measured run
replaces them. Do not edit published throughput values by hand.

## Compare tasks, not flag names

Start with these bounded comparison contracts:

| Contract | Configuration and limits |
| --- | --- |
| CommonMark, trusted | Core syntax, raw HTML passthrough, no extensions, equivalent output for each timed fixture. |
| Shared GFM subset, trusted | Add tables, double-tilde strikethrough, and task lists. Explicitly exclude bare autolinks and tag filtering; do not call this full GFM. |
| Individual extended features | Compare only supporting parsers, one feature and its output contract at a time. Mark missing support as unavailable. |
| Application defaults | Measure each documented default and disclose differences; this answers an integration question, not equal-work parser throughput. |

For example, pulldown-cmark's [0.13.4 options](https://docs.rs/pulldown-cmark/0.13.4/pulldown_cmark/struct.Options.html)
use `ENABLE_GFM` for special blockquote tags, not an all-GFM switch.
`ENABLE_HEADING_ATTRIBUTES` accepts explicit attributes; it does not mean
automatic heading slug generation. Its strikethrough behavior on single tildes
also differs from Ferromark's [double-tilde contract](../arch/ADR-0008-inline-markup-syntax-alignment.md).
Comrak exposes separate [extension options](https://docs.rs/comrak/0.54.0/comrak/options/struct.Extension.html).
Map those options to tested behavior rather than counting enabled switches.

Keep core syntax enabled in Ferromark when the competitor cannot disable it.
Removing HTML events after parsing is not equivalent to disabling HTML parsing.
Do not add artificial feature switches to competitors just to match our API.
If an extra enabled feature is absent from an input, disclose it and test both
neutral text and near-miss syntax before assuming zero detection cost.

A comparable untrusted-input pipeline is a separate contract. Define raw-HTML
and URL behavior first, then include any necessary external filtering in the
timed work. Similar option names do not prove equivalent safety policies.

## Keep lifecycle and environment visible

Report three separate API-use cases: fresh parser with owned output; fresh
parser with retained output; and retained parser state plus output where the
public API supports it. The last case measures available integration benefits,
not a universally available parser operation. Include parsing on every iteration;
do not compare rendering a cached AST with parsing a new document.

Construct constant options outside the timer in all adapters. Count unavoidable
per-document setup, output clearing, parsing, rendering, and destruction
consistently. Record whether buffers are warmed and how their initial capacities
are chosen. Report fresh calls as well as steady state for short documents.

The [native Bun comparison](../../benchmarks/bun-comparison/README.md) is a
separate experiment with a pinned Bun revision, nightly compiler, shared native
support/allocator, and owned-output lifecycle. Its numbers cannot be combined
with stable-toolchain, system-allocator, retained-output results into one ranking.

## Correctness and publication gate

Before timing, compare output for every selected fixture. Prefer exact HTML;
allow only explicit, tested serialization differences. Do not normalize away
visible text, URL behavior, code whitespace, IDs, classes, or checkbox semantics.
Preserve mismatches and publish inclusion/exclusion counts and reasons. Where
output differs materially, show a separate task instead of an equal-work ratio.

Use frozen real documents alongside synthetic cases: short comments, lightweight
documentation, a README, release notes, and a larger mixed document, with source
revision and content hashes. Dense feature inputs diagnose costs; they do not
establish a universal parser ranking. Retain edge cases as correctness controls.

Extend run metadata with every parser option, adapter/source revisions, lockfile
and input hashes, output sizes, toolchain, target CPU, allocator, build flags,
lifecycle, timer boundaries, and correctness outcomes. Use repeated alternating
runs and the existing publication protocol, not the short optimization probes,
for public competitor claims. Publish native x86 results only after native runs.

For each eligible parser/input/configuration/lifecycle combination, report
latency per document and input throughput, repeated-run variation, and output
size. Measure allocation calls and cumulatively requested bytes separately from
uninstrumented timing; do not label cumulative bytes as peak memory. Keep
activation cost on identical-output inputs separate from the cost of rendering
additional syntax. CPU profiles explain candidate bottlenecks, not rankings.

Generate README and homepage figures from the same reviewed run artifact. Keep
the current benchmark-number contract, and extend it to cover configuration
labels and provenance when the schema changes.

## Documentation shape

- **README:** explain syntax presets, default rendering policy, and Renderer
  reuse first; show a small, explicitly scoped benchmark summary and link to
  methodology. Replace subjective star scores and unprofiled causal claims with
  verifiable capability/API facts. Avoid “nothing cherry-picked” claims.
- **Homepage:** put corpus, lifecycle, and policy next to the result. Link the
  comparison guide; avoid a blanket fastest-parser promise from two fixtures.
- **Benchmark guide:** show the supported task matrix, mismatches, environment,
  and per-parser options. Separate activation/detection cost from syntax actually
  used, and fresh-call setup from retained-state work.

The [final feature-cost report](../reports/2026-09-10-markdown-feature-costs-final.md)
and [optimization report](../reports/2026-09-10-markdown-feature-optimizations.md)
already support the last distinction. Feature-heavy documents differ in block
count, syntax density, and HTML expansion: their time-per-KiB values are not
independent feature prices that can be added together. Unused options are often
cheap, but that does not make every option free or semantically interchangeable.

## Targeted next profiling round

Use production revision `41d5201c804e760f400a2d25e2ebe06394ec6f04` as the
recorded starting point and replay the frozen catalog. First collect current
CPU and allocation evidence; only then test one implementation hypothesis.

1. Isolate the confirmed retained-Renderer inline-code slowdown (+4.8%, about
   0.26 microseconds per 32-paragraph control). Compare capacity growth, retained
   state, and generated code; none is an established cause yet.
2. Profile reference-heavy documents to locate remaining normalization and
   allocation work. Check the historical experiment log before repeating a
   previously rejected cache or scan optimization.
3. Profile entity-heavy and list-heavy inputs alongside realistic mixed controls
   to distinguish decoding, block construction, and HTML output costs.

Keep exact output checks and paired fresh/retained controls for each experiment.
Confirm apparent regressions with longer alternating runs. Retain changes only
with repeatable benefits and an explicit assessment of their regressions; record
rejected ideas compactly, following [ARCH-EXP-018](../arch/ARCH-EXP-018-neon-position-and-match-cache.md).

The comparison contracts, native adapters, output gate, repeated cross-parser
measurements, and public documentation refresh are delivered by the linked report.
Resume targeted CPU profiling as a separate investigation, beginning with a fresh
recorded production revision and preserving the frozen controls. A new timing
snapshot does not establish which implementation change would help.
