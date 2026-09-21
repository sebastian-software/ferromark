#!/bin/bash
# usage: paired-fix.sh <name>  — build this worktree's core against baseline c4af9525, then screen + broad.
set -euo pipefail
NAME=$1; CAND=$2
REPO=/Users/sebastian/Workspace/ferromark/.claude/worktrees/homepage-performance-metrics-8a493a
P=/private/tmp/fm-paired
H=$REPO/benchmarks/optimization-rounds
CORPUS=/private/tmp/native-release-work/paired-corpus.json
LOG=$P/measure-log.txt
cd "$REPO"
rm -rf $P/build-$NAME
python3 $H/prepare.py --baseline-path $P/src-base --candidate-path $CAND --out $P/build-$NAME --baseline-revision c4af9525 --reuse-baseline-build $P/build-head > $P/build-$NAME.log 2>&1 || { echo "BUILD FAILED"; tail -20 $P/build-$NAME.log; exit 1; }
echo "built $NAME"
gate() { while pgrep -x rustc >/dev/null || pgrep -x cargo >/dev/null || pgrep -x clang >/dev/null; do sleep 20; done; }
stamp() { echo "[$(date -u +%FT%TZ)] $1 :: load $(sysctl -n vm.loadavg | tr -d '{}')" >> "$LOG"; }
screen() {
  local build=$1 name=$2 filter=$3 pairs=$4; shift 4
  gate; stamp "BEFORE $name"
  rm -rf $P/results/$name
  python3 $H/run.py $P/$build $CORPUS $P/results/$name --rounds 3 --pairs $pairs --window-ms 40 --filter "$(cat $filter)" --modes "$@" > $P/results/$name.log 2>&1
  python3 $H/summarize.py $P/results/$name $P/results/$name/grouped.json > $P/results/$name.summary 2>&1
  stamp "AFTER  $name"
}
screen build-$NAME $NAME-screen $P/filter-screen.txt 3 fresh reuse parse render
echo "[$(date -u +%FT%TZ)] $NAME DONE" >> "$LOG"
