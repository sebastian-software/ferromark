#!/bin/bash
# Build the paired workers: baseline c4af9525 (the 7c887a2b core) against each candidate revision.
set -euo pipefail
REPO=/Users/sebastian/Workspace/ferromark/.claude/worktrees/homepage-performance-metrics-8a493a
P=/private/tmp/fm-paired
H=$REPO/benchmarks/optimization-rounds
cd "$REPO"
python3 $H/prepare.py --baseline-path $P/src-base --candidate-path $P/src-head --out $P/build-head --baseline-revision c4af9525 > $P/build-head.log 2>&1 && echo "built head"
for name in aa c369 c372 c374 c377 c380 c382 c383; do
  python3 $H/prepare.py --baseline-path $P/src-base --candidate-path $P/src-$name --out $P/build-$name --baseline-revision c4af9525 --reuse-baseline-build $P/build-head > $P/build-$name.log 2>&1 && echo "built $name" || { echo "BUILD FAILED $name"; tail -20 $P/build-$name.log; exit 1; }
done
echo "ALL BUILT"
