# Native comparison after the release-fix regression was repaired — 2026-09-21

The release head, **v2 `60602a5a` and v1 `4e15141`**, were measured with explicit
syntax and renderer flags on the frozen **57 documents (37–113,609 UTF-8
bytes)**. Each scored column uses the same input set and equivalent HTML for
every included engine. **Speed relative to v2: higher is faster; v2 = 1.00×.**

| Engine | Fresh, 14 agreeing across six | Reuse, same 14 | Fresh, 50 agreeing across five | Reuse, same 50 |
| --- | ---: | ---: | ---: | ---: |
| Ferromark v2 | 1.00× | 1.00× | 1.00× | 1.00× |
| OX-Content original | 0.87× | 0.96× | — | — |
| Ferromark v1 | 0.62× | 0.70× | 0.50× | 0.54× |
| md4c | 0.20× | 0.18× | 0.28× | 0.27× |
| pulldown-cmark | 0.41× | 0.37× | 0.40× | 0.39× |
| Bun native bun_md | 0.14× | 0.12× | 0.17× | 0.16× |

On the all-six agreement subset, v2 runs at
**1.149× OX's throughput fresh** and
**1.040× with reuse**. On the broader
five-engine subset it runs at 1.98× v1's speed
fresh and 1.85× with reuse, 2.53×
pulldown-cmark's fresh, 3.54× md4c's and
5.74× Bun's native engine. These are measured
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
| <512 B | 10 | 0.60× | 0.18× | 0.46× | 0.22× |
| 32–128 KiB | 7 | 0.53× | 0.27× | 0.35× | 0.10× |
| comments | 11 | 0.62× | 0.20× | 0.46× | 0.21× |
| technical-docs | 21 | 0.50× | 0.33× | 0.39× | 0.18× |
| plain-prose | 4 | 0.63× | 0.25× | 0.29× | 0.06× |

Ferromark's published binaries are built with profile-guided optimization. The
same recipe was applied to every Rust engine in the shared executable and
measured on the **28 held-out documents** no profile saw. It gives v2
1.27× fresh and 1.26× with reuse over its default
build; against the PGO builds of the other engines, v2 stands at
2.06× v1, 2.49× pulldown-cmark,
4.06× md4c and 5.55× Bun's
native engine fresh. md4c is C and is not PGO-built; a PGO row is compared only
with PGO rows.


## What changed in v2, and what these numbers may be attributed to

This run measures `main` at `60602a5a`: the release head `bffc89f6` plus the
regression fix. The [preceding run](../2026-09-21-native-release/README.md) measured
`bffc89f6` itself and found it 7–12% behind `7c887a2b` against every engine;
the [paired harness](../2026-09-21-release-fixes-paired/README.md) attributed
the loss to the final review's laziness tracker (#383) and to the parser
struct the review's memo tables had grown, and the
[decision record](../../decisions/2026-09-21-lazy-tracker-cost.md) describes
the fix. This report answers whether the fix restored the position the
`7c887a2b` core had, so its comparison baseline is
[2026-09-16-native-segments](../2026-09-16-native-segments/README.md), not the regressed run.
On identical pins, flags, inputs and agreeing subsets:

| V2 throughput relative to | N | Lifecycle | Preceding run | This run | Change |
| --- | ---: | --- | ---: | ---: | ---: |
| OX-Content original | 14 | fresh | 1.189× | 1.149× | -3.35% |
| OX-Content original | 14 | reuse | 1.084× | 1.040× | -3.99% |
| Ferromark v1 | 50 | fresh | 2.074× | 1.981× | -4.51% |
| Ferromark v1 | 50 | reuse | 1.938× | 1.847× | -4.70% |
| md4c | 50 | fresh | 3.507× | 3.543× | +1.04% |
| md4c | 50 | reuse | 3.644× | 3.686× | +1.15% |
| pulldown-cmark | 50 | fresh | 2.648× | 2.529× | -4.48% |
| pulldown-cmark | 50 | reuse | 2.702× | 2.580× | -4.49% |
| Bun native bun_md | 50 | fresh | 6.277× | 5.740× | -8.56% |
| Bun native bun_md | 50 | reuse | 6.737× | 6.148× | -8.73% |

The regressed run had lost 7–12% on every row; the largest movement is now
8.73% and the smallest 1.04%, and against md4c v2 is
ahead of the `7c887a2b` run. The three process rounds of this run agree to
within 0.007 on every validation row below, and the other five
engines are unchanged pins. Two things make up what remains. The paired
harness measures the fixed core at 0.982× fresh and 0.967× parse against
the `7c887a2b` core, the cost the review's nesting cap (#372) and linear
link probe (#374) carry by design on link- and emphasis-dense technical
pages and on sub-microsecond comments; the paired report lists it per
document. The rest of the movement is between-run variation: the `7c887a2b`
run was taken at a 1-minute load of 7.96, this one at 2.6, and the six
engines do not share that load equally, which is why this report compares
positions and not nanoseconds. What it establishes is the **competitive
position of the release head with the fix** on matched flags and equivalent
outputs.

### Where v2 moved against v1

111 of the 114 document/lifecycle rows moved against v2
relative to v1 since the `7c887a2b` run, the largest by 14.53%; the
3 rows that moved the other way are within
2.1% (the twelve largest losses are listed):

| Document | Lifecycle | Bytes | Change vs v1 | Change vs pulldown |
| --- | --- | ---: | ---: | ---: |
| typescript-handbook-typescript-5-0 | reuse | 50714 | +14.53% | +11.49% |
| typescript-handbook-typescript-5-0 | fresh | 50714 | +13.27% | +9.59% |
| vue-docs-reactivity-in-depth | fresh | 24001 | +11.85% | +4.96% |
| guard-angle-link | reuse | 41 | +11.18% | +13.48% |
| vue-docs-reactivity-in-depth | reuse | 24001 | +10.29% | +4.01% |
| vite-docs-features | fresh | 39739 | +10.22% | +5.28% |
| vite-docs-features | reuse | 39739 | +10.16% | +7.31% |
| comment-links | reuse | 278 | +9.57% | +9.52% |
| vite-docs-api-plugin | fresh | 31890 | +9.46% | +5.84% |
| typescript-handbook-advanced-types | fresh | 36745 | +9.38% | +5.69% |
| typescript-handbook-advanced-types | reuse | 36745 | +9.23% | +5.65% |
| vue-docs-ways-of-using-vue | reuse | 5883 | +8.66% | +7.80% |


## Outputs did not change

Every one of the **342 archived HTML outputs** (57 documents × 6 engines) is
byte-identical to the regressed run and to the `7c887a2b` run, and every
agreement classification is unchanged, so the 14-case and 50-case scored
subsets are the same documents. `compare_previous.py` refuses to emit a
comparison otherwise and `audit.py` asserts the whole `verification.json` is
equal. The two held-out runs reproduce the same HTML from both the default and
the PGO executable. The fix's one intended output change and the review's
three do not reach this corpus under the benchmark flags; see
[OUTPUT-REVIEW.md](OUTPUT-REVIEW.md).

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
| Ferromark v2 | rust-pgo | 1.270× | 1.258× |
| OX-Content original | rust-pgo | 1.182× | 1.193× |
| Ferromark v1 | rust-pgo | 1.187× | 1.176× |
| md4c | none | 1.059× | 1.057× |
| pulldown-cmark | rust-pgo | 1.198× | 1.198× |
| Bun native bun_md | rust-pgo-partial | 1.147× | 1.146× |

Every Rust engine gains 18–27%. md4c is a C parser compiled with clang `-O3`
and is unchanged; Bun's engine gains through its Rust crates only. Per-document
fresh ranges: v2 1.10–1.41×, OX 1.08–1.27×,
pulldown-cmark 1.10–1.33×, v1 0.92–1.37×,
Bun 0.98–1.25×.

Because the gain is not uniform, **a PGO row is compared only with PGO rows**:

| V2 throughput relative to | N | Default fresh | Default reuse | PGO fresh | PGO reuse |
| --- | ---: | ---: | ---: | ---: | ---: |
| OX-Content original | 5 | 1.19× | 1.04× | 1.30× | 1.06× |
| Ferromark v1 | 24 | 1.97× | 1.85× | 2.06× | 1.94× |
| md4c | 24 | 3.45× | 3.60× | 4.06× | 4.22× |
| pulldown-cmark | 24 | 2.41× | 2.46× | 2.49× | 2.52× |
| Bun native bun_md | 24 | 5.09× | 5.48× | 5.55× | 5.93× |

The `7c887a2b` report measured the same split and reported v2 gaining
1.258× fresh / 1.251× reuse; the regressed run 1.287× / 1.285×. At `60602a5a` the
same recipe gains 1.270× / 1.258×. The other five engines are
unchanged pins; these are separate runs on a differently loaded machine, so
small movements are not attributed to anything.

## Validation

Direct comparison in each process round, using the same agreement sets:

| V2 throughput relative to | Inputs | Fresh rounds | Reuse rounds |
| --- | ---: | --- | --- |
| OX-Content original | 14 | 1.146×, 1.149×, 1.149× | 1.041×, 1.039×, 1.041× |
| Ferromark v1 | 50 | 1.978×, 1.980×, 1.986× | 1.842×, 1.846×, 1.849× |

These are round aggregates, not confidence intervals. The headline uses each
engine/document's median of round medians before aggregating document ratios.
The three rounds agree to within 0.007 on every row above, which is the
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
4.67 before the first run and 2.52 after the last, and no
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
