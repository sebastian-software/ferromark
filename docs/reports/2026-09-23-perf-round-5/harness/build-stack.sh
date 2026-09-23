#!/bin/bash
# usage: build-stack.sh <name> <baseline-dir> <baseline-revision> <candidate-dir>
# Re-screen builds: each merge candidate against the state that already holds the previous merges.
# The stacked states are commits made with `git merge-tree --write-tree` + `git commit-tree`
# (no worktree touched) and unpacked with `git archive`; see README.
#
# ROUND5_WORK is the scratch directory that held the round; FERROMARK_REPO is a checkout of this
# repository that contains the stacked commits.
set -euo pipefail
NAME=$1; BASE=$2; REV=$3; CAND=$4
P=${ROUND5_WORK:?set ROUND5_WORK to the round scratch directory}
REPO=${FERROMARK_REPO:?set FERROMARK_REPO to a ferromark checkout}
PREP=$REPO/benchmarks/optimization-rounds/prepare.py
rm -rf "$P/build-$NAME"
CARGO_BUILD_JOBS=4 python3 "$PREP" --baseline-path "$BASE" --candidate-path "$CAND" --out "$P/build-$NAME" \
  --baseline-revision "$REV" > "$P/build-$NAME.log" 2>&1 \
  && echo "built $NAME" || { echo "BUILD FAILED $NAME"; tail -20 "$P/build-$NAME.log"; exit 1; }
