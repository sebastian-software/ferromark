#!/usr/bin/env python3
"""Stage geomeans per screen and the per-document matrix across screens: numbers.py <report-dir> <out.md>"""
import json
import sys
from pathlib import Path

report = Path(sys.argv[1])
out = Path(sys.argv[2])
SCREENS = [
    ("aa", "A/A control (baseline against itself)"),
    ("c369", "`2887b2bd` #369 bound inline bracket nesting"),
    ("c372", "`e8439fd8` #372 bound emphasis nesting"),
    ("c374", "`ca82ec2f` #374 linear nested link probing"),
    ("c377", "`c84743cb` #376/#377 base URL, MDX closer memo"),
    ("c380", "`1037fb5d` #380 renderer fixes"),
    ("c382", "`e532dcf4` #382 tabs, HTML closers, MDX flow"),
    ("c383", "`8d957e30` #383 laziness, container and inline cost bounds"),
    ("head-screen", "`bffc89f6` #384 linear math/MDX/definition-list scans (HEAD)"),
    ("fix1-screen", "HEAD + tracker classifiers gated on the first byte"),
    ("abl-emph-screen", "ablation: the same, with `emphasis.rs` as of #382 (measures the delimiter list of #383)"),
    ("fix2-screen", "HEAD + tracker catching up on demand"),
    ("fix3-screen", "HEAD + on-demand tracker + lazily allocated memos"),
    ("fix4-screen", "HEAD + on-demand tracker + lazy memos + closer front cache (the fix)"),
]
BROAD = [
    ("head-broad", "HEAD `bffc89f6`"),
    ("fix1-broad", "HEAD + first-byte gate"),
    ("fix2-broad", "HEAD + on-demand catch-up"),
    ("fix3-broad", "HEAD + on-demand catch-up + lazy memos"),
    ("fix4-broad", "HEAD + on-demand catch-up + lazy memos + closer front cache (the fix)"),
]
MODES = ("fresh", "reuse", "parse", "render")


def load(name):
    directory = report / "results" / name
    return json.loads((directory / "summary.json").read_text()), json.loads((directory / "grouped.json").read_text())


lines = ["# Numbers", "",
         "Ratios are baseline time over candidate time (median of the paired windows); **higher is faster, below 1.000 the candidate is slower**.",
         "The baseline is `c4af9525` in every screen — the core the preceding native comparison measured as `7c887a2b`.",
         "Each candidate is the named revision of `main`, so every screen is cumulative: the difference between two consecutive rows is what the commits between them cost.", ""]

# Broad confirmations.
lines += ["## Against the baseline on the 57 broad documents", "",
          "| Candidate | Stage | N | Geomean | Rounds | Above 1 | Below 1 |", "| --- | --- | ---: | ---: | --- | ---: | ---: |"]
broad = {}
for name, label in BROAD:
    if not (report / "results" / name).exists():
        continue
    summary, grouped = load(name)
    broad[name] = grouped
    for mode in MODES:
        g = grouped["all"][mode]
        lines.append(f"| {label} | {mode} | {g['cases']} | {g['geometric_mean']:.4f} | {' / '.join(f'{x:.4f}' for x in g['round_geometric_means'])} | {g['measured_above_1']} | {g['measured_below_1']} |")
lines += ["", "By category and size group, all stages:", "",
          "| Group | N | Candidate | fresh | reuse | parse | render |", "| --- | ---: | --- | ---: | ---: | ---: | ---: |"]
for kind in ("categories", "sizes"):
    for group in broad["head-broad"][kind]:
        for name, label in BROAD:
            if name not in broad:
                continue
            values = broad[name][kind][group]
            n = values["fresh"]["cases"]
            lines.append(f"| {group} | {n} | {label} | " + " | ".join(f"{values[m]['geometric_mean']:.4f}" for m in MODES) + " |")
lines.append("")

# Screens.
lines += ["## Stage geomeans per screen, 16 documents", "",
          "| Candidate | fresh | reuse | parse | render |", "| --- | ---: | ---: | ---: | ---: |"]
per_case = {}
for name, label in SCREENS:
    if not (report / "results" / name).exists():
        continue
    summary, grouped = load(name)
    lines.append(f"| {label} | " + " | ".join(f"{grouped['all'][m]['geometric_mean']:.4f}" for m in MODES) + " |")
    for row in summary:
        per_case.setdefault((row["mode"], row["case"]), {})[name] = row["baseline_over_candidate_median"]
lines.append("")

for mode in ("parse", "fresh"):
    present = [(name, label) for name, label in SCREENS if (report / "results" / name).exists()]
    lines += [f"## Per-document {mode} ratios across screens", "",
              "| Document | " + " | ".join(name for name, _ in present) + " |",
              "| --- | " + " | ".join("---:" for _ in present) + " |"]
    cases = sorted({case for (m, case) in per_case if m == mode}, key=lambda c: per_case[mode, c].get("head-screen", 1.0))
    for case in cases:
        values = per_case[mode, case]
        lines.append(f"| `{case}` | " + " | ".join(f"{values[name]:.3f}" if name in values else "—" for name, _ in present) + " |")
    lines.append("")

out.write_text("\n".join(lines) + "\n")
print("wrote", out)
