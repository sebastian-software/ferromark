# Apple Silicon iteration round 3 — 2026-09-16

A third round on the open items of the [second round](../2026-09-15-arm-round-2/README.md):
the line-end rescans in block parsing, the fence-run searches, the owned
strings in `HtmlRendererOptions`, per-block fence metadata handling, the
per-paragraph GFM autolink pre-flight, and — as a build-level experiment
rather than a code change — profile-guided optimization. Baseline is the
branch tip after round 2, `f216b8da`. Same host (Apple M1 Pro), toolchain
(Rust 1.95), worker and paired harness; five pairs per round throughout.

Ratios are baseline time over candidate time, geometric means of per-document
medians; higher is faster.

## Results

Promoted set against `f216b8da`, 3 rounds × 5 pairs × 40 ms; every case and
stage verified for exact HTML and AST `Debug` equality before timing.

| Input set | N | Fresh | Reuse | Parse | Render |
| --- | ---: | ---: | ---: | ---: | ---: |
| Original broad documents | 57 | **1.142×** | **1.027×** | **1.036×** | 1.009× |
| Authored diagnostics | 45 | 1.173× | 1.002× | 1.014× | 0.987× |

Round geometric means agree to within 0.003 on every stage. On the broad set
all 57 documents parse faster, 54 of 57 reuse faster, 53 of 57 fresh faster;
render is split 25/32. Fresh gains scale inversely with size (up to 512 B
1.501×, 513–2,048 B 1.127×, larger than 64 KB 1.018×) because the options
change removes per-renderer allocations; parse gains are flat across sizes
(1.025–1.047×) because the line-scan work is per line.

| Category (broad) | N | Fresh | Reuse | Parse | Render |
| --- | ---: | ---: | ---: | ---: | ---: |
| Comments | 12 | 1.463× | 1.031× | 1.031× | 1.015× |
| Technical docs | 22 | 1.070× | 1.034× | 1.042× | 1.024× |
| Reference | 4 | 1.043× | 1.040× | 1.070× | 1.007× |
| Readme | 2 | 1.041× | 1.019× | 1.021× | 1.003× |
| Encyclopedia | 12 | 1.057× | 1.006× | 1.022× | 0.980× |
| Plain prose | 4 | 1.043× | 1.029× | 1.045× | 1.001× |

Measured losses, all retained in [TABLES.md](TABLES.md): the three largest
encyclopedia article bodies render at 0.939–0.951× and `rust-book-appendix-02-operators`
at 0.963×; `wiki-tea-article-body` is 0.977× fresh and 0.976× reuse. Among
the diagnostics `autolink-closers-512`/`-2048` render and reuse at 0.90–0.91×,
`autolink-unicode-64` render 0.918×, `scan-dense-escapes` render 0.908×.

Two checks separate code layout from real cost. First, each promoted
component was screened alone on the affected documents
([`results/*-attrib`](results/)): the options change and the fence fast path
render the encyclopedia bodies at 1.00–1.04×, and the parser-only line-scan
commits move their render time by 0.97–0.99× although they touch no render
code — the loss appears only in the combined binary. Second, the same sources
were built as a thin-LTO pair ([`results/combo3-thin-control`](results/combo3-thin-control/)):
under thin LTO `scan-dense-escapes` renders at 1.002× (fat 0.907×),
`rust-book-appendix-02-operators` at 1.001× (fat 0.968×),
`wiki-tea-article-body` at 1.002× (fat 0.968×) and 1.016× reuse (fat 0.989×),
`wiki-volcano-article-body` at 0.984× / 1.032× reuse, and `wiki-chess-article-body`
at 0.975× (fat 0.937×). Those losses are fat-LTO code placement, the class of
variance the first round documented at ±5–9% and the PGO section below
removes wholesale.

The `autolink-closers` diagnostics are different: 0.90–0.91× under both LTO
modes, and attributable to the options change alone. Profiles of both
builds ([`profiles/`](profiles/)) show the same 80–86% of render time in
`trim_trailing_punct` — the unbalanced-closer trimming loop this synthetic
input exists to stress — with its lazily computed bracket counts
(`Option::get_or_insert_with`) taking 8.7% instead of 4.1%: the compiler
inlines that untouched function differently once the pattern slice type
changes in the same module. A URL followed by hundreds of unbalanced
closers does not occur in the corpus; the loss is recorded rather than
chased.

Correctness: exact HTML and full AST `Debug` (including spans) agreed between
baseline and candidate for every case and stage; every timing window's
iteration checksum was verified; the branch passes `cargo fmt`, 896 workspace
tests, Clippy with warnings denied, and the benchmark builds. The Node
binding's JS tests (28) passed against the options change.


## Profile-guided optimization (build experiment, no code change)

The worker was built three times with `-Cprofile-generate`, trained by running
its own timed loop over a training set in all four stages (60 ms per case and
stage), merged with `llvm-profdata`, and rebuilt with `-Cprofile-use` on top of
the usual fat-LTO, generic-AArch64 release settings. The scripts are in
[`harness/pgo_build.sh`](harness/pgo_build.sh).

| Training set | Measured on | N | Fresh | Reuse | Parse | Render |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| 57 broad + 45 diagnostics (in-sample) | 19-case quick set | 19 | 1.199× | 1.233× | 1.261× | 1.152× |
| 45 authored diagnostics only | 57 broad documents (never seen) | 57 | 1.125× | 1.152× | 1.193× | 1.071× |
| 29 broad (every other document) + 45 diagnostics | the other 28 broad documents (never seen) | 28 | **1.204×** | **1.240×** | **1.255×** | **1.176×** |

The held-out split is the honest number: every one of the 28 unseen documents
is faster in every stage (worst fresh `comment-question` 1.069×, worst render
`typescript-handbook-the-handbook` 1.076×); encyclopedia articles parse 1.33×.
Training only on synthetic diagnostics already carries most of the gain but
leaves render losses on some reference pages (`typescript-handbook-compiler-options`
0.87×), so the training set should contain real documents of every category.

This is larger than everything the three code rounds delivered together, and
it is a release-pipeline decision rather than a commit: the native build
matrix produces eight targets, several of them cross-compiled, and a profile
has to be collected by running an instrumented binary. Two workable shapes:
collect the profile once per architecture on the runners that can execute the
binary (`darwin-arm64`, `darwin-x64`, `linux-x64-gnu`, `linux-arm64-gnu`,
both Windows targets) and reuse it for the matching cross targets, or ship
PGO only for the natively built targets first. `docs/releasing.md` and the
`native` job in `.github/workflows/ci.yml` are where it would go. The training
corpus would be the frozen benchmark corpus, which is already in the
repository.

## Code candidates

A/A control taken at the start of the measurement run, under the same
indexer load as the screens (`aa3-load`): fresh 1.001, reuse 1.000, parse
1.000, render 1.001.

| Candidate | Render | Fresh | Reuse | Parse | Outcome |
| --- | ---: | ---: | ---: | ---: | --- |
| H `HtmlRendererOptions` strings as `Cow<'static, str>` (breaking) | 1.000 | **1.112** | 1.004 | | promoted; `comment-ack` fresh 2.06×, `comment-review` 1.66× |
| I1 bare fence languages skip the metadata tokenizer | 1.008 | 1.010 | 1.008 | | |
| I1+I2 plain fence markup from merged literals, one reserve | **1.013** | 1.008 | 1.007 | | promoted; fence-heavy pages 1.03–1.08 |
| F1 list continuation and blank-line lookahead scan once | | 1.004 | 1.006 | 1.002 | |
| F1+F2 block dispatch and probe carry the line end | | 1.011 | 1.013 | 1.018 | |
| F1–F3 containers and leaves derive the next line start | | 1.017 | 1.019 | **1.028** | |
| F1–F4 definition lists and table metadata | | 1.017 | 1.020 | **1.029** | promoted; `typescript-handbook-compiler-options` parse 1.153, comments 1.01–1.08 |
| J3 reserve the merged length before coalescing text runs | | 1.000 | 1.001 | 0.997 | tie; archived |

Rounds agree to within 0.003 for every promoted candidate.

## Promoted commits

| Commit | Change |
| --- | --- |
| `87556c31` | List continuation and blank-line lookahead find each line end once; the marker line's length comes from the parsed item |
| `e3215b7c` | `skip_blank_lines` reports the first content byte; dispatch arms and `probe_line` carry the line end into the table guard and the paragraph loop |
| `7ff949b2` | Block quotes, HTML blocks, footnotes, indented code, fences, MDX children and the table header derive the next line start from the terminator they hold |
| `72fcb46e` | Definition-list term/body loops and table attribute probes stop rescanning lines |
| `8059f0fd` | A plain language token (`0x21..=0x7E` minus `{ [ : & < > " '`) is written directly; anything else takes the tokenizer |
| `1dcf3ea5` | The default fence route resolves the language first, writes merged literals and reserves once |
| `21e78746` | Decision record for the `Cow<'static, str>` option fields |
| `62cdcd01` | Breaking: option strings become `Cow<'static, str>`, the pattern list `Cow<'static, [Cow<'static, str>]>`; defaults and clones allocate nothing |
| `d3ec6461` | The matcher receives plain `&str` patterns resolved once per text node that can hold a match |

## Negative results

- **Fence-run finders (G).** The 11% `memmem::Finder::find` attribution on
  the fence-heavy Vite page turned out to be a shared symbol: the GFM autolink
  pre-flight holds 9.8–12.5% of it and the `]:` pre-pass scan 4.1–4.5%; the
  fence-run search itself is 3.1–3.5%. Instrumented counters showed no
  double search on that document and not one rejected closing candidate
  anywhere in the corpus; the searches average 121 bytes, so the cost is the
  finder's per-call setup. A `memchr`-driven single-byte scan with run
  verification measured worse in the profile (`find_fenced_close` 4.9% →
  6.4% on the most fence-dense page). Patch archived in [`rejected/`](rejected/).
- **Document-level autolink gate (J).** Scanning the whole source once for
  `@`, `://` and `www.` so needle-free documents skip the per-block
  pre-flight. Correct (a strict superset, differential-tested over the spec
  fixtures and 1,500 generated documents), but the pre-scan reads bytes the
  per-block scans never touch, so it gains 3–5% only on short needle-free
  comments and loses 2–3% on documents whose first needle sits late
  (`legacy-contributing`, `comment-review-long`). A `memchr3` over `@`, `:`
  and `w` was far worse because `w` is a prose letter. Reverted; patch archived.
- **Reserved text coalescing (J3).** Output-neutral allocation-count
  reduction, measured as a tie.

## Structural finding: the definition pre-pass re-parses the document

`collect_definitions` runs a full second block parse (`ParsePhase::Definitions`)
whenever the source holds a `]:` candidate, so such documents pay block
parsing twice; on `typescript-handbook-advanced-types` (13 definitions) the
fence closer search runs 116 times for 58 blocks. Only 3 of the 57 broad
documents contain `]:`, so the corpus barely shows it, but reference-style
Markdown (READMEs, changelogs) does. The fix is a parser restructure — block
structure first, inline content after, as cmark does — which would also
remove the pre-pass entirely. Recorded here as the next structural target,
not attempted in this round.

## Method notes

- Agents implemented in isolated worktrees with the same rules as round 2
  (byte-identical HTML and AST `Debug`, full workspace gate, no timings).
  Two agents' worktrees were created from `main` rather than the branch tip;
  their commits were cherry-picked onto `f216b8da` and one test literal from
  round 2 needed `.into()` after the options change.
- Measurements ran while Spotlight, Photos, Time Machine and a backup client
  were active (1-minute load average 4–14, once 31 with none of our processes
  running). The paired design absorbed it: the A/A control under that load
  was within 0.2% on every stage. What does not work is measuring while any
  `rustc` runs, and letting several queued measurement shells gate on each
  other's command lines — the final queue gates on exact process names and
  runs everything sequentially ([`harness/queue_all.sh`](harness/queue_all.sh)).
- Decision rule unchanged: geometric mean ≥ 1.010 on the target stage with
  every round above 1.000, fresh and reuse ≥ 1.000, no broad case below 0.970
  without an explanation.
