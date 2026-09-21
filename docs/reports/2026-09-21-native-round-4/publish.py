#!/usr/bin/env python3
"""Regenerate this report's result text, the website benchmark section, and the homepage figures."""

import argparse
import importlib.util
import json
from datetime import datetime
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
SPEC = importlib.util.spec_from_file_location("tables", HERE / "harness/report.py")
tables = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(tables)

PREVIOUS = "2026-09-21-native-release-fixed"
REGRESSED = "2026-09-21-native-release"
ROUND = "2026-09-21-perf-round-4"
REPORT = "docs/reports/2026-09-21-native-round-4"
DATE = "2026-09-21"
MACHINE = "Apple M1 Pro"
HELD_OUT_ANCHOR = "#profile-guided-optimization-measured-on-the-held-out-half"


def load_held_out():
    held = {}
    for label in ("default", "pgo"):
        directory = HERE / "pgo" / label
        held[label] = dict(summary=tables.read(directory, "summary.json"),
                           cases={c["name"] for c in tables.read(directory, "corpus.json")["cases"]})
    return held


def parse_time(value):
    return datetime.strptime(value, "%Y-%m-%dT%H:%M:%SZ")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--update-readme", action="store_true",
                        help="replace the native comparison section of the root README.md")
    parser.add_argument("--website-json", type=Path,
                        help="write the homepage figures derived from this archive")
    args = parser.parse_args()
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
    round_spread = 0.0
    for engine, members in (("ox-content", six), ("v1", five)):
        by_mode = {}
        for mode in ("fresh", "reuse"):
            selected = [r for r in summary if r["case"] in members and r["mode"] == mode]
            by_mode[mode] = [tables.geomean(
                r["engines"][engine]["round_medians_ns"][i] / r["engines"]["v2"]["round_medians_ns"][i]
                for r in selected) for i in range(run["rounds"])]
            round_spread = max(round_spread, max(by_mode[mode]) - min(by_mode[mode]))
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

    # Position against the preceding report, from compare_previous.py's output.
    comparison = tables.read(HERE, "previous-comparison.json")
    position = {}
    for row in comparison:
        if row["group"] in ("all-six", "configurable-five"):
            position[row["engine"], row["mode"]] = row
    position_table = ("| V2 throughput relative to | N | Lifecycle | Preceding run | This run | Change |\n"
                      "| --- | ---: | --- | ---: | ---: | ---: |\n")
    for engine in tables.ENGINES:
        if engine == "v2":
            continue
        for mode in ("fresh", "reuse"):
            row = position[engine, mode]
            position_table += (f"| {tables.LABELS[engine]} | {row['documents']} | {mode} | "
                               f"{row['previous_v2_relative_throughput']:.3f}× | "
                               f"{row['current_v2_relative_throughput']:.3f}× | "
                               f"{row['relative_position_change_percent']:+.2f}% |\n")
    largest_move = max(abs(r["relative_position_change_percent"]) for r in position.values())

    previous = tables.read(HERE, "previous-documents.json")
    losses = [r for r in previous if r["peers"]["v1"]["relative_time_change_percent"] > 0]
    loss_table = ("| Document | Lifecycle | Bytes | Change vs v1 | Change vs pulldown |\n"
                  "| --- | --- | ---: | ---: | ---: |\n")
    for row in sorted(losses, key=lambda r: -r["peers"]["v1"]["relative_time_change_percent"])[:12]:
        loss_table += (f"| {row['case']} | {row['mode']} | {row['bytes']} | "
                       f"{row['peers']['v1']['relative_time_change_percent']:+.2f}% | "
                       f"{row['peers']['pulldown-cmark']['relative_time_change_percent']:+.2f}% |\n")
    largest_loss = max((r["peers"]["v1"]["relative_time_change_percent"] for r in losses), default=0.0)
    largest_gain = -min((r["peers"]["v1"]["relative_time_change_percent"] for r in previous), default=0.0)
    smallest_move = min(abs(r["relative_position_change_percent"]) for r in position.values())

    # PGO, held out.
    held = load_held_out()
    cases = held["default"]["cases"]
    dmap = {(r["case"], r["mode"]): r for r in held["default"]["summary"]}
    pmap = {(r["case"], r["mode"]): r for r in held["pgo"]["summary"]}
    pgo_table = "| Engine | Profile data | Fresh | Reuse |\n| --- | --- | ---: | ---: |\n"
    profile = build_pgo["pgo"]["engine_profile_data"]
    gains = {}
    ranges = {}
    for engine in tables.ENGINES:
        cells = []
        for mode in ("fresh", "reuse"):
            per_document = [dmap[c, mode]["engines"][engine]["ns"] / pmap[c, mode]["engines"][engine]["ns"]
                            for c in cases]
            cells.append(tables.geomean(per_document))
            if mode == "fresh":
                ranges[engine] = (min(per_document), max(per_document))
        gains[engine] = tuple(cells)
        kind = profile[engine].split(":")[0]
        pgo_table += f"| {tables.LABELS[engine]} | {kind} | {cells[0]:.3f}× | {cells[1]:.3f}× |\n"
    rust_engines = ("v2", "ox-content", "v1", "pulldown-cmark")
    rust_gain_low = min(min(gains[e]) for e in rust_engines)
    rust_gain_high = max(max(gains[e]) for e in rust_engines)

    standing_rows = {}
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
        standing_rows[engine] = dict(documents=len(members), default_fresh=values[0],
                                     default_reuse=values[1], pgo_fresh=values[2], pgo_reuse=values[3])
        standing += (f"| {tables.LABELS[engine]} | {len(members)} | "
                     + " | ".join(f"{v:.2f}×" for v in values) + " |\n")

    windows = run["rounds"] * run["samples"] * len(run["jobs"]) * len(run["modes"]) * len(run["engines"])
    held_run = tables.read(HERE, "run-pgo-default.json")
    held_windows = (held_run["rounds"] * held_run["samples"] * len(held_run["jobs"])
                    * len(held_run["modes"]) * len(held_run["engines"]))
    last_run = tables.read(HERE, "run-pgo-pgo.json")
    load_first = run["host_before"]["load_average"][0]
    load_last = last_run["host_after"]["load_average"][0]
    broad_seconds = int((parse_time(run["host_after"]["time_utc"])
                         - parse_time(run["host_before"]["time_utc"])).total_seconds())
    broad_duration = f"{broad_seconds // 60}m{broad_seconds % 60:02d}s"
    v2 = build["engines"]["ferromark_v2"]["revision"][:8]
    v1 = build["engines"]["ferromark_v1"]["revision"][:7]

    common = f"""The release head, **v2 `{v2}` and v1 `{v1}`**, were measured with explicit
syntax and renderer flags on the frozen **57 documents (37–113,609 UTF-8
bytes)**. Each scored column uses the same input set and equivalent HTML for
every included engine. **Speed relative to v2: higher is faster; v2 = 1.00×.**

{main_table}

On the all-six agreement subset, v2 runs at
**{1 / scores['six', 'fresh']['ox-content']:.3f}× OX's throughput fresh** and
**{1 / scores['six', 'reuse']['ox-content']:.3f}× with reuse**. On the broader
five-engine subset it runs at {1 / scores['five', 'fresh']['v1']:.2f}× v1's speed
fresh and {1 / scores['five', 'reuse']['v1']:.2f}× with reuse, {1 / scores['five', 'fresh']['pulldown-cmark']:.2f}×
pulldown-cmark's fresh, {1 / scores['five', 'fresh']['md4c']:.2f}× md4c's and
{1 / scores['five', 'fresh']['bun']:.2f}× Bun's native engine. These are measured
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
on an {MACHINE}, one Rust compiler, shared mimalloc and the same pinned
registry resolution; three process rounds with six rotating windows per round.
Small differences on this shared workstation are not established significance.

Selected groups, **fresh**, using only the five-engine agreement subset and
the same speed scale:

{group_table}

Ferromark's published binaries are built with profile-guided optimization. The
same recipe was applied to every Rust engine in the shared executable and
measured on the **{len(cases)} held-out documents** no profile saw. It gives v2
{gains['v2'][0]:.2f}× fresh and {gains['v2'][1]:.2f}× with reuse over its default
build; against the PGO builds of the other engines, v2 stands at
{standing_rows['v1']['pgo_fresh']:.2f}× v1, {standing_rows['pulldown-cmark']['pgo_fresh']:.2f}× pulldown-cmark,
{standing_rows['md4c']['pgo_fresh']:.2f}× md4c and {standing_rows['bun']['pgo_fresh']:.2f}× Bun's
native engine fresh. md4c is C and is not PGO-built; a PGO row is compared only
with PGO rows.
"""

    text = f"""# Native comparison after optimization round 4 — {DATE}

{common}

## What changed in v2, and what these numbers may be attributed to

This run measures `main` at `{v2}`: the repaired release head `60602a5a`
plus the three optimizations round 4 kept — the forward window for the
closer probe (#386), the 2 KB arena reservation floor (#387) and the plain
inline depth cell (#388). The [round report](../{ROUND}/README.md) screened
each of them alone against `23a59bdf` with the paired harness and again on
top of each other before they were merged: `closer` 1.021× fresh and
1.029× parse over the 57 documents, `arena` 1.022× fresh on the documents
under 512 bytes, `depth` 1.009× parse on 51 of 57 documents. The
comparison baseline here is the [preceding run](../{PREVIOUS}/README.md)
at `60602a5a`, taken a few hours earlier on the same pins, flags, inputs and
agreeing subsets:

{position_table}
The largest movement is {largest_move:.2f}% and the smallest {smallest_move:.2f}%.
The three process rounds of this run agree to within {round_spread:.3f} on
every validation row below, and the other five engines are unchanged pins.
The paired harness puts the three merges together at about 2–3% of fresh
and parse time over the 57 documents; the rest of any movement here is
between-run variation, which is why this report compares positions and not
nanoseconds. What it establishes is the **competitive position of `main`
after round 4** on matched flags and equivalent outputs.

### Where v2 moved against v1

{len(losses)} of the {len(previous)} document/lifecycle rows moved against v2
relative to v1 since the preceding run, the largest by {largest_loss:.2f}%; the
{len(previous) - len(losses)} rows that moved the other way are within
{largest_gain:.1f}%{' (the twelve largest losses are listed)' if len(losses) > 12 else ''}:

{loss_table}

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
of the round-3 split. Both executables were then measured on the other **{len(cases)}
documents, which no engine's profile ever saw**. Same pinned nightly, generic
CPU baseline, fat LTO, allocator and registry resolution; only `RUSTFLAGS`
differ. Per-engine speedup is default-build time over PGO-build time,
geometric mean over the {len(cases)} held-out documents.

{pgo_table}
Every Rust engine gains {100 * (rust_gain_low - 1):.0f}–{100 * (rust_gain_high - 1):.0f}%. md4c is a C parser compiled with clang `-O3`
and is unchanged; Bun's engine gains through its Rust crates only. Per-document
fresh ranges: v2 {ranges['v2'][0]:.2f}–{ranges['v2'][1]:.2f}×, OX {ranges['ox-content'][0]:.2f}–{ranges['ox-content'][1]:.2f}×,
pulldown-cmark {ranges['pulldown-cmark'][0]:.2f}–{ranges['pulldown-cmark'][1]:.2f}×, v1 {ranges['v1'][0]:.2f}–{ranges['v1'][1]:.2f}×,
Bun {ranges['bun'][0]:.2f}–{ranges['bun'][1]:.2f}×.

Because the gain is not uniform, **a PGO row is compared only with PGO rows**:

{standing}
The preceding run measured the same split at `60602a5a` and reported v2
gaining 1.270× fresh / 1.258× reuse. At `{v2}` the same recipe gains
{gains['v2'][0]:.3f}× / {gains['v2'][1]:.3f}×. The other five engines are
unchanged pins; these are separate runs on a differently loaded machine, so
small movements are not attributed to anything.

## Validation

Direct comparison in each process round, using the same agreement sets:

{round_table}
These are round aggregates, not confidence intervals. The headline uses each
engine/document's median of round medians before aggregating document ratios.
The three rounds agree to within {round_spread:.3f} on every row above, which is the
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

Measurements ran on a shared workstation: the 1-minute load average was
{load_first:.2f} before the first run and {load_last:.2f} after the last, and no
compiler ran at any point between the builds and the end of timing (every run
was gated on `pgrep -x rustc`, `cargo` and `clang`). The 57-document run took
{broad_duration}. Load averages before and after every run are in
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
"""
    (HERE / "README.md").write_text(text)

    if args.update_readme:
        links = f"""

[Full results and per-document timings]({REPORT}/README.md),
[exact flags]({REPORT}/FLAGS.md), and
[HTML differences]({REPORT}/OUTPUT-REVIEW.md)
include raw measurements, source hashes, and reproducible configuration.
The [held-out PGO comparison]({REPORT}/README.md{HELD_OUT_ANCHOR})
measures the build recipe the published binaries use. The preceding runs at
[`60602a5a`](docs/reports/2026-09-21-native-release-fixed/README.md) (the
repaired release head, before round 4),
[`bffc89f6`](docs/reports/2026-09-21-native-release/README.md) (the regressed
release head),
[`7c887a2b`](docs/reports/2026-09-16-native-segments/README.md),
[`c232d97`](docs/reports/2026-09-15-release-native/README.md), and
[`e93394e`](docs/reports/2026-09-15-native-arm/README.md) remain historical
evidence. The [full 207-case before/after suite](docs/reports/2026-09-15-arm-full-suite/README.md)
uses a separate harness and reports all four stages.

"""
        path = ROOT / "README.md"
        readme = path.read_text()
        start = readme.index("## Native engine comparison\n")
        end = readme.index("The historical [two-engine broad Markdown comparison]", start)
        path.write_text(readme[:start] + "## Native engine comparison\n\n" + common + links + readme[end:])

    if args.website_json:
        def figure(engine, members):
            group = "six" if engine == "ox-content" else "five"
            return dict(id=engine, label=tables.LABELS[engine], documents=len(members),
                        fresh=round(1 / scores[group, "fresh"][engine], 2),
                        reuse=round(1 / scores[group, "reuse"][engine], 2))

        website = dict(
            note=f"Generated by scripts/publish-native-readme.py from {REPORT}. Do not hand-edit figures.",
            report=REPORT,
            revision=v2,
            measured=DATE,
            machine=MACHINE,
            documents=dict(all=len(corpus), fiveEngineAgreement=len(five), sixEngineAgreement=len(six),
                           heldOut=len(cases)),
            figures=[figure(e, five) for e in ("v1", "pulldown-cmark", "md4c", "bun")]
                    + [figure("ox-content", six)],
            pgo=dict(
                v2Gain=dict(fresh=round(gains["v2"][0], 2), reuse=round(gains["v2"][1], 2)),
                figures=[dict(id=e, label=tables.LABELS[e], documents=standing_rows[e]["documents"],
                              fresh=round(standing_rows[e]["pgo_fresh"], 2),
                              reuse=round(standing_rows[e]["pgo_reuse"], 2))
                         for e in ("v1", "pulldown-cmark", "md4c", "bun", "ox-content")],
            ),
        )
        args.website_json.write_text(json.dumps(website, indent=2) + "\n")


if __name__ == "__main__":
    main()
