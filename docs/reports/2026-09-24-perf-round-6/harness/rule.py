"""Decision-rule check per run: stage geomeans, per-round geomeans, cases below 0.970.

usage: rule.py <run-name>...   (reads results/<run>/summary.json of the report this harness belongs to)
"""
import json
import math
import sys
from pathlib import Path

REPORT = Path(__file__).resolve().parent.parent
MODES = ("fresh", "reuse", "parse", "render")


def geomean(values):
    return math.exp(sum(math.log(v) for v in values) / len(values))


for run in sys.argv[1:]:
    rows = json.loads((REPORT / "results" / run / "summary.json").read_text())
    print(f"== {run}")
    for mode in MODES:
        mine = [r for r in rows if r["mode"] == mode]
        if not mine:
            continue
        overall = geomean([r["baseline_over_candidate_median"] for r in mine])
        rounds = len(mine[0]["round_medians"])
        per_round = [geomean([r["round_medians"][i] for r in mine]) for i in range(rounds)]
        low = sorted((r["baseline_over_candidate_median"], r["case"]) for r in mine if r["baseline_over_candidate_median"] < 0.970)
        up = sum(r["baseline_over_candidate_median"] > 1.0 for r in mine)
        best = sorted(((r["baseline_over_candidate_median"], r["case"]) for r in mine), reverse=True)[:4]
        print(f"  {mode:6s} {overall:.3f}  rounds {' / '.join(f'{x:.3f}' for x in per_round)}  up {up}/{len(mine)}"
              f"  <0.970: {', '.join(f'{c} {v:.3f}' for v, c in low) or 'none'}")
        print(f"         best: {', '.join(f'{c} {v:.3f}' for v, c in best)}")
