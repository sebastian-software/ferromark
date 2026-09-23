#!/bin/bash
# usage: profile.sh <tag> — sample parse and render of the 57 broad documents for 10 s each.
#
# PROFILE_WORK is a scratch directory holding the built driver (cargo build --release in a copy of
# this directory, with Cargo.toml pointing at the ferromark revision to profile) and corpus.json
# (any paired corpus with the 57 `broad` cases; the round used the 207-case round-3 corpus).
PROFILE_WORK=${PROFILE_WORK:?set PROFILE_WORK to the profiling scratch directory}
B=$PROFILE_WORK/prof/target/release/fmprof
for mode in parse render; do
  "$B" "$PROFILE_WORK/corpus.json" "$mode" 14 &
  PID=$!
  sleep 1.5
  sample "$PID" 10 1 -mayDie -file "$PROFILE_WORK/sample-$1-$mode.txt" >/dev/null 2>&1
  wait "$PID"
done
