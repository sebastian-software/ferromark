#!/usr/bin/env python3
"""Regenerate this report's result text and optionally the root README section."""

import argparse
import importlib.util
from pathlib import Path


HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
SPEC = importlib.util.spec_from_file_location("tables", HERE / "harness/report.py")
tables = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(tables)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--update-readme", action="store_true")
    args = parser.parse_args()
    corpus = tables.read(HERE, "corpus.json")["cases"]
    verification = tables.read(HERE, "verification.json")
    summary = tables.read(HERE, "summary.json")
    run = tables.read(HERE, "run.json")
    six, five = tables.agreement_sets(verification)
    scores = {}
    for name, members, engines in (("six", six, tables.ENGINES), ("five", five, tables.CONFIGURABLE)):
        for mode in ("fresh", "reuse"):
            scores[name, mode] = tables.aggregate(summary, members, mode, engines)
    header = (
        f"| Engine | Fresh, {len(six)} agreeing across six | Reuse, same {len(six)} | "
        f"Fresh, {len(five)} agreeing across five | Reuse, same {len(five)} |\n"
        "| --- | ---: | ---: | ---: | ---: |\n"
    )
    rows = []
    for engine in tables.ENGINES:
        values = [scores[group, mode].get(engine) for group, mode in
                  (("six", "fresh"), ("six", "reuse"), ("five", "fresh"), ("five", "reuse"))]
        rows.append("| " + tables.LABELS[engine] + " | " + " | ".join(
            f"{value:.2f}×" if value is not None else "—" for value in values) + " |")
    main_table = header + "\n".join(rows)
    six_leader = tables.LABELS[max(scores["six", "fresh"], key=scores["six", "fresh"].get)]
    five_leader = tables.LABELS[max(scores["five", "fresh"], key=scores["five", "fresh"].get)]
    group_table = "| Group, five-engine agreement | N | V1 | md4c | pulldown | Bun native |\n| --- | ---: | ---: | ---: | ---: | ---: |\n"
    group_rows = []
    for label, predicate in (
        ("<512 B", lambda c: c["byte_count"] < 512),
        ("32–128 KiB", lambda c: 32768 <= c["byte_count"] < 131072),
        ("comments", lambda c: c["category"] == "comments"),
        ("technical-docs", lambda c: c["category"] == "technical-docs"),
        ("plain-prose", lambda c: c["category"] == "plain-prose"),
    ):
        names = {c["name"] for c in corpus if c["name"] in five and predicate(c)}
        values = tables.aggregate(summary, names, "fresh", tables.CONFIGURABLE)
        group_rows.append(f"| {label} | {len(names)} | " + " | ".join(
            f"{values[e]:.2f}×" for e in ("v1", "md4c", "pulldown-cmark", "bun")) + " |")
    group_table += "\n".join(group_rows)
    common = f"""The current local cores, **v2 `33c216b` and v1 `4e15141`**, were rerun with
explicit syntax and renderer flags on the original **57 documents (37–113,609
UTF-8 bytes)**. Each scored column uses the same input set and equivalent HTML
for every included engine. **Speed relative to v2: higher is faster; v2 = 1.00×.**

{main_table}

**{six_leader} leads the smaller all-six fresh aggregate;
{five_leader} leads the broader five-engine fresh aggregate.** On the latter,
v2 runs at {1 / scores['five', 'fresh']['v1']:.2f}× v1's speed fresh and
{1 / scores['five', 'reuse']['v1']:.2f}× with reuse. Individual documents can
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

{group_table}
"""
    report_intro = "# Native comparison with matched flags — 2026-09-14\n\n" + common
    windows = run["rounds"] * run["samples"] * len(run["jobs"]) * len(run["modes"]) * len(run["engines"])
    report_tail = f"""

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

All **{windows:,} timed windows** passed output-length checksums. Fresh/reuse
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
"""
    (HERE / "README.md").write_text(report_intro + report_tail)
    if args.update_readme:
        links = """

[Full results and per-document timings](docs/reports/2026-09-14-native-matched/README.md),
[exact flags](docs/reports/2026-09-14-native-matched/FLAGS.md), and
[HTML differences](docs/reports/2026-09-14-native-matched/OUTPUT-REVIEW.md)
include raw measurements, source hashes, and reproducible configuration.
The [older pre-correction six-engine run](docs/reports/2026-09-14-native-engines/README.md)
is retained as historical evidence.

"""
        path = ROOT / "README.md"
        text = path.read_text()
        start = text.index("## Native engine comparison\n")
        end = text.index("The historical [two-engine broad Markdown comparison]", start)
        path.write_text(text[:start] + "## Native engine comparison\n\n" + common + links + text[end:])


if __name__ == "__main__":
    main()
