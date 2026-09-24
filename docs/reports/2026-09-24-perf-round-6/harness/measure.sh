#!/bin/bash
# usage: measure.sh <name> <filter-name> [pairs] — one paired measurement, gated on a quiet machine.
# Waits until no rustc/cargo/clang runs and the one-minute load is below 8 (test binaries
# of implementation agents load the CPU without showing up as compilers); gives up waiting
# after 20 minutes and records that in the log. The load gate was added after the heading-id
# runs of #422 (see README); the runs before it waited for compilers only.
#
# ROUND6_WORK is the scratch directory that held the round (builds, corpus.json, filter-*.txt,
# results); FERROMARK_REPO is a checkout whose benchmarks/optimization-rounds runs the pairs (the
# round ran the copy in its 060b02d2 export; runner and worker hashes are in every run.json).
set -euo pipefail
NAME=$1; KIND=$2; PAIRS=${3:-3}
P=${ROUND6_WORK:?set ROUND6_WORK to the round scratch directory}
H=${FERROMARK_REPO:?set FERROMARK_REPO to a ferromark checkout}/benchmarks/optimization-rounds
LOG=$P/measure-log.txt
quiet() {
  ! pgrep -x rustc >/dev/null && ! pgrep -x cargo >/dev/null && ! pgrep -x clang >/dev/null \
    && awk -v load="$(sysctl -n vm.loadavg | awk '{print $2}')" 'BEGIN { exit !(load < 8) }'
}
waited=0
until quiet; do
  sleep 20
  waited=$((waited + 20))
  if [ "$waited" -ge 1200 ]; then echo "[$(date -u +%FT%TZ)] GAVE UP WAITING for quiet before $NAME-$KIND" >> "$LOG"; break; fi
done
echo "[$(date -u +%FT%TZ)] BEFORE $NAME-$KIND :: load $(sysctl -n vm.loadavg | tr -d '{}')" >> "$LOG"
rm -rf "$P/results/$NAME-$KIND"
python3 "$H/run.py" "$P/build-$NAME" "$P/corpus.json" "$P/results/$NAME-$KIND" --rounds 3 --pairs "$PAIRS" --window-ms 40 \
  --filter "$(cat "$P/filter-$KIND.txt")" --modes fresh reuse parse render > "$P/results/$NAME-$KIND.log" 2>&1
python3 "$H/summarize.py" "$P/results/$NAME-$KIND" "$P/results/$NAME-$KIND/grouped.json" > "$P/results/$NAME-$KIND.summary" 2>&1
echo "[$(date -u +%FT%TZ)] AFTER  $NAME-$KIND :: load $(sysctl -n vm.loadavg | tr -d '{}')" >> "$LOG"
head -6 "$P/results/$NAME-$KIND.summary"
