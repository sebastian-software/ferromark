# Definition-list and line-comment costs

Historical results: the later [line-comment dispatch study](../2026-09-14-line-comment-dispatch/README.md)
reduces the remaining +3.2% plain-prose parser overhead further by reusing
existing line-prefix scans. The measurements below remain unchanged.

The unusually high unused-feature costs came from repeated discovery and
speculative term storage. The retained implementation removes that work without
changing syntax, options, HTML, ASTs, or source positions. All attempted variants,
including two rejected approaches, are recorded in [ATTEMPTS.md](ATTEMPTS.md).

Baseline: `1728355`, whose core is identical to the earlier runtime-profile
study's `c57ef45`. This is a v2-before/v2-after investigation with identical
runtime options on both sides, not a new comparison against other libraries.
The original profile study remains a historical measurement.
Promoted locally as `203babb` (definition lists) and `3c868eb` (line comments).
The [production-source hashes](measured-production.json) confirm that the
retained implementation equals the timed candidate; subsequent edits were
confined to test fixtures and documentation.

## What was expensive

**Definition lists:** unsuccessful probes still collected potential terms into
an arena vector, repeatedly checked other block recognizers, and searched for
line endings before checking whether a line began with `:`. The new allocation
regression failed on the original code: an ordinary rejected probe caused the
previously empty arena to reserve 448 bytes. This is arena capacity, not the
exact size of the term vector. The retained probe uses zero arena bytes on the
tested rejected inputs, including ordinary prose before a later definition.

The parser now caches the next possible body marker, rejects non-colon prefixes
before searching for line endings, and represents validated terms by source
coordinates and a count. ASCII-letter term prefixes skip irrelevant block
recognizers while retaining the inline-backtick check. Every dedented subparser
gets a fresh marker cache. A real marker later in the source can still retain
speculative work on earlier paragraphs.

**Line comments:** paragraph parsing already identified eligible comments, but
the text-joining helper searched for them again. It now receives the first
known comment position. A paragraph without comments, or with comments only
beyond its content boundary, keeps its original source slice. Reference
definitions still need independent discovery and have a cheap marker preflight.
Actual removed lines still require joining text and preserving the mapping to
physical source positions. That necessary work remains.

## Confirmed same-option results

Three process rounds, three paired windows per round, 50 ms per window. Times
below are median microseconds per document. Savings use the median paired ratio;
they can differ slightly from the ratio of the displayed rounded times.

| Feature / input | Bytes | Parser before → after | Parser time saved | Complete reused before → after | Complete time saved |
| --- | ---: | ---: | ---: | ---: | ---: |
| Definition lists enabled, plain prose | 4,125 | 3.821 → 1.295 µs | 66.1% | 4.374 → 1.835 µs | 58.1% |
| Repeated active definition lists | 4,104 | 52.722 → 43.485 µs | 17.4% | 56.126 → 47.141 µs | 16.3% |
| Line comments enabled, plain prose | 4,125 | 1.458 → 1.219 µs | 16.6% | 2.002 → 1.756 µs | 12.2% |
| Repeated active line comments | 4,116 | 9.838 → 8.993 µs | 8.9% | 10.987 → 10.215 µs | 7.2% |

The smaller and larger probes confirm the direction. On plain prose from
330 B to 65,670 B, definition-list parser ratios are 2.486–2.986× and line-comment
ratios are 1.166–1.200×. Active definition-list parsing improves 1.198–1.211×;
active line-comment parsing improves 1.086–1.098×. These are specific repeated
input shapes, not fixed feature prices on arbitrary documents.

At the 64 KiB target, the additional diagnostics improve parser throughput by
1.239× for prose followed by a late definition, 1.416× for an indented definition
body, 1.331× for many terms sharing a body, 1.694× for inline-colon prose without
definitions, and 1.457× for slash-heavy text without eligible comments. The
4 KiB versions show the same direction. These shapes specifically check that
the fast rejection paths do not merely shift work into common near-misses.

## Mixed documents and controls

The 13 documents comprise authored comment examples, project documentation, and
Wikipedia-derived Markdown/prose views, from 37 to 113,609 bytes. Each document
has equal weight in the geometric mean of its median paired ratio. Related
Wikipedia views are correlated; this is not a population estimate.

| Configuration | Parse | Fresh complete processing | Reused complete processing | Render prebuilt AST |
| --- | ---: | ---: | ---: | ---: |
| Docs recipe plus definition lists | 1.390× | 1.211× | 1.297× | 1.000× |
| Strict CommonMark controls | 1.003× | 1.000× | 1.006× | 0.999× |

Ratios are baseline time / candidate time; above 1 favors the candidate.
The enabled recipe includes line comments as well as definition lists. Its
reused ratio corresponds to 22.9% less time. Every mixed document has a reused
ratio above 1, but TypeScript Compiler Options is only 1.003× and is effectively
unchanged. Its parser-only ratio is 0.984×.

Small losses remain visible: CommonMark Chess parsing is 0.977×, tiny
acknowledgment fresh processing is 0.984×, and rendering the prebuilt active
comment ASTs is 0.981–0.983×. Renderer code and verified ASTs are unchanged;
cross-binary code layout and measurement effects can still move these controls.
Their exact cause is unproven. No samples were removed, and the report does not
claim every stage or document improved. [All 300 final rows](final/summary.csv)
include per-round ratios and the full observed pair range.

## Remaining price of enabling an unused feature

A separate same-binary off/on comparison repeats the 4,125-byte plain probe
with each option enabled or disabled. Both settings produce identical HTML and
AST. Each binary was measured in three rounds of three 50 ms pairs; these are
new measurements of both versions, rather than ratios inferred from different
studies.

| Enabled option | Parser overhead before | Parser overhead after | Complete reused overhead before | Complete reused overhead after |
| --- | ---: | ---: | ---: | ---: |
| Definition lists | +217.2% | +9.4% | +148.9% | +6.6% |
| Line comments | +20.1% | +3.2% | +13.8% | +2.4% |

The old line-comment estimate was +22.5% in the earlier study; the new baseline
measurement reproduces its direction and approximate size at +20.1%. The
remaining cost is small but measurable on this shape. An enabled marker scan
or per-line eligibility branch still performs work. Active feature costs are
different: comparing feature-on to feature-off there changes the generated AST
and HTML, so the same-output optimization table above is the appropriate check.
Raw [before](off-on-baseline/summary.csv) and [after](off-on-candidate/summary.csv)
rows also retain the 300 B and 64 KiB targets and renderer controls.

## Verification and reproduction

All 75 benchmark cases passed exact HTML, complete AST Debug, and child-count
equality in all four lifecycle modes before timing. Every timed worker is
checked again before and after measurements, with output/child-count checksums
on every sample. The final sweep contains 2,700 paired observations.

The additional deterministic differential corpus contains 400 documents under
three feature combinations. It exercises CRLF/CR/LF, BOMs, Unicode terms,
backticks, nested and dedented sources, comment ordering, reference definitions,
opaque HTML/code, and source spans. Its 1,200 comparisons use the pre-change
implementation as a regression oracle; they do not establish an independent
specification conformance claim.

The final workspace run passes **770 tests in 54 suites**, including the existing
CommonMark/GFM and pinned-oracle regressions. Formatting, strict workspace Clippy,
all benchmark builds, and the three replay-harness tests pass. Commands and logs
are preserved under [checks](checks/commands.json); generated inputs and their
aggregate output digest are under [generated](generated/result.json).

The [harness](../../../benchmarks/feature-scan-optimization/README.md) freezes
source and worker hashes, dependency locks, compiler flags, runtime options,
input bytes, and host metadata. Both binaries use Rust 1.95, generic AArch64,
fat LTO, opt-level 3, one codegen unit, and identical pinned registry packages.
Measurements ran serially on the shared macOS arm64 workstation on AC power;
there was no CPU pinning. CPU-model and thermal probes were unavailable in the
sandbox. No builds ran concurrently with timed measurements.

Archived corpus profile paths reflect the original temporary directories.
The harness wrapper reconstructs them from `runtime_options`, so replay does
not depend on those paths. Original input provenance is preserved separately
in [attribution.json](attribution.json), keyed by case name and input hash;
the current generator also includes it directly in each case. The
[source corpus documentation and licenses](../../../benchmarks/broad-comparison/README.md)
remain authoritative for the third-party inputs.
