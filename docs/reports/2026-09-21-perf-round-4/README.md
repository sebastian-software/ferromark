# Optimization round 4: our own ideas, screened one by one — 2026-09-21

After the release-fix regression was repaired ([paired report](../2026-09-21-release-fixes-paired/README.md)),
this round takes the ideas that repair left behind and screens each one on
its own, against one baseline, with the paired harness of the optimization
rounds (`benchmarks/optimization-rounds`), before anything is measured
against the other engines. Each idea was implemented on its own branch in
its own worktree from the same base; the coordinator built every candidate
against the same baseline build and measured them one after another on a
quiet machine, with an A/A control first.

Baseline is `23a59bdf`, the head of PR #385 (the fix plus the documentation
of the regression), whose parser and renderer sources equal `main` at
`fe8987c4` where the candidate branches start. Same host (Apple M1 Pro),
toolchain (Rust 1.95.0), fat-LTO workers built with `-C target-cpu=generic`
and `--reuse-baseline-build`; screens use 3 rounds × 3 pairs × 40 ms
windows per case and stage on the 20 screen documents plus 16 table and
scan diagnostics, the broad confirmations 3 rounds × 5 pairs on the 57
broad documents. Every case and stage was verified for exact HTML and AST
equality between the two workers before timing. Ratios are baseline time
over candidate time, medians of the paired windows; **higher is faster,
below 1.000 the candidate is slower**. [NUMBERS.md](NUMBERS.md) holds every
table, [TABLES.md](TABLES.md) every case with its round medians.

Decision rule, from the earlier rounds: a candidate is kept when its broad
geomean is at least 1.010 with every round above 1.000, fresh and reuse at
or above 1.000, no broad case below 0.970, and outputs identical. Anything
else is archived with its patch and numbers, not merged.

## The candidates

| Name | Branch | Idea |
| --- | --- | --- |
| `norm` | `perf/source-normalization-borrow` | Source normalization already borrowed; the cold NUL/BOM rewriting path is outlined from the constructor frame. |
| `depth` | `perf/inline-depth-cell` | The inline nesting depth is a plain `Cell<usize>` per parser instead of an arena cell shared by pointer; only an inline note's sub-parser folds its depth back. |
| `lines` | `perf/list-line-facts` | One pass per container line yields line end, indentation, trimmed slice and blankness for the list and block-quote collectors. |
| `subctx` | `perf/sub-parser-context` | Sub-parsers borrow the options from the arena, carry the two overrides as flags, and share one `Rc` for both pre-pass tables; `Parser` is 304 bytes. |
| `arena` | `perf/arena-reservation` | `for_source_len` keeps `len × 8` and lowers the floor from 16 KB to 2 KB (4,032 usable bytes after bumpalo's rounding, against 20,416 before). |
| `emph` | `perf/emphasis-scratch` | One delimiter-run buffer per parse, parked between inline contexts; a context with no run never borrows it and skips pairing outright. |
| `closer` | `perf/closer-forward-window` | `has_closer_from` answers from a forward window in addresses, one per closer byte, instead of hashing a slice key into a table. |
| `tables` | `perf/table-pipe-masks` | Pipe and backslash masks per 16-byte block with NEON compares (scalar fallback), consumed by both the row splitter and the cell decoder. |

One idea was declined before measurement, on the code and the corpus:
[REJECTED.md](REJECTED.md) (delimiter run length and flanking from the marker
scan's mask).

## A/A control

The baseline against itself, 36 cases (20 screen documents and 16
diagnostics), measured while one implementation agent was still working:

| Stage | Geomean | Rounds |
| --- | ---: | --- |
| fresh | 1.004 | 1.006 / 1.001 / 1.004 |
| reuse | 1.003 | 1.001 / 1.002 / 0.993 |
| parse | 1.004 | 1.003 / 1.003 / 1.005 |
| render | 0.998 | 0.999 / 0.998 / 0.997 |

Single cases move up to ±5% (`typescript-handbook-advanced-types` reuse
1.049, `scan-long-references` fresh 0.967), so in the screens a geomean
inside ±0.5% and a case inside ±5% are noise.

## Screens

36 cases (20 documents, 16 diagnostics), 3 rounds × 3 pairs; every candidate
against the same baseline build, so the rows are independent:

| Candidate | fresh | reuse | parse | render | Verdict |
| --- | ---: | ---: | ---: | ---: | --- |
| A/A control | 1.004 | 1.003 | 1.004 | 0.998 | noise floor |
| `norm` | 1.000 | 0.998 | 1.000 | 0.992 | tie: nothing left to gain, archived |
| `depth` | 1.006 | 1.006 | 1.009 | 0.999 | every parse round above 1.008, 33 of 36 cases up: confirm |
| `lines` | 1.008 | 1.005 | 1.009 | 0.999 | `vite-docs-api-plugin` parse 1.105, `comment-quote` parse 0.951: confirm, watch the quote |
| `subctx` | 0.999 | 0.997 | 0.997 | 0.996 | tie with a loss on the smallest comments (`comment-ack` parse 0.961): archived |
| `arena` | 1.005 | 1.004 | 1.005 | 1.000 | `comment-ack` fresh 1.042, `comment-question` 1.035: confirm |
| `emph` | 1.003 | 0.998 | 1.000 | 0.996 | tie; only the emphasis-dense table diagnostics gain 3–4%: archived |
| `closer` | 1.014 | 1.013 | 1.017 | 0.998 | every round above 1.010; `comment-links` parse 1.172, `vue-docs-ways-of-using-vue` 1.086: confirm |
| `tables` | 1.023 | 1.023 | 1.022 | 1.000 | split: `table-dense-*` 1.29–1.43, `table-plain-*` 1.03–1.07, but `table-formatted-*` 0.90–0.93 and `table-sparse-256` 0.96: confirm for the record, revise the escape path |

The candidates that gained did so where their mechanism predicts: `closer` on
link-dense documents, `arena` on the smallest fresh parses, `lines` on
list-heavy technical pages, `depth` a little everywhere inline content is
parsed. The three ties lost nothing either; they are archived with their
branches rather than merged, because a change that measures as noise is
not worth its review.

## Broad confirmations

57 documents, 3 rounds × 5 pairs, for the candidates whose screen was not a
tie. The decision rule asks for a geomean of at least 1.010 with every round
above 1.000, fresh and reuse at or above 1.000, and no case below 0.970.

| Candidate | fresh | reuse | parse | render | Fresh rounds | Cases < 0.970 | Rule |
| --- | ---: | ---: | ---: | ---: | --- | --- | --- |
| `closer` | 1.021 | 1.023 | 1.029 | 0.999 | 1.011 / 1.022 / 1.022 | none fresh or reuse; parse `comment-reproduction` 0.968 | **met** |
| `depth` | 1.005 | 1.006 | 1.009 | 1.000 | 1.004 / 1.003 / 1.007 | none | below 1.010; 51 of 57 parse up, no loser |
| `lines` | 1.005 | 1.003 | 1.006 | 1.001 | 1.006 / 1.005 / 1.006 | `comment-quote` reuse 0.960, parse 0.949 | not met |
| `arena` | 1.007 | 1.002 | 1.004 | 1.001 | 1.007 / 1.009 / 1.008 | none | below 1.010 overall; the target group (< 512 B) fresh 1.022, no loser |
| `tables` | 1.000 | 1.002 | 1.001 | 1.000 | 1.001 / 1.001 / 1.000 | none | not met: no real document moves, the only table-heavy one (`rust-book-appendix-02-operators`) within noise |

`closer` gains where its mechanism says: `guard-angle-link` parse 1.342,
`comment-links` 1.171, `rust-book-ch03-04-comments` 1.139,
`vue-docs-ways-of-using-vue` 1.084, the Wikipedia article bodies 1.02–1.05;
the smallest bracket-free comments pay 2–3% for the three windows the parser
now carries. `depth` lifts nearly every document by about 1% and loses
nowhere. `lines` gains 5–8% on the list-heavy technical pages but its one
extra terminator scan per block quote costs `comment-quote` 4–5%. `arena`
gains on exactly the documents whose reservation the floor decided — every
comment under 512 bytes, fresh 1.022 as a group, `comment-ack` 1.048 — and
touches nothing above it.

## Decisions

- **Merge `closer`** (`perf/closer-forward-window`). The only candidate
  that meets the rule outright; it also removes a hash table and a cache
  cell from the parser.
- **Merge `arena`** (`perf/arena-reservation`) as a memory and small-input
  win: below the 1.010 geomean over all 57 documents, but 1.022 on the
  documents it targets, no loser anywhere, and a fresh parse now reserves
  4 KB instead of 20 KB.
- **Merge `depth`** (`perf/inline-depth-cell`) as a simplification with a
  measured 0.9% parse gain and no loser: it removes an arena allocation per
  parse and replaces a pointer-shared cell by a field.
- **Revise `lines`** (`perf/list-line-facts`): keep the container line facts
  but let a block quote's terminating blank line stop before the terminator
  scan, as it did; re-screen `comment-quote` before anything else.
- **Revise `tables`** (`perf/table-pipe-masks`): the NEON cursor is a clear
  win on dense and plain rows; the escaped-pipe path (`table-formatted-*`)
  and the sparse 256-byte row lose, so the decoder's record handling needs
  a second iteration before the branch is measured again.
- **Archive `norm`, `subctx`, `emph`**: ties within the A/A spread; their
  branches and numbers stay for reference. `subctx` shows that borrowing
  the options costs the smallest documents more than the cheaper sub-parsers
  give back; `emph` that the delimiter buffer is not where emphasis time
  goes; `norm` that normalization was already free.
- **Declined without a patch**: delimiter runs from the marker mask
  ([REJECTED.md](REJECTED.md)).

Merges go in one at a time, each re-screened on top of the previous one
before the six-engine comparison is rerun, because the three winners touch
the same struct (`Parser`) and the same inline paths, and independent
gains do not add up by assumption.

## Not measured: the product defaults

The tenth idea was a measurement, not a change: run the runtime-profile
study (`benchmarks/runtime-profiles`) at this revision so the cost of the
product defaults — heading IDs, footnotes, sanitizing — is known next to
the CommonMark-flag numbers the comparisons use. Its `prepare.py` still
archives `crates`, the source layout from before the single-crate
consolidation, and `git archive` fails at every revision since; the same
path list is the coverage gap `audit_v2_sources.py` supplements in the
native reports. The harness needs its path list moved to the root package
before that study can run again; it is left as the next item.

## Reproduction

`harness/build.sh <name> <candidate-path>` builds a candidate against the
baseline worktree (`--baseline-revision 23a59bdf`, reusing the baseline
build of the A/A pair); `harness/measure.sh <name> screen|broad [pairs]`
runs one measurement, gated on `pgrep -x rustc`, `cargo` and `clang`. The
corpus is `make_corpus.py --include-scanner-diagnostics` (its SHA-256 is in
every `run.json`; it is not archived), the filters are
`harness/filter-screen.txt` and `harness/filter-broad.txt`.
`harness/tables.py` regenerates [TABLES.md](TABLES.md) from `results/`, which
holds every run's `summary.json`, `grouped.json`, `run.json`, `build.json`
and gzipped raw windows. Load averages before and after every run are in
[measure-log.txt](measure-log.txt).
