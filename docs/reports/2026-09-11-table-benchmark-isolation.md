# Isolated table benchmarks and remeasurement — 2026-09-11

Historical intermediate experiment: the active benchmark now uses only plain and
CommonMark-formatted cells. See the [follow-up](2026-09-11-commonmark-table-costs.md).
The strikethrough cases below are archived diagnostics and are no longer active.
Their former shared fixture is preserved as
[`tables-inline.md`](2026-09-11-table-benchmark-isolation/tables-inline.md) next to
the historical runner; the descriptions below record the intermediate state.

The table benchmark at this stage separates ordinary table parsing from enabled inline
formatting and deliberately disabled syntax. On this run, the plain table case
costs Ferromark **34.25 µs versus
30.97 µs** for pulldown-cmark
(10.6% more elapsed time).
The previously conflated disabled-strikethrough case is
22.1% slower,
while enabled table/inline formatting differs by
2.7%.

These are different workloads, not a parser speedup. Production parser source
remains unchanged from the [initial profiling](2026-09-11-table-profiling.md).

## Benchmark changes

Two fixtures are shared by the focused comparison, profiling runner, and native
five-parser harness. Both contain 80 tables, two columns, one header row, and one
body row per table (320 cells total):

| Workload | Fixture | Enabled extensions | Purpose |
| --- | --- | --- | --- |
| `tables-plain` | `benches/fixtures/tables-plain.md`, 4,560 bytes | Tables | Plain cell text; no inline syntax |
| `tables-inline` | `benches/fixtures/tables-inline.md`, 5,200 bytes | Tables, strikethrough, task lists | Strong emphasis and strikethrough both rendered |
| `tables-disabled-syntax` | Same inline fixture | Tables | Explicit negative-syntax diagnostic; `~~pending~~` remains literal |

Strong emphasis is CommonMark syntax. Task-list parsing is enabled in the shared
GFM-overlap preset but this fixture contains no tasks. All used syntax in the
first two cases is enabled. Input sizes/output sizes differ, so subtracting their
times is not an additive price for a feature.

The focused harness adds a `tables-only` configuration plus stable Criterion
identifiers `parity/tables-only/plain`, `parity/tables-inline/enabled`, and
`parity/tables-disabled-syntax/literal-strikethrough`. `profile_driver` can select
`tables-plain` and `tables-inline` with the corresponding configuration. Semantic
tests require the plain fixture to emit only table structure and ordinary text,
check actual rendered table/cell counts for both parsers, and verify active versus
literal strikethrough. This prevents unrelated syntax from silently returning.

Native cases are `tables/tables-plain`, `gfm_overlap/tables-inline`, and
`tables/tables-disabled-syntax`. The first two join its repeated headline set.
The older `tables/gfm-tables` identifier remains available for historical evidence.
The public README/homepage now label its existing figures
**“Tables + inactive strikethrough (historical)”** and disclose the mixed workload.
Those archived figures were regenerated with the corrected label; no measured
number was edited or replaced with a result from a different environment.

## Measurement and results

Apple M1 Pro, macOS 26.6.2, Rust 1.97.1 / LLVM 22.1.6, ARM64, System allocator,
opt-level 3, fat LTO, one codegen unit, repository `apple-m1`/NEON flags.
Pulldown-cmark 0.13.4 uses its default Cargo features. Trusted syntax settings
match between the parsers; heading IDs, literal autolinks, and filtering are off.

Nine 250 ms windows per case/lifecycle, with rotating/reversing parser order,
32 warmup renders and a clock check every 16 renders. The raw corpus metadata
records the shared fixture hashes and exact production-source hashes. No competing
builds or other benchmark jobs were started by this task during timing/sampling.

Microseconds per render, median of nine window means. Both principal columns
create fresh parser state and reuse output. The final column additionally retains
Ferromark parser scratch and is a diagnostic, not a matched pulldown API.

| Case | Ferromark | pulldown-cmark | Difference | Retained Renderer |
| --- | ---: | ---: | ---: | ---: |
| Plain tables, 80 × 2 columns | 34.25 | 30.97 | +10.6% | 33.45 |
| Tables with enabled emphasis/strikethrough | 45.69 | 44.50 | +2.7% | 45.07 |
| Tables with literal inactive strikethrough | 44.82 | 36.72 | +22.1% | 42.13 |
| 404 short plain cells | 26.41 | 21.97 | +20.2% | 25.61 |
| 404 long plain cells | 33.47 | 64.79 | -48.3% | 32.24 |
| 8 columns × 101 rows | 49.03 | 40.10 | +22.3% | 47.56 |
| 9 columns × 101 rows | 57.67 | 44.27 | +30.3% | 56.25 |

Owned output confirms the direction: plain tables take
34.88 versus 32.63 µs,
and enabled inline formatting takes 47.52 versus
45.62 µs. These are local diagnostic windows, not the
five-parser publication protocol or a general ranking across document populations.

The plain 80-table input and the 404-cell input have different document structures.
Their 10.6% and
20.2% gaps are not
contradictory. The retained long-cell control still favors Ferromark, while wide
short-cell tables expose a larger gap. This keeps the structural table issue
visible after removing the misleading inactive-syntax contribution.

## Fresh CPU profiles and allocation counts

Six separate five-second macOS `sample` captures profiled both parsers on all
three primary cases, using the uninstrumented executable. Leaf/top-of-stack
attribution for selected Ferromark symbols:

| Symbol | Plain | Inline enabled | Inactive syntax |
| --- | ---: | ---: | ---: |
| `render_block_event` | 19.7% | 16.8% | 19.4% |
| `split_table_cells` | 13.5% | 8.2% | 9.1% |
| `InlineParser::parse_with_options_in_document` | 5.7% | 22.1% | 20.2% |
| `render_inline_content` | 5.4% | 7.2% | 6.5% |

The pure input takes the inline plain-text fast path **320/320** times; the other
two cases take it **160/320** times. All three emit 1,760 block events. Their inline
event counts are respectively 320, 720, and 480. The marked reduction in general
inline-parser samples is consistent with removing inline syntax from the table
baseline. Row splitting, table recognition, event handling, and rendering remain
significant in the plain-table profile.

The warmed Renderer allocates zero times in all three primary cases and in the
404-cell controls. Eight-column tables also allocate zero times; nine columns
still cause **103 allocations / 19,600 requested bytes** per document. This
confirms that the eight-cell local SmallVec threshold remains a distinct issue.
Counting ran in a separate instrumented build; requested bytes are cumulative,
not peak memory.

Inlining attributes HTML writing to enclosing symbols such as `render_block_event`.
The percentages do not isolate dispatch cost, cannot be added as promised speedups,
and are not precise independent phase timers. The next parser experiment should
use the plain short-cell cases to evaluate cell scanning/event handoffs, retaining
the enabled and inactive cases as separate controls.

## Validation and reproduction

- All seven focused cases passed normalized HTML comparison; each Ferromark
  lifecycle was also checked byte-for-byte.
- The rebuilt native five-parser harness admitted **45/45** workloads; all three
  new cases had matching normalized HTML across all five parsers. This was a
  verification-only run, not a new five-parser timing publication.
- The focused semantic/matrix tests, full `cargo test --locked --all-features`,
  root and focused-harness Clippy (all targets/features, warnings denied), and
  formatting checks passed.
- Native Python verification/publication tests, README/CONTRIBUTING/profiling
  contracts, generated-publication checks, and homepage benchmark checks passed.

Regular benchmark and profiling selectors:

```sh
cargo bench --locked --manifest-path benchmarks/pulldown-comparison/Cargo.toml --bench comparison -- 'parity/tables-'
cargo run --release --locked --manifest-path benchmarks/pulldown-comparison/Cargo.toml --bin profile_driver -- --config tables-only --corpus tables-plain --parser ferromark --seconds 5
```

Reproduce the uninstrumented measurements and separate counts/CPU profiles from
this report (Cargo dependencies must be cached for offline builds):

```sh
python3 docs/reports/2026-09-11-table-benchmark-isolation/run.py time --out target/table-isolation-repro
python3 docs/reports/2026-09-11-table-benchmark-isolation/run.py sample --out target/table-isolation-repro
python3 docs/reports/2026-09-11-table-benchmark-isolation/run.py counts --out target/table-isolation-repro
python3 docs/reports/2026-09-11-table-profiling/summarize_profiles.py target/table-isolation-repro
```

The runner reuses the original diagnosis's probe implementation and materializes
copies of the committed fixtures outside the timer. Use a new output directory;
CPU sampling requires macOS process inspection permission. The general-purpose
`profile_driver` exposes allocation instrumentation and is not the executable
used for the uninstrumented timing table above.

Evidence: [metadata](2026-09-11-table-benchmark-isolation/raw/metadata.json),
[raw timing windows](2026-09-11-table-benchmark-isolation/raw/timings.jsonl),
[summary](2026-09-11-table-benchmark-isolation/raw/summary.json),
[counts](2026-09-11-table-benchmark-isolation/raw/counts.json),
[CPU summary](2026-09-11-table-benchmark-isolation/raw/profile-summary.json),
[CPU capture index](2026-09-11-table-benchmark-isolation/raw/profiles.json),
[focused HTML checks](2026-09-11-table-benchmark-isolation/raw/verification.json),
[native HTML verification](2026-09-11-table-benchmark-isolation/native-verification/verification.json),
and [full test log](2026-09-11-table-benchmark-isolation/cargo-test.log).
