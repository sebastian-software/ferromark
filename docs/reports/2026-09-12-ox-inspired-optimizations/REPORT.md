# Ox-inspired optimization experiments

Date: September 12, 2026. This is a measured implementation experiment against
frozen `main`, following the [Ox Content audit](../2026-09-12-ox-content-audit/REPORT.md).
It does not replace the published README or homepage benchmark dataset.

The subsequent [native heap audit](../2026-09-12-ox-memory/REPORT.md) measures
peak live requested bytes with a growing Ox arena, including large mixed and
plain documents. It establishes the narrower claims that the allocation totals
in this report alone could not support.

## Outcome

Retain five complementary changes: exact NEON nibble classification with direct
lane location, borrowing contiguous paragraph/heading source text, borrowing
unescaped fenced-code language text, sizing the heading registry from a count
collected during block parsing, and directly emitting a single parsed text event.
The generated [results](RESULTS.md) include every screen, all confirmation runs,
allocation counts, and a fresh native Ox comparison.

The retained source improves the upstream mixed Markdown workload and several
specific costs. The largest gains occur on long scans. Plain paragraphs,
headings, and small language-tagged fences also improve. Lists and richly
formatted inline content change much less; MDX gains are modest because its
segmentation and JSX work remain. Empty input shows a small relative increase
at sub-microsecond scale. None of this establishes a universal parser ranking.

## Hypotheses and decisions

| Approach | Reason to try it | Decision |
|---|---|---|
| NEON nibble classification | Replace one comparison per special byte with two table lookups | Retain with direct lane location |
| Nibble classification plus direct lane location | Avoid rescanning a matching vector one byte at a time | Retain; short input stays scalar |
| Borrow the first paragraph/heading fragment | Avoid an unconditional text copy and scratch allocation | Extend to contiguous fragments |
| Borrow adjacent fragments and actual LF breaks | Keep normalized text borrowed when it is already contiguous | Retain; copy when indentation, CRLF or container prefixes separate it |
| Borrow simple fence info | Most language names contain no backslash escapes | Retain; escape/entity handling still runs |
| Flat heading-ID collision chains | Reduce per-hash value size and traversal overhead | Reject: insufficient benefit for added complexity |
| Count headings in a separate event pass | Avoid registry growth during slug generation | Replace: extra scan hurts list/table workloads |
| Count headings during block parsing | Reserve registry capacity without a second pass | Retain; no public event/API change |
| Cache generated slugs | Avoid repeated slug normalization | Reject: severe unique-heading regression |
| SWAR escaping | Scan escape candidates in eight-byte words | Reject: regressions on prose and long code blocks |
| Double initial output capacity | Reduce output growth and copying | Reject: extra capacity without a clear general gain |
| Carry the first special-character position | Avoid scanning the same prefix twice | Reject: mixed-input overhead; nibble scanning is the better tradeoff |
| Directly emit a single parsed text event | Avoid the generic image/link event-emitter setup | Retain; full inline parsing and entity handling are preserved |

Three combinations were screened before the final production cleanup. These are
independent prototypes from the same baseline, not sequentially accumulated
optimizations. Their gains are not additive. The final code was rebuilt and
measured separately; only its longer confirmation runs support the retained
performance claims. The prior audit also measured Ox's arena growth versus
source-sized reservation and fresh versus reusable renderers. A wholesale AST or
arena rewrite of Ferromark was not attempted: that would change the streaming
architecture and feature/API costs rather than isolate the observed hotspots.

## Correctness boundaries

The ByteSet classifier groups only identical high-nibble membership rows. More
than eight distinct nonempty rows use exact byte comparisons, so arbitrary
binary sets, duplicate bytes and NUL remain supported. The packed lane-location
path is limited to little-endian NEON; big-endian ARM keeps scalar location.
SSE2 and scalar-only targets retain their existing search implementation.

Borrowed ranges are scoped to the current paragraph or heading and cleared on
finish/start/reset. Only a literal LF can extend a borrowed soft break; removed
prefixes, CRLF and other normalization materialize the existing scratch buffer.
MDX transitions receive different source slices, so tests cover Markdown/JSX
boundaries as well as heading-ID continuity. Paragraph-copy profiling counts
actual copies, including materialization. Heading IDs still compare slug bytes
inside hash collisions and preserve naturally written suffixes.

Every screen compares fresh HTML, reusable-renderer transitions, heading metadata
and resource-limit diagnostics for 2,028 cases against frozen baseline hashes.
The final driver adds MDX body/ESM/front-matter checks, for 2,046 cases. These are
equivalence checks, not a new claim that all extensions conform to CommonMark.
The repository's full all-feature test suite separately checks its expected
CommonMark, GFM, MDX and application behavior. Added regression tests cover byte
membership across SIMD boundaries, normalized text fragments, code-language
escaping and changing MDX source segments.

## Measurement protocol

Baseline: `5b680e5447e525a7a58781e91849c66242c286c3` (Ferromark 0.9.0).
The core source and dependency lock match the preceding audit's `66892cf`; the
intervening main commit changes the README company badge. Source hashes and the
retained patch are archived. Rust 1.97.1, Apple M1 Pro, macOS 26.6.2, AC power;
release opt-level 3, fat LTO, one codegen unit, panic abort, system allocator.

All builds in this experiment ran outside the repository Cargo configuration,
without a target-cpu override. Rust 1.97.1 defaults aarch64-apple-darwin to
`apple-m1`; the earlier generic-target descriptor was incorrect. Both baseline
and candidate, and the separate Ox comparison, used the same default target.
The preceding audit explicitly inherited Apple M1/NEON flags. Absolute times
from different harnesses should not be subtracted to calculate an improvement.
The September 12 corpus-optimization report records the target-specification check.

Screening uses 42 timing cases, five 35 ms windows per engine, and 100 ms warmup.
Confirmation uses 45 cases including real MDX rendering, three separate process
pairs, nine 75 ms windows per engine and 250 ms warmup. The order alternates
between baseline and candidate; the starting order rotates between process
pairs. Each window executes batches of 16 renders. Fresh convenience APIs
produce an owned result and drop it inside timing. Explicit reuse controls use
retained renderer scratch for both sides. Input loading, options construction,
JSON, output comparison and allocation instrumentation are outside timing.

The case set includes Ox's actual 497-byte fixture repeated 100 and 2,150 times,
short/fresh/reused input, plain and multiline prose, inline syntax, lists,
fences, tables, repeated/unique headings, long scans and the earlier native
hotspot guards. The final output checks also include all 652 CommonMark examples
under three flag configurations and adversarial normalization/limit cases.
No experiment build or test command ran concurrently with its timing windows.
Background OS activity is uncontrolled; repeat ranges are not confidence
intervals, and small differences should not be treated as portable guarantees.

Allocation counters use separate instrumented binaries and report cumulative
requested bytes, including reallocations. They do not measure peak memory.
The reproduction helper touches copied sources before each build to prevent
Cargo from reusing a binary when a restored source file has an older timestamp.
Both recorded timed production builds explicitly compiled Ferromark, and their
binary hashes differ. A stale allocation-only pilot was discarded and rebuilt
before recording the allocation results.

The Ox comparison uses its current native Rust core at
`026d1859d1c35e5fb1ea65e7e855b428a918b9bb` (3.2.0), with its intended source-sized
arena. Tables and heading IDs are aligned, bare autolink conversion and extra
link attributes disabled. Ox's normalizer checks output outside timing; it
ignores incidental whitespace, heading IDs and link target/rel attributes.
Consequently this is a workload comparison, not byte-identical full-feature
conformance. The exact before/after Ferromark check remains stricter. The
normalizer and text codec are archived with their upstream MIT license.
The standalone dependency resolutions are archived separately: Ferromark retains
its main lock's smallvec 1.15.2; Ox resolves smallvec 1.16.1 (its upstream
workspace lock has 1.16.0). This is a pinned library-consumer build, not a claim
to have run the upstream workspace binary. No Node.js wrapping or JSX compilation
is included in that comparison.

## Validation and reproduction

[REPRODUCE.md](REPRODUCE.md) describes rebuilding every variant and generating the
tables. Raw windows, summaries, source hashes, lockfiles and build/test logs are
included. Required all-feature tests, all-target/all-feature Clippy, formatting,
shared-byte-search tests/Clippy, x86 library compilation, scalar-only byte-search
compilation, and relevant documentation/profiling/benchmark contracts passed.
Performance was measured on ARM only; x86 and scalar checks establish compilation,
not performance or runtime correctness on those platforms.
