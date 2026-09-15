#!/bin/zsh
# usage: profile.sh <worker-binary> <profile> <stage> <name> <input.md...>
# Records ~4s of the timed loop with xctrace Time Profiler and exports a per-symbol summary.
set -e
S=/private/tmp/claude-501/-Users-sebastian-Workspace-ferromark--claude-worktrees-code-audit-review-bb9235/487e9fd2-c2bd-451b-8ce7-afd6d14441ca/scratchpad
export DEVELOPER_DIR=/Applications/Xcode.app/Contents/Developer
BIN=$1; PROFILE=$2; STAGE=$3; NAME=$4; shift 4
OUT=$S/profiles/$NAME
rm -rf $OUT; mkdir -p $OUT
FIFO=$OUT/stdin.fifo; mkfifo $FIFO
# keep fifo open for writing from this shell
$BIN $PROFILE $STAGE "$@" < $FIFO > $OUT/worker.out 2> $OUT/worker.err &
WPID=$!
exec 3> $FIFO
echo "verify" >&3
sleep 0.5
xcrun xctrace record --template 'Time Profiler' --attach $WPID --time-limit 9000ms --output $OUT/trace.trace > $OUT/xctrace.log 2>&1 &
XPID=$!
sleep 0.8
echo "bench 8000000000" >&3
wait $XPID || true
echo "quit" >&3
exec 3>&-
wait $WPID || true
xcrun xctrace export --input $OUT/trace.trace --xpath '/trace-toc/run[@number="1"]/data/table[@schema="time-profile"]' > $OUT/time-profile.xml 2> $OUT/export.err || true
python3 $S/aggregate_profile.py $OUT/time-profile.xml > $OUT/summary.txt
head -60 $OUT/summary.txt
