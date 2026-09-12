# Inline emission after the Ox corpus optimizations

This investigation starts from merged main `96e78323a9d263530e2564cdd7ef8fa3facc0ac0`
(PR #308). It revisits the inline work identified in the earlier
[corpus profiles](../2026-09-12-ox-corpus-profiling/REPORT.md). The resulting change
combines code opener/content points in eligible paragraphs, reduces private sort
payloads to indices into existing resolution records, and accelerates the search
for inline HTML openers.

[RESULTS.md](RESULTS.md) contains generated measurements. The decision and
compatibility constraints are recorded in
[ARCH-EXP-022](../../arch/ARCH-EXP-022-inline-emission-and-html-search.md).

## Why these changes

Fresh sampling profiles retain separate captures for Vue Suspense, the Rust Book
recoverable-errors chapter, Vue render functions, the TypeScript compiler-options
reference, and a rich-inline control. The first two still spend a substantial
share of samples in inline processing. Sorting is much more prominent in the
Rust chapter than in Suspense. Source locations in Suspense instead also identify
`find_html_spans_into`, which advances byte by byte between candidate openers.

The selected changes address these distinct costs. A resolved code span usually
needs one content point and one closing point rather than three points. Its
closing boundary still matters, and paragraphs containing other resolved
construct families retain the original sequence. Rich sort payloads are already
stored in the parser's resolution records, so the sort buffer can refer to them
by index. The HTML recognizer and its precedence rules remain unchanged; only
finding the next `<` is accelerated.

## Correctness and rejected experiments

The frozen guards include the previous specification/extension/renderer-reuse
cases and every Ox corpus input, including existing engine mismatches. Additional
deterministic inputs compare exact HTML, public inline events, and MDX inline
events against the frozen baseline. These comparisons preserve behavior rather
than treating every baseline result as normative Markdown.

The first one-point code-span experiment changes one original Vite document and
its two corpus concatenations. Additional syntax combinations also expose changes
in the unconditional two-point version. The retained eligibility guard avoids
those changes. Existing unusual link/code and math interactions remain separate
correctness work; the performance patch does not silently change them.
The minimized cases are ``$`a`$`` (math enabled) and ``[](`a`)``: their existing
resolution emits both a surrounding construct and code content. See
[the minimized baseline/prototype outputs](code-two/minimized.json). The new
regression test explicitly identifies these as compatibility snapshots rather
than normative syntax expectations.

Three approaches to combining the already ordered mark family were screened:
iterator merging, slice merging, and bounded insertion. Their complete control
results are retained, including regressions. The public event stream, source
ranges, ordering of simultaneous points, suppression behavior, extension options,
and reused renderer state remain part of the compatibility boundary.

## Measurement boundaries

All local timing uses native Rust operations with owned output and the system
allocator. Rust 1.97.1 defaults to `apple-m1` on this aarch64-apple-darwin host;
omitting `target-cpu` does not mean generic. Release builds use optimization level
3, fat LTO, one codegen unit, and panic abort. Profiling builds additionally retain
debug information and frame pointers; their samples are not latency measurements.
The build environment removes inherited Rust flags and runs outside the
repository's Cargo configuration.

Screens are exploratory. Longer confirmations use fresh processes, warm each
case, and rotate engine order. Raw windows and exact build identities are retained.
Small fluctuations and sample percentages are not confidence intervals. No build,
test, profiler, or second timing suite runs concurrently with a timing suite.

Heap observations use the existing counting allocator and fresh render state,
with output still live at the end of each observation. Requested live bytes are
not process RSS. Large inputs remain in the suite even when their savings are
small or zero. The primary competitor lifecycle uses a growing Ox arena;
source-reserved results remain separately identified.

The standalone optimization harness and the Ox corpus comparison use different
drivers. Their absolute times must not be subtracted from one another. The README
and homepage comparison tables remain independently dated publication snapshots;
this report records the new optimization experiment and its corpus comparison.

The mismatch archive retains full outputs for the original corpus files.
Concatenated mismatches are reproducible from the frozen inputs and driver;
their duplicated full outputs are omitted from the archive. Every corpus case
remains in the verification manifest.

## Source and validation

The production implementation is commit `6f3602c10a090c40fad9103928166105c5b38d64`. Subsequent report changes do
not alter the measured parser source. `metadata.json` records every source hash,
and `implementation.patch` records its difference from the frozen baseline.

The complete Rust test suite with all features, all-target/all-feature Clippy
with warnings denied, and the format check pass. Dedicated public-event tests
cover code boundaries, nested link/image payloads, HTML candidates, math/note
ranges, and the explicitly documented overlap compatibility cases.

The archive integrity contract passes. A separate scratch checkout rebuilds the
archived baseline and production implementation and successfully replays every
frozen output guard. Validation logs are retained under `validation/`.

The post-change profiles still identify inline mark collection/resolution in the
prose cases and block-level HTML processing in the compiler-options reference.
The latter remains a separate candidate for consuming contiguous root-level HTML
blocks earlier in the parser. Neither the remaining Ox gap nor a sample share
establishes an attainable speedup for that future change.
