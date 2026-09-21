#!/usr/bin/env python3
"""Stage geomeans per candidate and the per-document matrix: numbers.py <report-dir> <out.md>"""
import json
import sys
from pathlib import Path

report = Path(sys.argv[1])
out = Path(sys.argv[2])
CANDIDATES = [
    ("aa", "A/A control (baseline against itself)"),
    ("norm", "`norm` source normalization, cold path outlined"),
    ("depth", "`depth` inline depth as a plain cell"),
    ("lines", "`lines` container line facts once per line"),
    ("subctx", "`subctx` borrowed sub-parser context"),
    ("arena", "`arena` 2 KB reservation floor"),
    ("emph", "`emph` one delimiter buffer per parse"),
    ("closer", "`closer` forward window for has_closer_from"),
    ("tables", "`tables` NEON pipe cursor"),
]
MODES = ("fresh", "reuse", "parse", "render")


def load(name):
    directory = report / "results" / name
    return json.loads((directory / "summary.json").read_text()), json.loads((directory / "grouped.json").read_text())


lines = ["# Numbers", "",
         "Ratios are baseline time over candidate time (median of the paired windows); **higher is faster, below 1.000 the candidate is slower**.",
         "The baseline is `23a59bdf` for every candidate; each candidate is one branch on its own, so the rows are independent, not cumulative.", ""]
for kind, label_kind, pairs in (("screen", "Screens: 20 documents and 16 diagnostics, 3 rounds × 3 pairs", 3), ("broad", "Broad confirmations: 57 documents, 3 rounds × 5 pairs", 5)):
    present = [(n, l) for n, l in CANDIDATES if (report / "results" / f"{n}-{kind}").exists()]
    if not present:
        continue
    lines += [f"## {label_kind}", "", "| Candidate | fresh | reuse | parse | render | fresh rounds | parse rounds |", "| --- | ---: | ---: | ---: | ---: | --- | --- |"]
    per_case = {}
    for name, label in present:
        summary, grouped = load(f"{name}-{kind}")
        g = grouped["all"]
        lines.append(f"| {label} | " + " | ".join(f"{g[m]['geometric_mean']:.4f}" for m in MODES)
                     + " | " + " / ".join(f"{x:.3f}" for x in g["fresh"]["round_geometric_means"])
                     + " | " + " / ".join(f"{x:.3f}" for x in g["parse"]["round_geometric_means"]) + " |")
        for row in summary:
            per_case.setdefault((row["mode"], row["case"]), {})[name] = row["baseline_over_candidate_median"]
    lines.append("")
    for mode in ("fresh", "parse"):
        lines += [f"### Per-document {mode} ratios, {kind}", "",
                  "| Document | " + " | ".join(n for n, _ in present) + " |",
                  "| --- | " + " | ".join("---:" for _ in present) + " |"]
        cases = sorted({case for (m, case) in per_case if m == mode})
        for case in cases:
            values = per_case[mode, case]
            lines.append(f"| `{case}` | " + " | ".join(f"{values[n]:.3f}" if n in values else "—" for n, _ in present) + " |")
        lines.append("")
out.write_text("\n".join(lines) + "\n")
print("wrote", out)
