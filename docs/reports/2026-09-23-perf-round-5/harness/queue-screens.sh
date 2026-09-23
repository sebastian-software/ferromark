#!/bin/bash
# Screens in sequence: A/A control first, then each candidate against the same baseline build.
# ROUND5_WORK is the round scratch directory holding measure.sh and the builds.
P=${ROUND5_WORK:?set ROUND5_WORK to the round scratch directory}
for name in aa trim prescan tagfilter; do
  echo "=== $name screen"
  "$P/measure.sh" "$name" screen 3 || { echo "FAILED $name"; tail -5 "$P/results/$name-screen.log"; }
done
