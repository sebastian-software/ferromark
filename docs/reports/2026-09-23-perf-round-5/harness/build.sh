#!/bin/bash
# usage: build.sh <name> <candidate-path> — build a candidate against the round baseline (cb352020), reusing the A/A baseline build.
#
# ROUND5_WORK is the scratch directory that held the round (it was a session scratchpad);
# FERROMARK_REPO is a checkout of this repository. src-base is `git archive cb352020` unpacked
# into $ROUND5_WORK/src-base, each candidate a `git archive` of its pushed branch (see README).
set -euo pipefail
NAME=$1; CAND=$2
P=${ROUND5_WORK:?set ROUND5_WORK to the round scratch directory}
REPO=${FERROMARK_REPO:?set FERROMARK_REPO to a ferromark checkout}
rm -rf "$P/build-$NAME"
if [ -f "$P/build-aa/build.json" ] && [ "$NAME" != aa ]; then REUSE="--reuse-baseline-build $P/build-aa"; else REUSE=""; fi
# prepare.py resolves --baseline-revision in the repository it lives in, so run a checkout's copy.
PREP=$REPO/benchmarks/optimization-rounds/prepare.py
CARGO_BUILD_JOBS=4 python3 "$PREP" --baseline-path "$P/src-base" --candidate-path "$CAND" --out "$P/build-$NAME" \
  --baseline-revision cb3520202d34d7b390f0703ea2efff25de02dbf2 $REUSE > "$P/build-$NAME.log" 2>&1 \
  && echo "built $NAME" || { echo "BUILD FAILED $NAME"; tail -20 "$P/build-$NAME.log"; exit 1; }
