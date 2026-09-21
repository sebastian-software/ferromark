#!/bin/bash
# usage: round-measure.sh <name> <screen|broad> [pairs]
set -euo pipefail
NAME=$1; KIND=$2; PAIRS=${3:-3}
REPO=/Users/sebastian/Workspace/ferromark/.claude/worktrees/homepage-performance-metrics-8a493a
P=/private/tmp/fm-round
H=$REPO/benchmarks/optimization-rounds
CORPUS=/private/tmp/native-release-work/paired-corpus.json
LOG=$P/measure-log.txt
cd "$REPO"
while pgrep -x rustc >/dev/null || pgrep -x cargo >/dev/null || pgrep -x clang >/dev/null; do sleep 20; done
echo "[$(date -u +%FT%TZ)] BEFORE $NAME-$KIND :: load $(sysctl -n vm.loadavg | tr -d '{}')" >> "$LOG"
rm -rf $P/results/$NAME-$KIND
python3 $H/run.py $P/build-$NAME $CORPUS $P/results/$NAME-$KIND --rounds 3 --pairs $PAIRS --window-ms 40 --filter "$(cat $P/filter-$KIND.txt)" --modes fresh reuse parse render > $P/results/$NAME-$KIND.log 2>&1
python3 $H/summarize.py $P/results/$NAME-$KIND $P/results/$NAME-$KIND/grouped.json > $P/results/$NAME-$KIND.summary 2>&1
echo "[$(date -u +%FT%TZ)] AFTER  $NAME-$KIND :: load $(sysctl -n vm.loadavg | tr -d '{}')" >> "$LOG"
head -4 $P/results/$NAME-$KIND.summary
