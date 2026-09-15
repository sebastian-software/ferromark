#!/bin/zsh
# Sequential round-3 measurements. Gate on exact process names so queued shells cannot block each other.
source $(ls -d /private/tmp/claude-501/-Users-sebastian-Workspace-ferromark--claude-worktrees-code-audit-review-bb9235/*/scratchpad)/env.sh
gate() { until ! pgrep -x rustc >/dev/null && ! pgrep -x xctrace >/dev/null && ! pgrep -x cargo >/dev/null && ! pgrep -f "^python3 .*optimization-rounds/run.py" >/dev/null && [ "$(sysctl -n vm.loadavg | awk '{print int($2)}')" -lt 12 ]; do sleep 15; done; }
export PAIRS=5
AA='^(comment-ack|comment-review|comment-incident|legacy-docs-readme|rust-book-ch17-00-async-await|vite-docs-features|wiki-tea-lead|wiki-chess-article-body|scan-dense-escapes|table-formatted-4096)$'
RENDER='^(comment-ack|comment-review|comment-links|comment-incident|legacy-docs-readme|legacy-docs-mdx|legacy-contributing|rust-book-ch17-00-async-await|vite-docs-features|vite-docs-performance|typescript-handbook-compiler-options|typescript-handbook-typescript-5-0|vue-docs-slots|wiki-tea-lead|wiki-chess-article-body|wiki-volcano-plain-prose|scan-dense-escapes|table-formatted-4096|autolink-broad--comment-links|autolink-broad--wiki-tea-lead)$'
PARSE='^(comment-ack|comment-review|comment-links|comment-incident|comment-checklist|comment-table|legacy-docs-readme|legacy-docs-mdx|legacy-contributing|rust-book-ch17-00-async-await|rust-book-appendix-02-operators|vite-docs-features|typescript-handbook-compiler-options|wiki-tea-lead|wiki-chess-article-body|wiki-tea-article-body|wiki-volcano-plain-prose|scan-many-short-links|scan-long-references|table-formatted-4096)$'
gate; echo "start $(uptime)"
$S/screen.sh $S/build-aa3 aa3-load "$AA" fresh reuse parse render > /dev/null 2>&1
echo "A/A: $(grep -E '^(fresh|reuse|parse|render) ' $S/results/aa3-load.summary | tr '\n' ';')"
gate; $S/screen.sh $S/build-pgo-split pgo-split-test "$(cat $S/filter-broad-test.txt)" fresh reuse parse render > /dev/null 2>&1
echo "pgo-split: $(grep -E '^(fresh|reuse|parse|render) ' $S/results/pgo-split-test.summary | tr '\n' ';')"
for c in optH fenceI1 fenceI12; do gate; $S/screen.sh $S/build-$c $c-screen "$RENDER" render fresh reuse > /dev/null 2>&1; echo "$c: $(grep -E '^(render|fresh|reuse) ' $S/results/$c-screen.summary | tr '\n' ';')"; done
for c in lineF1 lineF12 lineF123 lineF1234; do gate; $S/screen.sh $S/build-$c $c-screen "$PARSE" parse reuse fresh > /dev/null 2>&1; echo "$c: $(grep -E '^(parse|reuse|fresh) ' $S/results/$c-screen.summary | tr '\n' ';')"; done
export REUSE=$S/build-aa3 BASELINE_REV=HEAD BASELINE_DIR=$S/baseline3
$S/make_candidate.sh coalJ HEAD 0fe75ccd 2>&1 | grep -v "^cand-"
gate; $S/screen.sh $S/build-coalJ coalJ-screen "$PARSE" parse reuse fresh > /dev/null 2>&1; echo "coalJ: $(grep -E '^(parse|reuse|fresh) ' $S/results/coalJ-screen.summary | tr '\n' ';')"
echo "end $(uptime)"
