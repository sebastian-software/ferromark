#!/bin/bash
# usage: round-build.sh <name> <candidate-path>  — build a candidate against the round baseline (23a59bdf), reusing its build.
set -euo pipefail
NAME=$1; CAND=$2
REPO=/Users/sebastian/Workspace/ferromark/.claude/worktrees/homepage-performance-metrics-8a493a
P=/private/tmp/fm-round
H=$REPO/benchmarks/optimization-rounds
cd "$REPO"
rm -rf $P/build-$NAME
if [ -f $P/build-aa/build.json ] && [ "$NAME" != aa ]; then REUSE="--reuse-baseline-build $P/build-aa"; else REUSE=""; fi
python3 $H/prepare.py --baseline-path $P/src-base --candidate-path "$CAND" --out $P/build-$NAME --baseline-revision 23a59bdf $REUSE > $P/build-$NAME.log 2>&1 && echo "built $NAME" || { echo "BUILD FAILED $NAME"; tail -20 $P/build-$NAME.log; exit 1; }
