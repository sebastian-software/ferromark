#!/bin/bash
# Merge re-screens on top of each other, then the cumulative effect against the round baseline.
# ROUND5_WORK is the round scratch directory holding measure.sh and the builds.
P=${ROUND5_WORK:?set ROUND5_WORK to the round scratch directory}
for run in "stack1 screen 3" "stack2 screen 3" "total broad 5"; do
  set -- $run
  echo "=== $1 $2"
  "$P/measure.sh" "$1" "$2" "$3" || { echo "FAILED $1"; tail -5 "$P/results/$1-$2.log"; }
done
