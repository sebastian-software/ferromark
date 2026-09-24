#!/bin/bash
# usage: build.sh <name> <candidate-dir> — round 6: build a candidate against main 060b02d2, reusing the A/A baseline build.
#
# ROUND6_WORK is the scratch directory that held the round (it was a session scratchpad); main210 in it
# is `git archive 060b02d2` unpacked, each candidate a `git archive` of its pushed branch (see README).
# FERROMARK_REPO is a checkout of this repository: prepare.py resolves --baseline-revision in the
# repository it lives in, and main210 is a plain export without git.
set -euo pipefail
NAME=$1; CAND=$2
P=${ROUND6_WORK:?set ROUND6_WORK to the round scratch directory}
PREP=${FERROMARK_REPO:?set FERROMARK_REPO to a ferromark checkout}/benchmarks/optimization-rounds/prepare.py
rm -rf "$P/build-$NAME"
if [ -f "$P/build-aa6/build.json" ] && [ "$NAME" != aa6 ]; then REUSE="--reuse-baseline-build $P/build-aa6"; else REUSE=""; fi
CARGO_BUILD_JOBS=4 python3 "$PREP" --baseline-path "$P/main210" --candidate-path "$CAND" --out "$P/build-$NAME" \
  --baseline-revision 060b02d2 $REUSE > "$P/build-$NAME.log" 2>&1 \
  && echo "built $NAME" || { echo "BUILD FAILED $NAME"; tail -20 "$P/build-$NAME.log"; exit 1; }
