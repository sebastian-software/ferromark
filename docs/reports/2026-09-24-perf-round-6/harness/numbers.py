#!/usr/bin/env python3
"""Stage geomeans per run, per-document matrices and group breakdowns: numbers.py [out.md]

Reads results/ of the report this harness belongs to (the Apple Silicon runs in results/<run>/, the
x86-64 CI runs in results/x86/<name>/host-N/) and writes NUMBERS.md next to it by default.
"""
import gzip
import json
import math
import re
import sys
from collections import defaultdict
from pathlib import Path

REPORT = Path(__file__).resolve().parent.parent
OUT = Path(sys.argv[1]) if len(sys.argv) > 1 else REPORT / "NUMBERS.md"
MODES = ("fresh", "reuse", "parse", "render")
LABELS = {
    "aa6": "A/A control (`060b02d2` against itself)",
    "tagdispatch": "`tagdispatch` #418 tag filter per-initial candidate sets",
    "headingid": "`headingid` #422 heading ids in reusable claim storage",
    "tables2": "`tables2` #424 NEON table pipe cursor, second iteration",
    "lists2": "`lists2` #425 container line facts, second iteration",
    "autoidx": "`autoidx` #427 v1, autolink triggers from the root scan",
    "autoidx2": "`autoidx2` #427 v2, lazy per-block trigger record",
    "arm426": "`arm426` #426 x86 ByteClass paths, aarch64 check",
}
BASELINES = {"aa6": "060b02d2", "tagdispatch": "060b02d2", "headingid": "060b02d2", "tables2": "060b02d2",
             "lists2": "9e741a37", "autoidx": "9e741a37", "autoidx2": "9e741a37", "arm426": "af65cb43"}
SECTIONS = [
    ("Screens: 20 documents and 16 diagnostics, 3 rounds × 3 pairs",
     ["aa6-screen", "headingid-screen", "tables2-screen", "lists2-screen", "autoidx-screen", "autoidx2-screen"]),
    ("Broad confirmations: 57 documents, 3 rounds × 5 pairs",
     ["tagdispatch-broad", "headingid-broad", "tables2-broad", "lists2-broad", "autoidx-broad", "autoidx2-broad",
      "arm426-broad"]),
    ("Recheck of #418: 5 documents, 3 rounds × 7 pairs", ["aa6-recheck6", "tagdispatch-recheck6"]),
    ("Recheck of #422: 6 documents, 3 rounds × 7 pairs", ["aa6-recheck422", "headingid-recheck422"]),
    ("Recheck of #424: 7 cases, 3 rounds × 9 pairs", ["aa6-recheck424", "tables2-recheck424"]),
]
X86 = [
    ("aa", "A/A control, #419's head `a4d2fd0a` against itself", "a4d2fd0a"),
    ("414", "#414 ASCII-only block trimming, `7eda5a4b`", "aceafd32"),
    ("426", "#426 first commit `09538a76`, dispatch inlined", "af65cb43"),
    ("426b", "#426 revision `b64cc1e6`, dispatch out of line (merged)", "af65cb43"),
    ("428", "#428 first commit `bdde4db0`, AVX2 at every size", "05a55a01"),
    ("428b", "#428 revision `2bf6ba56`, SSE2 below 512 bytes, run 1", "05a55a01"),
    ("428c", "#428 revision `2bf6ba56`, run 2", "05a55a01"),
    ("430", "#430 `5782b91c` inlined SSE2 escape classifiers (merged)", "f8fff4c9"),
    ("431", "#431 first commit `273b6bf6`, whole scan out of line", "dfcfe983"),
    ("431b", "#431 revision `d6e0bad3`, inline 16-byte probe (merged)", "dfcfe983"),
    ("431inl", "#431 comparison `78429a94`, fully inline", "dfcfe983"),
    ("431pl", "#431 placebo `41f4e99e`, the revision's structure with the word scan", "dfcfe983"),
    ("total", "cumulative: `c6a29e34` (#426 + #430 + #431)", "05a55a01"),
]


def geomean(values):
    return math.exp(sum(math.log(v) for v in values) / len(values))


def median(values):
    ordered = sorted(values)
    middle = len(ordered) // 2
    return ordered[middle] if len(ordered) % 2 else (ordered[middle - 1] + ordered[middle]) / 2


def quantiles(values, n):
    """statistics.quantiles(values, n=n) with the default exclusive method (this file shadows `numbers`)."""
    data, count = sorted(values), len(values)
    cuts = []
    for i in range(1, n):
        j = min(max(i * (count + 1) // n, 1), count - 1)
        delta = i * (count + 1) - j * n
        cuts.append((data[j - 1] * (n - delta) + data[j] * delta) / n)
    return cuts


def load(run):
    directory = REPORT / "results" / run
    return json.loads((directory / "summary.json").read_text()), json.loads((directory / "grouped.json").read_text())


def label(run):
    name = run.rsplit("-", 1)[0]
    return f"{LABELS[name]}, base `{BASELINES[name]}`"


def x86_hosts(name):
    """[(host number, CPU model, summary rows)] of one x86 run."""
    hosts = []
    for directory in sorted((REPORT / "results" / "x86" / name).glob("host-*")):
        model = re.search(r"Model name:\s+(.*)", (directory / "host.txt").read_text()).group(1).strip()
        rows = json.loads(gzip.decompress((directory / "summary.json.gz").read_bytes()))
        hosts.append((directory.name.split("-")[1], short_cpu(model), rows))
    return hosts


def short_cpu(model):
    model = re.sub(r"\(R\)|CPU @ [\d.]+GHz|\d+-Core Processor", "", model, flags=re.I)
    model = re.sub(r"\s+", " ", model).strip()
    return model.replace("INTEL XEON PLATINUM", "Intel Xeon Platinum").replace("Intel Xeon Platinum", "Intel Xeon")


def stage(rows, mode):
    mine = [r for r in rows if r["mode"] == mode]
    ratios = [r["baseline_over_candidate_median"] for r in mine]
    rounds = [geomean([r["round_medians"][i] for r in mine]) for i in range(len(mine[0]["round_medians"]))]
    return geomean(ratios), rounds, sum(x > 1.0 for x in ratios), sum(x < 0.970 for x in ratios), len(ratios)


lines = [
    "# Numbers", "",
    "Ratios are baseline time over candidate time (median of the paired windows); "
    "**higher is faster, below 1.000 the candidate is slower**.",
    "Every Apple Silicon row measures one branch on its own against the `main` state it started from, named in the "
    "row, so the rows are independent, not cumulative. The x86-64 rows come from the `x86-64 paired benchmark` "
    "workflow, one line per CI host. Generated by `harness/numbers.py` from `results/`.", "",
    "# Apple Silicon (Apple M1 Pro)", "",
]
for title, runs in SECTIONS:
    lines += [f"## {title}", "",
              "| Run | fresh | reuse | parse | render | fresh rounds | reuse rounds | parse rounds | render rounds |",
              "| --- | ---: | ---: | ---: | ---: | --- | --- | --- | --- |"]
    counts = []
    for run in runs:
        summary, grouped = load(run)
        g = grouped["all"]
        lines.append(f"| {label(run)} | " + " | ".join(f"{g[m]['geometric_mean']:.4f}" for m in MODES) + " | "
                     + " | ".join(" / ".join(f"{x:.3f}" for x in g[m]["round_geometric_means"]) for m in MODES) + " |")
        row = []
        for m in MODES:
            _, _, up, low, total = stage(summary, m)
            row.append(f"{up}/{total} ({low})")
        counts.append(f"| {label(run)} | " + " | ".join(row) + " |")
    lines += ["", "Cases faster than the baseline, and in parentheses cases below 0.970:", "",
              "| Run | fresh | reuse | parse | render |", "| --- | ---: | ---: | ---: | ---: |"] + counts + [""]

# Per-document matrices.
for kind, runs, modes in (
    ("screen", SECTIONS[0][1], ("fresh", "parse", "render")),
    ("broad", SECTIONS[1][1], MODES),
    ("recheck", [r for _, rs in SECTIONS[2:] for r in rs], MODES),
):
    per_case = defaultdict(dict)
    for run in runs:
        for row in load(run)[0]:
            per_case[row["mode"], row["case"]][run] = row["baseline_over_candidate_median"]
    names = [r.rsplit("-", 1)[0] if kind != "recheck" else r for r in runs]
    for mode in modes:
        lines += [f"## Per-document {mode} ratios, {kind}", "",
                  "| Document | " + " | ".join(names) + " |", "| --- | " + " | ".join("---:" for _ in runs) + " |"]
        for case in sorted({c for (m, c) in per_case if m == mode}):
            values = per_case[mode, case]
            lines.append(f"| `{case}` | " + " | ".join(f"{values[r]:.3f}" if r in values else "—" for r in runs) + " |")
        lines.append("")
    if kind != "recheck":
        lines += ["`lists2`, `autoidx` and `autoidx2` use `9e741a37` as their baseline, `arm426` uses `af65cb43`, "
                  "the others `060b02d2`.", ""]

# Group breakdown of the broad runs.
cases = json.loads((REPORT / "results" / "cases.json").read_text())["cases"]
runs = SECTIONS[1][1]
for mode in ("parse", "render"):
    groups = defaultdict(lambda: defaultdict(list))
    for run in runs:
        for row in load(run)[0]:
            if row["mode"] != mode:
                continue
            meta = cases[row["case"]]
            groups[f"profile `{meta['profile']}`"][run].append(row["baseline_over_candidate_median"])
            groups[f"category `{meta['category']}`"][run].append(row["baseline_over_candidate_median"])
    lines += [f"## Broad {mode} geomeans by profile and category", "",
              "| Group | Documents | " + " | ".join(r.rsplit("-", 1)[0] for r in runs) + " |",
              "| --- | ---: | " + " | ".join("---:" for _ in runs) + " |"]
    for group in sorted(groups, key=lambda g: (not g.startswith("profile"), g)):
        values = groups[group]
        lines.append(f"| {group} | {len(values[runs[0]])} | " + " | ".join(f"{geomean(values[r]):.3f}" for r in runs) + " |")
    lines.append("")

# x86-64 CI runs.
lines += ["# x86-64 (GitHub-hosted `ubuntu-latest` runners)", "",
          "Every run: 57 broad documents, 3 rounds × 5 pairs, fat-LTO workers built with `-C target-cpu=generic`, "
          "three hosts measuring the same baseline/candidate pair independently. The CPU is whatever GitHub assigned; "
          "`host.txt` in each host directory records it.", ""]
for name, title, base in X86:
    hosts = x86_hosts(name)
    lines += [f"## `{name}`: {title}, base `{base}`", "",
              "| Host | CPU | fresh | reuse | parse | render | fresh rounds | reuse rounds | parse rounds | render rounds |",
              "| --- | --- | ---: | ---: | ---: | ---: | --- | --- | --- | --- |"]
    counts = []
    for number, cpu, rows in hosts:
        stats = {m: stage(rows, m) for m in MODES}
        lines.append(f"| {number} | {cpu} | " + " | ".join(f"{stats[m][0]:.4f}" for m in MODES) + " | "
                     + " | ".join(" / ".join(f"{x:.3f}" for x in stats[m][1]) for m in MODES) + " |")
        counts.append(f"| {number} | {cpu} | " + " | ".join(f"{stats[m][2]}/{stats[m][4]} ({stats[m][3]})" for m in MODES)
                      + " |")
    lines += ["", "Cases faster than the baseline, and in parentheses cases below 0.970:", "",
              "| Host | CPU | fresh | reuse | parse | render |", "| --- | --- | ---: | ---: | ---: | ---: |"] + counts + [""]
    if name == "aa":
        lines += ["Per-case spread of the A/A control (5th and 95th percentile, extremes, cases outside ±3%):", "",
                  "| Host | Stage | p5 | p95 | min | max | outside ±3% |", "| --- | --- | ---: | ---: | ---: | ---: | ---: |"]
        for number, cpu, rows in hosts:
            for m in MODES:
                ratios = sorted(r["baseline_over_candidate_median"] for r in rows if r["mode"] == m)
                q = quantiles(ratios, 20)
                outside = sum(not 0.97 <= x <= 1.03 for x in ratios)
                lines.append(f"| {number} | {m} | {q[0]:.3f} | {q[-1]:.3f} | {ratios[0]:.3f} | {ratios[-1]:.3f} | "
                             f"{outside} |")
        lines.append("")

# x86 per-document matrices: median over the three hosts.
lines += ["## x86-64 per-document ratios, median over the three hosts", "",
          "Each cell is the median of the three hosts' per-case ratios; the baselines differ per column (see above). "
          "[X86-TABLES.md](X86-TABLES.md) lists every host.", ""]
medians = {}
for name, _, _ in X86:
    per = defaultdict(list)
    for _, _, rows in x86_hosts(name):
        for row in rows:
            per[row["mode"], row["case"]].append(row["baseline_over_candidate_median"])
    medians[name] = {key: median(values) for key, values in per.items()}
for mode in MODES:
    lines += [f"### {mode}", "", "| Document | " + " | ".join(name for name, _, _ in X86) + " |",
              "| --- | " + " | ".join("---:" for _ in X86) + " |"]
    for case in sorted({c for (m, c) in medians["aa"] if m == mode}):
        lines.append(f"| `{case}` | " + " | ".join(f"{medians[name][mode, case]:.3f}" for name, _, _ in X86) + " |")
    lines.append("")

OUT.write_text("\n".join(lines))
print("wrote", OUT)
