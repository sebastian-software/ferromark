#!/usr/bin/env python3
"""Compare retained feature-probe executables with exact HTML and paired timing."""
import argparse
import hashlib
import json
import pathlib
import statistics
import subprocess

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("before", type=pathlib.Path)
parser.add_argument("after", type=pathlib.Path)
parser.add_argument("catalog", type=pathlib.Path)
parser.add_argument("output", type=pathlib.Path)
parser.add_argument("--rounds", type=int, default=7)
parser.add_argument("--window-ms", type=int, default=100)
parser.add_argument(
    "--filters", nargs="+",
    help="Case ID substrings to time; exact HTML checks always cover the full catalog",
)
args = parser.parse_args()
assert args.rounds > 0 and args.window_ms > 0
binaries = {"before": args.before.resolve(), "after": args.after.resolve()}


def run(binary, mode, case_filter="all", lane="all"):
    result = subprocess.run(
        [str(binary), mode, str(args.catalog), case_filter, lane,
         str(args.window_ms), "1"],
        capture_output=True, text=True, check=True,
    )
    rows = json.loads(result.stdout)
    assert rows, f"No cases matched {case_filter!r}"
    return rows


html = {key: run(binary, "html") for key, binary in binaries.items()}
assert html["before"] == html["after"], "HTML changed; aborting measurement"
filters = args.filters or [
    "lifecycle/", "syntax/heading_ids/medium", "syntax/allow_link_refs/medium",
    "syntax/footnotes/medium", "syntax/tables/medium", "core/emphasis/medium",
    "core/links/medium", "core/inline_code/medium",
]
rows = []
for round_index in range(args.rounds):
    offset = round_index % len(filters)
    order = filters[offset:] + filters[:offset]
    for case_filter in order:
        labels = ["before", "after"] if round_index % 2 == 0 else ["after", "before"]
        pair = {label: run(binaries[label], "measure", case_filter) for label in labels}
        for first, second in zip(pair["before"], pair["after"], strict=True):
            key = (first["id"], first["variant"], first["lane"])
            assert key == (second["id"], second["variant"], second["lane"])
            rows.append({
                "id": key[0], "variant": key[1], "lane": key[2],
                "round": round_index, "before": first["ns"], "after": second["ns"],
            })
    print(f"round {round_index + 1}/{args.rounds}", flush=True)

summary = []
for key in dict.fromkeys((r["id"], r["variant"], r["lane"]) for r in rows):
    group = [r for r in rows if (r["id"], r["variant"], r["lane"]) == key]
    deltas = [(r["after"] / r["before"] - 1) * 100 for r in group]
    summary.append({
        "id": key[0], "variant": key[1], "lane": key[2],
        "before_ns": statistics.median(r["before"] for r in group),
        "after_ns": statistics.median(r["after"] for r in group),
        "paired_change_percent": statistics.median(deltas),
        "paired_min_percent": min(deltas), "paired_max_percent": max(deltas),
    })

result = {
    "exact_html_pairs": len(html["before"]),
    "catalog_sha256": hashlib.sha256(args.catalog.read_bytes()).hexdigest(),
    "binary_sha256": {
        key: hashlib.sha256(binary.read_bytes()).hexdigest()
        for key, binary in binaries.items()
    },
    "rounds": args.rounds, "window_ms": args.window_ms, "filters": filters,
    "summary": summary, "samples": rows,
}
args.output.parent.mkdir(parents=True, exist_ok=True)
args.output.write_text(json.dumps(result, indent=2) + "\n")
for row in summary:
    if row["variant"] in ["on", "syntax", "default", "commonmark"]:
        print(f"{row['id']:40} {row['variant']:10} {row['lane']:8} "
              f"{row['paired_change_percent']:+6.2f}%")
