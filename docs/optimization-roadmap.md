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
from one cell. The [comparison at the fixed revision](reports/2026-09-21-native-release-fixed/README.md) measures v2 at 1.98× v1, 2.53× pulldown-cmark, 3.54× md4c and 5.74× Bun's native engine fresh on the 50 five-engine documents, and 1.15× OX on the 14 all-six documents; the paired harness puts the fixed core at 0.982× fresh and 0.967× parse against the `7c887a2b` core, the remainder being the review's nesting cap and link probe on link-dense prose and sub-microsecond comments. These direct measurements replace
extrapolation from optimization speedups; the older reports retain their
frozen evidence.

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
   function is a useful next profiling target; retain stage controls.
2. **Tiny-input fresh allocation.** The source-size heuristic reserves at least
   16 KB even for a 37-byte comment. The retained-arena and fresh-arena cases
   should remain separate when considering a different reservation strategy.
3. **Fuse table pipe discovery and unescaping.** The row splitter and cell
   decoder still inspect pipe/escape locations independently. Carrying exact
   positions may remove work, but must preserve trimming, code-span semantics,
   malformed rows, and source mappings.
4. **Other architectures.** Run differential tests and real/diagnostic suites on
   x86-64 SSSE3/AVX2 and scalar targets. The current evidence is Apple M1 Pro NEON;
   cross-target speedups are not established.

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

Report every stage and workload tradeoff. Geometric means summarize the chosen
cases; overlapping document views are not independent observations or evidence
that every input wins. Promote based on measured complete processing plus
correctness, while recording isolated stage losses and allocation behavior.
