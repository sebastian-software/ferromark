#!/bin/bash
# usage: measure.sh <name> <screen|broad|recheck> [pairs] — one paired measurement, gated on a quiet machine.
#
# ROUND5_WORK is the scratch directory that held the round (builds, corpus.json, filters, results);
# FERROMARK_REPO is a checkout of this repository whose benchmarks/optimization-rounds runs the pairs.
set -euo pipefail
NAME=$1; KIND=$2; PAIRS=${3:-3}
P=${ROUND5_WORK:?set ROUND5_WORK to the round scratch directory}
H=${FERROMARK_REPO:?set FERROMARK_REPO to a ferromark checkout}/benchmarks/optimization-rounds
LOG=$P/measure-log.txt
while pgrep -x rustc >/dev/null || pgrep -x cargo >/dev/null || pgrep -x clang >/dev/null; do sleep 20; done
echo "[$(date -u +%FT%TZ)] BEFORE $NAME-$KIND :: load $(sysctl -n vm.loadavg | tr -d '{}')" >> "$LOG"
rm -rf "$P/results/$NAME-$KIND"
python3 "$H/run.py" "$P/build-$NAME" "$P/corpus.json" "$P/results/$NAME-$KIND" --rounds 3 --pairs "$PAIRS" --window-ms 40 \
  --filter "$(cat "$P/filter-$KIND.txt")" --modes fresh reuse parse render > "$P/results/$NAME-$KIND.log" 2>&1
python3 "$H/summarize.py" "$P/results/$NAME-$KIND" "$P/results/$NAME-$KIND/grouped.json" > "$P/results/$NAME-$KIND.summary" 2>&1
echo "[$(date -u +%FT%TZ)] AFTER  $NAME-$KIND :: load $(sysctl -n vm.loadavg | tr -d '{}')" >> "$LOG"
head -6 "$P/results/$NAME-$KIND.summary"
