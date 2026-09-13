# Vue Suspense: inline code and range membership

This round starts from Ferromark `39c4a0164626404e226d40441f155142342b17fa`,
including the ARM escaping change in PR #313. It targets the exact Vue Suspense
guide discussed in the previous profiles. [RESULTS.md](RESULTS.md) is generated
from the archived timing windows. [REPRODUCE.md](REPRODUCE.md) describes the
frozen native harness and validation.

## Scope and tradeoff

This is a complexity fix for paragraphs with many inline code tags. Limited
small regressions are acceptable when the measured distribution supports the
tradeoff; the complete corpus is checked instead of assuming that the favorable
examples are more common. [CORPUS.md](CORPUS.md) contains every individual result,
neutral-band counts and longer follow-ups. [RESULTS.md](RESULTS.md) retains the
focused controls, scaling curves and native Ox comparison.

<!-- measured-summary:start -->
Of 638 individual documents, 415 have a lower median
runtime and 156 are lower in all three broad pairs.
Using a descriptive ±1% band, 96 are faster,
497 are within the band and 45 are
slower. The sum of per-document medians changes by
-0.02%; this is not a separately timed
site build. The geometric mean change is
-0.23% with equal document weighting.

Small changes do not establish a universal speedup. The original Suspense
input is practically unchanged; long code/tag paragraphs have the substantial
improvement. The focused TypeScript 6.0 loss and heading outliers remain in
[RESULTS.md](RESULTS.md), alongside the broader follow-ups in
[CORPUS.md](CORPUS.md). No source or timing observation is dropped to improve the
reported distribution.
<!-- measured-summary:end -->

## Review follow-up: independent complexity guards

Review exposed a hole in the original combined probe count: restoring one
quadratic search without instrumentation left the whole-render test green.
The three walks now have independent counters, each with a nonzero coverage
assertion and a linear-growth bound. In a temporary crate, restoring each old
search with and without instrumentation produces six expected test failures;
the corrected source passes. See
[the negative controls](validation/review-negative-controls.json).

This follow-up changes only test instrumentation and assertions. The timed
`final` source remains the snapshot at `3927be5`; none of its timings or output
checks were relabeled as measurements of the review update. The additional
patch and source hashes in `metadata.json` identify the reviewed tests, and
`check-evidence.py --check-current` validates that source separately.

## Document and competing operation

The input is `vue-docs/src/guide/built-ins/suspense.md` from Vue docs commit
`b75d188ab16bf83bd1f364a77dfd2315be8f3fa4`: 8,291 bytes, 170 lines, 66 inline
code spans, seven fenced blocks, nine headings and eight Markdown links. All
36 less-than signs outside fences occur in inline code. The file is unchanged;
frontmatter and VitePress-specific notation receive each engine's ordinary
Markdown handling. This is not a Vue runtime or VitePress site-build benchmark.

The comparison pins native Ox Content 3.2.0 at
`026d1859d1c35e5fb1ea65e7e855b428a918b9bb`. Both engines render fresh owned HTML
with destruction inside timing. Ox uses a fresh growing arena. Input loading,
JSON transport, option setup and normalization are outside the timed operation.
CommonMark plus tables, strikethrough, task lists, trusted raw HTML and heading
IDs are enabled. MDX compilation, highlighting, bare URL autolinking and
frontmatter extraction are outside the matched profile.

The 16 selected native comparisons have equal normalized output. All 649 corpus
cases are checked; the same 594 normalize equally. The normalizer ignores
heading-ID values and selected spelling/whitespace differences. This is not
byte-identical cross-engine conformance. Corpus attribution and licenses remain
in the [original archive](../2026-09-12-ox-corpus-profiling/licenses/ATTRIBUTION.md).

## Findings and proposed change

Three general-path membership searches restarted at the first range for every
code opener, HTML candidate or autolink. Their inputs are already ordered.
Forward cursors avoid repeated work without changing membership or boundaries.
The whole-render work test failed before the fix: doubling its mixed paragraph
changed 20,480 range probes to 81,920. The proposed change bounds these three
searches linearly. Test-only counters compile out of release builds.
The autolink range filter remains a separate, outlined helper; the other
searches keep their existing locations. Resolver order, HTML/code precedence
and the public event stream remain unchanged.

This is a complexity fix for larger paragraphs using the document's syntax;
it is not evidence that the complete parser is linear or that the original
8 KB Suspense document becomes substantially faster. The separate native table
shows the remaining gap to Ox.

## Experimental decisions

The general pipeline recognizes possible autolinks and HTML before resolving
code spans, then filters candidates inside code and constructs ordered output
events. A prototype validated text/code-only paragraphs and emitted them early,
skipping the general pipeline. It required paired backticks with less-than
markers inside the pairs and rejected other mark families or HTML extending
past the closing backtick. Both scratch-backed and two-pass versions preserved
the checked outputs.

Those shortcuts are **not retained**. The intermediate formatted `production`
variant improved Suspense but repeatedly slowed the HTML prose control. Moving
the shortcut later, removing scratch writes and skipping empty filters did not
eliminate the tradeoffs. The precise machine-code cause of the HTML cost was
not established. Their measured sources and full observations remain archived.

The fused autolink prefilter regressed actual URL/email controls. Reordering
autolink recognition, an event-emission shortcut and broader code-prose
variants also exposed control losses. Their sources, outputs and favorable
and unfavorable timing observations remain archived.

The first autolink-code screen has 93 inputs; later screens have 103, including
ten added code/tag/autolink controls. Longer exploratory pairs cover 75 inputs
selected to include every initial over-2% loss, ordinary controls and target
corpus files. `linear-ranges` isolates the three ordered searches. The first
formatted version, `linear-formatted`, exposed a repeatable heading-control
cost in both full and focused comparisons. `outline-probe` keeps the autolink
filter in a separate helper and receives two wider checks. `final` rebuilds that
outlined version with formatting, comments and regression tests, then measures
it independently. Remaining losses and heading-control outliers stay visible
in the final tables.
The archived name `production`
refers to the rejected intermediate candidate, not the committed source.

## Measurement boundaries

Release builds use Rust 1.97.1, optimization level 3, fat LTO, one codegen unit,
panic abort and the system allocator on Apple M1 Pro. Standalone lockfiles pin
each harness. Ambient Rust flags are removed.

Initial screens use five alternating 30 ms windows after 60 ms warmup. Longer
paired runs use seven alternating 50 ms windows after 60 ms warmup. The final
source has three fresh pairs over all 103 selected inputs. Native Ox comparisons
use three fresh process groups, nine alternating 60 ms windows and 35 ms warmup.
Scaling controls use seven alternating 50 ms windows and 35 ms warmup. Workers
check elapsed time after batches of 16 renders in the paired harness and eight
in the native comparison, so slow cases can exceed the requested window length.

Three identical-binary process pairs diagnose variability on selected controls.
Focused follow-ups use seven 100 ms windows on any final over-2% losses plus
heading, Suspense and HTML-prose controls.

Final timing windows do not overlap builds, correctness suites or profiling.
Background OS activity is uncontrolled. Repeat ranges are not confidence
intervals, and these selected cases do not establish an overall corpus ranking.
No new heap/RSS or x86 performance claim is made. README/homepage engine tables
remain their separately dated complete-comparison snapshots.
