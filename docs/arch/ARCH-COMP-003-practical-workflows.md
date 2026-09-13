# ARCH-COMP-003: Measure publishing workflows and output lifetimes

**Status:** Accepted
**Date:** 2026-09-13

## Questions before measurements

The existing native comparisons isolate syntax and parser implementation costs.
This complementary series starts with decisions an application author makes:

1. What does a burst of short user-authored previews cost with the actual secure
   defaults? Does retaining parser scratch through `Renderer` help?
2. What does rendering documentation together with front matter and heading
   metadata cost? A body-only renderer does not perform that complete job.
3. What does the Markdown stage of a documentation build cost? How does freeing
   each result immediately differ from keeping all HTML until the build ends?

The corpus and options are frozen before timing. Preview messages are openly
authored fixtures, not production traffic. The other inputs are verbatim project
documentation snapshots, with source revision, path, byte count, and hash. They
are a useful reproducible example, not a representative sample of all websites.
No document is padded, repeated to hit a size bucket, or chosen by its timing.

## Lanes and completed work

- **Previews:** Ferromark's default untrusted HTML API, fresh calls versus a
  retained `Renderer`, plus pulldown-cmark and Comrak application adapters.
  Every integration escapes user HTML, checks link/image URLs against the same
  scheme allowlist, renders callouts, generates heading IDs, and returns owned
  HTML that is dropped after each document. Adapter work is part of the cost.
- **Guide metadata:** Ferromark `parse()` and the two application adapters return
  HTML, borrowed raw front matter, and owned heading level/text/ID records.
  Metadata and rendered IDs must agree for every archived input. Complete
  results are consumed and dropped. Ferromark's resource-limit report must be
  empty; other engines do not expose the same diagnostics. Body-only rendering
  is not eligible for this comparison.
- **Documentation HTML:** Ferromark, pulldown-cmark, and Comrak render the same
  complete files with trusted CommonMark plus tables, double-tilde strikethrough,
  and task lists. Heading IDs, bare autolinks, footnotes, and other extensions
  are off. Normal allocation and parser APIs are preserved. Each engine is
  measured with both immediate release and retention of all rendered pages.

Protocol 1 measured previews and guide metadata only with Ferromark. Protocol 2
adds the missing comparisons, preserving the exact original corpus and sampling
durations. It measures all 13 variants afresh in one alternating run, including
the Ferromark baselines. The original archive stays immutable and reproducible;
its timings are not mixed into the new run.

The adapters use public events, ASTs, and rendering hooks. They parse Markdown
once, include required policy and metadata work inside time and heap scopes,
and use the engines' own HTML generation. No extra syntax features are silently
disabled to admit a comparison. Corpus admission checks every preview and guide
as well as the documentation collection. Tests reject missing metadata, broken
navigation IDs, omitted content, literal callouts, and unsafe HTML. This is a
comparison of documented integrations, not an assertion that native defaults
or every possible integration have the same semantics or cost.

Inputs with renderer differences follow [ARCH-COMP-002](ARCH-COMP-002-workload-comparability.md).
Raw outputs and reviewed differences are archived before timing; missing
features, missing content, or resource-limit fallback prevent publication of a
comparison. Do not silently trim the corpus to favorable or agreeing files.

## Time, memory, and lifecycle

Report elapsed time per complete workload and input/output sizes. Per-document
time is an explicitly labeled average of that batch, never request p95/p99.
One warmed single-threaded native process per variant excludes process startup,
input loading, I/O, IPC, normalization, templates, syntax highlighting, and
JavaScript bindings. Output allocation, retained-output containers, and dropping
outputs are timed. A reusable renderer persists across measured iterations.

Three fresh process rounds rotate variant order. Each variant warms up for three
seconds and contributes 80 alternating windows of at least 63 ms per round.
Keep every window's duration, completed-workload count, output byte count, and
derived time. Publish the median of round medians and show round variation.
Do not call window percentiles per-request tail latency. Record host, OS, CPU,
compiler, flags, dependencies, source and binary hashes, and power state.

Use separate instrumented binaries for memory; never time allocation counters.
Measure peak simultaneously live requested Rust heap above the infrastructure
baseline. Include parser state, retained renderer scratch, owned HTML, metadata,
and retained-output containers. Exclude already loaded input and bookkeeping.
Warm a newly created session, then measure with that session's retained bytes
still included above the pre-session baseline. Capture cold-session peak too.
Drop the session and require return to baseline. Repeat ten observations in each
of three fresh processes and retain every observation.

Peak requested heap is not RSS: stack, allocator rounding, metadata, cached pages,
and hidden transient storage inside realloc are outside these counters. Also
report requested allocation volume and call count, labeling volume as cumulative
traffic rather than memory capacity. Never infer peak process RAM or multiply
per-page heap peaks to estimate a sequential build. The explicit retained-output
lane measures that application's lifetime choice instead.

## Evidence and publication

The standalone harness lives in `benchmarks/workflows/`. Its result archive
includes the corpus, provenance, locked dependencies, effective options, outputs,
admission checks, raw timings, memory observations, and host observations. A
publisher recomputes summaries from raw evidence and validates hashes, counts,
durations, equality of timed output sizes, and allocation balance. Incomplete or
screening measurements cannot populate the documentation.

Keep these secure-default and system-allocator workflow results distinct from
the historical shared-mimalloc experiment. State the native-only and single-host
scope next to the public figures. Node.js end-to-end measurements, concurrent
services, and total-site builds need their own boundaries and remain unmeasured
by this series.
