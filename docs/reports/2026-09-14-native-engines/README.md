# Six native Markdown engines — 2026-09-14

Ferromark v2 leads the geometric mean of this 57-document Markdown→HTML mix.
With fresh state, it is **1.39× as fast as current local Ferromark v1, 1.96× as
fast as pulldown-cmark, 2.62× as fast as md4c, and 4.44× as fast as Bun's native
Markdown core**. The gap to original OX-Content is much smaller: **1.03×**.
These are measurements in the pinned common environment below, not universal
rankings or performance claims about JavaScript APIs.

[All tables](TABLES.md) · [CSV timings](timings.csv) · [HTML review](OUTPUT-REVIEW.md) ·
[Source/build provenance](PROVENANCE.md) · [Harness and reproduction](../../../benchmarks/native-comparison/README.md)

## Aggregate results

Speed relative to v2 in the same lifecycle; **higher is faster**, v2 = 1.00×.
Each document has equal weight in the geometric mean. “Fresh” includes creation
of parser state, full parsing/rendering, owned output allocation, and destruction.
“Reuse” retains arena/scratch/output where each original public API supports it.
Bun exposes owned output here and still makes the same fresh call in the reuse schedule.

| Engine | Fresh, all 57 | Reuse, all 57 | Fresh, 14 agreeing outputs | Reuse, 14 agreeing outputs |
| --- | ---: | ---: | ---: | ---: |
| Ferromark v2 | 1.00× | 1.00× | 1.00× | 1.00× |
| OX-Content original | 0.97× | 0.97× | 1.00× | 0.99× |
| Ferromark v1 | 0.72× | 0.75× | 0.81× | 0.84× |
| md4c | 0.38× | 0.36× | 0.26× | 0.21× |
| pulldown-cmark | 0.51× | 0.48× | 0.46× | 0.39× |
| Bun native core | 0.23× | 0.20× | 0.17× | 0.13× |

The full mix spans 37–113,609 UTF-8 bytes, with 12 authored comments, 12 legacy
Ferromark documents, 16 external project documents, 16 overlapping Wikipedia
views, and one syntax guard. All 57 are timed. CommonMark is used for 17 cases;
40 use the shared GFM subset of tables, strikethrough, and task lists. It is not
full GFM or an MDX benchmark. Raw HTML is preserved and product-specific
sanitization, renderer URL autolinking, and new-tab link targets are disabled.

Only **14 cases have equivalent output across all six engines**: ten comments
and four plain-prose Wikipedia views. Thus the strict subset is narrower than
the corpus and is not a substitute for the full mix. The 43 diagnostic cases
include 34 heading-ID-only differences and nine additional rendering differences.
OX/v2 always emit heading IDs; other engines have that option disabled. These
IDs were not stripped inside timing or silently treated as equivalent output.
Full raw HTML and classifications are retained. **OX and v2 are byte-identical
on every one of the 57 documents.** V1 hit no resource-limit fallback.

## What the distribution shows

- V2 leads the group geometric mean in all five size bins against V1, md4c,
  pulldown-cmark, and Bun. It does not win every document. The 310-byte table
  comment favors V1 by about 11% fresh and 16% with reuse. Several reference
  documents also favor V1; the reference category is approximately tied with reuse.
- The OX/v2 gap is concentrated in encyclopedia Markdown: OX has about 0.90×
  v2's speed there. Their plain-prose and short-comment averages are close.
  The strict subset shows under 1% separation, so this run does not support a
  broad claim that every v2 document beats the original OX core.
- Plain prose strongly separates these native implementations: in fresh mode,
  V1 has about 0.80× v2's speed, md4c 0.25×, pulldown 0.30×, and Bun 0.06×.
  These are four views from four Wikipedia topics, not a representative sample
  of all prose or an explanation of the underlying CPU costs.
- The first two full process rounds produced almost the same group ratios
  (for example, V1/v2 fresh 0.721 and 0.717 before final median aggregation).
  All three rounds and their ranges are retained. No statistical significance
  or cross-machine portability claim is attached to small differences.

Some absolute latencies make the size range concrete. Fresh complete operations
in microseconds; lower is faster:

| Input | Bytes | V2 | OX | V1 | md4c | pulldown | Bun native |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Acknowledgment comment | 37 | 0.158 | 0.156 | 0.159 | 0.777 | 0.187 | 0.293 |
| Review comment | 282 | 0.218 | 0.220 | 0.266 | 1.071 | 0.496 | 1.327 |
| Chess plain prose | 80,966 | 28.780 | 29.028 | 34.366 | 101.097 | 86.041 | 429.329 |
| Chess article Markdown | 113,609 | 174.130 | 190.577 | 251.519 | 355.148 | 325.130 | 741.833 |

The first three rows satisfy all-six HTML agreement. The article row remains
diagnostic because of heading IDs and the documented link differences.
Full document latencies, category and size tables, and the two rotating batches
are in [TABLES.md](TABLES.md). A rotating-batch operation processes the entire
profile corpus and is not included in the equal-document geometric means.

## Validation and evidence

One shared native executable on Apple M1 Pro/macOS, same nightly Rust compiler,
release/fat-LTO profile, mimalloc, and pinned shared dependency resolution.
The compiler, allocator, and some dependency versions differ from earlier
v1/v2 reports; those old measurements must not be combined with these ratios.
Bun is its original Rust Markdown engine plus original native SIMD/allocation
support, with a standalone stack boundary adapter. No full Bun runtime, JS,
WASM, process launch, input I/O, or JSON work is inside a timed call.

Three process rounds × six windows × 59 individual/batch workloads × two
lifecycles × six engines produced **12,744 timed windows**. Each window lasts
at least 40 ms, after 60 ms per-engine warmup. Engine positions rotate; document
order is shuffled. Timed output-length checksums, fresh/reuse equality,
pre/post-timing equality, effective feature guards, and repeated document
transitions all passed. The root workspace's 638 Rust tests, formatting, strict
Clippy, and seven benchmark builds passed; 11 Python harness contract tests passed.

- [Raw timing windows](samples.json.gz), [run conditions](run.json),
  [per-document summary](summary.json), [aggregate data](aggregates.json).
- [Frozen corpus with source attribution](corpus.json.gz),
  [all HTML outputs](verification.json.gz), [feature/transition guards](behavior.json.gz).
- [Measured build metadata](build.json), [exact resolved Cargo lock](Cargo.lock),
  [independent source audit](source-audit.json.gz), [validation metadata](validation.json).
- Compressed workspace/build/run logs and the original source lockfiles are included.
  The source audit checked 35 V1 files, 354 V2 files, 3,495 Bun files, 10 md4c
  files, and 393 OX files against pinned Git objects or the original archive,
  plus the two native dependency archives; no parser source mismatch was found.

The native library build emitted md4c's existing C99 typedef warning and Cargo's
unused Bun profile-entry warning. Neither changed source or stopped the build.
Initial adapter/setup failures were corrected before measurement; the archived
build logs retain that history. The final harness was also rebuilt from a fresh
disposable workspace with the archived lock and its outputs checked against the
measured build; the replay record is in [validation.json](validation.json).

`SHA256SUMS` covers archived evidence and generated tables. Regenerate the tables
with `python3 benchmarks/native-comparison/report.py docs/reports/2026-09-14-native-engines`.
