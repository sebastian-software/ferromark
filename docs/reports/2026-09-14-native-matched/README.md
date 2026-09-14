# Native comparison with matched flags — 2026-09-14

The current local cores, **v2 `33c216b` and v1 `4e15141`**, were rerun with
explicit syntax and renderer flags on the original **57 documents (37–113,609
UTF-8 bytes)**. Each scored column uses the same input set and equivalent HTML
for every included engine. **Speed relative to v2: higher is faster; v2 = 1.00×.**

| Engine | Fresh, 14 agreeing across six | Reuse, same 14 | Fresh, 50 agreeing across five | Reuse, same 50 |
| --- | ---: | ---: | ---: | ---: |
| Ferromark v2 | 1.00× | 1.00× | 1.00× | 1.00× |
| OX-Content original | 1.09× | 1.12× | — | — |
| Ferromark v1 | 0.78× | 0.82× | 0.66× | 0.69× |
| md4c | 0.26× | 0.21× | 0.37× | 0.34× |
| pulldown-cmark | 0.44× | 0.37× | 0.46× | 0.44× |
| Bun native bun_md | 0.18× | 0.14× | 0.23× | 0.21× |

**OX-Content original leads the smaller all-six fresh aggregate;
Ferromark v2 leads the broader five-engine fresh aggregate.** On the latter,
v2 runs at 1.52× v1's speed fresh and
1.45× with reuse. Individual documents can
favor v1, including the 310-byte table comment in both lifecycles.

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
| <512 B | 10 | 0.81× | 0.25× | 0.56× | 0.29× |
| 32–128 KiB | 7 | 0.62× | 0.31× | 0.34× | 0.11× |
| comments | 11 | 0.82× | 0.26× | 0.54× | 0.28× |
| technical-docs | 21 | 0.61× | 0.41× | 0.43× | 0.22× |
| plain-prose | 4 | 0.70× | 0.28× | 0.25× | 0.07× |


## What changed and how to read it

The earlier comparison used v2 before its correctness fixes and a renderer
adapter shared with OX. This run uses current source pins and v2's explicit
CommonMark renderer profile. V1 also has eight changed production source files
since the previous pin. Other engine sources and the common Cargo lock are
unchanged. Differences from the old leaderboard therefore cannot isolate the
benefit of a single optimization or flag.

All 57 inputs remain timed. Their aggregate is a native-workload diagnostic,
not the headline equal-output comparison. The [full tables](TABLES.md) include
those aggregates, every document, both lifecycles, matched category/size groups,
per-engine agreement counts, and rotating batches. Ratios use equal document
weight; Wikipedia views overlap and are not independent populations.

## Validation

All **12,744 timed windows** passed output-length checksums. Fresh/reuse
equality, repeated transitions, and exact pre/post-timing output checks passed
for every engine. Each of the 59 workloads (57 documents and two rotating
batches) was measured in both lifecycles, with three process rounds, six
40 ms minimum windows per round, and 60 ms per-engine warmup. A rotating batch
operation processes its entire profile collection and is not included in the
equal-document aggregates.

The 771 workspace tests, formatting, strict Clippy, and benchmark builds passed
before timing. The native harness has 13 passing comparator/protocol/aggregation
tests, including guards that prevent OX from receiving a score in the
five-engine subset. No parser or renderer source was edited for this rerun.

- [Exact flag contract](FLAGS.md), [output differences](OUTPUT-REVIEW.md),
  [source/build provenance](PROVENANCE.md).
- [Raw windows](samples.json.gz), [run metadata](run.json),
  [per-document timings](timings.csv), [aggregates](aggregates.json).
- [Frozen inputs and attribution](corpus.json.gz), [all HTML](verification.json.gz),
  [executable option guards](behavior.json.gz).
- [Build metadata](build.json), [dependency lock](Cargo.lock),
  [source audit](source-audit.json.gz), [checks](checks/commands.json).

The complete harness is preserved under `harness/`. Rebuild using its
`prepare.py`, supplying explicit `--ferromark-v1-source` and
`--ferromark-v2-source` paths, `--worker harness/worker.rs`, this report's
`--lockfile Cargo.lock`, and `--compile`. Existing Bun/md4c/OX source caches are
required as documented by `prepare.py --help`. Then use its `run.py` with the
new build metadata and this report's `corpus.json.gz`. Regenerate result tables
with `python3 harness/report.py .` and this text with `python3 publish.py` from
the report directory. `publish.py --update-readme` also refreshes the root
README section. Large raw JSON artifacts are gzip-compressed without removing
samples or output; `SHA256SUMS` covers the archived report.
