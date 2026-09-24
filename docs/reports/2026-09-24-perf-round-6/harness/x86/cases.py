"""Per-case ratios across hosts for one stage of one x86 run: cases.py <name> <mode> [limit]

Reads results/x86/<name>/host-N/ of the report this harness belongs to and prints the cases sorted by
their median ratio over hosts, worst first and best last.
"""
import gzip
import json
import statistics
import sys
from pathlib import Path

REPORT = Path(__file__).resolve().parent.parent.parent
run, mode = REPORT / "results" / "x86" / sys.argv[1], sys.argv[2]
limit = int(sys.argv[3]) if len(sys.argv) > 3 else 8
table = {}
for host in sorted(run.glob("host-*")):
    for row in json.loads(gzip.decompress((host / "summary.json.gz").read_bytes())):
        if row["mode"] == mode:
            table.setdefault(row["case"], []).append(row["baseline_over_candidate_median"])
ranked = sorted(table.items(), key=lambda item: statistics.median(item[1]))
for case, values in ranked[:limit] + [("...", [])] + ranked[-limit:]:
    if not values:
        print("   ...")
        continue
    print(f"   {case:40} median {statistics.median(values):.3f}  hosts {' '.join(f'{v:.3f}' for v in values)}")
