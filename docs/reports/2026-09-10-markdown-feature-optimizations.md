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
