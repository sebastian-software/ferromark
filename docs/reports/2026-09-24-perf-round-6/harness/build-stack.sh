#!/bin/bash
# usage: build-stack.sh <name> <baseline-dir> <baseline-revision> <candidate-dir>
# Builds a candidate against any other baseline tree, building both workers. Round 5 used it for
# its stacked re-screens; round 6 used it for the branches that started from 9e741a37 (#425, #427:
# `main9e7`) and for the aarch64 check of #426 against af65cb43 (`baseaf6`), each a `git archive`
# of that revision.
#
# ROUND6_WORK is the scratch directory that held the round; FERROMARK_REPO is a checkout of this
# repository in which the baseline revision resolves.
set -euo pipefail
NAME=$1; BASE=$2; REV=$3; CAND=$4
P=${ROUND6_WORK:?set ROUND6_WORK to the round scratch directory}
PREP=${FERROMARK_REPO:?set FERROMARK_REPO to a ferromark checkout}/benchmarks/optimization-rounds/prepare.py
rm -rf "$P/build-$NAME"
CARGO_BUILD_JOBS=4 python3 "$PREP" --baseline-path "$BASE" --candidate-path "$CAND" --out "$P/build-$NAME" \
  --baseline-revision "$REV" > "$P/build-$NAME.log" 2>&1 \
  && echo "built $NAME" || { echo "BUILD FAILED $NAME"; tail -20 "$P/build-$NAME.log"; exit 1; }
