#!/usr/bin/env python3
"""Archive this SIMD experiment and derive tables from its paired measurements."""

import argparse
import gzip
import json
import math
from pathlib import Path
import shutil

from prepare import verify_baseline

MODES = ("fresh", "reuse", "parse", "render")
SCREENS = {
    "repeated-search": ("screen-fixed", "source.patch"),
    "search-after-scalar": ("screen-hybrid", "hybrid.patch"),
    "eight-byte-probe": ("screen-local", "local.patch"),
    "single-probe": ("screen-once", "once.patch"),
}


def archive_run(source, target):
    target.mkdir(parents=True, exist_ok=True)
    for name in ("samples.json", "verification.json", "arena-capacities.json"):
        (target / f"{name}.gz").write_bytes(gzip.compress((source / name).read_bytes(), mtime=0))
    for name in ("build.json", "run.json", "summary.json", "summary.csv"):
        shutil.copyfile(source / name, target / name)


def geomean(values):
    values = list(values)
    return math.exp(sum(map(math.log, values)) / len(values))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("experiment", type=Path, help="Directory containing measured/, screens, builds, and patches")
    parser.add_argument("report", type=Path)
    args = parser.parse_args()
    root, report = args.experiment, args.report
    report.mkdir(parents=True, exist_ok=True)
    archive_run(root / "measured", report)
    for directory in ("followup", "same-binary-control"):
        archive_run(root / directory, report / directory)
    (report / "source-verification.json").write_text(
        json.dumps(verify_baseline(root / "baseline"), indent=2) + "\n"
    )
    (report / "corpus.json.gz").write_bytes(gzip.compress((root / "corpus.json").read_bytes(), mtime=0))
    shutil.copyfile(root / "once.patch", report / "candidate.patch")
    for label, (directory, patch) in SCREENS.items():
        target = report / "screens" / label
        archive_run(root / directory, target)
        shutil.copyfile(root / patch, target / "candidate.patch")
    for name in ("allocations.json", "build.json"):
        (report / f"allocation-{name}.gz").write_bytes(
            gzip.compress((root / "allocations-checked" / name).read_bytes(), mtime=0)
        )
    for engine in ("baseline", "candidate"):
        for label, directory in (("timing", "build-once"), ("allocation", "alloc-build-checked")):
            shutil.copyfile(root / directory / engine / "Cargo.lock", report / f"{label}-{engine}.Cargo.lock")

    corpus = json.loads((root / "corpus.json").read_text())["cases"]
    summary = json.loads((root / "measured/summary.json").read_text())
    rows = {(row["case"], row["mode"]): row for row in summary}
    broad = [case for case in corpus if case["suite"] == "broad"]
    groups = {"All broad cases": broad}
    for category in sorted({case["category"] for case in broad}):
        groups[category] = [case for case in broad if case["category"] == category]
    for size in dict.fromkeys(case["size_bin"] for case in broad):
        groups[size] = [case for case in broad if case["size_bin"] == size]
    aggregates = []
    for group, cases in groups.items():
        item = {"group": group, "n": len(cases), "modes": {}}
        for mode in MODES:
            selected = [rows[case["name"], mode] for case in cases]
            item["modes"][mode] = {
                "geomean_paired_ratio": geomean(row["baseline_over_candidate_median"] for row in selected),
                "round_geomeans": [geomean(row["round_medians"][index] for row in selected)
                                   for index in range(len(selected[0]["round_medians"]))],
            }
        aggregates.append(item)
    (report / "aggregates.json").write_text(json.dumps(aggregates, indent=2) + "\n")
    lines = ["# Derived measurements", "", "Ratios are baseline time / candidate time; values above 1 favor the candidate.",
             "The aggregate is the geometric mean of each document's median paired ratio.",
             "Broad groups exclude synthetic diagnostics. Wikipedia views overlap.", "",
             "| Group | Cases | Fresh | Reuse | Parse | Render control |", "| --- | ---: | ---: | ---: | ---: | ---: |"]
    for row in aggregates:
        ratios = " | ".join(f'{row["modes"][mode]["geomean_paired_ratio"]:.3f}' for mode in MODES)
        lines.append(f'| {row["group"]} | {row["n"]} | {ratios} |')
    lines += ["", "## Individual cases", "", "Absolute nanosecond measurements and all paired/round ranges are in `summary.csv`.", "",
              "| Case | Bytes | Fresh | Reuse | Parse | Render control | Reuse round range |",
              "| --- | ---: | ---: | ---: | ---: | ---: | ---: |"]
    for case in corpus:
        ratios = " | ".join(f'{rows[case["name"], mode]["baseline_over_candidate_median"]:.3f}' for mode in MODES)
        rounds = rows[case["name"], "reuse"]["round_medians"]
        lines.append(f'| {case["name"]} | {case["byte_count"]} | {ratios} | {min(rounds):.3f}–{max(rounds):.3f} |')
    (report / "RESULTS.md").write_text("\n".join(lines) + "\n")
    print(report / "RESULTS.md")


if __name__ == "__main__":
    main()
