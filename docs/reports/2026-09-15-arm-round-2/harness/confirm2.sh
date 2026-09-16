#!/bin/zsh
# usage: confirm2.sh <build-name>
# Robust variant: gates on our own heavy processes (rustc/cargo/xctrace) instead of the load average,
# runs an A/A control under the same conditions first, and uses 5 pairs per round.
source $(ls -d /private/tmp/claude-501/-Users-sebastian-Workspace-ferromark--claude-worktrees-code-audit-review-bb9235/*/scratchpad)/env.sh
NAME=$1
export PAIRS=5 ROUNDS=3 WINDOW=40
until ! pgrep -x rustc >/dev/null && ! pgrep -x xctrace >/dev/null && ! pgrep -f "cargo (build|test|clippy|bench)" >/dev/null; do sleep 10; done
echo "start $(uptime)"
AA='^(comment-ack|comment-review|comment-incident|legacy-docs-readme|rust-book-ch17-00-async-await|vite-docs-features|wiki-tea-lead|wiki-chess-article-body|scan-dense-escapes|table-formatted-4096)$'
$S/screen.sh $S/build-aa aa-load "$AA" fresh reuse parse render > /dev/null 2>&1
echo "A/A under current load: $(grep -E '^(fresh|reuse|parse|render) ' $S/results/aa-load.summary | tr '\n' ';')"
python3 $S/percase.py $S aa-load | awk 'NR>1 && ($3<0.98 || $3>1.02) {print "  A/A outlier:",$0}'
$S/screen.sh $S/build-$NAME $NAME-broad "$(cat $S/filter-broad.txt)" fresh reuse parse render > /dev/null 2>&1
echo "$NAME broad: $(grep -E '^(fresh|reuse|parse|render) ' $S/results/$NAME-broad.summary | tr '\n' ';')"
$S/screen.sh $S/build-$NAME $NAME-diag "$(cat $S/filter-diag.txt)" fresh reuse parse render > /dev/null 2>&1
echo "$NAME diag: $(grep -E '^(fresh|reuse|parse|render) ' $S/results/$NAME-diag.summary | tr '\n' ';')"
echo "end $(uptime)"
