"""Stage geomeans, round geomeans and per-case ratio quantiles per host of one x86 run: spread.py [name]

Reads results/x86/<name>/host-N/ of the report this harness belongs to (default: the A/A control `aa`).
"""
import gzip
import json
import math
import statistics
import sys
from pathlib import Path

REPORT = Path(__file__).resolve().parent.parent.parent
RUN = REPORT / "results" / "x86" / (sys.argv[1] if len(sys.argv) > 1 else "aa")


def geomean(values):
    return math.exp(sum(math.log(v) for v in values) / len(values))


for host_dir in sorted(RUN.glob("host-*")):
    host = (host_dir / "host.txt").read_text().strip().splitlines()
    rows = json.loads(gzip.decompress((host_dir / "summary.json.gz").read_bytes()))
    print(f"== {host_dir.name}: {' | '.join(line for line in host if 'Model' in line or 'simd' in line or 'cpus' in line)}")
    for mode in ("fresh", "reuse", "parse", "render"):
        mine = [r for r in rows if r["mode"] == mode]
        ratios = sorted(r["baseline_over_candidate_median"] for r in mine)
        rounds = [geomean([r["round_medians"][i] for r in mine]) for i in range(len(mine[0]["round_medians"]))]
        q = statistics.quantiles(ratios, n=20)
        outside = sum(not (0.97 <= x <= 1.03) for x in ratios)
        print(f"   {mode:6s} geomean {geomean(ratios):.4f} rounds {' / '.join(f'{x:.3f}' for x in rounds)} "
              f"p5 {q[0]:.3f} p95 {q[-1]:.3f} min {ratios[0]:.3f} max {ratios[-1]:.3f} outside±3% {outside}/{len(ratios)}")
