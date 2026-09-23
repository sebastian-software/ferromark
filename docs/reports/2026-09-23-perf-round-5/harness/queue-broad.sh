#!/bin/bash
# Broad confirmations (57 documents, 3 rounds x 5 pairs), then the Node-level comparison for #412.
# ROUND5_WORK is the round scratch directory; measure.sh, node-bench.mjs and
# optimization-rounds/run.py sit in it next to corpus.json and the two addons.
P=${ROUND5_WORK:?set ROUND5_WORK to the round scratch directory}
for name in trim prescan tagfilter; do
  echo "=== $name broad"
  "$P/measure.sh" "$name" broad 5 || { echo "FAILED $name"; tail -5 "$P/results/$name-broad.log"; }
done
echo "=== node #412"
while pgrep -x rustc >/dev/null || pgrep -x cargo >/dev/null || pgrep -x clang >/dev/null; do sleep 20; done
echo "[$(date -u +%FT%TZ)] BEFORE node-412 :: load $(sysctl -n vm.loadavg | tr -d '{}')" >> "$P/measure-log.txt"
python3 "$P/optimization-rounds/run.py" 10 | tee "$P/results/node-412.txt"
echo "[$(date -u +%FT%TZ)] AFTER  node-412 :: load $(sysctl -n vm.loadavg | tr -d '{}')" >> "$P/measure-log.txt"
