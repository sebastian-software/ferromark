#!/usr/bin/env python3
"""Regenerate this report's result text from the archived evidence."""

import importlib.util
import json
from pathlib import Path

HERE = Path(__file__).resolve().parent
SPEC = importlib.util.spec_from_file_location("tables", HERE / "harness/report.py")
tables = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(tables)

PREVIOUS = HERE.parent / "2026-09-15-release-native"


def main():
    corpus = tables.read(HERE, "corpus.json")["cases"]
    verification = tables.read(HERE, "verification.json")
    summary = tables.read(HERE, "summary.json")
    run = tables.read(HERE, "run.json")
    build = tables.read(HERE, "build.json")
    build_pgo = tables.read(HERE, "build-pgo.json")
    six, five = tables.agreement_sets(verification)

    scores = {}
    for name, members, engines in (("six", six, tables.ENGINES),
                                   ("five", five, tables.CONFIGURABLE)):
        for mode in ("fresh", "reuse"):
            scores[name, mode] = tables.aggregate(summary, members, mode, engines)

    header = (
        f"| Engine | Fresh, {len(six)} agreeing across six | Reuse, same {len(six)} | "
        f"Fresh, {len(five)} agreeing across five | Reuse, same {len(five)} |\n"
        "| --- | ---: | ---: | ---: | ---: |\n"
    )
    rows = []
    for engine in tables.ENGINES:
        values = [scores[g, m].get(engine) for g, m in
                  (("six", "fresh"), ("six", "reuse"), ("five", "fresh"), ("five", "reuse"))]
        rows.append("| " + tables.LABELS[engine] + " | " + " | ".join(
            f"{v:.2f}×" if v is not None else "—" for v in values) + " |")
    main_table = header + "\n".join(rows)

    round_comparisons = {}
    round_table = ("| V2 throughput relative to | Inputs | Fresh rounds | Reuse rounds |\n"
                   "| --- | ---: | --- | --- |\n")
    for engine, members in (("ox-content", six), ("v1", five)):
        by_mode = {}
        for mode in ("fresh", "reuse"):
            selected = [r for r in summary if r["case"] in members and r["mode"] == mode]
            by_mode[mode] = [tables.geomean(
                r["engines"][engine]["round_medians_ns"][i] / r["engines"]["v2"]["round_medians_ns"][i]
                for r in selected) for i in range(run["rounds"])]
        round_comparisons[engine] = {"documents": len(members), "v2_relative_throughput": by_mode}
        round_table += "| " + tables.LABELS[engine] + f" | {len(members)} | " + " | ".join(
            ", ".join(f"{v:.3f}×" for v in by_mode[m]) for m in ("fresh", "reuse")) + " |\n"
    (HERE / "comparison-rounds.json").write_text(json.dumps(round_comparisons, indent=2) + "\n")

    group_table = ("| Group, five-engine agreement | N | V1 | md4c | pulldown | Bun native |\n"
                   "| --- | ---: | ---: | ---: | ---: | ---: |\n")
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

    # Per-document movement against the preceding report.
    previous = tables.read(HERE, "previous-documents.json")
    gains = sorted(previous, key=lambda r: r["peers"]["v1"]["relative_time_change_percent"])
    losses = [r for r in previous if r["peers"]["v1"]["relative_time_change_percent"] > 0]
    loss_table = ("| Document | Lifecycle | Bytes | Change vs v1 | Change vs pulldown |\n"
                  "| --- | --- | ---: | ---: | ---: |\n")
    for row in sorted(losses, key=lambda r: -r["peers"]["v1"]["relative_time_change_percent"]):
        loss_table += (f"| {row['case']} | {row['mode']} | {row['bytes']} | "
                       f"{row['peers']['v1']['relative_time_change_percent']:+.2f}% | "
                       f"{row['peers']['pulldown-cmark']['relative_time_change_percent']:+.2f}% |\n")

    issue_table = ("| Document | Lifecycle | v2 time vs OX | vs v1 | vs pulldown |\n"
                   "| --- | --- | ---: | ---: | ---: |\n")
    by_case = {(r["case"], r["mode"]): r for r in previous}
    for name in ("comment-incident", "rust-book-ch00-00-introduction",
                 "typescript-handbook-advanced-types"):
        for mode in ("fresh", "reuse"):
            row = by_case[name, mode]
            cells = []
            for engine in ("ox-content", "v1", "pulldown-cmark"):
                peer = row["peers"][engine]
                after = 1 / peer["current_v2_relative_throughput"]
                before = after / (1 + peer["relative_time_change_percent"] / 100)
                cells.append(f"{before:.3f} → {after:.3f}")
            issue_table += f"| {name} | {mode} | " + " | ".join(cells) + " |\n"

    # PGO, held out.
    held = {}
    for label in ("default", "pgo"):
        directory = HERE / "pgo" / label
        held[label] = dict(summary=tables.read(directory, "summary.json"),
                           cases={c["name"] for c in tables.read(directory, "corpus.json")["cases"]},
                           verification=tables.read(directory, "run.json"))
    cases = held["default"]["cases"]
    dmap = {(r["case"], r["mode"]): r for r in held["default"]["summary"]}
    pmap = {(r["case"], r["mode"]): r for r in held["pgo"]["summary"]}
    pgo_table = ("| Engine | Profile data | Fresh | Reuse |\n| --- | --- | ---: | ---: |\n")
    profile = build_pgo["pgo"]["engine_profile_data"]
    for engine in tables.ENGINES:
        cells = []
        for mode in ("fresh", "reuse"):
            cells.append(tables.geomean(
                dmap[c, mode]["engines"][engine]["ns"] / pmap[c, mode]["engines"][engine]["ns"]
                for c in cases))
        kind = profile[engine].split(":")[0]
        pgo_table += f"| {tables.LABELS[engine]} | {kind} | {cells[0]:.3f}× | {cells[1]:.3f}× |\n"

    standing = ("| V2 throughput relative to | N | Default fresh | Default reuse | PGO fresh | PGO reuse |\n"
                "| --- | ---: | ---: | ---: | ---: | ---: |\n")
    six_h, five_h = tables.agreement_sets({k: v for k, v in verification.items() if k in cases})
    for engine in tables.ENGINES:
        if engine == "v2":
            continue
        members = (six_h if engine == "ox-content" else five_h) & cases
        engines = tables.ENGINES if engine == "ox-content" else tables.CONFIGURABLE
        values = []
        for source in (held["default"]["summary"], held["pgo"]["summary"]):
            for mode in ("fresh", "reuse"):
                values.append(1 / tables.aggregate(source, members, mode, engines)[engine])
        standing += (f"| {tables.LABELS[engine]} | {len(members)} | "
                     + " | ".join(f"{v:.2f}×" for v in values) + " |\n")

    windows = run["rounds"] * run["samples"] * len(run["jobs"]) * len(run["modes"]) * len(run["engines"])
    held_run = tables.read(HERE, "run-pgo-default.json")
    held_windows = (held_run["rounds"] * held_run["samples"] * len(held_run["jobs"])
                    * len(held_run["modes"]) * len(held_run["engines"]))
    v2 = build["engines"]["ferromark_v2"]["revision"][:8]
    v1 = build["engines"]["ferromark_v1"]["revision"][:7]

    text = f"""# Native comparison after the segmented definition pass — 2026-09-16

The six-engine native comparison was rerun with **v2 `{v2}`** (the head of the
segmented-definition-pass branch) and v1 `{v1}`, on the same frozen **57 documents
(37–113,609 UTF-8 bytes)** and with the same flags as the
[preceding release-readiness report](../2026-09-15-release-native/README.md).
Each scored column uses one input set and equivalent HTML for every included
engine. **Speed relative to v2: higher is faster; v2 = 1.00×.**

{main_table}

On the all-six agreement subset v2 runs at
**{1 / scores['six', 'fresh']['ox-content']:.3f}× OX's throughput fresh** and
**{1 / scores['six', 'reuse']['ox-content']:.3f}× with reuse**. On the broader
five-engine subset it runs at {1 / scores['five', 'fresh']['v1']:.2f}× v1's speed
fresh and {1 / scores['five', 'reuse']['v1']:.2f}× with reuse. These are measured
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

{group_table}


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

{issue_table}
`comment-incident` and `rust-book-ch00-00-introduction` are in the held-out
half of the PGO split; `typescript-handbook-advanced-types` is in the training
half. All three are measured here in the 57-document default-build run, which
uses no profile data at all.

### Where v2 lost ground

{len(losses)} of the {len(previous)} document/lifecycle rows moved against v2
relative to v1, all within the round-to-round spread of this run:

{loss_table}

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

{pgo_table}
Every Rust engine gains 18–26%. md4c is a C parser compiled with clang `-O3`
and is unchanged; Bun's engine gains through its Rust crates only. Per-document
fresh ranges: v2 1.10–1.35×, OX 1.06–1.27×, pulldown-cmark 1.10–1.33×, v1
0.91–1.35× (two short encyclopedia paragraphs lose), Bun 1.00–1.30×.

Because the gain is not uniform, **a PGO row is compared only with PGO rows**:

{standing}
The round-3 PGO section measured the same split with v2 pinned at `e93394e` and
reported v2 gaining 1.220× fresh / 1.236× reuse. At `{v2}` the same recipe
gains {tables.geomean(dmap[c, 'fresh']['engines']['v2']['ns'] / pmap[c, 'fresh']['engines']['v2']['ns'] for c in cases):.3f}× / {tables.geomean(dmap[c, 'reuse']['engines']['v2']['ns'] / pmap[c, 'reuse']['engines']['v2']['ns'] for c in cases):.3f}×, so the faster core has not
exhausted what profile data buys. The other five engines are unchanged pins and
their gains reproduce the round-3 figures to within 0.02 (OX 1.175→1.192 fresh,
v1 1.195→1.177, pulldown-cmark 1.195→1.208, md4c 1.004→1.010), except Bun's
Rust crates at 1.165→1.203. These are separate runs on a differently loaded
machine, so small movements here are not attributed to anything.

## Validation

Direct comparison in each process round, using the same agreement sets:

{round_table}

These are round aggregates, not confidence intervals. The headline uses each
engine/document's median of round medians before aggregating document ratios.
The three rounds agree to within 0.009 on every row above, which is the
objective check that no round was disturbed.

All **{windows:,} timed windows** in the 57-document run passed output-length
checksums, as did the **{held_windows:,} windows** of each held-out run. Fresh/reuse
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
"""
    (HERE / "README.md").write_text(text)
    print("README.md and comparison-rounds.json written")


if __name__ == "__main__":
    main()
