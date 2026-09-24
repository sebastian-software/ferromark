# Measured optimization work

The [optimization rounds](reports/2026-09-14-optimization-rounds/README.md) now
contain five locally promoted optimizations and the rejected variants that led
to them. The [initial SIMD study](reports/2026-09-14-simd-round/README.md) remains
an immutable account of its original measurements apart from a follow-up link.

## Implemented and measured

- Clean link components return their borrowed slice after one `memchr2` probe;
  escapes/entities retain the original scalar semantics. Repeated SIMD probing
  was rejected for dense-escape regressions.
- Optional inline markers share one exact SIMD classifier selected from eight
  precomputed option sets. A combined `memchr3` memo introduced repeated suffix
  scans; an all-marker table with retry filtering regressed on disabled markers.
- Escaped table-cell source spans use 64-byte bitsets with prefix counts and
  constant-time rank lookup. Binary-search sparse lists saved memory but slowed
  formatted cells. Defensive span clamping is preserved.
- Bare-URL punctuation trimming caches bracket counts, bounding work to three
  linear scans instead of rescanning once per trailing closer.
- AArch64 NEON skips ASCII URL spans, with Unicode/CJK handling and portable
  fallbacks retained. An immediate Unicode host avoids unnecessary SIMD setup.
- Definition-list probes cache possible markers, retain term source ranges,
  and skip irrelevant block recognizers on ordinary ASCII terms. Line-comment
  paragraphs reuse already discovered comment positions. The
  [feature-cost follow-up](reports/2026-09-14-feature-scan-optimization/README.md)
  records these changes, their remaining runtime costs, and rejected variants.
- The [Apple Silicon iteration round](reports/2026-09-15-arm-iterations/README.md)
  added ten commits: NEON stop-byte classifiers for link destinations and
  bracket bodies (`ByteClass`), a one-scan URL escaper, a one-pass autolink
  pre-flight, cached fence-close search, a boxed `ParseError`, out-of-line
  17–64-byte copies, and reuse of scan facts in link parsing. Against
  `a7f0a00`, the [complete 207-case confirmation](reports/2026-09-15-arm-full-suite/README.md)
  measures the finished branch on all four stages. The 57 broad documents
  gain 1.298× in parsing, 1.257× with reuse, 1.224× fresh, and 1.199× in
  rendering. Every broad fresh/reuse/parse median improves; isolated render
  and diagnostic losses remain visible in the full tables. The original
  attempts, rejected variants, patches, and raw results are archived with the
  iteration report.
- The [second Apple Silicon round](reports/2026-09-15-arm-round-2/README.md)
  added twelve commits on the single-crate core: renderer fixed costs (lazy
  heading scratch buffers, the autolink first-byte index built once per
  renderer, a setup walk skipped when no option reads it, a 64-byte output
  floor, a minimal footnote reset), heading ids (no escape pass for generated
  slugs, slugifying single-text headings from the source, an ASCII slug
  cursor), an inline source-span gate, and parser work (the definition
  pre-pass driven from `]:` with a static searcher, SWAR short-slice probes,
  emphasis bookkeeping). Against `fea50462` the 57 broad documents gain
  1.050× fresh, 1.041× with reuse, 1.012× in parsing, and 1.102× in
  rendering; comment-sized inputs render 1.16× and the 45 diagnostics
  1.129×. Two of the first round's top-ranked hot spots were tried and
  lost: a raw output cursor in the escapers (render 0.973× / 0.940×) and a
  fused block/inline paragraph scan (parse 0.949×). Patches and screens are
  archived with the report.
- The [third round](reports/2026-09-16-arm-round-3/README.md) added nine
  commits: every block-level line walk finds a line end once (lists, block
  dispatch and probe, containers and leaves, definition lists and table
  metadata), bare fence languages bypass the metadata tokenizer and the
  plain fence markup is assembled from merged literals, and — as a recorded
  v2 API decision — `HtmlRendererOptions` strings became `Cow<'static, str>`
  so default options and their clones allocate nothing. Against `f216b8da`
  the 57 broad documents gain 1.142× fresh, 1.027× with reuse, 1.036× in
  parsing (every document), and 1.009× in rendering. The fence-run finders,
  a document-level autolink gate, and reserved text coalescing were measured
  and rejected. The same report measures profile-guided optimization as a
  build experiment: trained on half the broad documents plus the
  diagnostics and measured on the other half, PGO gives 1.204× fresh,
  1.240× reuse, 1.255× parse and 1.176× render on documents it never saw.

The [native comparison](reports/2026-09-15-native-arm/README.md) separately
remeasures all six engines with matched syntax and renderer settings, and has
been rerun at every milestone since: the
[release-readiness run](reports/2026-09-15-release-native/README.md), the
[segmented definition pass](reports/2026-09-16-native-segments/README.md),
and the [release head](reports/2026-09-21-native-release/README.md). That
last run found the head 7–12% behind the position `7c887a2b` had held
against every engine; the [paired attribution](reports/2026-09-21-release-fixes-paired/README.md)
traced it to the final review's laziness tracker, to the parser struct its
memo tables had grown, and to two hashed lookups per bracket, and the
[decision record](decisions/2026-09-21-lazy-tracker-cost.md) describes the
fix: the tracker runs on demand and classifies lines from their first byte,
the memo tables allocate on first use, and the closer memo answers repeats
from one cell. The [comparison at the fixed revision](reports/2026-09-21-native-release-fixed/README.md) measured v2 at 1.98× v1, 2.53× pulldown-cmark, 3.54× md4c and 5.74× Bun's native engine fresh on the 50 five-engine documents, and 1.15× OX on the 14 all-six documents. [Optimization round 4](reports/2026-09-21-perf-round-4/README.md) then screened nine of our own ideas one by one with the paired harness and merged three — the forward window for the closer probe, the 2 KB arena reservation floor and the plain inline depth cell — each re-screened on top of the previous; the [comparison after round 4](reports/2026-09-21-native-round-4/README.md) puts `main` back at the `7c887a2b` position: 2.07× v1, 2.65× pulldown-cmark, 3.51× md4c and 6.03× Bun's native engine fresh on the 50 documents, 1.19× OX on the 14, and 1.27× from PGO on the held-out half. These direct measurements replace
extrapolation from optimization speedups; the older reports retain their
frozen evidence.

[Optimization round 5](reports/2026-09-23-perf-round-5/README.md) started
from a sample profile of release 2.0.1 (`cb352020`) over the 57 broad
documents and merged four changes: block-boundary trimming over ASCII
whitespace only (#414, a CommonMark conformance and span fix with its own
[decision](decisions/2026-09-23-commonmark-whitespace-trim.md), which removed
the 6% of parse time spent in Unicode `str::trim`), one NEON root scan for the
first NUL and the first `]:` (#413, revised once to end on an overlapping
vector after it lost on sub-kilobyte comments), a first-byte gate for the GFM
tag filter (#411, render 1.02–1.18 on documents with raw HTML), and a warm
output buffer for the reusable Node renderer (#412, 1.027 at the Node level).
Re-screened one on top of the other and measured together against
`cb352020`, the three Rust changes give 1.049× fresh, 1.055× with reuse,
1.080× in parsing (every one of the 57 documents faster) and 1.005× in
rendering. Moving the GFM autolink pre-flight into the inline marker scan
(#415) measured 0.968× in parsing and was closed.

[Optimization round 6](reports/2026-09-24-perf-round-6/README.md) worked
through round 5's remaining hot spots on Apple Silicon and made x86-64
measurable. On the M1 Pro it merged two render changes: the tag filter compares
a raw HTML tag only with the filtered names that share its initial (#418,
render 1.29× on the raw-HTML reference page), and heading ids are planned in
reusable claim storage (#422, broad render 1.035×, reuse 1.012×). It closed the
second iterations of round 4's table and container revisions (#424, #425) and
a third design for the autolink pre-flight (#427). Two CI workflows now run the
paired harness (#419) and a `perf` profile (#423) on GitHub's x86-64 runners;
the paired workflow's A/A control stays within ±0.5% on three hosts. The first
x86 profile led to three merged x86 SIMD paths: `ByteClass` with SSSE3 and AVX2
(#426, parse 1.060–1.069×), the escape classifiers as inlined SSE2 (#430,
render 1.091–1.093×) and an SSE2 `line_end` (#431, parse 1.062–1.065×). The
port of round 5's fused root scan (#428) was closed. Against release 2.1.1, the
code of 2.1.2 measures 1.116–1.148× fresh, 1.125–1.152× with reuse,
1.135–1.160× in parsing and 1.112–1.126× in rendering on an Intel Xeon 6973P-C
and two AMD EPYC 7763. The round also found that on x86, code placement moves
stages a change does not touch by ±2–3%, consistently on all three hosts, and it
used a placebo build to tell that apart from real cost.

## Next questions

The [iteration-round report](reports/2026-09-15-arm-iterations/README.md) ranked
the measured hot spots after the first round. The [second round](reports/2026-09-15-arm-round-2/README.md)
settled most of them: the reference-definition pre-pass bail, heading slugs,
and the renderer preparation scan are done; the escaper's output-buffer growth
checks and the double paragraph scan were implemented as designed and measured
slower, so they should be treated as structural floors unless a genuinely
different mechanism is proposed. URL sanitization searches remain untested
(the benchmark profiles do not enable `sanitize`). The questions below remain
open as well. The third round settled the `HtmlRendererOptions` string
ownership and the line-end rescans, and measured the fence-run finders and a
document-level autolink gate as non-wins. Its two forward-looking items are
(a) profile-guided optimization, by far the largest measured lever
(1.18–1.26× on unseen documents) — now applied to the published native
addons ([ADR-0019](arch/ADR-0019-profile-guided-native-addon.md)), with the
native comparison able to build every Rust engine the same way so the
published numbers stay fair — and (b) the definition
pre-pass, which block-parses a document twice whenever it holds a `]:`
candidate. Item (b) is now addressed structurally: the pass runs on the
segments that can hold a definition, each bounded by a line start where the
real parser is at the document root, and a document whose candidates all sit
in code or raw HTML runs no structural pass at all. The exactness argument,
what still falls back, and the differential proof are recorded in
[the decision](decisions/2026-09-16-segmented-definition-pass.md); the
measured effect is in [its report](reports/2026-09-16-definition-segments/README.md). A blocks-first parse with inline
content resolved afterwards remains the larger, unattempted variant.


1. **Code layout and render-only variance.** Unchanged render paths can shift
   across combined fat-LTO binaries. The earlier link-probe losses disappear
   without LTO and change with the expanded worker. A precise instruction/cache
   or memory-placement mechanism remains unproven. The large `render_node`
   function is a useful next profiling target; retain stage controls. Round 6
   measured the effect on x86-64: an inlined runtime dispatch that grew its
   callers cost 4–7% on documents that never run the dispatched code (#426's
   first commit), and stages a change does not touch move by ±2–3%. The three
   CI hosts build byte-identical workers, so a layout effect looks consistent
   across hosts. A placebo build — the candidate's code structure with the old
   algorithm — is the control that separates layout from cost; see the
   [round 6 methodology](reports/2026-09-24-perf-round-6/README.md#methodology-what-the-round-learned-about-measuring).
2. **Tiny-input fresh allocation.** Round 4 lowered the reservation floor from
   16 KB to 2 KB, which gave comment-sized documents 2.2% fresh. The
   retained-arena and fresh-arena cases should remain separate when
   considering a different reservation strategy.
3. **Tables and container lines.** Tables hold 10% and lists 12% of parse time
   after round 5. Round 6 measured the second iterations of both round 4
   revisions and closed them ([record](reports/2026-09-24-perf-round-6/REJECTED.md)).
   The NEON pipe cursor (#424) parses the dense table diagnostics 2.3–3.1×
   faster but loses 2–7% on realistic cells of 5–30 bytes, so the one idea left
   is a gate on pipe density in front of it. The container line facts (#425)
   measured a tie, because #414 had already removed the Unicode trims they
   saved. The structural route for containers — parsing block quotes and list
   items on the original source instead of copying and reparsing them — is
   tracked in #432, outside the rounds.
4. **The GFM autolink pre-flight.** About 10% of parse time after round 5: a
   second `memchr2` + `memmem` pass over each block's content. Every attempt
   to avoid the pass lost or tied: round 3's document-level gate, round 5's
   fusion into the inline marker scan (#415, parse 0.968×), a standalone
   single-pass NEON pre-flight (about 1% in a prototype) and round 6's trigger
   index in two versions (#427: 1.010× with a 13% loss on raw HTML, then a tie
   with small-document losses; see its
   [record](reports/2026-09-24-perf-round-6/REJECTED.md#427-the-autolink-pre-flight-from-recorded-trigger-offsets)).
   Treat the per-block pre-flight as a floor unless autolinking moves into
   inline parsing itself.
5. **Render: heading ids and the tag filter.** Round 6 addressed both: #418
   compares a tag only with the filtered names sharing its initial, and #422
   plans heading ids in reusable claim storage. What remains is the slugifier
   (`slugify_heading_into`, about 5% of render on both architectures); a
   table-driven or branchless ASCII slug loop is unmeasured.
6. **Other architectures.** x86-64 is measured since round 6. The
   `x86-64 paired benchmark` workflow (#419) runs the paired harness on three
   GitHub-hosted hosts (its A/A control stays within ±0.5% in every stage, with
   per-case ratios within about 0.975–1.025), and the `x86-64 profile`
   workflow (#423) samples parse and render with `perf`. The link-destination
   and bracket-body classifiers (`byte_class`, SSSE3 and AVX2, #426), the line
   scan (`line_scan`, SSE2, #431) and the escape scanners (inlined SSE2, #430)
   now have x86-64 SIMD paths. The root scan deliberately stays on separate
   `memchr`/`memmem` searches on x86-64 (#428): `memchr` runs AVX2 there, so
   the two passes are cheap and a fused scan gained about 1% while losing on
   small documents. The renderer's ASCII URL-span scan for autolinks is still
   NEON-only and has not been timed on x86-64. Targets without a vector path
   of their own, such as `wasm32`, are neither run in CI nor timed.
7. **Next measurements.** The six-engine native comparison has only run on
   Apple Silicon. It should run on Linux x86-64, together with a fresh Apple
   Silicon run, before new comparison numbers are published. The Node N-API
   boundary — how much of a Node call is spent crossing into and out of Rust
   rather than parsing and rendering — has not been measured on its own.

Ferroni's candidate scanner (`src/regset.rs`) and Ferrocat's structural scans
(`crates/ferrocat-po/src/scan.rs`) remain useful references in those repositories.
OX-Content already supplied SIMD nibble classifiers, SWAR line
scans, code-span fast paths, arena sizing, and an autolink prefix index; these
existing facilities should not be counted as new ports.

## Experiment discipline

Freeze source, inputs, toolchain, dependencies, and build settings. State a
falsifiable workload prediction, change one concept per variant, and preserve
its patch and result even when reverted. Validate exact output and ownership/
span invariants before timing. Keep one optimization per commit, portable
fallbacks, and full conformance checks. Broader redesigns are welcome when they
have a representative feedback loop rather than only favorable synthetic cases.

When a change moves a stage it does not touch, measure a placebo build — the
same code structure with the old algorithm — before attributing the movement
to the change. Report every stage and workload tradeoff. Geometric means
summarize the chosen cases; overlapping document views are not independent
observations or evidence that every input wins. Promote based on measured
complete processing plus correctness, while recording isolated stage losses and
allocation behavior.
