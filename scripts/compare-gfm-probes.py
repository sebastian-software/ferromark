#!/usr/bin/env python3
"""Alternate uninstrumented probe binaries; verify HTML before timing."""
import argparse
import hashlib
import json
import statistics
import subprocess
from pathlib import Path

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("before", type=Path)
parser.add_argument("after", type=Path)
parser.add_argument("output", type=Path)
parser.add_argument("--rounds", type=int, default=7)
parser.add_argument("--window-ms", type=int, default=150)
parser.add_argument("--lanes", default="buffer,renderer")
args = parser.parse_args()
assert args.rounds > 0 and args.window_ms > 0
binaries = {"before": args.before.resolve(), "after": args.after.resolve()}


def invoke(binary, *arguments):
    result = subprocess.run([str(binary), *arguments], check=True, capture_output=True, text=True)
    return json.loads(result.stdout)


html = {label: invoke(binary, "html") for label, binary in binaries.items()}
assert html["before"] == html["after"], "HTML changed; timing comparison aborted"
configurations = [
    ("plain", "commonmark"), ("plain", "gfm"),
    ("commonmark-5k", "commonmark"), ("commonmark-5k", "gfm"),
    ("commonmark-50k", "commonmark"), ("commonmark-50k", "gfm"),
    ("gfm-overlap-tables", "overlap"), ("gfm-overlap-tables", "gfm"),
    ("tables-5k", "gfm"), ("autolinks", "gfm"), ("tasks", "gfm"),
    ("mixed-gfm", "gfm"), ("readme", "commonmark"), ("readme", "gfm"),
]
rows = []
for round_index in range(args.rounds):
    order = configurations[round_index:] + configurations[:round_index]
    if round_index % 2:
        order.reverse()
    for case, preset in order:
        for lane in args.lanes.split(","):
            labels = ["before", "after"] if round_index % 2 == 0 else ["after", "before"]
            pair = {}
            for label in labels:
                row = invoke(binaries[label], "measure", case, preset, lane,
                             str(args.window_ms), "1")[0]
                pair[label] = row["ns"]
            rows.append({"round": round_index, "case": case, "preset": preset,
                         "lane": lane, **pair})
    print(f"round {round_index + 1}/{args.rounds}", flush=True)
summary = []
for case, preset in configurations:
    for lane in args.lanes.split(","):
        group = [r for r in rows if (r["case"], r["preset"], r["lane"]) == (case, preset, lane)]
        changes = [(r["after"] / r["before"] - 1) * 100 for r in group]
        summary.append({"case": case, "preset": preset, "lane": lane,
                        "before_ns": statistics.median(r["before"] for r in group),
                        "after_ns": statistics.median(r["after"] for r in group),
                        "paired_change_percent": statistics.median(changes),
                        "paired_min_percent": min(changes), "paired_max_percent": max(changes)})
result = {"binary_sha256": {k: hashlib.sha256(v.read_bytes()).hexdigest() for k, v in binaries.items()},
          "html_configurations_verified": len(html["before"]), "rounds": args.rounds,
          "window_ms": args.window_ms, "change_sign": "negative means faster",
          "summary": summary, "samples": rows}
args.output.parent.mkdir(parents=True, exist_ok=True)
args.output.write_text(json.dumps(result, indent=2) + "\n")
for r in summary:
    print(f"{r['case']:20} {r['preset']:10} {r['lane']:8} {r['paired_change_percent']:+6.2f}%")
