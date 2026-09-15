# Native comparison with matched flags — 2026-09-15

The current local cores, **v2 `c232d97` and v1 `4e15141`**, were rerun with
explicit syntax and renderer flags on the original **57 documents (37–113,609
UTF-8 bytes)**. Each scored column uses the same input set and equivalent HTML
for every included engine. **Speed relative to v2: higher is faster; v2 = 1.00×.**

| Engine | Fresh, 14 agreeing across six | Reuse, same 14 | Fresh, 50 agreeing across five | Reuse, same 50 |
| --- | ---: | ---: | ---: | ---: |
| Ferromark v2 | 1.00× | 1.00× | 1.00× | 1.00× |
| OX-Content original | 1.01× | 1.01× | — | — |
| Ferromark v1 | 0.72× | 0.73× | 0.54× | 0.56× |
| md4c | 0.27× | 0.21× | 0.32× | 0.29× |
| pulldown-cmark | 0.47× | 0.39× | 0.42× | 0.40× |
| Bun native bun_md | 0.17× | 0.13× | 0.19× | 0.17× |

On the all-six agreement subset, v2 runs at
**0.986× OX's throughput fresh** and
**0.991× with reuse**. On the broader
five-engine subset it runs at 1.85× v1's speed
fresh and 1.80× with reuse. These are measured
corpus aggregates; ratios close to 1.00 are not evidence of a universal lead.

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
startup, file I/O, or output normalization is timed. One macOS arm64 executable,
one Rust compiler, shared mimalloc and the same pinned dependency lock; three
process rounds with six rotating windows per round. Small differences on this
shared workstation are not established significance.

Selected groups, **fresh**, using only the five-engine agreement subset and
the same speed scale:

| Group, five-engine agreement | N | V1 | md4c | pulldown | Bun native |
| --- | ---: | ---: | ---: | ---: | ---: |
| <512 B | 10 | 0.73× | 0.26× | 0.56× | 0.26× |
| 32–128 KiB | 7 | 0.56× | 0.28× | 0.36× | 0.10× |
| comments | 11 | 0.77× | 0.29× | 0.58× | 0.26× |
| technical-docs | 21 | 0.51× | 0.35× | 0.40× | 0.18× |
| plain-prose | 4 | 0.66× | 0.26× | 0.31× | 0.06× |


## What changed and how to read it

This run measures the release-preparation branch at `c232d97`, including
the parser/renderer refactors and document-wide footnote correction. The
previous matched-flags ARM comparison measured `e93394e`. V1 and every external
engine retain their previous source pins, the native adapters and timed loops
are unchanged, and the shared dependency lock is byte-identical. All six engines
were rebuilt together; this is a direct comparison in the new executable, not
an extrapolation from a different harness.

**The aggregate does not clear every document for release.** The incident
comment and Rust-book introduction have material regressions against the
earlier v2 core. A paired comparison of the existing binaries reproduces them;
removing definition markers reduces both gaps sharply. See the
[outlier diagnosis](DIAGNOSIS.md) and the
[per-document position changes](previous-documents.md). The structural reference
prepass is the next optimization target; its correctness fixes must survive.

This comparison spans several commits and cannot attribute the whole change to
the latest footnote fix. The [isolated footnote measurement](../2026-09-15-footnote-scope/README.md)
uses its immediate predecessor and a separate harness; those ratios are not
pooled with this native comparison. Footnotes are disabled in these native lanes.

All 57 inputs remain timed. Their aggregate is a native-workload diagnostic,
not the headline equal-output comparison. The [full tables](TABLES.md) include
those aggregates, every document, both lifecycles, matched category/size groups,
per-engine agreement counts, and rotating batches. Ratios use equal document
weight; Wikipedia views overlap and are not independent populations.

## Validation

Direct comparison in each process round, using the same agreement sets:

| V2 throughput relative to | Inputs | Fresh rounds | Reuse rounds |
| --- | ---: | --- | --- |
| OX-Content original | 14 | 0.987×, 0.986×, 0.985× | 0.992×, 0.993×, 0.992× |
| Ferromark v1 | 50 | 1.852×, 1.855×, 1.857× | 1.801×, 1.801×, 1.804× |


These are round aggregates, not confidence intervals. The headline uses each
engine/document's median of round medians before aggregating document ratios.

All **12,744 timed windows** passed output-length checksums. Fresh/reuse
equality, repeated transitions, and exact pre/post-timing output checks passed
for every engine. Each of the 59 workloads (57 documents and two rotating
batches) was measured in both lifecycles, with three process rounds, six
40 ms minimum windows per round, and 60 ms per-engine warmup. A rotating batch
operation processes its entire profile collection and is not included in the
equal-document aggregates.

The 835 workspace tests and doctests, formatting, strict Clippy, and benchmark builds passed
before timing. The native harness has 13 passing comparator/protocol/aggregation
tests, including guards that prevent OX from receiving a score in the
five-engine subset. No parser or renderer source was edited for this rerun.

- [Position against the preceding run](previous-comparison.md),
  [exact flag contract](FLAGS.md), [output differences](OUTPUT-REVIEW.md),
  [source/build provenance](PROVENANCE.md).
- [Raw windows](samples.json.gz), [run metadata](run.json),
  [per-document timings](timings.csv), [aggregates](aggregates.json).
- [Frozen inputs and attribution](corpus.json.gz), [all HTML](verification.json.gz),
  [executable option guards](behavior.json.gz).
- [Build metadata](build.json), [dependency lock](Cargo.lock),
  [source audit](source-audit.json.gz), [checks](checks/commands.json),
  [archived-data audit](artifact-audit.json).

The required source caches were reused from the preceding restoration. The
OX, mimalloc, and Highway archive checksums match the original run exactly;
`restore.py` and `restore.json` preserve the URLs, pins, and verification.

The complete harness is preserved under `harness/`. Rebuild using its
`prepare.py`, supplying explicit `--ferromark-v1-source` and
`--ferromark-v2-source` paths, `--ferromark-v2-revision c232d97`, `--worker harness/worker.rs`, this report's
`--lockfile Cargo.lock`, and `--compile`. Supply the restored Bun/md4c Git checkouts, OX archive, and native support
cache with the explicit source arguments documented in `PROVENANCE.md`. Then use its `run.py` with the
new build metadata and this report's `corpus.json.gz`. Regenerate result tables
with `python3 harness/report.py .` and this text with `python3 publish.py` from
the report directory. `publish.py --update-readme` also refreshes the root
README section. Large raw JSON artifacts are gzip-compressed without removing
samples or output; `SHA256SUMS` covers the archived report.
`python3 audit.py` rechecks input hashes, complete timing coverage, every stored
timing checksum, and unchanged HTML against the previous matched-flags report.
