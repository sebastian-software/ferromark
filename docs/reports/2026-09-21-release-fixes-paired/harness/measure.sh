#!/bin/bash
# Gate on compilers, then: A/A control, HEAD on the 57 broad documents (4 stages), and per-revision screens on the affected documents.
set -euo pipefail
REPO=/Users/sebastian/Workspace/ferromark/.claude/worktrees/homepage-performance-metrics-8a493a
P=/private/tmp/fm-paired
H=$REPO/benchmarks/optimization-rounds
CORPUS=/private/tmp/native-release-work/paired-corpus.json
LOG=$P/measure-log.txt
cd "$REPO"
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
screen build-aa aa $P/filter-screen.txt 3 fresh reuse parse render
screen build-head head-broad $P/filter-broad.txt 5 fresh reuse parse render
for name in c369 c372 c374 c377 c380 c382 c383; do
  screen build-$name $name $P/filter-screen.txt 3 fresh reuse parse render
done
screen build-head head-screen $P/filter-screen.txt 3 fresh reuse parse render
echo "[$(date -u +%FT%TZ)] ALL DONE" >> "$LOG"
