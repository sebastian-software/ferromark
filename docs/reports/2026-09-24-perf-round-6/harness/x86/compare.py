"""Median-over-hosts ratio per case for several x86 runs side by side.

usage: compare.py <mode> <case-regex> <name>...   (names of results/x86/<name>/, e.g. 431 431b 431inl 431pl)
"""
import gzip
import json
import re
import statistics
import sys
from pathlib import Path

REPORT = Path(__file__).resolve().parent.parent.parent
mode, pattern = sys.argv[1], re.compile(sys.argv[2])
names = sys.argv[3:]
table = {}
for name in names:
    for host in sorted((REPORT / "results" / "x86" / name).glob("host-*")):
        for row in json.loads(gzip.decompress((host / "summary.json.gz").read_bytes())):
            if row["mode"] == mode and pattern.search(row["case"]):
                table.setdefault(row["case"], {}).setdefault(name, []).append(row["baseline_over_candidate_median"])
print(f"{mode:7} {'case':34} " + " ".join(f"{name:>10}" for name in names))
for case in sorted(table):
    cells = []
    for name in names:
        values = table[case].get(name)
        cells.append(f"{statistics.median(values):10.3f}" if values else f"{'-':>10}")
    print(f"{'':7} {case:34} " + " ".join(cells))
