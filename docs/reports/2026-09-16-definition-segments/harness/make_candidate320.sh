#!/bin/zsh
# usage: make_candidate320.sh <name> <rev>  -> worktree $S/cand-<name> at <rev>, builds $S/build-<name> against the HEAD baseline of build-aa320
set -e
source /private/tmp/claude-501/-Users-sebastian-Workspace-ferromark--claude-worktrees-code-audit-review-bb9235/487e9fd2-c2bd-451b-8ce7-afd6d14441ca/scratchpad/env.sh
REPO=/Users/sebastian/Workspace/ferromark/.claude/worktrees/code-audit-review-bb9235
NAME=$1; REV=$2
rm -rf $S/cand-$NAME; git -C $REPO worktree prune
git -C $REPO worktree add --detach $S/cand-$NAME $REV >/dev/null 2>&1
echo "cand-$NAME at $(git -C $S/cand-$NAME rev-parse --short HEAD): $(git -C $S/cand-$NAME log --oneline origin/main..HEAD | wc -l | tr -d ' ') commits on origin/main"
python3 $REPO/benchmarks/optimization-rounds/prepare.py --baseline-path $REPO --candidate-path $S/cand-$NAME --out $S/build-$NAME --baseline-revision HEAD --reuse-baseline-build $S/build-aa320 > $S/build-$NAME.log 2>&1 && echo "built build-$NAME" || { echo "BUILD FAILED"; tail -30 $S/build-$NAME.log; exit 1; }
