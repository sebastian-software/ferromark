# Native profiles after PR #312

Date: September 12, 2026.

The next small implementation candidates are a single shared text-escape scan
and early rejection of impossible autolinks. Larger remaining opportunities are
in inline mark collection and event construction. The measured HTML specialization
and short-copy prototype have counterexamples; the soft-break experiment either
changes existing output or becomes slower when conservatively guarded.

The [stricter escape follow-up](../2026-09-12-neon-text-escape/REPORT.md) subsequently
found repeated counterexamples in five integration variants and adopted none.
Its additional large quoted-code controls preserve a useful rescan reproducer.

This is an investigation of merged Ferromark
`e731dc8162828e0ac2df30b7567107f560159ff8` against the same pinned native Ox Content
`026d1859d1c35e5fb1ea65e7e855b428a918b9bb` used in the preceding audit. Production
source is the merged baseline. All candidate patches are isolated diagnostic
sources, with their results and failures retained under `experiments/`.

[RESULTS.md](RESULTS.md) contains generated comparison tables, profile attribution,
every initial screen, longer confirmations, and repeated control regressions.
[REPRODUCE.md](REPRODUCE.md) explains the harness reconstruction and checks.

## What the new comparison establishes

Fresh measurements reproduce the small Compiler Options gap and the larger gaps
on MSBuild, Vue render function, Vue Suspense, and the Rust Book recoverable-errors
chapter. Being close on the HTML-heavy reference does not establish similar
performance on prose-heavy documents.

Both engines use native Rust calls, a fresh parser/renderer operation and fresh
owned HTML, including output destruction inside timing. Ox's growing arena is
the primary comparison; its presized arena remains a separate diagnostic column.
The matched feature profile enables tables, strikethrough, task lists, raw HTML
and heading IDs. Loading, option construction, JSON and normalization are outside
timing. No Node wrapper, MDX compilation, framework render or highlighting is
part of the competing operation.

All five selected inputs pass the existing limited normalized-output comparison.
Across the full corpus, all 649 comparison records remain identical to the
preceding report; 594 normalize equally. Normalization ignores heading-ID values,
selected attribute/entity spellings and incidental HTML whitespace, so this is
not a byte-identical cross-engine conformance claim. The remaining mismatches
are retained and are not used to support these five timing comparisons.

## Profile findings

Twenty regular captures cover both engines on five documents, twice each.
Profile-build HTML matches release-build HTML in all 1,298 checks. Four additional
captures outline three Ferromark functions to distinguish costs hidden by LTO:
mark collection, autolink search and event construction. That attribution-only
source also preserves all 649 corpus outputs.

Compiler Options still concentrates in HTML/block work on both engines. Ferromark
spends visible time finding line ends and maintaining root HTML continuations;
Ox likewise spends much of its operation in HTML block parsing. Similar sample
shares do not imply equal absolute costs. In particular, the new main-source
profiles do not show final string conversion as the dominant explanation for
the gap.

Vue Suspense and the Rust chapter concentrate much more heavily in Ferromark's
inline pipeline. The attribution probe separates substantial mark collection
and event construction costs. Sorting is part of that work, but is much smaller
on Suspense than on Rust: another sort-only change cannot address both cases.
Autolink search is also visible on Suspense, including attempts to interpret
ordinary angle-bracket tags as links before raw-HTML recognition runs.

Ox's inline parser builds nodes while walking constructs and folds ordinary
soft line breaks into text runs. Ferromark resolves marks and then orders and
renders events. That source difference motivates experiments; it does not show
that removing Ferromark events preserves its existing behavior. Allocation-stack
shares do not support making an arena rewrite the next priority. This round
does not measure heap peaks or RSS; the preceding memory audit remains the
evidence for memory comparisons.

## Experiments and decisions

### First candidate: one text-escape scan

`escape-single-scan` uses the existing four-byte `ByteSet` for the full text
escape search. The baseline uses that scanner for short suffixes, then combines
separate `memchr3` and quote searches for longer suffixes. The candidate keeps
the same escape byte set and replacement policy, while avoiding the second
search and its setup on those suffixes.

The repeated improvements are modest but spread across MSBuild, Vue and Rust.
No input in the longer confirmation set is more than 2% slower in both pairs.
Compiler Options is effectively unchanged. This is the best first follow-up.
Measurements are on Apple M1 Pro: the shared scanner has different NEON and SSE2
implementations, so these results do not justify replacing the long-input x86
path without measuring it. A production patch should establish the supported
architecture boundary and repeat the normal regression and performance gates.

### Second candidate: reject impossible autolinks earlier

`autolink-prefilter` follows the existing bounded scan to the closing bracket,
then rejects content containing neither `:` nor `@` before URI/email validation.
A URI requires the former, and an email requires the latter. Complete validation
still runs for possible links; no syntax option is disabled.

This directly targets the extra failed recognition visible in Suspense's
profile. The longer pairs confirm a small improvement there, with mixed or small
changes elsewhere. A headings control is consistently around 2% slower in its
additional pairs, so the tradeoff needs attention even though it does not cross
the over-2% threshold in both runs. This is a secondary candidate, with actual
autolink-heavy cases and headings retained as guards.
The two leading candidates were measured independently: their gains are not
additive, and their combination has not been measured.

### Defer the HTML specialization and short copies

`html-blank-run` specializes private root type-6/7 HTML continuation handling,
deferring parser-state updates until the stopping line. It preserves the tested
public block/MDX streams and improves Compiler Options, but the deeper-emphasis
control repeatedly becomes slower. That control does not use the new HTML path;
code layout is a plausible explanation, not an established mechanism. The
current evidence does not justify merging this exact patch.

`escape-short-copy` uses explicit bounded overlapping copies for text runs of
at most 16 bytes, inspired by Ox's short-run renderer helper. The initial Rust
gain is not equally strong in the longer pairs, and several small fence controls
repeatedly regress. The added low-level copy implementation is not justified by
these results.

### Reject the soft-break prototypes

`softbreak-ranges` keeps literal soft newlines inside private-render text ranges,
while retaining public parser events and falling back for images. Six extended
guards expose changed HTML around unusual overlapping code-span events. These
are compatibility failures, even where the baseline behavior itself deserves a
separate correctness investigation. The source is not timed after that failure.

`softbreak-no-code` additionally excludes paragraphs containing code spans.
It passes the other suites but fails 267 fence/MDX guards: joining text that
contains a NUL byte with a later entity changes whether the existing entity
decoder normalizes that byte. One retained example changes a literal NUL into
U+FFFD. Text segmentation therefore affects more than the number of renderer
calls in the current implementation.

`softbreak-guarded` also excludes NUL and CR input. It passes every exact-output
guard and improves the simple multiline control, but repeatedly regresses
several real documents and long-delimiter controls. The extra eligibility work
and fallback restrictions erase the intended benefit. This formulation is
rejected, rather than retaining only its favorable control.

## Scope and measurement limits

- Native release builds use Rust 1.97.1, opt-level 3, fat LTO, one codegen unit,
  panic abort and the system allocator on Apple M1 Pro. Standalone workspaces
  avoid repository Cargo settings; ambient Rust flags are removed. Each engine
  retains its pinned dependency resolution.
- Current engine comparison: three fresh process groups, nine rotating 60 ms
  windows per engine/input, 35 ms warmup. Each window batches eight renders.
- Initial prototype screen: 93 inputs, five alternating 30 ms windows per
  baseline/candidate, 60 ms warmup. Confirmation: two additional fresh process
  pairs, seven alternating 50 ms windows on 37 inputs. The selection includes
  the five target documents, ordinary controls, and every over-2% loss in the
  first four correct screens. It was also used for the later autolink probe; its additional headings
  screen regression received two separate follow-up pairs.
- Each timed prototype passes 111,902 exact before/after output comparisons,
  including public events, MDX, fence boundaries and adversarial cases.
  This is differential evidence, not a substitute for full production checks.
  No parser change or implementation PR is created by this investigation.
- Profiling uses separate optimized debug builds with frame pointers and
  six-second `sample` captures at a requested 1 ms interval. Symbol groups are
  visible-stack attribution, not exact phase timers. The outlined diagnostic
  build can change code generation and sample shares. Conversion, allocation
  and sort percentages overlap other categories.
- Builds, output guards and sample collection do not overlap recorded timing
  windows. Background OS activity remains uncontrolled. Short screens are
  exploratory; repeated ranges are not confidence intervals. The deliberately
  selected outliers are not a site-build benchmark or a universal engine ranking.

The source hashes, binary hashes, raw windows, profile stacks and failed guard
outputs are archived. `check-evidence.py` checks file/source identities and
recomputes timing summaries from raw observations. README and homepage tables
remain their separately dated full-comparison snapshots.
