# Comprehensive performance profiling design

Status: proposed for review
Date: 2026-07-11

## Context

Ferromark is close enough to `pulldown-cmark` that a difference below 2% can be
measurement noise, compiler drift, or a real parser cost. The repository already
contains important low-level optimizations: reusable buffers, range-based events,
`memchr` scanning, AArch64 NEON paths, fat LTO, one release codegen unit, and PGO
profiling scripts. It also records several neutral or regressive pre-scan, SIMD,
streaming, and allocation experiments.

The next phase therefore profiles the complete parse-and-render pipeline before
making more production changes. It is intended to find unexpected costs, bound
the value of larger structural changes, and distinguish portable source wins
from compiler- or CPU-specific wins.

The profiling work must not prematurely collapse Ferromark into one HTML-only
execution path. A future ecosystem may need remark-like transformations,
plugins, alternate renderers, or event consumers. Performance evidence may
justify an optional direct-render fast path, but not the removal of a useful
intermediate representation without a separate architecture decision.

## Goals

1. Produce repeatable CPU, allocation, memory, and pipeline-work measurements.
2. Cover representative syntax, document sizes, profiles, and trust policies.
3. Compare Ferromark and `pulldown-cmark` only where the performed work is
   semantically comparable.
4. Identify both local hotspots and structural sources of repeated work.
5. Measure the ceiling from Apple Silicon tuning, native code generation, and
   PGO without mixing those gains into portable source claims.
6. Rank follow-up experiments by expected return, risk, portability, and impact
   on future extensibility.

## Non-goals

- Optimizing production parser or renderer code in the profiling change.
- Designing or committing to a public plugin API.
- Replacing the event pipeline with a direct HTML sink.
- Publishing new headline benchmark claims from one development-machine run.
- Adding hand-written SIMD before profiles and instruction data justify it.
- Treating lower security work as a parser optimization.

## Approaches considered

### One larger CPU sample

Rejected as insufficient. A CPU flame graph can locate time, but it does not
explain allocation pressure, bytes copied, buffer growth, event volume, peak
memory, branch behavior, or code-generation differences.

### Begin a structural parser rewrite immediately

Rejected as premature. Direct rendering, aggregated block events, borrowed
paragraph slices, and compact scratch layouts are credible experiments, but
their benefit and architectural cost are not yet measured well enough.

### A bounded multidimensional profiling campaign

Accepted. One profiling-focused change adds reproducible harnesses, metadata,
corpora, counters, and reporting. Production behavior remains unchanged. The
result is an evidence package and a ranked experiment backlog.

## Architecture constraint: preserve transformability

Profiling and later decisions distinguish three conceptual layers:

1. **Parse state and syntax resolution**
   discovers block and inline structure, references, and precedence.
2. **Intermediate semantic representation**
   exposes resolved structure that future transforms or plugins could inspect
   or alter.
3. **Output sinks**
   turn resolved semantics into HTML or another representation.

The current event model does not automatically become the final plugin API, but
it represents an important architectural capability. Profiling may show that
event construction is expensive. If so, acceptable follow-up designs include:

- keeping the event path as the canonical transformable path;
- adding an optional direct HTML sink for callers that request no transforms;
- sharing syntax resolution between event and direct-output sinks;
- using a more compact internal representation that remains traversable;
- fusing stages only behind an internal or explicit fast-path boundary.

An optimization is not accepted solely because it improves HTML throughput. Its
evaluation must also state whether it preserves, narrows, duplicates, or removes
a plausible transformation boundary.

## Profiling harness

Extend the existing `benchmarks/pulldown-comparison` crate with the focused
profiling harness. It is already independent of md4c and comrak, excluded from
the published Ferromark package, and owns the semantically guarded parity
configurations. Keeping one comparison crate avoids duplicating those option
contracts.

The harness has two execution modes:

### Throughput mode

Criterion measures steady-state end-to-end throughput with reusable output
buffers. It stores machine-readable estimates for repeated baseline/candidate
comparisons.

### Diagnostic mode

A long-running binary repeatedly executes exactly one named parser, corpus,
configuration, and phase. This gives Instruments, `xctrace`, `sample`, and
Samply a stable process with little benchmark-harness noise.

Every run records:

- git commit and dirty state;
- `rustc -Vv` and active target;
- Cargo profile and effective `RUSTFLAGS`;
- CPU architecture and operating system;
- corpus name, input bytes, and output bytes;
- parser and complete option set;
- trust/rendering policy;
- iteration count and elapsed time;
- whether PGO or CPU-specific flags were used.

## Corpus matrix

The matrix is broad enough to reveal phase-specific behavior without becoming a
combinatorial benchmark suite.

### Core corpora

- simple prose with little markup;
- mixed CommonMark;
- code-heavy documents;
- safe and unsafe URL-heavy documents;
- reference-definition and reference-link-heavy documents;
- table-heavy documents;
- list and blockquote container-heavy documents;
- delimiter-heavy inline markup;
- raw-HTML-heavy documents;
- Unicode- and entity-heavy documents.

### Sizes

- approximately 5 KB for fixed setup and allocation cost;
- approximately 20 KB for ordinary application documents;
- approximately 50 KB for the existing published comparison range;
- one larger fixture, at least 250 KB, for cache, growth, and scaling behavior.

Not every focused corpus requires every size. The three CommonMark sizes and one
large mixed fixture are mandatory; focused corpora use a representative size.

### Configurations

- Ferromark `Essentials`, `Extended`, and `Full` on compatible input;
- secure-default and trusted rendering as separately named lanes;
- CommonMark, GFM-overlap, and extended-overlap parity configurations;
- `pulldown-cmark` only in semantically guarded parity lanes.

## Measurement dimensions

### CPU time and stacks

Collect repeated time profiles for the primary 5 KB and 50 KB lanes plus each
focused corpus. Reports retain both inclusive and exclusive samples and group
symbols into block parsing, inline parsing, reference work, event production,
rendering, escaping, allocation, and copying.

Use Apple Time Profiler or `xctrace` as the primary local source and Samply as a
second view where useful. A finding is actionable only when it appears in a
repeatable run or is supported by another measurement dimension.

### Allocations and memory

A benchmark-only counting allocator records allocation count, reallocation
count, deallocation count, and allocated bytes inside a controlled measurement
window. Instruments Allocations provides stack attribution for selected lanes.

Where the public API permits clean isolation, measure:

- parser and renderer setup;
- block parsing;
- inline parsing and event emission;
- rendering;
- full end-to-end conversion.

Also record output-buffer capacity and peak resident memory for diagnostic runs.
If internal phase isolation would require production API changes, record only
end-to-end allocation data and use stack attribution rather than exposing new
APIs for the profiler.

### Pipeline work counters

Benchmark-only instrumentation records work that wall-clock profiles cannot
explain reliably:

- block event count by kind;
- inline event count by kind;
- inline marks and emit points;
- buffer growth events and maximum capacities;
- bytes copied into paragraph or scratch buffers;
- bytes visited by major inline stages;
- URL normalization and escaping passes;
- reference-label normalization calls;
- fast-path hits and fallbacks.

Counters are enabled only by a non-default profiling feature. Small internal
hooks behind that feature are allowed, but the feature-off build must have no
counter fields, branches, public API additions, or changed behavior. The
instrumented build is never benchmarked as if it were the normal release build.

### Code generation and hardware

Run a separate compiler matrix on a small set of primary lanes:

- portable/default target behavior;
- current `apple-m1` target configuration;
- `target-cpu=native` as a local ceiling;
- non-PGO and PGO builds.

Where Instruments exposes reliable counters, compare instructions, cycles,
branches, branch misses, and cache-related events. If counters are unavailable
or unstable, state that limitation rather than estimating them from wall time.

Inspect assembly or LLVM output only for functions that are both hot and
sensitive to the compiler matrix. CPU-specific wins remain separately labeled
and do not become the default portable performance claim.

Potential SIMD work is gated by this evidence. Existing NEON and `memchr` paths
are baselines, not assumptions that custom intrinsics are faster. Any later SIMD
experiment needs a scalar or portable fallback and cross-platform correctness
tests.

## Comparison protocol

- Pin and record the benchmark Rust toolchain.
- Run on AC power with stable machine conditions.
- Use at least 80 samples, a 5-second measurement window, and a 3-second warmup
  for screening.
- Repeat promising or surprising results three times.
- Alternate baseline and comparison order when practical.
- Treat changes below 1% as noise unless confidence intervals and medians agree
  across repeated runs.
- Keep secure-default and trusted/parity results separate.
- Store raw Criterion estimates and profiler summaries as artifacts.
- Do not compare instrumented-counter throughput with normal release throughput.

## Analysis output

The profiling report contains:

1. environment and reproducibility metadata;
2. throughput matrix and confidence intervals;
3. CPU hotspot tables by corpus and configuration;
4. allocation and memory tables;
5. pipeline work and scaling counters;
6. compiler/CPU/PGO deltas;
7. unexpected findings and disproven assumptions;
8. a ranked opportunity map.

Each opportunity is scored on:

- observed cost or bounded maximum gain;
- expected throughput and memory benefit;
- implementation and correctness risk;
- portability;
- effect on transformation/plugin flexibility;
- smallest experiment that can validate it;
- stop condition.

The report explicitly identifies negative results so later work does not repeat
them without new evidence.

## Likely follow-up classes

These are hypotheses to measure, not approved implementations:

- lazy or smaller scratch buffers for rare syntax;
- fewer URL safety, normalization, encoding, and escaping passes;
- borrowed contiguous paragraph input with a buffered fallback;
- compact event or mark layouts and less copied metadata;
- reduced repeated inline-stage scans;
- shared syntax resolution with optional event and direct-output sinks;
- better cache locality in block/container state;
- PGO or target-specific deployment recommendations;
- focused SIMD only for a demonstrated scan hotspot.

## Validation

The profiling change must pass:

- formatting and Clippy for all added Rust targets;
- unit tests for metadata and counter-window correctness;
- semantic parity tests for every cross-parser lane;
- smoke execution of every corpus/configuration selector;
- a check that instrumentation is absent from ordinary production builds;
- the existing Ferromark test and CommonMark suites.

Raw profiler traces may be too large or platform-specific for git. In that case,
the repository stores scripts, metadata, compact exports, and summaries while
documenting the location and reproduction command for raw artifacts.

## Deliverables

1. Reproducible throughput and diagnostic runners.
2. The bounded corpus/configuration matrix.
3. Allocation and pipeline-work measurement support.
4. CPU and compiler-matrix profiling scripts.
5. Machine-readable run metadata and Criterion output retention.
6. A completed profiling report and ranked experiment backlog.
7. Updates to the existing performance checklist based on measured evidence.

## Acceptance criteria

- A clean checkout can reproduce every retained measurement command.
- Every result identifies code, compiler, CPU mode, corpus, options, and trust
  policy.
- CPU, allocation, memory, and pipeline-work evidence cover the primary lanes.
- At least one large-document run verifies scaling and buffer-growth behavior.
- Ferromark and `pulldown-cmark` are compared only under semantically guarded
  parity configurations.
- Portable, Apple-specific, and PGO results are reported separately.
- No production parser behavior or public API changes are required.
- The opportunity map includes extensibility impact and a stop condition for
  every proposed structural experiment.
- Direct HTML rendering, if later justified, remains optional unless a separate
  architecture decision explicitly replaces the transformable path.
