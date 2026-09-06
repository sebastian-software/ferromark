#!/usr/bin/env python3
"""Alternate retained executables, verify HTML, and save paired timing evidence."""
import argparse
import hashlib
import json
import statistics
import subprocess
from pathlib import Path

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("baseline")
parser.add_argument("candidate")
parser.add_argument("output", type=Path)
parser.add_argument("--pairs", type=int, default=3)
parser.add_argument("--window-ms", type=int, default=30)
parser.add_argument("--filter", default="")
parser.add_argument("--preset", default="")
args = parser.parse_args()
if args.pairs < 1 or args.window_ms < 1:
    parser.error("pairs and window-ms must be positive")
runs = []
expected = {}
expected_keys = None
for pair in range(args.pairs):
    order = ["baseline", "candidate"] if pair % 2 == 0 else ["candidate", "baseline"]
    run = {}
    for role in order:
        rows = json.loads(subprocess.check_output([
            getattr(args, role), str(args.window_ms), args.filter, args.preset,
        ]))
        keys = [(row["case"], row["preset"], row["lane"]) for row in rows]
        assert keys, "no cases matched the filters"
        if expected_keys is None:
            expected_keys = keys
        assert keys == expected_keys, "baseline/candidate case matrices differ"
        for row in rows:
            key = (row["case"], row["preset"], row["lane"])
            html = row.pop("html")
            if key in expected:
                assert html == expected[key], f"HTML changed: {key}"
            else:
                expected[key] = html
            row["html_sha256"] = hashlib.sha256(html.encode()).hexdigest()
        run[role] = rows
    runs.append(run)
    print(f"pair {pair + 1}/{args.pairs} complete; HTML identical", flush=True)
summary = []
for i, row in enumerate(runs[0]["baseline"]):
    ratios = [run["candidate"][i]["ns"] / run["baseline"][i]["ns"] for run in runs]
    result = {key: row[key] for key in ["case", "preset", "lane"]}
    result.update(change_percent=100 * (statistics.median(ratios) - 1), ratios=ratios)
    summary.append(result)
args.output.parent.mkdir(parents=True, exist_ok=True)
args.output.write_text(json.dumps({"arguments": vars(args) | {"output": str(args.output)},
                                 "runs": runs, "summary": summary}, indent=2) + "\n")
for row in summary:
    print(f'{row["case"]:18} {row["preset"]:10} {row["lane"]:6} {row["change_percent"]:+6.2f}%')
