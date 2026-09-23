#!/usr/bin/env python3
"""Render one paired run as Markdown: stage geomeans, rounds, and the cases a decision reads.

usage: step_summary.py <run-dir> [--title TEXT] [--host FILE]

Reads `summary.json` (written by run.py) and `grouped.json` (written by
summarize.py) from the run directory. Ratios are baseline time over candidate
time: above 1.000 the candidate is faster.
"""
import argparse
import json
from pathlib import Path

FLOOR = 0.970


def render(run: Path, title: str, host: str | None = None) -> str:
    rows = json.loads((run / "summary.json").read_text())
    grouped = json.loads((run / "grouped.json").read_text())["all"]
    lines = [f"### {title}", ""]
    if host:
        lines += ["```text", host.strip(), "```", ""]
    lines += [
        "| Stage | Cases | Geomean | Rounds | Above 1 | Below 0.970 |",
        "| --- | ---: | ---: | --- | ---: | --- |",
    ]
    for mode, group in grouped.items():
        low = sorted(
            (r["baseline_over_candidate_median"], r["case"])
            for r in rows
            if r["mode"] == mode and r["baseline_over_candidate_median"] < FLOOR
        )
        rounds = " / ".join(f"{value:.3f}" for value in group["round_geometric_means"])
        below = ", ".join(f"`{case}` {value:.3f}" for value, case in low) or "none"
        lines.append(
            f"| {mode} | {group['cases']} | {group['geometric_mean']:.4f} | {rounds} "
            f"| {group['measured_above_1']} | {below} |"
        )
    lines += ["", "Ratios are baseline time over candidate time; above 1.000 the candidate is faster.", ""]
    return "\n".join(lines)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("run", type=Path)
    parser.add_argument("--title", default="Paired run")
    parser.add_argument("--host", type=Path)
    args = parser.parse_args()
    host = args.host.read_text() if args.host else None
    print(render(args.run, args.title, host))


if __name__ == "__main__":
    main()
