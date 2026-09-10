# Measured Markdown feature optimizations — 2026-09-10

See [the baseline feature study](2026-09-10-markdown-feature-costs.md) for the
catalog, machine, interpretation, and measurement method. This log preserves
individual hypotheses, paired measurements, and decisions. Production starts at
`99fbf01`; measurement infrastructure is committed in `bfb3687`. Negative timing
changes mean less elapsed time. These are local Apple M1 Pro results.

## 1. Allocate inline scratch buffers on demand — retained

Hypothesis: preallocating scratch space for every possible inline construct
accounts for much of the fresh-parser cost, even on documents with no markup.
CPU samples of the tiny CommonMark workload are dominated by allocator paths.
Replace initial capacities with empty buffers; existing reserve/resize/growth
paths allocate them when needed, and a retained Renderer keeps that capacity.

Seven paired >=100 ms windows, alternating binary order, with exact HTML checks
for all 292 feature configurations:

| Workload | Fresh owned time change | Retained Renderer change |
| --- | ---: | ---: |
| Tiny CommonMark | -63.7% | +1.0% |
| Tiny default | -60.8% | +0.3% |
| Plain 1 KiB CommonMark | -32.2% | -3.5% |
| Light 1 KiB CommonMark | -5.9% | +0.2% |
| Light 1 KiB default | -5.6% | +0.8% |
| README CommonMark | +1.8% | +1.3% |
| README default | -0.4% | +1.4% |

Tiny CommonMark drops from 40 allocation calls / 11,239 requested bytes to
9 / 3,103. Default drops from 43 / 14,703 to 12 / 6,567. The separate GFM guard
checks 90 HTML configurations: complex CommonMark/GFM cases stay within about
2.2%, and plain input with a fresh parser improves by 7–8%. A +4.6% retained
heading-ID result in the feature probe needs rechecking in the final combined
comparison; warmed buffers should not repeatedly pay initialization costs.

Validation: `cargo test --locked --all-features` passed. Evidence is in
`2026-09-10-markdown-feature-costs/optimizations/lazy-{comparison,counts,gfm-guard}.json`.

## 2. Initialize heading IDs at the first heading — retained

Hypothesis: the ID tracker accounts for three allocations even when a document
has no headings. Initialize it on the first heading, keep its storage on Renderer
reuse, and continue sharing the document registry with nested footnotes. The
constructor no longer needs options; the MDX caller follows that private change.

Compared with experiment 1 in seven paired >=100 ms windows: default tiny owned
improves another 15.1%, empty default 23.6%, and plain 1 KiB default 4.1%.
Heading-ID syntax improves 2.8% owned / 3.2% retained; most controls remain within
3%. The plain 16 KiB CommonMark retained control moves +3.9%, although this
workload never initializes the tracker; the final combined run rechecks it.

All-feature tests and all 292 exact HTML comparisons passed. A new regression
covers first headings appearing only inside footnotes, shared duplicate-ID
resolution, empty/plain documents, and repeated Renderer reuse. Evidence:
`optimizations/heading-comparison.json`. Baseline for this step: `950f014`.

## 3. Reuse opener scratch across inline extensions — retained

Hypothesis: highlight, subscript, and superscript each allocate an opener stack
for every parsed paragraph. Their resolution phases run sequentially, so they
can share the existing strikethrough scratch buffer, clearing it before each use.
Matching, link boundaries, and precedence remain unchanged.

All three medium workloads drop from 16 retained-renderer allocation calls to
zero. Compared with `9b5ce86`, seven paired >=100 ms windows measure:

| Feature | Medium fresh owned | Medium retained Renderer |
| --- | ---: | ---: |
| Highlight | -11.4% | -13.4% |
| Subscript | -11.8% | -15.5% |
| Superscript | -11.3% | -13.5% |

Single-unit retained workloads improve 13–14%; fresh single-unit calls still need
one stack allocation, so stay unchanged. Light/README/strikethrough controls
stay within 1.7%. All-feature tests and 292 exact HTML comparisons passed. Added
regressions cover mixed feature phases, unmatched openers, document/paragraph
boundaries, code spans, links, and table cells. Allocation output also confirms
experiment 2 leaves nine fresh allocations for tiny default rendering, matching
CommonMark. Evidence: `optimizations/openers-{comparison,counts}.json`.

The comparison driver now accepts explicit timing filters and records them;
exact HTML checks always cover the complete catalog.

## 4. Write footnote numbers without temporary Strings — retained

Hypothesis: decimal formatting allocates repeatedly for visible numbers, element
IDs, and backreferences. Reuse the existing stack-based decimal helper to write
numbers directly into HtmlWriter's output buffer. This introduces no dependency
or public API and does not change HTML escaping (the output is decimal digits).

Compared with `db74d79`, seven paired >=100 ms windows measure 16.2% less time for
medium inline footnotes with a retained Renderer and 14.2% less for fresh owned
rendering. Reference footnotes improve 6.6% retained. Inline-note allocation calls
fall from 131 to 35 retained (six temporary strings per note removed); reference
notes fall from 106 to 74 (two per note). The small inline-note workload improves
11.9% retained and 5.9% fresh owned.

All-feature tests and all 292 HTML comparisons passed. New checks cover zero,
decimal boundaries, usize::MAX, and mixed reference/inline numbering through 101,
including target IDs and backreferences. Evidence:
`optimizations/numbers-{comparison,counts}.json`.

## 5. Retain resolved-math storage — retained

Hypothesis: math resolution replaces and drops its span vector for every parsed
paragraph. Fill the existing parser-owned vector instead, clearing it before
resolution; preserve the existing code/math/link precedence and matching rules.

Compared with `37657c8`, seven paired >=100 ms windows show 13.0% less time for
medium math with fresh owned rendering and 14.5% less with a retained Renderer.
The medium retained workload drops from 16 allocation calls to zero. Small
retained math improves 12.0%; small fresh calls still allocate their first vector.

All-feature tests and all 292 exact HTML comparisons passed. A new regression
checks math-to-plain transitions, unmatched delimiters, code precedence, cells,
paragraph boundaries, and repeated documents. Evidence:
`optimizations/math-{comparison,counts}.json`.

## 6. Borrow inline-footnote content during emission — retained

Hypothesis: cloning an already-owned inline-note definition adds one allocation
and copy per note during emission. Borrow its bytes instead; nested rendering
uses separate numbering state and disables nested notes, so it does not mutate
the parent definition store. Rust verifies the disjoint field borrows.

Compared with `cddefb8`, seven paired >=100 ms windows measure another 10.7%
improvement for medium inline notes with a retained Renderer and 8.9% for fresh
owned rendering. Small retained notes improve 9.4%. Medium retained allocation
calls fall from 35 to 19, leaving 752 cumulatively requested bytes for 16 notes.
All-feature tests and all 292 exact HTML comparisons passed; the existing inline
note suite covers formatting, links, multiline content, and mixed numbering.
Evidence: `optimizations/borrow-comparison.json` and `final-counts.json`.

## Combined result against the original production baseline

Final production: `41d5201c804e760f400a2d25e2ebe06394ec6f04`. The feature comparison
uses seven paired >=150 ms windows per row, while the independent GFM guard uses
seven >=200 ms windows. Both alternate old/new binary order. All 292 feature
and 90 GFM HTML configurations match exactly. The earlier retained heading-ID
outlier is absent in this combined comparison (+0.3%); the plain 16 KiB retained
controls improve rather than reproduce their earlier slowdown.

| Workload | Fresh owned before → after (µs) | Fresh owned change | Retained Renderer change |
| --- | ---: | ---: | ---: |
| 17 B sentence / CommonMark | 1.073 → 0.384 | -64.5% | -0.3% |
| 17 B sentence / default | 1.136 → 0.378 | -66.9% | +0.7% |
| Plain 1 KiB / default | 2.243 → 1.536 | -31.1% | -4.2% |
| Light 1 KiB / CommonMark | 7.813 → 7.279 | -7.1% | -0.2% |
| Light 1 KiB / default | 8.551 → 7.978 | -6.2% | -0.2% |
| README / default | 136.707 → 128.617 | -4.0% | -2.3% |
| Heading ids | 4.196 → 3.516 | -16.4% | +0.3% |
| Tables | 8.026 → 7.115 | -10.9% | +0.8% |
| Highlight | 4.371 → 3.359 | -23.7% | -15.9% |
| Subscript | 4.037 → 2.924 | -27.4% | -17.7% |
| Superscript | 4.208 → 3.297 | -22.1% | -13.3% |
| Math | 3.839 → 2.861 | -25.3% | -14.6% |
| Footnotes | 12.241 → 10.921 | -11.1% | -8.3% |
| Inline footnotes | 12.344 → 9.040 | -26.8% | -24.8% |

The GFM guard's larger cases mostly move within 3%. Two retained controls merit
a longer recheck: inline code is +4.5% in the feature comparison and CommonMark
README is +3.5% in the GFM harness (the feature harness's CommonMark README is
-0.4%). The recheck and its outcome are recorded below rather than omitting these
outliers. Synthetic controls can also improve from allocation/layout differences
without their parsing algorithm changing, as with retained plain paragraphs.

## Reproduction and verification

The original production revision predates the new probe. Use `bfb3687` to build
a baseline probe with identical production code, and `41d5201` for the final
probe. In separate checkouts, build without profiling and preserve both binaries:

```sh
cargo build --locked --profile release-debug --example feature_cost_probe --example gfm_optimization_probe
```

Use the committed `2026-09-10-markdown-feature-costs/catalog.json` for exact replay;
regenerating the catalog reads the checkout's README, which may change later.
Pass the saved binaries to `scripts/compare-feature-probes.py` with the filters
and windows in `combined-comparison.json`, and to `scripts/compare-gfm-probes.py`
with `--window-ms 200`. Build the profiling-feature probe separately for counts.
The CPU-capture helper used `profile_harness` built at `321512b` (production
identical to the study baseline) and saved as `target/gfm-profile/profile-url`.

Validation on the final production code:

- `cargo test --locked --all-features`: 979 tests passed, including doctests.
- Workspace/all-target/all-feature Clippy with `-D warnings`: passed.
- `cargo fmt --all --check`: passed.
- All-feature library compile checks for x86_64 Linux and Windows MSVC: passed.
- Catalog and original baseline report regenerate byte for byte.
- Native x86 execution/performance was unavailable; all timing claims are M1 Pro.

Raw measurements and binary/catalog hashes live in the adjacent data directory;
`validation.json` records versions, revisions, and check results. These changes
add no unsafe parser code and no dependencies. The durable design summary is
[ARCH-EXP-017](../arch/ARCH-EXP-017-markdown-feature-costs-and-lazy-scratch.md).

## Longer control recheck and accepted limitation

Nine paired >=500 ms windows per binary, alternating order, using the same
retained executables and input catalog:

| Retained control | Before → after (µs) | Paired median change | Range across pairs |
| --- | ---: | ---: | ---: |
| inline-code | 4.980 → 5.235 | +4.8% | +0.0% to +6.7% |
| readme-commonmark | 108.703 → 108.185 | -0.4% | -1.7% to +1.7% |

The README slowdown does not reproduce. The inline-code control **does reproduce**
and is retained as a known local trade-off: about 0.26 µs extra per complete
32-paragraph document with a warmed Renderer, while fresh owned rendering of the
same workload improves 9.0%. This is not classified as noise. The cause has not
been isolated; changed buffer capacities/code layout are hypotheses, not findings.
The larger mixed CommonMark/GFM controls do not show this consistent regression.
The overall changes are retained for their substantial fresh-call and targeted
feature gains, without claiming every workload improves.

Evidence: `control-recheck.json`; `recheck-controls.py` reproduces the capture
against the saved binaries. The original outlier measurements remain preserved.

## What the current feature costs mean

The [final feature tables](2026-09-10-markdown-feature-costs-final.md) are regenerated
from five complete measurement rounds on the final implementation. Most unused
boolean options add little detection overhead; autolink detection is a measurable
exception. Link-base rewriting also clones configuration for a fresh parser.

In these deliberately syntax-dense, retained-renderer workloads, plain paragraphs
cost about 1.2 µs/KiB; math, inline code, strikethrough, and highlight about
3.3–3.7; emphasis and links about 5.7–5.9; tables, entities, and lists about
8.4–9.1; reference links and footnotes about 10.5–11.7. These groups compare
*different documents*. They include syntax density, block count, HTML expansion,
and actual parsing work; they are not independently summable feature prices.

Small documents benefit especially from reducing initialization costs. Reusing a
Renderer is still valuable: the final 17-byte default workload takes about
0.37 µs fresh versus 0.09 µs retained, with nine versus zero new allocation calls.
