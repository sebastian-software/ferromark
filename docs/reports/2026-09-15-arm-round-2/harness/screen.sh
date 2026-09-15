#!/bin/zsh
# usage: screen.sh <build-dir> <result-name> <filter-regex> <modes...>
set -e
S=/private/tmp/claude-501/-Users-sebastian-Workspace-ferromark--claude-worktrees-code-audit-review-bb9235/487e9fd2-c2bd-451b-8ce7-afd6d14441ca/scratchpad
H=/Users/sebastian/Workspace/ferromark/.claude/worktrees/code-audit-review-bb9235/benchmarks/optimization-rounds
BUILD=$1; NAME=$2; FILTER=$3; shift 3
OUT=$S/results/$NAME
rm -rf $OUT
python3 $H/run.py $BUILD $S/corpus.json $OUT --rounds ${ROUNDS:-3} --pairs ${PAIRS:-3} --window-ms ${WINDOW:-40} --filter "$FILTER" --modes "$@" > $OUT.log 2>&1
python3 $H/summarize.py $OUT $OUT/grouped.json > $OUT.summary
python3 - "$OUT" <<'PY'
import json,sys,statistics
out=sys.argv[1]
rows=json.load(open(out+'/summary.json'))
g=json.load(open(out+'/grouped.json'))
print("== geomeans (baseline/candidate; >1 = candidate faster) ==")
for mode,v in g['all'].items():
    print(f"{mode:7s} n={v['cases']:3d} geo={v['geometric_mean']:.4f} rounds={[round(x,4) for x in v['round_geometric_means']]} above1={v['measured_above_1']} below1={v['measured_below_1']}")
for cat,cv in g['categories'].items():
    print(f"  [{cat}] " + " ".join(f"{m}={v['geometric_mean']:.4f}" for m,v in cv.items()))
print("== per case ==")
for r in sorted(rows,key=lambda r:(r['mode'],r['case'])):
    print(f"{r['mode']:7s} {r['case']:45s} {r['baseline_over_candidate_median']:.4f}  [{r['baseline_over_candidate_min']:.3f},{r['baseline_over_candidate_max']:.3f}] rounds={[round(x,3) for x in r['round_medians']]} base={r['baseline_ns_per_document']/1000:.1f}us")
PY
