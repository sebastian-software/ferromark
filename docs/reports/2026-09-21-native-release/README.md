# Native comparison before the 2.0.0 release — 2026-09-21

The release head, **v2 `bffc89f6` and v1 `4e15141`**, were measured with explicit
syntax and renderer flags on the frozen **57 documents (37–113,609 UTF-8
bytes)**. Each scored column uses the same input set and equivalent HTML for
every included engine. **Speed relative to v2: higher is faster; v2 = 1.00×.**

| Engine | Fresh, 14 agreeing across six | Reuse, same 14 | Fresh, 50 agreeing across five | Reuse, same 50 |
| --- | ---: | ---: | ---: | ---: |
| Ferromark v2 | 1.00× | 1.00× | 1.00× | 1.00× |
| OX-Content original | 0.94× | 1.05× | — | — |
| Ferromark v1 | 0.67× | 0.76× | 0.52× | 0.56× |
| md4c | 0.25× | 0.22× | 0.31× | 0.30× |
| pulldown-cmark | 0.44× | 0.41× | 0.41× | 0.41× |
| Bun native bun_md | 0.16× | 0.13× | 0.18× | 0.17× |

On the all-six agreement subset, v2 runs at
**1.069× OX's throughput fresh** and
**0.954× with reuse**. On the broader
five-engine subset it runs at 1.92× v1's speed
fresh and 1.78× with reuse, 2.43×
pulldown-cmark's fresh, 3.24× md4c's and
5.55× Bun's native engine. These are measured
corpus aggregates on one machine, not a universal ranking.

The all-six subset contains ten comments and four plain-prose views. The broader
five-engine subset also covers technical docs, linked encyclopedia excerpts,
references, and READMEs. Both agreeing subsets span 37–80,966 bytes. OX has no
score in the five-engine columns: its original renderer cannot disable heading
IDs, callouts, inline TOCs, or fence metadata cleanup. Its 39 heading-ID-only
differences are not normalized away. The seven cases outside five-engine
agreement remain measured as diagnostics, including task CSS and link/content
differences.

All engines parse and render natively. The CommonMark lane disables optional
syntax; the extension lane enables only **tables, strikethrough, and task lists**.
It is not full GFM. Bare URL autolinking, footnotes, frontmatter, line comments,
definition lists, MDX, and optional renderer extras are off where configurable.
Raw HTML passes through; these explicit benchmark settings differ from library
defaults.

Fresh includes parser/renderer setup, complete processing, owned output, and
destruction. Reuse retains state where the public API permits; Bun's native
`bun_md` still uses its fresh owned-output API. No JavaScript, WASM, process
startup, file I/O, or output normalization is timed. One macOS arm64 executable
on an Apple M1 Pro, one Rust compiler, shared mimalloc and the same pinned
registry resolution; three process rounds with six rotating windows per round.
Small differences on this shared workstation are not established significance.

Selected groups, **fresh**, using only the five-engine agreement subset and
the same speed scale:

| Group, five-engine agreement | N | V1 | md4c | pulldown | Bun native |
| --- | ---: | ---: | ---: | ---: | ---: |
| <512 B | 10 | 0.62× | 0.23× | 0.49× | 0.23× |
| 32–128 KiB | 7 | 0.54× | 0.28× | 0.36× | 0.10× |
| comments | 11 | 0.68× | 0.25× | 0.51× | 0.23× |
| technical-docs | 21 | 0.51× | 0.35× | 0.40× | 0.18× |
| plain-prose | 4 | 0.68× | 0.27× | 0.32× | 0.06× |

Ferromark's published binaries are built with profile-guided optimization. The
same recipe was applied to every Rust engine in the shared executable and
measured on the **28 held-out documents** no profile saw. It gives v2
1.29× fresh and 1.29× with reuse over its default
build; against the PGO builds of the other engines, v2 stands at
1.95× v1, 2.37× pulldown-cmark,
3.86× md4c and 5.20× Bun's
native engine fresh. md4c is C and is not PGO-built; a PGO row is compared only
with PGO rows.


## What changed in v2, and what these numbers may be attributed to

This run measures `main` at `bffc89f6`, the head of the v2 line after the final
pre-release review. The preceding report measured `7c887a2b`; **33 commits
separate them, 3 of them `perf` and 9 `fix`**. The parser and renderer work
in that range bounds costs that were quadratic or unbounded on specific
shapes — nested link probing, missing MDX closers, math/MDX/definition-list
scans, container and inline nesting, code-annotation ranges — and corrects
three CommonMark and MDX conformance departures
([container and inline fixes](../../decisions/2026-09-21-container-and-inline-fixes.md),
[extension scan memos](../../decisions/2026-09-21-extension-scan-memos.md),
[MDX flow and block conformance](../../decisions/2026-09-21-mdx-flow-and-block-conformance.md),
[renderer fixes](../../decisions/2026-09-21-renderer-fixes.md),
[linear link probe](../../decisions/2026-09-17-linear-link-probe.md)).

None of those shapes occurs at scale in the 57 frozen documents, so this rerun
answers one question: **did the release fixes cost the broad corpus anything?**
The position against the preceding run, on identical pins, flags, inputs and
agreeing subsets, is:

| V2 throughput relative to | N | Lifecycle | Preceding run | This run | Change |
| --- | ---: | --- | ---: | ---: | ---: |
| OX-Content original | 14 | fresh | 1.189× | 1.069× | -10.04% |
| OX-Content original | 14 | reuse | 1.084× | 0.954× | -11.93% |
| Ferromark v1 | 50 | fresh | 2.074× | 1.922× | -7.35% |
| Ferromark v1 | 50 | reuse | 1.938× | 1.784× | -7.95% |
| md4c | 50 | fresh | 3.507× | 3.240× | -7.60% |
| md4c | 50 | reuse | 3.644× | 3.345× | -8.22% |
| pulldown-cmark | 50 | fresh | 2.648× | 2.432× | -8.16% |
| pulldown-cmark | 50 | reuse | 2.702× | 2.468× | -8.64% |
| Bun native bun_md | 50 | fresh | 6.277× | 5.546× | -11.64% |
| Bun native bun_md | 50 | reuse | 6.737× | 5.920× | -12.12% |

**They did.** V2 lost 7–12% of its relative
throughput against every engine and in both lifecycles, most on the 14-document
all-six subset, which is ten short comments and four plain-prose views. The
three process rounds of this run agree to within 0.004 on every
validation row below, the other five engines are unchanged pins whose absolute
times reproduce the preceding run, and the direction is the same on
108 of the 114 document/lifecycle rows, so this is not
round noise or a machine effect: **the release head is slower than `7c887a2b`
on ordinary documents.** Two separate runs of the six-engine harness cannot say
which commits cost what; the paired A/B measurement in
[2026-09-21-release-fixes-paired](../2026-09-21-release-fixes-paired/README.md)
builds `c4af9525` (the `7c887a2b` core on `main`) against each intermediate
revision of the range on the same documents and attributes the loss. Those
ratios are not pooled here. What this report establishes is the **competitive
position of the release head** on matched flags and equivalent outputs, which
remains ahead of every engine on the 50-document set and, fresh, of OX on the
14-document set.

### Where v2 lost ground

108 of the 114 document/lifecycle rows moved against v2
relative to v1, the largest by 53.57% (the twelve largest are listed).
The largest losses sit on comments with task lists and block quotes and on
technical pages with many list items and fences; the 6 rows that moved the
other way are within 2.7%:

| Document | Lifecycle | Bytes | Change vs v1 | Change vs pulldown |
| --- | --- | ---: | ---: | ---: |
| comment-checklist | reuse | 287 | +53.57% | +53.17% |
| comment-quote | reuse | 290 | +50.13% | +48.16% |
| comment-checklist | fresh | 287 | +47.18% | +47.93% |
| comment-review-long | reuse | 957 | +40.01% | +39.12% |
| comment-quote | fresh | 290 | +37.82% | +39.68% |
| comment-review-long | fresh | 957 | +35.67% | +36.74% |
| typescript-handbook-the-handbook | fresh | 5337 | +34.44% | +34.84% |
| typescript-handbook-the-handbook | reuse | 5337 | +33.91% | +36.90% |
| comment-incident | reuse | 1124 | +31.04% | +30.95% |
| comment-incident | fresh | 1124 | +29.57% | +29.03% |
| vite-docs-api-plugin | fresh | 31890 | +28.16% | +27.56% |
| vite-docs-api-plugin | reuse | 31890 | +24.92% | +26.75% |


## Outputs did not change

Every one of the **342 archived HTML outputs** (57 documents × 6 engines) is
byte-identical to the preceding report, and every agreement classification is
unchanged, so the 14-case and 50-case scored subsets are the same documents.
`compare_previous.py` refuses to emit a comparison otherwise and `audit.py`
asserts the whole `verification.json` is equal. The two held-out runs reproduce
the same HTML from both the default and the PGO executable. The review's three
intentional output corrections do not reach this corpus under the benchmark
flags; see [OUTPUT-REVIEW.md](OUTPUT-REVIEW.md).

## Profile-guided optimization, measured on the held-out half

`prepare.py --pgo` applies one recipe to every Rust engine in the shared
executable and trains it on the 29 broad documents plus 45 authored diagnostics
of the round-3 split. Both executables were then measured on the other **28
documents, which no engine's profile ever saw**. Same pinned nightly, generic
CPU baseline, fat LTO, allocator and registry resolution; only `RUSTFLAGS`
differ. Per-engine speedup is default-build time over PGO-build time,
geometric mean over the 28 held-out documents.

| Engine | Profile data | Fresh | Reuse |
| --- | --- | ---: | ---: |
| Ferromark v2 | rust-pgo | 1.287× | 1.285× |
| OX-Content original | rust-pgo | 1.222× | 1.240× |
| Ferromark v1 | rust-pgo | 1.236× | 1.228× |
| md4c | none | 1.031× | 1.030× |
| pulldown-cmark | rust-pgo | 1.229× | 1.229× |
| Bun native bun_md | rust-pgo-partial | 1.198× | 1.202× |

Every Rust engine gains 22–29%. md4c is a C parser compiled with clang `-O3`
and is unchanged; Bun's engine gains through its Rust crates only. Per-document
fresh ranges: v2 1.12–1.38×, OX 1.09–1.32×,
pulldown-cmark 1.12–1.37×, v1 0.95–1.43×,
Bun 1.01–1.32×.

Because the gain is not uniform, **a PGO row is compared only with PGO rows**:

| V2 throughput relative to | N | Default fresh | Default reuse | PGO fresh | PGO reuse |
| --- | ---: | ---: | ---: | ---: | ---: |
| OX-Content original | 5 | 1.10× | 0.94× | 1.19× | 0.95× |
| Ferromark v1 | 24 | 1.89× | 1.76× | 1.95× | 1.82× |
| md4c | 24 | 3.10× | 3.20× | 3.86× | 3.98× |
| pulldown-cmark | 24 | 2.29× | 2.32× | 2.37× | 2.39× |
| Bun native bun_md | 24 | 4.87× | 5.20× | 5.20× | 5.51× |

The preceding report measured the same split at `7c887a2b` and reported v2
gaining 1.258× fresh / 1.251× reuse. At `bffc89f6` the same recipe gains
1.287× / 1.285×. The other five engines are unchanged pins; these are
separate runs on a differently loaded machine, so small movements are not
attributed to anything.

## Validation

Direct comparison in each process round, using the same agreement sets:

| V2 throughput relative to | Inputs | Fresh rounds | Reuse rounds |
| --- | ---: | --- | --- |
| OX-Content original | 14 | 1.071×, 1.068×, 1.069× | 0.956×, 0.954×, 0.955× |
| Ferromark v1 | 50 | 1.922×, 1.924×, 1.925× | 1.786×, 1.784×, 1.782× |

These are round aggregates, not confidence intervals. The headline uses each
engine/document's median of round medians before aggregating document ratios.
The three rounds agree to within 0.004 on every row above, which is the
objective check that no round was disturbed.

All **12,744 timed windows** in the 57-document run passed output-length
checksums, as did the **6,480 windows** of each held-out run. Fresh/reuse
equality, repeated transitions, and exact pre/post-timing output checks passed
for every engine in all three runs. Each of the 59 workloads (57 documents and
two rotating batches) was measured in both lifecycles, with three process
rounds, six 40 ms minimum windows per round, and 60 ms per-engine warmup.

The native harness has **32 passing** comparator/protocol/aggregation/PGO tests,
including the guards that prevent OX from receiving a score in the five-engine
subset and that the PGO training and measured halves partition the corpus. No
parser or renderer source was edited for this rerun.

Measurements ran on a shared workstation: the 1-minute load average was
4.49 before the first run and 1.80 after the last, and no
compiler ran at any point between the builds and the end of timing (every run
was gated on `pgrep -x rustc`, `cargo` and `clang`). The 57-document run took
11m05s. Load averages before and after every run are in
[measure-log.txt](measure-log.txt).

- [Position against the preceding run](previous-comparison.md),
  [per-document position changes](previous-documents.md),
  [exact flag contract](FLAGS.md), [output differences](OUTPUT-REVIEW.md),
  [source/build provenance](PROVENANCE.md).
- [Raw windows](samples.json.gz), [run metadata](run.json),
  [per-document timings](timings.csv), [aggregates](aggregates.json),
  [full tables](TABLES.md).
- [Frozen inputs and attribution](corpus.json.gz), [all HTML](verification.json.gz),
  [executable option guards](behavior.json.gz).
- [Default build metadata](build.json), [PGO build metadata](build-pgo.json),
  [dependency lock](Cargo.lock), [source audit](source-audit.json.gz) and its
  [v2 supplement](source-audit-v2.json), [checks](checks/commands.json),
  [archived-data audit](artifact-audit.json).
- Held-out PGO evidence: [default run](run-pgo-default.json) and
  [`pgo/default/`](pgo/default/), [PGO run](run-pgo-pgo.json) and
  [`pgo/pgo/`](pgo/pgo/).

The source caches were restored from the same upstream commits with this
report's `restore.py`; the OX, mimalloc, and Highway archive checksums match
the original run exactly, and `restore.json` preserves the URLs, pins, and
verification.

The complete harness at this revision is preserved under `harness/`.
Regenerate the tables with `python3 harness/report.py .`, this text with
`python3 publish.py`, the preceding-run comparison with
`python3 compare_previous.py`, and recheck the archive with `python3 audit.py`.
`publish.py --update-readme` refreshes the website's native comparison section
and `--website-json` writes the homepage figures; the repository wrapper
`scripts/publish-native-readme.py` runs both from a temporary copy.
Reproduction commands are in [PROVENANCE.md](PROVENANCE.md).
