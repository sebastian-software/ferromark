# Optimization round 5: from a profile to four merges — 2026-09-23

Round 4 screened ideas the release-fix repair had left behind. This round
started from a measurement instead: a sample-based profile of release 2.0.1
over the 57 broad documents ([PROFILES.md](PROFILES.md)). Parsing took about
70% of a reuse iteration, and the profile named four places that looked
removable: block trimming, the root pre-scans, the GFM autolink pre-flight
and the tag filter. A fifth change came from reading the Node binding. Each
became one change on its own branch, implemented by an agent in its own
worktree from `cb352020`; the coordinator built every candidate against the
same baseline build and measured them one after another with the paired
harness of the optimization rounds (`benchmarks/optimization-rounds`), with
an A/A control first.

Baseline is `cb352020` (release 2.0.1, `main` when the round started). Same
host (Apple M1 Pro), toolchain (Rust 1.95.0), fat-LTO workers built with
`-C target-cpu=generic` and `--reuse-baseline-build`, and the same corpus as
round 4. Screens use 3 rounds × 3 pairs × 40 ms windows per case and stage on
the 20 screen documents plus 16 table and scan diagnostics, the broad
confirmations 3 rounds × 5 pairs on the 57 broad documents, the recheck
3 rounds × 7 pairs on four documents. Every case and stage was verified for
exact HTML and AST equality between the two workers before timing. Ratios are
baseline time over candidate time, medians of the paired windows; **higher is
faster, below 1.000 the candidate is slower**. [NUMBERS.md](NUMBERS.md) holds
every stage table, [TABLES.md](TABLES.md) every case with its round medians,
[REJECTED.md](REJECTED.md) the closed candidate.

Decision rule, unchanged from the earlier rounds: a candidate is kept when its
broad geomean is at least 1.010 with every round above 1.000, fresh and reuse
at or above 1.000, no broad case below 0.970, and outputs identical.

## Where the round started

The [first profile](PROFILES.md#what-the-first-profile-pointed-at) (fat LTO
with symbols, `sample` at 1 ms) attributed, at `cb352020`:

| Item | Share | Where |
| --- | ---: | --- |
| Unicode `str::trim` | 6.1% of parse, plus 1.1% `trim_start` | table cells (the largest caller), paragraphs, list items |
| GFM autolink pre-flight | about 7–9% of parse on `gfm` documents | a second `memchr2` + `memmem` pass per block |
| Root NUL `memchr` + `]:` `memmem` | about 5–6% of parse | two whole-document scans that find nothing in most documents |
| `escape_into` | 36% of render | measured as a floor in [round 2](../2026-09-15-arm-round-2/README.md) |
| Heading ids | 11.5% of render | slugs and the id map |
| `tagfilter::matching_tag` | 3.7% of render | every `<` in raw HTML tried nine names |

Probing the trim finding turned up two bugs before any speed question: the
parser trimmed block content with `str::trim`, which strips every Unicode
`White_Space` character where CommonMark and GFM remove only spaces, tabs and
line endings (`to_html("a\u{a0}")` lost the no-break space that cmark keeps),
and paragraphs passed the untrimmed line start as the inline offset, shifting
every inline span of an indented paragraph. That candidate therefore became a
fix with a [decision record](../../decisions/2026-09-23-commonmark-whitespace-trim.md),
measured like the others. Heading ids were left alone: #408–#410 were
reworking that code during the round (unique ids, prefixes, level offsets)
and merged after the round's measurements.

## The candidates

| Name | Pull request | Branch | Idea |
| --- | --- | --- | --- |
| `trim` | #414 | `fix/commonmark-whitespace-trim` | Block-boundary trimming over ASCII whitespace only (space, tab, LF, VT, FF, CR), byte-level; inline offsets start at the trimmed content; `clippy.toml` disallows the `str` trims in the parser. |
| `prescan` | #413 | `perf/fused-root-prescan` | One NEON root scan finds the first NUL and the first `]:` together; the pre-pass starts from the handed-in closer. First commit `35a6db46`. |
| `prescan2` | #413 | same | Revision `c6e83512`: bodies of 16 bytes or more end on one overlapping vector instead of a byte loop. |
| `tagfilter` | #411 | `perf/tagfilter-first-byte` | A 256-entry table of the disallowed names' first bytes, built at compile time, gates `matching_tag`. |
| `node` | #412 | `perf/node-renderer-output-buffer` | The reusable Node `Renderer.toHtml` renders with `render_borrowed`, so its output buffer stays warm between calls. |
| `autolink` | #415 | `perf/autolink-preflight-marker-scan` | The autolink pre-flight's facts collected during the inline marker scan instead of a second pass. |

`node` changes only the Node binding and was measured at the Node level; the
Rust candidates went through the paired harness.

## A/A control

The baseline against itself, 36 cases (20 screen documents and 16
diagnostics):

| Stage | Geomean | Rounds |
| --- | ---: | --- |
| fresh | 0.9995 | 1.000 / 1.002 / 0.999 |
| reuse | 1.0015 | 1.002 / 1.002 / 1.000 |
| parse | 1.0005 | 0.999 / 0.999 / 1.001 |
| render | 0.9987 | 1.004 / 1.002 / 0.999 |

Single cases move up to about ±3% (`typescript-handbook-advanced-types`
reuse 1.031, `legacy-docs-migration-0-2` parse 0.980,
`wiki-volcano-article-body` render 0.981). The last document is the round's
recurring outlier: it also shows render 0.939 under `prescan2`, a parser
change, and reuse 0.944 under `tagfilter`, whose code never runs on this
`commonmark` document. So a screen geomean inside ±0.2% is noise, and a
single case inside ±3% (±6% for `wiki-volcano-article-body`) is too.

## Screens

36 cases, 3 rounds × 3 pairs, every candidate against the same baseline
build, so the rows are independent:

| Candidate | fresh | reuse | parse | render | Verdict |
| --- | ---: | ---: | ---: | ---: | --- |
| A/A control | 0.9995 | 1.0015 | 1.0005 | 0.9987 | noise floor |
| `trim` | 1.042 | 1.049 | 1.063 | 1.000 | every round above 1.040; `rust-book-appendix-02-operators` parse 1.341, `table-plain-256` 1.329: confirm |
| `prescan` | 1.004 | 1.003 | 1.006 | 1.000 | `wiki-chess-plain-prose` parse 1.092, but `comment-question` parse 0.921, `comment-ack` 0.957: confirm, watch the small comments |
| `tagfilter` | 1.001 | 0.999 | 1.001 | 1.001 | tie; the screen holds none of the raw-HTML documents it later gained on: confirm on the broad set |
| `autolink` | 0.991 | 0.993 | 0.988 | 1.001 | loss; `table-formatted-*` parse 0.87–0.90, technical docs 0.92–0.96: confirm to see how far it goes |

## Broad confirmations

57 documents, 3 rounds × 5 pairs:

| Candidate | fresh | reuse | parse | render | Fresh rounds | Cases < 0.970 | Rule |
| --- | ---: | ---: | ---: | ---: | --- | --- | --- |
| `trim` | 1.031 | 1.037 | 1.051 | 1.000 | 1.032 / 1.032 / 1.030 | none | **met**; reuse faster on 57 of 57 |
| `prescan` | 1.010 | 1.013 | 1.021 | 1.000 | 1.008 / 1.006 / 1.012 | `comment-question` fresh 0.965, reuse 0.934, parse 0.920; `comment-ack` parse 0.960; `comment-inline-code` reuse 0.966, parse 0.965 | not met |
| `tagfilter` | 1.000 | 1.001 | 0.999 | 1.005 | 1.002 / 0.998 / 0.998 | `vite-docs-features` fresh 0.958 | not met; render rounds 1.004 / 1.005 / 1.007 |
| `autolink` | 0.980 | 0.979 | 0.968 | 1.002 | 0.978 / 0.982 / 0.983 | 30 documents in parse, down to 0.905 | not met: a loss |

`trim` gains most where the old code decoded most: `trim_cell` trimmed every
table cell twice with UTF-8 decoding, so `rust-book-appendix-02-operators`
parses at 1.354 and `comment-table` at 1.316; the `gfm` documents, which hold
the tables, parse at 1.062 against 1.025 for the `commonmark` ones. Its
lowest fresh case is 0.997.

`tagfilter` is a render change and gains where raw HTML is dense:
`typescript-handbook-compiler-options` render 1.172,
`rust-book-ch17-00-async-await` 1.056, `vue-docs-slots` 1.026, no render case
below 0.970. `vite-docs-features` fresh 0.958 is a stage the change does not
touch; see the recheck below.

## Revising `prescan`

The first commit of #413 lost exactly on the inputs where the old path was
cheapest. `comment-question` (160 bytes), `comment-ack` (37 bytes) and
`comment-inline-code` (285 bytes) hold no `[`, no `]:` and no NUL, so on
`cb352020` they cost two `memchr` calls, each finishing on one overlapping
16-byte vector, and the `[` probe returned before any `memmem`. The fused
scan ran its NEON loop and then left the last 1–16 bytes to a byte loop with
two compares and a branch per byte: 16, 5 and 13 bytes for those three
inputs. The revision (`c6e83512`) ends every body of 16 bytes or more on one
vector over its last 16 bytes, re-reading bytes the loop already cleared,
with a zero lane standing in for the lookahead after the final byte. The
loops over the rest of the body are unchanged.

| Run | fresh | reuse | parse | render |
| --- | ---: | ---: | ---: | ---: |
| `prescan`, broad | 1.010 | 1.013 | 1.021 | 1.000 |
| `prescan2`, screen | 1.006 | 1.004 | 1.009 | 0.999 |
| `prescan2`, broad | **1.014** | **1.013** | **1.021** | 0.999 |

The revised broad run meets the rule: fresh rounds 1.015 / 1.016 / 1.014, no
fresh or reuse case below 0.970. `comment-question` is back at parse 1.002,
`comment-ack` at 1.000; `comment-inline-code` stays at parse 0.969 (fresh
0.977, reuse 0.982), the pattern round 4 accepted for the closer window.
Three of the four plain-prose documents parse at 1.08–1.12
(`wiki-tea-plain-prose` 1.123): long documents without a reference
definition, where the `]:` `memmem` used to run to the end.

## Recheck of `tagfilter`

The four documents that moved in the broad run, 3 rounds × 7 pairs, with an
A/A control on the same four right before:

| Document | A/A render | `tagfilter` render | A/A fresh | `tagfilter` fresh |
| --- | ---: | ---: | ---: | ---: |
| `typescript-handbook-compiler-options` | 0.991 | 1.179 | 0.986 | 1.116 |
| `rust-book-ch17-00-async-await` | 0.988 | 1.053 | 0.996 | 0.997 |
| `vue-docs-slots` | 0.993 | 1.023 | 0.996 | 1.005 |
| `vite-docs-features` | 1.011 | 0.997 | 1.007 | 0.995 |

The render gains reproduce against an A/A spread of 0.986–1.011, and
`vite-docs-features` fresh is 0.995: its broad 0.958 was fat-LTO placement
noise on a document known for it.

## Node level: #412

`Renderer.toHtml` used `HtmlRenderer::render`, which moves the output buffer
out, so every call on a reused renderer grew a fresh buffer that N-API copied
and freed. The paired Rust harness does not apply (the core is unchanged), so
`harness/node-bench.mjs` alternates two local `release-node` addons without
PGO — `cb352020` and the branch — in separate child processes over 10 rounds,
order swapped every round, rendering the 57 broad documents at equal byte
volume per document:

| Lane | Median ratio | Rounds |
| --- | ---: | --- |
| reused `Renderer.toHtml` | **1.027** | 1.024 1.021 1.028 1.000 1.027 0.998 1.031 1.032 1.039 0.982 |
| one-shot `toHtml`, unchanged code, control | 0.997 | 0.997 0.999 1.000 1.004 0.991 0.949 0.996 0.996 0.977 1.019 |

Seven of ten rounds put the reused lane at 1.021–1.039. Of the other three,
rounds 5 and 9 (numbered from 0, as in the raw output) also moved the
unchanged one-shot lane (0.949 and 1.019), so they were disturbed runs rather
than a property of the change. The script
reports the upper median, the sixth of the ten sorted ratios. Raw output:
[`results/node-412.txt`](results/node-412.txt).

## Decisions

- **Merge `trim` (#414, `7eda5a4b`)** as a correctness fix that also meets the
  rule outright: non-ASCII Unicode whitespace is content at block boundaries,
  inline spans start at the trimmed content, and the broad set parses at
  1.051. No snapshot or conformance baseline changed; the behavior
  change is recorded in the [decision](../../decisions/2026-09-23-commonmark-whitespace-trim.md).
- **Merge `prescan2` (#413, `c12698bd`)**: meets the rule after the revision;
  other targets keep the two separate searches, since the fused scan is only
  measured on aarch64.
- **Merge `tagfilter` (#411, `68332286`)** as a targeted render win for GFM
  documents with raw HTML: below the 1.010 geomean over all 57 documents, as
  the arena floor was in round 4, but render 1.02–1.18 on the documents it
  targets, reproduced in the recheck, no render case below 0.970, and output
  unchanged by construction (the previous matcher is the test oracle).
- **Merge `node` (#412, `1a4b6064`)**: reused-renderer lane 1.027 with the
  one-shot control at 0.997; it also makes the class match what the PGO
  trainer already modeled.
- **Close `autolink` (#415)** as a measured loss: [REJECTED.md](REJECTED.md).

## Merges, re-screened one on top of the other

The three Rust changes touch different code (block trimming, the root
pre-scan, the renderer's tag filter), but independent gains do not add up by
assumption, so each was screened again on a state that already held the
previous ones (36 cases, 3 rounds × 3 pairs):

| Re-screen | Baseline | fresh | reuse | parse | render |
| --- | --- | ---: | ---: | ---: | ---: |
| `prescan2` solo | `cb352020` | 1.006 | 1.004 | 1.009 | 0.999 |
| `stack1`: #413 on top of #414 | `85c0031e` (`cb352020` + #414) | 1.007 | 1.008 | 1.011 | 1.001 |
| `stack2`: #411 on top of both | `170dd4f2` (`cb352020` + #414 + #413) | 1.001 | 1.000 | 0.999 | 1.000 |

`stack1` reproduces the solo screen of #413 (every parse round 1.007–1.010,
`comment-question` parse 1.012). `stack2` is a tie, as the solo screen of #411
was: the screen set holds none of the raw-HTML documents it gains on, and its
one case below 0.970 (`comment-links` parse 0.967) is a stage #411 does not
touch. The stacking order is not the merge order — `main` took #411 first,
then #412, #409, #414 and #413 — which does not matter for changes that share
no code.

## Cumulative effect

`cb352020` against the state with all three Rust changes (the `stack2`
candidate), 57 documents, 3 rounds × 5 pairs:

| Stage | Geomean | Rounds | Faster | Lowest case |
| --- | ---: | --- | ---: | --- |
| fresh | **1.049** | 1.048 / 1.056 / 1.049 | 57 of 57 | `comment-question` 1.003 |
| reuse | **1.055** | 1.055 / 1.057 / 1.056 | 56 of 57 | `typescript-handbook-advanced-types` 0.997 |
| parse | **1.080** | 1.071 / 1.079 / 1.079 | 57 of 57 | `wiki-tea-lead` 1.019 |
| render | 1.005 | 1.003 / 1.008 / 1.007 | 35 of 57 | `typescript-handbook-typescript-5-0` 0.985 |

Every document parses faster. The solo parse gains multiply to about 1.073
(`trim` 1.051 × `prescan2` 1.021); the combination measured 1.080, so the
changes do not eat into each other. By category, plain prose parses at 1.156,
reference pages at 1.137, READMEs at 1.110, comments at 1.088, technical
documentation at 1.069 and the encyclopedia pages at 1.046; the largest
single gains are `rust-book-appendix-02-operators` 1.358, `comment-table`
1.323 and `wiki-chess-plain-prose` 1.206. Render gains concentrate on the
documents with raw HTML (`typescript-handbook-compiler-options` 1.176,
`rust-book-ch17-00-async-await` 1.047, `vue-docs-slots` 1.020).

This measures the round's three Rust changes, not release 2.1.0: `060b02d2`
also holds the heading-id features #408–#410, which were not part of the
round and were not measured with the paired harness.

## Follow-up commits

Two commits landed on the round's pull requests after their measurements.

- **#414's `20c8ebdf`** keeps tab columns that a list marker expands inside a
  container out of inline span offsets (list items with a tab after the
  marker; HTML unaffected). It touches list parsing, so it was measured
  afterwards on its own: #414's measured commit `85c0031e` against
  `20c8ebdf`, 36 screen cases, 3 rounds × 3 pairs, built like the stacked
  states. **A tie**: fresh 0.998, reuse 0.999, parse 0.998, render 0.999, no
  case below 0.970 (lowest `wiki-tea-article-body` parse 0.972), the
  list-heavy `comment-checklist` and `vite-docs-api-plugin` at parse 0.989
  and 0.987. The run took place later in the day under a much higher load
  (one-minute load 32.8 at its start, 10.5 at its end), so its per-case
  spread is wider than the round's A/A control; the geomeans stay within
  0.3% of 1.000. `results/fix414-screen`, [NUMBERS.md](NUMBERS.md#follow-up-re-measurement-36-cases-3-rounds--3-pairs-414s-measured-commit-85c0031e-against-its-follow-up-20c8ebdf).
- **#413's `0610f606`** restores the previous pre-pass search order on
  targets without the fused scan. It changes nothing on aarch64, so the Apple
  Silicon harness cannot see it; it can be measured with the x86-64 CI
  workflow proposed in #419.

## After the round: remaining hot spots

The profiling driver rebuilt against 2.1.0 (`060b02d2`) completed 753 parse
sweeps in 14 s against 675 at `cb352020`, about 12% more, and render about 1%;
the two runs were hours apart under different load, so this is indicative
only. Unicode trimming is gone from the profile (6.1% → 0.2%), the `]:`
`memmem` no longer appears under the root phase, and the table cell closure
fell from 5.1% to 3.0%. What is left at the top ([PROFILES.md](PROFILES.md#after-the-round-at-060b02d2)):

- **The autolink pre-flight, about 10% of parse**, still a second pass per
  block; the fused variant lost ([REJECTED.md](REJECTED.md)).
- **Tables, 10.1%** and **lists, 11.8%** of parse, inclusive.
- **Heading ids, 11.5% of render**: `slugify_heading_into` 5.7% and the new
  `HeadingIdPlanner::plan_into` 4.4% from the unique-id and prefix work.
- **The tag filter, still 4.3% of render**: six first letters cover the nine
  names, so common tags such as `<td`, `<span` and `<p` pass the gate and walk
  all nine names.
- **The escaper, 36% of render**, a measured floor.

## How the round was run

- **Agents implemented, the coordinator measured.** Each implementation agent
  worked in its own worktree and waited for
  `pgrep -f optimization-rounds/run.py` to clear before every `cargo`
  command, so no compile overlapped a timing window. The Node comparison ran
  through a wrapper named [`optimization-rounds/run.py`](harness/optimization-rounds/run.py)
  for the same reason, and `harness/measure.sh` in turn waited for `rustc`,
  `cargo` and `clang` to finish before each run.
- **Candidates came from `git archive` of the pushed branches**, not from the
  agents' worktrees (`harness/extract.py` unpacks them), so a later push or
  an uncommitted edit could not leak into a measurement, and the measured
  tree is identified by its commit.
- **Stacked states were built without touching any worktree**, with
  `git merge-tree --write-tree` and `git commit-tree`: `85c0031e` is #414's
  measured commit (parent `cb352020`), `170dd4f2` a local merge of it with
  #413's `c6e83512`, and the `stack2` candidate adds #411's `d2a8b189` the
  same way; `harness/build-stack.sh` builds a candidate against such a state.
- The round's runs took place between 08:02 and 09:38 UTC. The one-minute
  load average ranged from 2.1 to 9.5 (round 4: up to 5.7) and was highest
  around the `tagfilter` broad run and during the stacked runs. The
  follow-up re-measurement of `20c8ebdf` ran at 16:08 UTC under a load of
  32.8 to 10.5. [measure-log.txt](measure-log.txt) records the load before
  and after every run of this report; runs of later work that shared the log
  are left out.

## Reproduction

The scripts in [`harness/`](harness/) take the round's scratch directory from
`ROUND5_WORK` and a ferromark checkout from `FERROMARK_REPO` (both were
session paths during the round). `harness/build.sh <name> <candidate-path>`
builds a candidate against `src-base` (`git archive cb352020`) with
`--baseline-revision cb352020`, reusing the A/A baseline build;
`harness/build-stack.sh` builds against a stacked state;
`harness/measure.sh <name> screen|broad|recheck [pairs]` runs one
measurement; `queue-screens.sh`, `queue-broad.sh` and `queue-stack.sh` are
the three sequences as run. The corpus is
`make_corpus.py --include-scanner-diagnostics` (SHA-256 `31583750…` in every
`run.json`, the same as round 4; it is not archived), the filters are
`harness/filter-screen.txt`, `filter-broad.txt` and `filter-recheck.txt`.

[`results/`](results/) holds every run's `summary.json`, `grouped.json`,
`run.json`, `build.json`, gzipped raw windows and `.summary` line, plus
`cases.json` (category, profile and size of every measured case, taken from
the corpus) and `node-412.txt`. `harness/rule.py <run>...` prints the
decision-rule view of a run, `harness/numbers.py` and `harness/tables.py`
regenerate [NUMBERS.md](NUMBERS.md) and [TABLES.md](TABLES.md), and
`harness/percase.py` compares runs side by side. The Node comparison is
`node harness/node-bench.mjs <corpus.json> <addonA.node> <addonB.node> 10`.
The profiling driver, its sampling script and the call-graph aggregator are
in [`harness/profile/`](harness/profile/), the four call graphs in
[`profiles/`](profiles/). The closed candidate's patch is
[`patches/autolink.patch`](patches/autolink.patch); the merged changes are in
`main`'s history. [SHA256SUMS](SHA256SUMS) covers every file of the report.
