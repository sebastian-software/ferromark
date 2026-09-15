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

The [new native comparison](reports/2026-09-15-native-arm/README.md) separately
remeasures all six engines with matched syntax and renderer settings. V2 is
effectively tied with OX fresh (1.000×) and reaches 1.018× OX throughput with
reuse on the 14 all-six agreeing inputs. On 50 inputs agreeing among the five
configurable engines, v2 reaches 1.90× v1 fresh and 1.85× with reuse. OX has
no score on that broader set. These direct measurements replace extrapolation
from optimization speedups; the older reports retain their frozen evidence.

## Next questions

The [iteration-round report](reports/2026-09-15-arm-iterations/README.md) ranks
the currently measured hot spots: output-buffer growth checks in the escaper,
the double scan of paragraph lines, the reference-definition pre-pass bail, URL
sanitization searches, heading slugs, and the renderer preparation scan. The
questions below remain open as well.


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
