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

## Addendum: follow-up pass (same day)

Two of the follow-up candidates were implemented after the first pass merged:

1. **Footnote allocation pressure** — footnote *reference* resolution no
   longer allocates: `FootnoteStore::get_index_bytes` normalizes the label
   into a stack buffer instead of building a `String` per `[^…]` candidate.
   The footnote section renderer now reuses the parent context's idle
   `InlineParser` (via `mem::swap`) for reference footnotes and the parent's
   parser + event buffer for inline footnotes, instead of allocating a fresh
   parser (10+ vectors) per footnote. On `mixed-extended-5k` with every
   extension on: allocations drop from 483 to 251 per document (108 → 60
   KiB), instructions drop 5.5%, and wall-clock on footnote-heavy input
   improves ~6% (paired-ratio median vs merged main).

2. **Autolink candidate scan: attempted, reverted.** Folding the three
   single-needle passes of `has_autolink_candidates` into one
   `memchr3(b'@', b':', b'.')` pass (or a `memchr2` + dot-loop hybrid) cuts
   the flag's instruction cost from +6.7% to +4.4% — but measured
   consistently *slower* in paired wall-clock runs on x86-64 AVX2. Multi-
   needle memchr runs at a lower per-byte rate than single-needle, and `.`
   is frequent in prose, so the combined scan stops constantly while the
   `@`/`:` passes it replaces run essentially stop-free. The three-pass
   structure is now documented in the code as deliberate. The remaining
   real opportunity is folding the candidate check into the SIMD specials
   scan itself (one shared pass over the text), which is a larger refactor
   of `collect_marks` — and any such change should be validated on Apple
   Silicon (the published benchmark platform), where relative memchr pass
   costs differ.

3. **Heading-ID dedup arena** (third pass, same day) — the remaining
   allocation source flagged in finding 1 is gone: `HeadingIdTracker` no
   longer clones each new base slug into a `HashMap<Vec<u8>, usize>` key.
   Base slugs are appended to a single arena buffer and the dedup map is
   keyed by the slug's 64-bit Fx hash, storing arena ranges; hash collisions
   (astronomically rare, but handled) share a map entry and are resolved by
   comparing arena bytes. On `commonmark-50k` (247 headings) with default
   options: allocations drop from 363 to 121 blocks per document (−67%),
   instructions drop 2.6%, and the `heading_ids` flag overhead falls from
   +8.6% to +5.6% over `Options::commonmark()`. Criterion medians on the
   container: `parsing/commonmark_5k` −5.4%, `parsing/commonmark_50k` −8.4%
   (allocator round-trips cost disproportionately more wall-clock than
   instructions). Rendered HTML is byte-identical across the fixture ×
   preset matrix including slug-collision edge cases. The flag's remaining
   cost is the slug scan and heading-content buffering itself, which is
   proportional to heading text and has no obvious fat left.

4. **HTML escaping scanned every text segment twice via SIMD dispatch**
   (fourth pass, same day) — `first_text_escape` ran two full memchr passes
   per text segment (`memchr3(<,>,&)` plus `memchr(")`), and segments
   between escapes are short: median 17 bytes, >95% under 64 bytes on the
   prose fixtures. At those lengths memchr's per-call cost is dominated by
   its ifunc trampoline (indirect call through an `AtomicPtr`) and SIMD
   setup, paid twice per segment — measured ~60 instructions per call
   against a ~20-instruction payload. Both escape scanners now use a
   table-driven scalar scan for segments ≤64 bytes (reusing the existing
   escape LUTs) and, on the rare long segment, bound the second pass by the
   first `memchr3` hit so it never scans past the earliest match.
   Instructions: −3.7% (commonmark-50k/gfm), −4.2% (commonmark-5k/default),
   −6.6% (tables-5k/gfm). Wall-clock across the whole parsing suite: −6.5%
   to −12.6% with no regressions — the win exceeds the instruction delta
   because the removed indirect calls also cost branch-prediction and
   pipeline stalls that callgrind does not model. Output byte-identical
   across the fixture × preset matrix.

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
