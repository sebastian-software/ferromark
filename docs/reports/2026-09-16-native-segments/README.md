# Native comparison after the segmented definition pass — 2026-09-16

The six-engine native comparison was rerun with **v2 `7c887a2b`** (the head of the
segmented-definition-pass branch) and v1 `4e15141`, on the same frozen **57 documents
(37–113,609 UTF-8 bytes)** and with the same flags as the
[preceding release-readiness report](../2026-09-15-release-native/README.md).
Each scored column uses one input set and equivalent HTML for every included
engine. **Speed relative to v2: higher is faster; v2 = 1.00×.**

| Engine | Fresh, 14 agreeing across six | Reuse, same 14 | Fresh, 50 agreeing across five | Reuse, same 50 |
| --- | ---: | ---: | ---: | ---: |
| Ferromark v2 | 1.00× | 1.00× | 1.00× | 1.00× |
| OX-Content original | 0.84× | 0.92× | — | — |
| Ferromark v1 | 0.60× | 0.67× | 0.48× | 0.52× |
| md4c | 0.22× | 0.19× | 0.29× | 0.27× |
| pulldown-cmark | 0.39× | 0.36× | 0.38× | 0.37× |
| Bun native bun_md | 0.13× | 0.11× | 0.16× | 0.15× |

On the all-six agreement subset v2 runs at
**1.189× OX's throughput fresh** and
**1.084× with reuse**. On the broader
five-engine subset it runs at 2.07× v1's speed
fresh and 1.94× with reuse. These are measured
corpus aggregates on one machine, not a universal ranking.

The all-six subset contains ten comments and four plain-prose views. The broader
five-engine subset also covers technical docs, linked encyclopedia excerpts,
references, and READMEs. Both agreeing subsets span 37–80,966 bytes. OX has no
score in the five-engine columns: its original renderer cannot disable heading
IDs, callouts, inline TOCs, or fence metadata cleanup. Its 39 heading-ID-only
differences are not normalized away. The seven cases outside five-engine
agreement remain measured as diagnostics.

All engines parse and render natively. The CommonMark lane disables optional
syntax; the extension lane enables only **tables, strikethrough, and task lists**.
It is not full GFM. Fresh includes parser/renderer setup, complete processing,
owned output, and destruction. Reuse retains state where the public API permits;
Bun's native `bun_md` still uses its fresh owned-output API. No JavaScript, WASM,
process startup, file I/O, or output normalization is timed. One macOS arm64
executable, one Rust compiler, shared mimalloc and the same pinned registry
resolution; three process rounds with six rotating windows per round.

Selected groups, **fresh**, using only the five-engine agreement subset and
the same speed scale:

| Group, five-engine agreement | N | V1 | md4c | pulldown | Bun native |
| --- | ---: | ---: | ---: | ---: | ---: |
| <512 B | 10 | 0.58× | 0.21× | 0.44× | 0.20× |
| 32–128 KiB | 7 | 0.51× | 0.26× | 0.33× | 0.09× |
| comments | 11 | 0.60× | 0.22× | 0.45× | 0.19× |
| technical-docs | 21 | 0.47× | 0.33× | 0.37× | 0.16× |
| plain-prose | 4 | 0.61× | 0.24× | 0.28× | 0.05× |


## What changed in v2, and what these numbers may be attributed to

The nine commits under review are the segmented definition pass: the document-wide
discovery of link reference and footnote definitions keeps the real block grammar
as its only authority but no longer block-parses the whole document to use it
([decision record](../../decisions/2026-09-16-segmented-definition-pass.md),
[paired measurement](../2026-09-16-definition-segments/README.md)).

**The change against the preceding report is not attributable to those nine
commits.** The preceding report measured v2 at `c232d97d`; this one measures
`7c887a2b`, and **137 commits separate them, 44 of them `perf`**. That range
contains the whole Apple Silicon rounds 2 and 3 sequence (`71a051de`, PR #323 —
renderer option strings as `Cow<'static, str>`, the fence fast path, the
line-scan commits) and the consolidation of the Rust components into one
package (`23a212d8`, `885e5b1c`), in addition to the definition pass. The
per-document movement says the same thing: the largest gains are on the
shortest comments — `comment-ack` (37 bytes) improves 34.9% against v1 and has
no definition marker at all — which is the signature of the round-3 options
change, not of a definition-discovery change.

The isolated, paired evidence for the segmented pass alone is in its own report:
`comment-incident` 1.295× fresh, `rust-book-ch00-00-introduction` 1.197×,
`typescript-handbook-advanced-types` 1.061×, and 1.007× over the 57 broad
documents as a whole. Those ratios are measured against `main` (`e35e9f64`) with
a paired harness and are not pooled with the competitive ratios here.

What this report does establish is the **competitive position of the PR head**,
on matched flags and equivalent outputs, which is the last item of the issue-320
checklist.

### The three documents the previous report flagged

The release-readiness report could not clear two documents: the incident comment
and the Rust-book introduction, where v2 was slower than the earlier v2 core and
lost to OX outright. Both are now ahead of OX. Values are v2 processing time
divided by the peer's; **lower is better for v2**.

| Document | Lifecycle | v2 time vs OX | vs v1 | vs pulldown |
| --- | --- | ---: | ---: | ---: |
| comment-incident | fresh | 1.262 → 0.814 | 1.108 → 0.706 | 0.759 → 0.486 |
| comment-incident | reuse | 1.310 → 0.837 | 1.204 → 0.764 | 0.757 → 0.486 |
| rust-book-ch00-00-introduction | fresh | 1.203 → 0.930 | 0.942 → 0.730 | 0.618 → 0.480 |
| rust-book-ch00-00-introduction | reuse | 1.203 → 0.934 | 1.005 → 0.780 | 0.629 → 0.490 |
| typescript-handbook-advanced-types | fresh | 0.788 → 0.692 | 0.549 → 0.476 | 0.478 → 0.432 |
| typescript-handbook-advanced-types | reuse | 0.769 → 0.665 | 0.562 → 0.484 | 0.475 → 0.427 |

`comment-incident` and `rust-book-ch00-00-introduction` are in the held-out
half of the PGO split; `typescript-handbook-advanced-types` is in the training
half. All three are measured here in the 57-document default-build run, which
uses no profile data at all.

### Where v2 lost ground

2 of the 114 document/lifecycle rows moved against v2
relative to v1, all within the round-to-round spread of this run:

| Document | Lifecycle | Bytes | Change vs v1 | Change vs pulldown |
| --- | --- | ---: | ---: | ---: |
| wiki-volcano-first-paragraph | reuse | 2644 | +0.62% | -0.10% |
| rust-book-appendix-02-operators | fresh | 22595 | +0.20% | +4.41% |


## Outputs did not change

Every one of the **342 archived HTML outputs** (57 documents × 6 engines) is
byte-identical to the preceding report, and every agreement classification is
unchanged, so the 14-case and 50-case scored subsets are the same documents.
`compare_previous.py` refuses to emit a comparison otherwise and `audit.py`
asserts the whole `verification.json` is equal. The two held-out runs reproduce
the same HTML from both the default and the PGO executable. See
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
| Ferromark v2 | rust-pgo | 1.258× | 1.251× |
| OX-Content original | rust-pgo | 1.192× | 1.206× |
| Ferromark v1 | rust-pgo | 1.177× | 1.188× |
| md4c | none | 1.010× | 1.006× |
| pulldown-cmark | rust-pgo | 1.208× | 1.207× |
| Bun native bun_md | rust-pgo-partial | 1.203× | 1.205× |

Every Rust engine gains 18–26%. md4c is a C parser compiled with clang `-O3`
and is unchanged; Bun's engine gains through its Rust crates only. Per-document
fresh ranges: v2 1.10–1.35×, OX 1.06–1.27×, pulldown-cmark 1.10–1.33×, v1
0.91–1.35× (two short encyclopedia paragraphs lose), Bun 1.00–1.30×.

Because the gain is not uniform, **a PGO row is compared only with PGO rows**:

| V2 throughput relative to | N | Default fresh | Default reuse | PGO fresh | PGO reuse |
| --- | ---: | ---: | ---: | ---: | ---: |
| OX-Content original | 5 | 1.23× | 1.08× | 1.37× | 1.12× |
| Ferromark v1 | 24 | 2.07× | 1.94× | 2.19× | 2.02× |
| md4c | 24 | 3.40× | 3.54× | 4.23× | 4.39× |
| pulldown-cmark | 24 | 2.53× | 2.58× | 2.59× | 2.63× |
| Bun native bun_md | 24 | 5.58× | 6.02× | 5.77× | 6.17× |

The round-3 PGO section measured the same split with v2 pinned at `e93394e` and
reported v2 gaining 1.220× fresh / 1.236× reuse. At `7c887a2b` the same recipe
gains 1.258× / 1.251×, so the faster core has not
exhausted what profile data buys. The other five engines are unchanged pins and
their gains reproduce the round-3 figures to within 0.02 (OX 1.175→1.192 fresh,
v1 1.195→1.177, pulldown-cmark 1.195→1.208, md4c 1.004→1.010), except Bun's
Rust crates at 1.165→1.203. These are separate runs on a differently loaded
machine, so small movements here are not attributed to anything.

## Validation

Direct comparison in each process round, using the same agreement sets:

| V2 throughput relative to | Inputs | Fresh rounds | Reuse rounds |
| --- | ---: | --- | --- |
| OX-Content original | 14 | 1.188×, 1.185×, 1.188× | 1.084×, 1.078×, 1.082× |
| Ferromark v1 | 50 | 2.074×, 2.070×, 2.079× | 1.934×, 1.938×, 1.942× |


These are round aggregates, not confidence intervals. The headline uses each
engine/document's median of round medians before aggregating document ratios.
The three rounds agree to within 0.009 on every row above, which is the
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

Measurements ran on a shared workstation under a Spotlight indexing load: the
1-minute load average was 7.96 before the first run and 5.53 after the last, and
no compiler ran at any point between the builds and the end of timing. The
57-document run took 11m11s, against 11m06s for the preceding report at load
2.69. Load averages before and after every run are in
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

The required source caches were reused from the preceding restoration; the OX,
mimalloc, and Highway archive checksums match the original run exactly, and
`restore.py`/`restore.json` preserve the URLs, pins, and verification.

The complete harness at this revision is preserved under `harness/`.
Regenerate the tables with `python3 harness/report.py .`, this text with
`python3 publish.py`, the preceding-run comparison with
`python3 compare_previous.py`, and recheck the archive with `python3 audit.py`.
Reproduction commands are in [PROVENANCE.md](PROVENANCE.md).
