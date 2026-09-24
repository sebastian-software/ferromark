#!/usr/bin/env python3
"""Side-by-side per-case ratios of several runs: percase.py <run-name>...  (e.g. aa6-recheck422 headingid-recheck422)"""
import json
import sys
from pathlib import Path

REPORT = Path(__file__).resolve().parent.parent
names = sys.argv[1:]
data = {n: {(r["mode"], r["case"]): r for r in json.loads((REPORT / "results" / n / "summary.json").read_text())} for n in names}
keys = sorted({k for d in data.values() for k in d})
print(f"{'mode':7s} {'case':44s} " + " ".join(f"{n:>20s}" for n in names))
for k in keys:
    row = []
    for n in names:
        r = data[n].get(k)
        row.append(f"{r['baseline_over_candidate_median']:20.3f}" if r else f"{'-':>20s}")
    print(f"{k[0]:7s} {k[1]:44s} " + " ".join(row))
