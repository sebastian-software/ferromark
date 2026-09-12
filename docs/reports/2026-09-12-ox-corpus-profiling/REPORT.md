# Profiling the merged core on Ox Content's document corpus

Date: September 12, 2026.

[PR #307](https://github.com/sebastian-software/ferromark/pull/307) passed every
reported CI check, including performance regression and native platform builds,
and was squash-merged as `a1c308cba63f7bb038699df8560daa8959159b4e`.
That exact source is the baseline here. This follow-up changes no production
parser, README, or homepage benchmark figures.

The new profiles identify two different priorities: HTML block scanning on the
TypeScript compiler-option reference, and inline mark/event work on Rust Book
prose and Vue's Suspense guide. Code-heavy documents expose a third opportunity:
processing and emitting larger contiguous code ranges. The evidence does not
support another broad arena or heading-allocation rewrite as the first step.

All figures below are generated in [RESULTS.md](RESULTS.md); raw windows, CPU
samples, counters, source hashes, lockfiles, and diagnostic patches accompany
this report. [REPRODUCE.md](REPRODUCE.md) reconstructs the pinned inputs.

## What Ox actually runs

At Ox Content revision `026d1859d1c35e5fb1ea65e7e855b428a918b9bb` (3.2.0),
[`fetch-bench-corpus.mjs`](https://github.com/ubugeeei-prod/ox-content/blob/026d1859d1c35e5fb1ea65e7e855b428a918b9bb/tools/scripts/fetch-bench-corpus.mjs)
selects Vue docs, Vite docs, Rust Book, and the English TypeScript documentation.
Its refs are moving `main`/`v2` branches. This investigation freezes the resolved
commits and every Markdown file's SHA-256 in `corpus-manifest.json`.

The native [`corpus.rs`](https://github.com/ubugeeei-prod/ox-content/blob/026d1859d1c35e5fb1ea65e7e855b428a918b9bb/crates/ox_content_parser/benches/corpus.rs)
recursively collects `.md` files, appends a newline after each, and benchmarks
one concatenation per project. It uses a fresh growing `Allocator::new()`,
`ParserOptions::gfm()`, and fresh `HtmlRenderer::new()` for parse-plus-render.
It also has a separate parse-only benchmark, which is not an equivalent
competitor to Ferromark's complete HTML operation. MDX compilation, framework
rendering, syntax highlighting, and Node wrappers are not in this measurement.

The upstream directory traversal has no defined file order. We sort relative
paths before concatenation for reproducibility. We also measure the original
files individually: concatenation changes document boundaries, heading-ID
collisions, and reference-definition scope. Upstream sparse patterns select
some Markdown license/readme files as well as documentation; these remain in
the manifest and screening coverage, with actual guides selected for profiling.

## Comparison and correctness boundaries

The main matched profile enables CommonMark, tables, strikethrough, task lists,
trusted raw HTML, and heading IDs. Bare URL autolinking, footnotes, tagfilter,
MDX, front-matter extraction, and other optional syntax are off. Ox custom
renderer options also disable automatic link target/rel attributes. Both sides
produce fresh owned HTML, which is dropped inside the timed operation.

A separate `upstream` profile uses Ox's exact GFM parser preset and default
renderer. Ferromark enables autolink literals and footnotes in addition to the
matched profile. This is a diagnostic reproduction of the upstream workload,
not a claim that the two presets have identical output semantics. Ox's renderer
also performs behaviors such as callout rendering without a corresponding
parser option to disable them.

The normalizer ignores heading IDs, link target/rel attributes, incidental
HTML whitespace, attribute order, void-tag spelling, and selected entity/URL
spellings. It is a limited structural comparison, not a browser DOM proof,
full feature-conformance result, or guarantee of equal heading-anchor behavior.
It preserves meaningful code whitespace, tag structure, text, and other
attributes. Two measurement-helper fixes were needed: searching an entity
window as bytes rather than slicing through UTF-8, and recognizing `input`
and the other HTML void elements. The UTF-8 test was observed failing before
its fix. Neither change touches either engine.

The generated coverage table records which original files pass. Every combined
project has at least one remaining mismatch, so combined-project times are
shown only as diagnostics. Examples include VitePress fence-language metadata,
callouts, front-matter/line-ending handling, code spans, and link parsing.
Some are dialect choices; others, such as duplicated link-destination text in
Ferromark, warrant a separate correctness investigation. They must not be
presented as merely cosmetic differences.

`baseline-mismatch-check.json` checks every failing matched case against the
pre-optimization `5b680e5` binary: the complete Ferromark HTML is unchanged.
These are not regressions introduced by #307. `verification.json` preserves
output lengths, normalized hashes, and first-difference excerpts for all cases;
`mismatches.jsonl.gz` retains complete outputs for the mismatching original
files. Large concatenations can be reconstructed rather than archiving repeated
copies of the entire documentation corpus.

## Findings and relevant Ox patterns

### First: line scans in raw HTML blocks

TypeScript's Compiler Options reference is mostly raw HTML. The two captures
consistently concentrate in `BlockParser::parse_html_block_line` and
`peek_html_block_start`; there is very little inline work. Work counters show
many block/text events and only a few inline parses. The hot source locations
lead to Ferromark's scalar `find_line_end` and `current_line_slice` scans.

Ox's [`HTML block parser`](https://github.com/ubugeeei-prod/ox-content/blob/026d1859d1c35e5fb1ea65e7e855b428a918b9bb/crates/ox_content_parser/src/parser/html.rs)
borrows a whole HTML block. It searches type 2–5 terminators across the remaining
source, scans type-1 closing tags with a targeted search, and advances other
blocks until a blank line. Its shared
[`line scanner`](https://github.com/ubugeeei-prod/ox-content/blob/026d1859d1c35e5fb1ea65e7e855b428a918b9bb/crates/ox_content_parser/src/parser/line_scan.rs)
uses NEON on aarch64 and a SWAR path for short/portable cases. Its own HTML
block parsing still dominates that workload, but its total operation is shorter.
Equal relative sample shares across engines would not mean equal absolute cost.

Two bounded Ferromark experiments replace the scalar advancing scan and the
scalar peeking scan independently with the already available `memchr`. Their
combination is measured separately. They preserve exact HTML on the entire
case set, including heading IDs and the known engine mismatches. The generated
prototype table shows repeatable improvements on both compiler-option documents,
with much smaller or inconsistent effects elsewhere. Faster line search is a
real, narrow opportunity; it does not close the whole Ox gap.

The first implementation follow-up should take the narrow scan change through
normal parser regression tests and cross-platform benchmarks. A larger second
step could coalesce contiguous HTML events or consume a whole root-level HTML
block, preserving container prefixes, end conditions, CRLF behavior, and public
event semantics. The bulk-block idea has source/profile support but has not
been implemented or assigned an estimated speedup here.

### Second: inline marks, soft breaks, and event ordering

Rust Book's recoverable-errors chapter repeatedly concentrates in
`InlineParser::parse_with_options_in_document`. Source locations identify mark
collection and event emission; the visible `EmitPoint` sorting stacks are a
substantial part of the whole operation in both captures. The chapter's
work counters show many marks and emit points relative to its paragraph count.
Vue's Suspense guide is also inline-heavy, but its sort share is much smaller:
optimizing sorting alone would not resolve both cases.

Ox's [`inline parser`](https://github.com/ubugeeei-prod/ox-content/blob/026d1859d1c35e5fb1ea65e7e855b428a918b9bb/crates/ox_content_parser/src/parser/inline.rs)
walks the input into nodes with local construct handling and a delimiter stack.
Its [`marker scanner`](https://github.com/ubugeeei-prod/ox-content/blob/026d1859d1c35e5fb1ea65e7e855b428a918b9bb/crates/ox_content_parser/src/parser/inline/marker_scan.rs)
caches the next hit for independent optional syntax scans. That is a distinct
pattern from a bitmap cache for every SIMD vector, which our earlier experiments
rejected. It is not evidence that copying either cache into Ferromark helps:
Ferromark's mark-collection loop already advances monotonically.

A useful next experiment is to measure event construction and ordering
separately on newline-heavy prose with code spans. Keeping already ordered
families ordered, or merging them while preserving the current same-position
precedence, could avoid some global sorting. This needs exact boundary tests
for nested emphasis, links, code spans, soft breaks, and suppression ranges.
Skipping the ordering/overlap rules, disabling features, or replacing the
three-phase parser wholesale is not justified by these profiles.

### Third: contiguous fenced-code ranges and rendering

Vue's render-function guide and the TypeScript release notes have more code
block work and more time in the remaining rendering/pipeline frames than the
Rust prose case. The existing paragraph-borrowing optimization is already doing
its job: representative Vue/TypeScript documents copy no paragraph scratch
bytes. Continuing to optimize those copies would miss the present workload.

Ox's [`fenced-code parser`](https://github.com/ubugeeei-prod/ox-content/blob/026d1859d1c35e5fb1ea65e7e855b428a918b9bb/crates/ox_content_parser/src/parser/fenced_code.rs)
finds the closing fence and borrows the entire unindented body when line-ending
normalization is unnecessary. It materializes the indented case. Ferromark
currently emits and processes code ranges line by line. A guarded contiguous
code-body path is therefore a better next candidate than another code-info
allocation tweak. Container indentation, unclosed fences, CRLF, MDX boundaries,
custom fence renderers, and event consumers remain part of the required behavior.
This bulk-code approach is a candidate, not a measured implementation result.

HTML escaping remains visible in both engines. The previous SWAR escaping
experiment regressed other workloads, so these profiles alone do not justify
reviving it. Any new escaping experiment should target the actual fragment-size
and escape-density distribution of the code-heavy files.

### Lower priority: arena and heading bookkeeping

Growing versus source-reserved Ox arenas give very similar times for the
confirmed original documents. Ferromark's visible allocation/heading stacks do
not explain the two leading outliers. Small-document setup costs still exist,
but they are not the primary explanation for the large-reference gap.

This profiling round measures CPU work, not peak memory. The
[separate growing-arena heap audit](../2026-09-12-ox-memory/REPORT.md) remains the
basis for memory claims, including its large plain-text counterexamples.
There is no universal speed or memory winner implied by the absence of an AST.

## Measurement protocol and limits

- Native Rust cores only, identical generic aarch64 build target, system
  allocator, Rust 1.97.1, Apple M1 Pro, macOS 26.6.2. Ferromark and Ox use
  separate locked dependency workspaces. Ferromark retains its main dependency
  versions; Ox retains its standalone resolution from the preceding audit.
- Uninstrumented timing builds use opt-level 3, fat LTO, one codegen unit, and
  panic abort. Builds run outside repository Cargo configuration with ambient
  Rust flags removed. Repository-specific Apple M1 flags are not inherited.
- Input loading, original option construction, JSON I/O, and normalization
  happen outside timing. Each operation constructs parser/renderer state and
  owns/drops its output. Required Ox custom-option cloning is part of its
  matched fresh-renderer operation; the upstream profile uses allocation-free
  static defaults via `HtmlRenderer::new()`.
- Screen: all cases, three 8 ms windows per engine, 3 ms warmup. Confirmation:
  selected cases, two fresh sets of engine processes, nine 60 ms windows per
  engine, 35 ms warmup. Engine order rotates. Every window performs batches of
  eight operations, so large-document windows can exceed the requested length.
- CPU profiles: separate optimized binaries with debug information and frame
  pointers, six-second `sample` captures at a requested 1 ms interval; both
  engines on each selected original/combined document and the large parser
  fixture. The two leading outliers are captured twice. Profile binary output
  was verified against the uninstrumented binaries before sampling.
- The profile loop performs the same fresh operation. It also includes timer
  and loop overhead. Sampling is statistical, captures fewer stacks than the
  nominal interval suggests, and does not recover every optimized inline frame.
  Symbol groups are not exact non-overlapping parser-phase timings. Sorting,
  heading, and allocation percentages overlap other groups. They cannot be
  summed into a predicted speedup or substituted for timed engine comparisons.
- Work counters use another build with the existing `profiling` feature. They
  count events and copied bytes, not time or peak heap. No counter, profiling,
  compilation, or test workload overlaps any recorded timing window.
- The two line-scan probes and their combination are based independently on
  the same frozen source. Both timing passes use seven 40 ms windows, 25 ms
  warmup, rotating order, and the same four processes across passes. Exact HTML
  is checked before timing. Full production/spec/platform validation has not
  been performed on these diagnostic patches, and production source is unchanged.
- The selected follow-up documents deliberately include outliers. They are not
  a random sample or representative headline ranking. Short-screen sums are
  not site-build measurements. Host scheduling and frequency effects remain;
  repeat ranges are not confidence intervals and small changes are inconclusive.

## Archive validation

The pinned preparation helper was run in a fresh temporary workspace. Both
standalone harnesses passed their seven normalizer tests and Clippy with
warnings denied. Rebuilt release binaries reproduced every verification record
for all cases, including normalized hashes and output lengths. Generated result
tables reproduced byte-for-byte, and the production source hashes still match
the merge commit. Logs and the machine-readable result are in `validation/`.
The merge record in `merge-ci.json` archives all successful PR checks.

## Disposition

The previous optimization/memory PR is merged with green CI. This report and
its isolated patches preserve the next-round evidence. The practical order is
narrow line scanning, then inline event ordering, then contiguous HTML/code
ranges. Existing output divergences should receive a separate correctness
investigation before these corpora are used in public feature or speed claims.
