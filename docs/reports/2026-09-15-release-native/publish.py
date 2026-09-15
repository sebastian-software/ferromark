#!/usr/bin/env python3
"""Regenerate this report's result text and optionally the root README section."""

import argparse
import importlib.util
import json
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
    round_comparisons = {}
    round_table = "| V2 throughput relative to | Inputs | Fresh rounds | Reuse rounds |\n| --- | ---: | --- | --- |\n"
    for engine, members in (("ox-content", six), ("v1", five)):
        by_mode = {}
        for mode in ("fresh", "reuse"):
            selected = [r for r in summary if r['case'] in members and r['mode'] == mode]
            by_mode[mode] = [tables.geomean(r['engines'][engine]['round_medians_ns'][i] /
                                          r['engines']['v2']['round_medians_ns'][i] for r in selected)
                             for i in range(run['rounds'])]
        round_comparisons[engine] = {'documents': len(members), 'v2_relative_throughput': by_mode}
        round_table += '| ' + tables.LABELS[engine] + f' | {len(members)} | ' + ' | '.join(
            ', '.join(f'{v:.3f}×' for v in by_mode[m]) for m in ('fresh', 'reuse')) + ' |\n'
    (HERE / 'comparison-rounds.json').write_text(json.dumps(round_comparisons, indent=2) + '\n')
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
    build = tables.read(HERE, "build.json")
    v2_revision = build["engines"]["ferromark_v2"]["revision"][:7]
    v1_revision = build["engines"]["ferromark_v1"]["revision"][:7]
    common = f"""The current local cores, **v2 `{v2_revision}` and v1 `{v1_revision}`**, were rerun with
explicit syntax and renderer flags on the original **57 documents (37–113,609
UTF-8 bytes)**. Each scored column uses the same input set and equivalent HTML
for every included engine. **Speed relative to v2: higher is faster; v2 = 1.00×.**

{main_table}

On the all-six agreement subset, v2 runs at
**{1 / scores['six', 'fresh']['ox-content']:.3f}× OX's throughput fresh** and
**{1 / scores['six', 'reuse']['ox-content']:.3f}× with reuse**. On the broader
five-engine subset it runs at {1 / scores['five', 'fresh']['v1']:.2f}× v1's speed
fresh and {1 / scores['five', 'reuse']['v1']:.2f}× with reuse. These are measured
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

{group_table}
"""
    report_intro = "# Native comparison with matched flags — 2026-09-15\n\n" + common
    windows = run["rounds"] * run["samples"] * len(run["jobs"]) * len(run["modes"]) * len(run["engines"])
    report_tail = f"""

## What changed and how to read it

This run measures the release-preparation branch at `{v2_revision}`, including
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

{round_table}

These are round aggregates, not confidence intervals. The headline uses each
engine/document's median of round medians before aggregating document ratios.

All **{windows:,} timed windows** passed output-length checksums. Fresh/reuse
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
`--ferromark-v2-source` paths, `--ferromark-v2-revision {v2_revision}`, `--worker harness/worker.rs`, this report's
`--lockfile Cargo.lock`, and `--compile`. Supply the restored Bun/md4c Git checkouts, OX archive, and native support
cache with the explicit source arguments documented in `PROVENANCE.md`. Then use its `run.py` with the
new build metadata and this report's `corpus.json.gz`. Regenerate result tables
with `python3 harness/report.py .` and this text with `python3 publish.py` from
the report directory. `publish.py --update-readme` also refreshes the root
README section. Large raw JSON artifacts are gzip-compressed without removing
samples or output; `SHA256SUMS` covers the archived report.
`python3 audit.py` rechecks input hashes, complete timing coverage, every stored
timing checksum, and unchanged HTML against the previous matched-flags report.
"""
    (HERE / "README.md").write_text(report_intro + report_tail)
    if args.update_readme:
        links = """

[Full results and per-document timings](docs/reports/2026-09-15-release-native/README.md),
[exact flags](docs/reports/2026-09-15-release-native/FLAGS.md), and
[HTML differences](docs/reports/2026-09-15-release-native/OUTPUT-REVIEW.md)
include raw measurements, source hashes, and reproducible configuration.
The [previous ARM run](docs/reports/2026-09-15-native-arm/README.md),
[earlier matched-flags run](docs/reports/2026-09-14-native-matched/README.md)
and [original six-engine run](docs/reports/2026-09-14-native-engines/README.md)
remain historical evidence. The [full 207-case before/after suite](docs/reports/2026-09-15-arm-full-suite/README.md)
uses a separate harness and reports all four stages.

"""
        path = ROOT / "README.md"
        text = path.read_text()
        start = text.index("## Native engine comparison\n")
        end = text.index("The historical [two-engine broad Markdown comparison]", start)
        path.write_text(text[:start] + "## Native engine comparison\n\n" + common + links + text[end:])


if __name__ == "__main__":
    main()
