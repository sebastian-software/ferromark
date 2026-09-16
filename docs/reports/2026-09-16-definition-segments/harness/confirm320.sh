#!/bin/zsh
# usage: confirm320.sh <build-name>
# A/A control under current conditions, then the issue-320 documents, then 57 broad + 45 diagnostics (5 pairs, 3 rounds).
source /private/tmp/claude-501/-Users-sebastian-Workspace-ferromark--claude-worktrees-code-audit-review-bb9235/487e9fd2-c2bd-451b-8ce7-afd6d14441ca/scratchpad/env.sh
NAME=$1
export PAIRS=5 ROUNDS=3 WINDOW=40 CORPUS=$S/corpus320.json
until ! pgrep -x rustc >/dev/null && ! pgrep -x xctrace >/dev/null; do sleep 10; done
echo "start $(uptime)"
AA='^(comment-ack|comment-review|comment-incident|legacy-docs-readme|rust-book-ch17-00-async-await|vite-docs-features|wiki-tea-lead|wiki-chess-article-body|scan-dense-escapes|table-formatted-4096)$'
$S/screen320.sh $S/build-aa320 aa320-load "$AA" fresh reuse parse render > /dev/null 2>&1
echo "A/A under current load: $(grep -E '^(fresh|reuse|parse|render) ' $S/results/aa320-load.summary | tr '\n' ';')"
python3 $S/percase.py $S aa320-load | awk 'NR>1 && ($3<0.98 || $3>1.02) {print "  A/A outlier:",$0}'
ISSUE='^(comment-incident|comment-incident-nodef|rust-book-ch00-00-introduction|rust-book-ch00-00-introduction-nodef|typescript-handbook-advanced-types|typescript-handbook-advanced-types-nodef|scan-extension-links|scan-long-references|scan-short-reference)$'
$S/screen320.sh $S/build-$NAME $NAME-issue "$ISSUE" fresh reuse parse render > /dev/null 2>&1
echo "$NAME issue docs: $(grep -E '^(fresh|reuse|parse|render) ' $S/results/$NAME-issue.summary | tr '\n' ';')"
grep -E '^(fresh|reuse|parse|render) +(comment-incident|rust-book-ch00|typescript-handbook-advanced|scan-)' $S/results/$NAME-issue.summary
$S/screen320.sh $S/build-$NAME $NAME-broad "$(cat $S/filter-broad.txt)" fresh reuse parse render > /dev/null 2>&1
echo "$NAME broad: $(grep -E '^(fresh|reuse|parse|render) ' $S/results/$NAME-broad.summary | tr '\n' ';')"
$S/screen320.sh $S/build-$NAME $NAME-diag "$(cat $S/filter-diag.txt)" fresh reuse parse render > /dev/null 2>&1
echo "$NAME diag: $(grep -E '^(fresh|reuse|parse|render) ' $S/results/$NAME-diag.summary | tr '\n' ';')"
echo "end $(uptime)"
