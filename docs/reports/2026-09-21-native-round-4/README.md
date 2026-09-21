# Native comparison after optimization round 4 — 2026-09-21

The release head, **v2 `39b1f0b7` and v1 `4e15141`**, were measured with explicit
syntax and renderer flags on the frozen **57 documents (37–113,609 UTF-8
bytes)**. Each scored column uses the same input set and equivalent HTML for
every included engine. **Speed relative to v2: higher is faster; v2 = 1.00×.**

| Engine | Fresh, 14 agreeing across six | Reuse, same 14 | Fresh, 50 agreeing across five | Reuse, same 50 |
| --- | ---: | ---: | ---: | ---: |
| Ferromark v2 | 1.00× | 1.00× | 1.00× | 1.00× |
| OX-Content original | 0.84× | 0.92× | — | — |
| Ferromark v1 | 0.60× | 0.67× | 0.48× | 0.52× |
| md4c | 0.22× | 0.19× | 0.29× | 0.27× |
| pulldown-cmark | 0.39× | 0.36× | 0.38× | 0.37× |
| Bun native bun_md | 0.14× | 0.12× | 0.17× | 0.15× |

On the all-six agreement subset, v2 runs at
**1.192× OX's throughput fresh** and
**1.081× with reuse**. On the broader
five-engine subset it runs at 2.07× v1's speed
fresh and 1.93× with reuse, 2.65×
pulldown-cmark's fresh, 3.51× md4c's and
6.03× Bun's native engine. These are measured
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
| <512 B | 10 | 0.57× | 0.21× | 0.44× | 0.21× |
| 32–128 KiB | 7 | 0.50× | 0.26× | 0.33× | 0.09× |
| comments | 11 | 0.60× | 0.22× | 0.45× | 0.20× |
| technical-docs | 21 | 0.47× | 0.32× | 0.37× | 0.17× |
| plain-prose | 4 | 0.61× | 0.24× | 0.28× | 0.06× |

Ferromark's published binaries are built with profile-guided optimization. The
same recipe was applied to every Rust engine in the shared executable and
measured on the **28 held-out documents** no profile saw. It gives v2
1.27× fresh and 1.26× with reuse over its default
build; against the PGO builds of the other engines, v2 stands at
2.13× v1, 2.59× pulldown-cmark,
4.24× md4c and 5.75× Bun's
native engine fresh. md4c is C and is not PGO-built; a PGO row is compared only
with PGO rows.


## What changed in v2, and what these numbers may be attributed to

This run measures `main` at `39b1f0b7`: the repaired release head `60602a5a`
plus the three optimizations round 4 kept — the forward window for the
closer probe (#386), the 2 KB arena reservation floor (#387) and the plain
inline depth cell (#388). The [round report](../2026-09-21-perf-round-4/README.md) screened
each of them alone against `23a59bdf` with the paired harness and again on
top of each other before they were merged: `closer` 1.021× fresh and
1.029× parse over the 57 documents, `arena` 1.022× fresh on the documents
under 512 bytes, `depth` 1.009× parse on 51 of 57 documents. The
comparison baseline here is the [preceding run](../2026-09-21-native-release-fixed/README.md)
at `60602a5a`, taken a few hours earlier on the same pins, flags, inputs and
agreeing subsets:

| V2 throughput relative to | N | Lifecycle | Preceding run | This run | Change |
| --- | ---: | --- | ---: | ---: | ---: |
| OX-Content original | 14 | fresh | 1.149× | 1.192× | +3.76% |
| OX-Content original | 14 | reuse | 1.040× | 1.081× | +3.96% |
| Ferromark v1 | 50 | fresh | 1.981× | 2.074× | +4.73% |
| Ferromark v1 | 50 | reuse | 1.847× | 1.929× | +4.46% |
| md4c | 50 | fresh | 3.543× | 3.508× | -1.00% |
| md4c | 50 | reuse | 3.686× | 3.638× | -1.29% |
| pulldown-cmark | 50 | fresh | 2.529× | 2.645× | +4.60% |
| pulldown-cmark | 50 | reuse | 2.580× | 2.695× | +4.45% |
| Bun native bun_md | 50 | fresh | 5.740× | 6.026× | +4.98% |
| Bun native bun_md | 50 | reuse | 6.148× | 6.462× | +5.11% |

The largest movement is 5.11% and the smallest 1.00%.
The three process rounds of this run agree to within 0.002 on
every validation row below, and the other five engines are unchanged pins.
The paired harness puts the three merges together at about 2–3% of fresh
and parse time over the 57 documents; the rest of any movement here is
between-run variation, which is why this report compares positions and not
nanoseconds. What it establishes is the **competitive position of `main`
after round 4** on matched flags and equivalent outputs.

### Where v2 moved against v1

1 of the 114 document/lifecycle rows moved against v2
relative to v1 since the preceding run, the largest by 0.76%; the
113 rows that moved the other way are within
19.2%:

| Document | Lifecycle | Bytes | Change vs v1 | Change vs pulldown |
| --- | --- | ---: | ---: | ---: |
| comment-ack | reuse | 37 | +0.76% | -1.57% |


## Outputs did not change

Every one of the **342 archived HTML outputs** (57 documents × 6 engines) is
byte-identical to the preceding run, the regressed run and the `7c887a2b` run, and every
agreement classification is unchanged, so the 14-case and 50-case scored
subsets are the same documents. `compare_previous.py` refuses to emit a
comparison otherwise and `audit.py` asserts the whole `verification.json` is
equal. The two held-out runs reproduce the same HTML from both the default and
the PGO executable. None of the three round-4 changes alters output by design,
and the paired harness verified that on every measured case; see
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
| Ferromark v2 | rust-pgo | 1.272× | 1.263× |
| OX-Content original | rust-pgo | 1.197× | 1.209× |
| Ferromark v1 | rust-pgo | 1.211× | 1.201× |
| md4c | none | 1.010× | 1.008× |
| pulldown-cmark | rust-pgo | 1.211× | 1.210× |
| Bun native bun_md | rust-pgo-partial | 1.170× | 1.169× |

Every Rust engine gains 20–27%. md4c is a C parser compiled with clang `-O3`
and is unchanged; Bun's engine gains through its Rust crates only. Per-document
fresh ranges: v2 1.11–1.37×, OX 1.09–1.30×,
pulldown-cmark 1.11–1.34×, v1 0.94–1.39×,
Bun 0.98–1.27×.

Because the gain is not uniform, **a PGO row is compared only with PGO rows**:

| V2 throughput relative to | N | Default fresh | Default reuse | PGO fresh | PGO reuse |
| --- | ---: | ---: | ---: | ---: | ---: |
| OX-Content original | 5 | 1.24× | 1.09× | 1.36× | 1.10× |
| Ferromark v1 | 24 | 2.07× | 1.94× | 2.13× | 2.00× |
| md4c | 24 | 3.40× | 3.54× | 4.24× | 4.39× |
| pulldown-cmark | 24 | 2.52× | 2.58× | 2.59× | 2.63× |
| Bun native bun_md | 24 | 5.37× | 5.78× | 5.75× | 6.14× |

The preceding run measured the same split at `60602a5a` and reported v2
gaining 1.270× fresh / 1.258× reuse. At `39b1f0b7` the same recipe gains
1.272× / 1.263×. The other five engines are
unchanged pins; these are separate runs on a differently loaded machine, so
small movements are not attributed to anything.

## Validation

Direct comparison in each process round, using the same agreement sets:

| V2 throughput relative to | Inputs | Fresh rounds | Reuse rounds |
| --- | ---: | --- | --- |
| OX-Content original | 14 | 1.192×, 1.191×, 1.192× | 1.082×, 1.080×, 1.082× |
| Ferromark v1 | 50 | 2.073×, 2.076×, 2.076× | 1.929×, 1.931×, 1.929× |

These are round aggregates, not confidence intervals. The headline uses each
engine/document's median of round medians before aggregating document ratios.
The three rounds agree to within 0.002 on every row above, which is the
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
4.90 before the first run and 2.71 after the last, and no
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
