# Profiling report: hotspots across option combinations (2026-07-25)

Instruction-level profiling pass over the current feature set, including the
recently added definition lists, merged table cells, and table column widths.
Deterministic counts come from `valgrind --tool=callgrind`; allocation
behavior from `valgrind --tool=dhat`; wall-clock sanity checks from the new
`profile_harness` example (medians over repeated runs). Wall-clock numbers in
this report were taken on a shared 4-core x86-64 container and are only
meaningful relative to each other, not comparable to the published Apple
Silicon numbers.

## Method

```bash
cargo build --profile release-debug --example profile_harness
valgrind --tool=callgrind \
  target/release-debug/examples/profile_harness benches/fixtures/tables-5k.md gfm 50
callgrind_annotate callgrind.out.<pid>
```

Fixtures: the existing `commonmark-{5k,20k,50k}` and `tables-5k`, plus three
new ones added by this pass — `deflists-5k` (definition lists),
`tables-merged-5k` (merged cells + column-width hints), and
`mixed-extended-5k` (every extension in one document, incl. footnotes, math,
callouts, front matter). Presets cover `minimal`, `commonmark`, `gfm`,
`default`, `all`, per-feature combos, and single-flag bisection presets
(`cm+<flag>`).

## Per-flag cost on a text-heavy document

Callgrind totals for `commonmark-50k` (10 iterations), one flag enabled on
top of `Options::commonmark()`, measured before any of the fixes below:

| Flag | Instruction overhead |
|------|---------------------:|
| `heading_ids` | **+11.8%** |
| `autolink_literals` | +6.7% |
| `tables` | +6.1% (the fixture contains real tables) |
| `highlight` | +3.6% (switches the SIMD specials scan variant) |
| `superscript`+`subscript` | +3.1% |
| `strikethrough`, `task_lists`, `math`, `callouts`, `disallowed_raw_html` | ~0% |

Notable: `strikethrough`, `math`, and `callouts` are effectively free on
documents that do not use them. `heading_ids` — enabled in
`Options::default()` — was the most expensive single flag.

## Hotspots found and fixed in this pass

### 1. Table cell splitting scanned byte-by-byte (largest win)

`split_table_cells` + `advance_table_cell_scan` accounted for ~20% of all
instructions on table-heavy input. The scanner made one non-inlined call per
content byte (152k calls per 50 renders of a 4.5 KB document, ~22
instructions per byte). Both the GFM splitter and the merged-cell splitter
now jump between structurally relevant bytes with `memchr3(b'|', b'\\',
b'`')`; plain cell content is skipped at SIMD speed.

### 2. Table cell content was copied per cell

`CellState::add_text` copied every cell into a scratch buffer byte-by-byte
(with a capacity check per byte) purely to rewrite `\|` escapes. Cells
almost never contain backslashes: the renderer now keeps a borrowed range
into the input for the single-text/no-escape case and renders it zero-copy;
the escape-rewriting copy only happens when a backslash is actually present.

### 3. `heading_ids` slug generation

`HeadingIdTracker::make_id` validated UTF-8 three times per heading,
allocated a `String` per heading for the dedup map, and classified slug
bytes through a chain of branches. The slug loop is now driven by a 256-byte
lookup table, the dedup map is keyed by raw bytes (`Vec<u8>`), and UTF-8 is
validated once on return. Flag cost dropped from +11.8% to +8.5%.

## Results (callgrind, identical HTML output verified)

| Fixture / preset | Before | After | Δ |
|------------------|-------:|------:|--:|
| tables-5k / default | 16.33M | 14.52M | **−11.1%** |
| tables-merged-5k / gfm-merged | 33.17M | 29.40M | **−11.4%** |
| tables-5k / gfm | 43.36M | 39.09M | **−9.9%** |
| commonmark-50k / default | 82.62M | 78.67M | **−4.8%** |
| mixed-extended-5k / all | 36.21M | 34.87M | −3.7% |
| commonmark-5k / gfm | 22.10M | 21.48M | −2.8% |
| deflists-5k / gfm-deflists | 22.23M | 22.27M | ±0% |

Wall-clock medians on the container agreed in direction (tables-5k +15%,
commonmark-5k +7–10%, merged +5–12%, deflists ±0%). The full test suite and
CommonMark conformance corpus pass unchanged; rendered HTML is byte-identical
across the fixture × preset matrix.

## Findings without code changes (candidates for follow-up)

1. **Allocation pressure under `all`** — on `mixed-extended-5k` with every
   extension on, ~10% of instructions are inside `malloc`/`free` (~483
   allocations per 5 KB document, DHAT). Main sources: per-heading dedup-map
   key allocation (reduced but not eliminated by this pass),
   `normalize_footnote_label` allocating per footnote definition *and*
   per reference, a fresh `InlineParser` per footnote-section render, and
   growth of per-parse scratch vectors. An arena or reuse of the
   footnote-section parser would remove most of this.
2. **`autolink_literals` candidate scan** — +6.7% on text without a single
   autolink, spent in extra `memchr` passes over all inline text
   (`has_autolink_candidates`). Folding the candidate check into the
   existing SIMD specials scan would make the flag near-free on
   non-autolink text.
3. **Definition lists are cheap** — the new feature adds no measurable
   overhead when unused and no dedicated hotspot when heavily used; no
   action needed.
4. **`merged_table_cells` / `table_column_widths`** — +2–4% instructions on
   table-heavy input, ~0 when unused. Early wall-clock outliers that
   suggested `table_column_widths` was expensive on non-table documents did
   not reproduce under callgrind; they were scheduler noise on the shared
   container.
5. **`render_block_event` dispatch** — ~1.5% of instructions go to the
   per-event prologue (destructuring ~20 `&mut` state fields per call).
   Splitting hot state (text/cell paths) from cold state would shrink this,
   but the payoff is small relative to the churn.

## Reproducing

```bash
# throughput matrix
cargo build --profile release-debug --example profile_harness
target/release-debug/examples/profile_harness benches/fixtures/<fixture>.md <preset> 2000

# instruction profile for one combination
valgrind --tool=callgrind --callgrind-out-file=out.cg \
  target/release-debug/examples/profile_harness benches/fixtures/tables-5k.md default 50
callgrind_annotate --auto=yes out.cg | less

# allocation profile
valgrind --tool=dhat \
  target/release-debug/examples/profile_harness benches/fixtures/mixed-extended-5k.md all 20

# criterion lens on the new fixtures
cargo bench --bench parsing -- extensions
```
