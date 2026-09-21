# The release fixes, paired against the `7c887a2b` core — 2026-09-21

The [native comparison of the release head](../2026-09-21-native-release/README.md)
found v2 at `bffc89f6` 7–12% behind the position `7c887a2b` had held against
every engine, on the same 57 frozen documents with identical HTML. Two runs of
the six-engine harness cannot say which commits cost what, so this report
measures the range with the paired harness of the optimization rounds
(`benchmarks/optimization-rounds`): baseline and candidate workers built the
same way, measured in alternating windows of one process, output verified
equal before timing. It attributes the loss, and it measures the fix
([decision record](../../decisions/2026-09-21-lazy-tracker-cost.md)).

Baseline is `c4af9525` — `main` right after the segmented definition pass
merged, whose parser and renderer sources are byte-identical to the
`7c887a2b` core the preceding native comparison measured. Candidates are
first-parent revisions of `main` between it and `bffc89f6`, then `bffc89f6`
with each stage of the fix applied. Same host (Apple M1 Pro), toolchain
(Rust 1.95.0), fat-LTO workers built with `-C target-cpu=generic`; the broad
runs use 3 rounds × 5 pairs × 40 ms windows per case and stage, the 16-document
screens 3 rounds × 3 pairs. Every case and stage was verified for exact HTML
and AST equality between the two workers before timing. Ratios are baseline
time over candidate time, medians of the paired windows; **higher is faster,
below 1.000 the candidate is slower**. The [numbers](NUMBERS.md) hold every
table; the [per-case tables](TABLES.md) every case with its round medians.

## Attribution

The A/A control — the baseline against itself, under the same load — stays
within 0.999–1.008 on every stage over the 16 documents, single cases within
±4%, so anything below about 0.97 per case or 0.99 per geomean is real.

The 16 screen documents are the twelve the native comparison had moved most,
plus four that had not moved, as controls. Each screen is cumulative, so the
difference between two consecutive rows is what the commits between them
cost:

| Candidate | fresh | reuse | parse | render |
| --- | ---: | ---: | ---: | ---: |
| A/A control | 1.007 | 1.004 | 0.999 | 1.008 |
| `2887b2bd` #369 bound inline bracket nesting | 1.003 | 1.003 | 0.996 | 1.006 |
| `e8439fd8` #372 bound emphasis nesting | 0.982 | 0.979 | 0.985 | 0.959 |
| `ca82ec2f` #374 linear nested link probing | 0.977 | 0.969 | 0.975 | 0.954 |
| `c84743cb` #376/#377 base URL, MDX closer memo | 0.974 | 0.969 | 0.970 | 0.957 |
| `1037fb5d` #380 renderer fixes | 0.976 | 0.968 | 0.973 | 0.958 |
| `e532dcf4` #382 tabs, HTML closers, MDX flow | 0.977 | 0.974 | 0.977 | 0.961 |
| `8d957e30` #383 laziness, container and inline cost bounds | **0.869** | **0.855** | **0.813** | 0.997 |
| `bffc89f6` #384 linear math/MDX/definition-list scans | 0.856 | 0.841 | 0.796 | 0.998 |

#369, #376, #377, #380 and #382 cost nothing. #372 and #374 together cost
about 2.5% of parse time on these documents. **#383 cost 17%** of parse time
and #384 a further 2%. Over the 57 broad documents `bffc89f6` measures
0.929× fresh, 0.922× reuse, 0.894× parse and 1.000× render against the
baseline, 56 of 57 documents slower in parsing; the loss is a parser loss.

Per document, #383 took `comment-checklist` (four task items) from 0.975 to
0.626, `comment-quote` to 0.685, `comment-review-long` to 0.715,
`typescript-handbook-the-handbook` to 0.720 and `vite-docs-api-plugin` to
0.754 — every document with list items or block quotes — while
`rust-book-appendix-02-operators` (no containers) stayed at 1.000. The render
column of the #372–#382 screens sits at 0.955–0.961 and returns to 1.000 at
#383; #372 changes no renderer code, so that is fat-LTO layout, as the
render column of the broad run (1.000) and the A/A control confirm.

## What #383 and the review had done

- The laziness tracker `lazy_paragraph::OpenParagraph` classified every
  line a block quote or list item collected — thematic break, ATX, setext,
  fence, HTML block start, type-7 HTML, list-marker probes — on top of the
  sub-parse that reads the same bytes again. Its type-7 check also handed
  lines to the inline tag scanner without a leading `<`, which read `ab>`
  as a tag and ended a paragraph the sub-parser keeps open.
- The review's memo tables — the last `]`/`>`/`}` per slice, link-probe and
  bracket-match maps, MDX, brace, math and wiki tables and gap cells — sat
  inline in `Parser`, which grew from 272 to 736 bytes between `7c887a2b`
  and `bffc89f6`. Every parse zeroed and moved it, and every container
  builds a sub-parser.
- The linear link probe (#374) asks two hashed tables per `[`: whether a
  `]` follows in the slice, and whether a recording walk already answered
  the bracket.

The ablation `abl-emph-screen` — the first fix stage with `emphasis.rs` as
of #382 — measures the delimiter linked list of #383 at about 1% on the 16
documents, within the spread; it is not part of the loss and was kept.

## The fix, stage by stage

| Candidate, 16 screen documents | fresh | reuse | parse | render |
| --- | ---: | ---: | ---: | ---: |
| `e532dcf4` #382, the last revision before the loss | 0.977 | 0.974 | 0.977 | 0.961 |
| `bffc89f6`, the release head | 0.856 | 0.841 | 0.796 | 0.998 |
| + tracker classifiers gated on the first byte | 0.937 | 0.929 | 0.904 | 1.001 |
| + tracker catching up on demand | 0.972 | 0.965 | 0.950 | 1.000 |
| + memo tables allocated on first use | 0.983 | 0.983 | 0.971 | 0.997 |
| + closer front cache, no-allocation bracket reads | 0.987 | 0.984 | 0.975 | 0.998 |

| Candidate, 57 broad documents | fresh | reuse | parse | render |
| --- | ---: | ---: | ---: | ---: |
| `bffc89f6`, the release head | 0.929 | 0.922 | 0.894 | 1.000 |
| + first-byte gate | 0.959 | 0.953 | 0.934 | 1.001 |
| + on-demand catch-up | 0.972 | 0.968 | 0.952 | 1.000 |
| + lazy memos | 0.979 | 0.974 | 0.962 | 0.998 |
| + closer front cache (the fix) | 0.982 | 0.978 | 0.967 | 1.000 |

With the fix, the release head measures **0.982× fresh, 0.978× reuse,
0.967× parse and 1.000× render** against the `7c887a2b` core over the 57
documents, from 0.929×/0.922×/0.894×/1.000× before it; no document is below
0.95× fresh. What remains is the cost the review's nesting cap (#372) and
linear link probe (#374) carry by design — a depth cell threaded through
every inline context and, per bracket, the cell check and the null check
that replaced two hashed lookups — and it sits where brackets are dense:
the Wikipedia article bodies parse at 0.93×, `wiki-tea-first-paragraph` at
0.93×, and the smallest comments, where a few nanoseconds of fixed cost are
several percent of a 150 ns parse, between 0.91× and 0.94×. Those mechanisms
are recorded here as measured, not removed: the decision records that
introduced them measured on a sandbox that could not resolve changes under
20%, which is why a few percent went unseen.

## Reproduction

`harness/build.sh` builds the baseline and every attribution candidate from
detached worktrees (`git worktree add --detach`) with
`benchmarks/optimization-rounds/prepare.py --baseline-revision c4af9525`;
`harness/measure.sh` runs the A/A control, the broad confirmation and the
screens back to back, gated on `pgrep -x rustc`, `cargo` and `clang`;
`harness/measure-fix.sh <name>` builds the working tree against the same
baseline and runs its screen and broad measurement; `harness/measure-ablation.sh`
does the same for a prepared source directory. The corpus is
`make_corpus.py --include-scanner-diagnostics` (its SHA-256 is in every
`run.json`; it is not archived), the filters are `harness/filter-*.txt`,
and `harness/fix1-first-byte-gate.patch` is the first stage as measured.
`harness/numbers.py` regenerates [NUMBERS.md](NUMBERS.md) and
`harness/tables.py` [TABLES.md](TABLES.md) from `results/`, which holds
every run's `summary.json`, `grouped.json`, `run.json`, `build.json` and
gzipped raw windows. Load averages before and after every run are in
[measure-log.txt](measure-log.txt).
