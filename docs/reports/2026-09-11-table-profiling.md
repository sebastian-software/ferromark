# Table profiling against pulldown-cmark — 2026-09-11

The current code reproduces the table slowdown, but it has three distinct causes:
**disabled inline syntax still enters the general inline pipeline; short cells
pay repeated parsing/rendering overhead; rows wider than eight columns spill a
fresh cell buffer to the heap.** The published tables-only input amplifies the
first issue because it contains literal `~~pending~~` despite strikethrough being
disabled. Long plain cells favor Ferromark on this machine.

This is a diagnosis, not an optimization or a replacement for public benchmark
figures. No production code, dependency, README figure, or architecture decision
was changed. The previous [GFM experiments](2026-09-10-gfm-optimizations.md) were
reviewed; their rejected row prescan and text-rendering shortcuts were not repeated.

## Method and reproduction

- Baseline: `c5ca4ac9f18e7173ebf44a75dd26a660686f287c` (initially clean checkout).
- Apple M1 Pro, macOS 26.6.2, Rust 1.97.1 / LLVM 22.1.6, ARM64.
- Both parsers in one optimized executable: opt-level 3, fat LTO, one codegen unit,
  aborting panics, debug symbols, repository `apple-m1`/NEON target flags, System
  allocator. pulldown-cmark is locked to **0.13.4**, default Cargo features
  (`getopts`, `html`; no `simd` feature). This differs from the public
  nightly/generic/shared-mimalloc harness; do not mix their absolute numbers.
- CommonMark plus tables, trusted HTML, no heading IDs, autolinks, filtering,
  merged cells, width hints, or other extensions. The separately labeled GFM
  overlap enables strikethrough and task lists in both parsers.
- Nine 250 ms windows per lane/case, clock checked every 16 renders, 32 warmup
  renders per process; rotating/reversing parser order and rotating case order.
  Values below are medians of window means. Timings, counting, and sampling ran
  separately, without competing builds or benchmarks started by this task.
- Fresh parser/reused output is the principal comparison. Owned-output lanes
  create and drop the result each iteration; each API keeps its own allocation
  strategy. Retained Renderer is a Ferromark-only diagnostic, not a matched API
  comparison. Options and input generation stay outside the timer.
- All 15 primary and six supplemental input/configuration combinations passed
  the existing native harness's normalized HTML comparison, including alignment
  and content. Ferromark's owned, reused-output, and retained-Renderer outputs
  were also checked byte-for-byte. Original HTML and input are retained.
- Allocation/pipeline counts are one warmed render in a **separate instrumented
  build**. Requested bytes are cumulative, not peak/live memory. Seven independent
  five-second macOS `sample` captures use the uninstrumented timing executable.

Run from the repository root (dependencies must already be available to Cargo's
offline cache; CPU sampling needs macOS process inspection permission):

```sh
python3 docs/reports/2026-09-11-table-profiling/run.py time --out target/table-profile-repro
python3 docs/reports/2026-09-11-table-profiling/run.py sample --out target/table-profile-repro
python3 docs/reports/2026-09-11-table-profiling/run.py counts --out target/table-profile-repro
python3 docs/reports/2026-09-11-table-profiling/supplement.py target/table-profile-repro/supplemental
python3 docs/reports/2026-09-11-table-profiling/summarize_profiles.py target/table-profile-repro
```

The reproduction probe received formatting and a narrow lint allowance after
measurement; the original measured source remains archived verbatim.
Use a new output directory for each run. The supplemental script reuses the exact
uninstrumented timing binary and the separate counting binary; it supplies altered
fixtures through the probe's existing file-input path. The saved
[lockfile](2026-09-11-table-profiling/Cargo.lock),
[measured probe source](2026-09-11-table-profiling/raw/probe-measured.rs), and
[environment/binary hashes](2026-09-11-table-profiling/raw/metadata.json) identify the run.

## Measured behavior

Microseconds per complete render. Difference is Ferromark elapsed time relative
to pulldown-cmark, using fresh parser state and reused output for both.
The last column retains Ferromark parser scratch as well.

| Input | Ferromark | pulldown-cmark | Difference | Retained Renderer |
| --- | ---: | ---: | ---: | ---: |
| Published table input | 43.98 | 36.31 | +21.1% | 40.95 |
| Published input, GFM overlap | 45.52 | 43.65 | +4.3% | 43.61 |
| Existing tables-5k fixture | 33.86 | 30.89 | +9.6% | 32.45 |
| 404 plain cells, 4 bytes/cell | 25.91 | 21.76 | +19.1% | 25.25 |
| 404 plain cells, 128 bytes/cell | 32.26 | 64.21 | -49.8% | 31.84 |
| 404 bold cells | 53.15 | 49.93 | +6.5% | 51.85 |
| 404 cells with escaped pipes | 31.81 | 27.56 | +15.4% | 30.73 |
| 8 columns × 101 rows | 48.19 | 39.15 | +23.1% | 47.04 |
| 9 columns × 101 rows | 56.46 | 43.40 | +30.1% | 55.21 |
| 4 columns × 1,001 rows | 250.93 | 202.15 | +24.1% | 249.26 |
| 404 plain paragraphs, 128 bytes each | 27.95 | 72.98 | -61.7% | 27.54 |

For owned output, the published table case is
**44.49 vs 37.42 µs**
(+18.9%);
the existing table fixture is 34.92 vs
32.15 µs. Reusing output therefore does not create the
reported ranking. The GFM-overlap gap is substantially smaller than tables-only.

The `short` and `long` inputs have identical row/column counts: increasing cell
content from four to 128 bytes adds only
6.35 µs to Ferromark versus
42.46 µs to pulldown-cmark. This supports a
substantial fixed cost per cell in Ferromark alongside an advantage on long text.
The plain-paragraph control also favors Ferromark, so the long-text advantage is
not specific to tables. The separate `control` case in raw data uses the existing
mixed CommonMark fixture, which itself contains tables; it is not a table-free control.

The 100-to-1,000-body-row experiment increases Ferromark time by
9.68× for approximately ten times
as many cells. No superlinear table scaling is demonstrated within this range.

## 1. Disabled tilde syntax explains much of the published gap

The published 5,200-byte input repeats 80 small tables whose body is:

```markdown
| **table** | ~~pending~~ done |
```

In tables-only mode, `~~pending~~` must remain literal. Ferromark's
[`BASE` byte set](../../src/inline/simd.rs) nevertheless always includes `~`
(and `$`). [`parse_with_options_in_document`](../../src/inline/mod.rs) therefore
collects marks and enters general resolution/event emission, although the
strikethrough resolver itself correctly stays disabled. In pulldown-cmark's
pinned `src/firstpass.rs`, `special_bytes` (lines 2353–2384) adds `~` only when
strikethrough or subscript is enabled.

Replacing only `~` with `x` preserves input length, table structure, and output
length. Both outputs still agree after normalization. This is a controlled input
experiment, not a patch or an assertion that the text is unchanged.

| Variant, tables-only | Ferromark | pulldown-cmark |
| --- | ---: | ---: |
| Original published input | 44.07 | 36.28 |
| Only `~` replaced with `x` | 38.31 | 36.20 |
| Only `*` replaced with `x` | 35.96 | 31.30 |
| Both replaced | 32.57 | 31.24 |
| 404 cells containing literal `~~data~~` | 41.67 | 22.57 |
| 404 cells containing `xxdataxx` | 25.28 | 22.75 |

On the published input, the replacement removes
**5.76 µs** from Ferromark and
0.08 µs from pulldown-cmark.
Ferromark still performs 320 inline parses and emits 480 inline events, but its
plain-text fast paths increase **160 → 240**. All retained-Renderer variants
allocate zero times. This specifically identifies avoidable inline work rather
than allocation, fewer cells, or fewer output events.

The isolated 404-cell `~~data~~` test goes from zero fast paths to 404 when changed
to `xxdataxx`; both variants emit the same number of block/inline events and
requested output bytes. The counter change and timing difference support an
option-aware inline-special scan as the first bounded optimization experiment.
They do **not** prove a future patch will recover the entire measured difference.
The same question for `$` is a source-level follow-up; it was not measured here.

## 2. The ordinary cell pipeline remains a structural cost

Even without any inline markup, every nonempty cell goes through:

```text
split_table_cells + trim
  → TableCellStart / Text(range) / TableCellEnd block events
  → CellState::add_text (another backslash check)
  → finish / render_inline_content
  → inline-special scan / InlineEvent::Text
  → general inline event rendering / escaping / HTML writes
```

[`split_table_cells`](../../src/block/parser.rs) first records trimmed source
ranges. [`CellState`](../../src/lib.rs) normally borrows those ranges rather than
copying their text. At cell end, `render_inline_content` clears/reserves event
scratch, invokes the inline parser, and renders its events. The ordinary
plain-text fast path already exists; it still pays these surrounding steps.

The 404-cell plain fixture produces **1,420 block events, 404 inline parses, and
404 inline events**; every inline parse takes its fast path. Its warmed Renderer
allocates **zero** times, yet takes 25.25 µs versus
21.76 µs for pulldown-cmark with fresh parser state.

CPU leaf-symbol attribution, with 4,128 samples on the published input and 4,131
on the retained-Renderer plain-cell case:

| Ferromark symbol | Published input | 404 plain cells |
| --- | ---: | ---: |
| `render_block_event` | 17.9% | 25.7% |
| `parse_with_options_in_document` | 21.4% | 6.6% |
| `render_inline_content` | 6.0% | 11.5% |
| `split_table_cells` | 9.0% | 12.9% |
| `CellState::add_text` | 3.2% | 6.2% |
| `emit_table_row` | 2.5% | 7.4% |

These are self/top-of-stack percentages, not inclusive phase totals. Inlined
rendering and HTML writing can be attributed to their enclosing symbol;
`render_block_event` is **not** a measurement of dispatch alone. Percentages
cannot be added up into a promised speedup. Full stack trees and all seven
profile summaries are retained.

The separate public BlockParser-only probe takes
6.39 µs on short cells and
13.66 µs on the published input. These are diagnostic
bounds, not directly subtractable phase measurements: it reuses its event buffer
and has a different caller/lifecycle. The `pull-events` probe performs both block
and inline parsing, so comparing it directly to `ferro-block` would be misleading.

Pulldown-cmark's pinned `parse_table_row_inner` calls `parse_line` in active table
mode; that scan recognizes both cell boundaries and inline candidates and stores
nodes in its internal tree. Ferromark separates those scans and event handoffs.
The source and profiles support investigating fewer per-cell handoffs and setup
steps. They do not establish that replacing the architecture or adding SIMD is
necessary. The earlier generic text shortcuts already had measured regressions.

## 3. Nine columns introduce per-row allocations

`split_table_cells` creates a fresh `SmallVec<[TableCell; 8]>` each time.
At nine cells it spills to the heap. Retaining the public Renderer cannot retain
this local buffer. Header/delimiter/alignment processing contributes additional
allocations around the body rows.

| Plain table, 100 body rows | Retained Renderer allocations | Requested bytes | Time |
| --- | ---: | ---: | ---: |
| 8 columns | 0 | 0 | 47.04 µs |
| 9 columns | 103 | 19,600 | 55.21 µs |
| 16 columns | 103 | 19,600 | 89.69 µs |

Nine columns add 12.5% more cells but
17.4% elapsed time in
this experiment. The allocation discontinuity is proven; the precise avoidable
time requires a paired implementation experiment. This cannot explain the
published two-column case, whose retained Renderer allocates zero times.

For that published input, fresh parser/reused output uses 18 allocations plus
three reallocations in Ferromark, versus 85 plus eight in pulldown-cmark.
A blanket claim that Ferromark loses because it allocates more is contradicted
by these counts.

## Recommended follow-up and limits

1. Test option-aware handling of inactive tilde syntax, including subscript and
   mixed active/inactive marks. Preserve exact HTML and measure both extension
   enabled/disabled paths. This is the most concrete isolated cause found here.
2. For ordinary GFM cells, prototype fewer scans/event handoffs using the existing
   borrowed range and shared escaping rules. Include ragged/empty cells,
   backslash/code-pipe behavior, renderer reuse, and unrelated URL-heavy workloads.
3. Test retained row scratch or direct row-cell consumption above eight columns.
   Measure narrow and wide rows separately; simply increasing inline capacity
   moves the threshold and may increase stack/copy costs.

This task deliberately stops at profiling. No speedup, regression fix, full-GFM
ranking, or x86 performance claim is made. The nine short-window repetitions on
one ARM64 host are diagnostic, not the public publication protocol. Raw windows
include min/max spread; CPU sampling has inlining and sampling noise. Full GFM
with literal autolinks/security defaults was not compared because pulldown-cmark
does not perform that same work in this harness.

Evidence: [primary timings](2026-09-11-table-profiling/raw/summary.json),
[raw windows](2026-09-11-table-profiling/raw/timings.jsonl),
[allocation/pipeline counts](2026-09-11-table-profiling/raw/counts.json),
[HTML checks](2026-09-11-table-profiling/raw/verification.json),
[CPU profiles](2026-09-11-table-profiling/raw/profiles.json),
[leaf-symbol summary](2026-09-11-table-profiling/raw/profile-summary.json),
[tilde experiments](2026-09-11-table-profiling/supplemental/summary.json), and
[tilde counters](2026-09-11-table-profiling/supplemental/counts.json).

Validation completed: all 1,107 recorded timing windows meet the requested duration;
all summary medians reproduce from raw windows; 123 counting runs and seven CPU
captures are present; measured source, executable, and lock hashes agree; report
links resolve. Both diagnostic build configurations pass Clippy with warnings
denied, and repository formatting plus the reproduction probe's formatting pass.
No production regression test was added because no implementation was changed.
