#!/bin/zsh
# Round-3 measurement queue: strict gate (no own heavy processes, 1-min load < 12), A/A control, then PGO split test and candidate screens.
source $(ls -d /private/tmp/claude-501/-Users-sebastian-Workspace-ferromark--claude-worktrees-code-audit-review-bb9235/*/scratchpad)/env.sh
gate() { until ! pgrep -x rustc >/dev/null && ! pgrep -x xctrace >/dev/null && ! pgrep -f "cargo" >/dev/null && ! pgrep -f "run.py" >/dev/null && [ "$(sysctl -n vm.loadavg | awk '{print int($2)}')" -lt 12 ]; do sleep 15; done; }
export PAIRS=5
AA='^(comment-ack|comment-review|comment-incident|legacy-docs-readme|rust-book-ch17-00-async-await|vite-docs-features|wiki-tea-lead|wiki-chess-article-body|scan-dense-escapes|table-formatted-4096)$'
RENDER='^(comment-ack|comment-review|comment-links|comment-incident|legacy-docs-readme|legacy-docs-mdx|legacy-contributing|rust-book-ch17-00-async-await|vite-docs-features|vite-docs-performance|typescript-handbook-compiler-options|typescript-handbook-typescript-5-0|vue-docs-slots|wiki-tea-lead|wiki-chess-article-body|wiki-volcano-plain-prose|scan-dense-escapes|table-formatted-4096|autolink-broad--comment-links|autolink-broad--wiki-tea-lead)$'
gate; echo "start $(uptime)"
$S/screen.sh $S/build-aa3 aa3-load "$AA" fresh reuse parse render > /dev/null 2>&1
echo "A/A: $(grep -E '^(fresh|reuse|parse|render) ' $S/results/aa3-load.summary | tr '\n' ';')"
python3 $S/percase.py $S aa3-load | awk 'NR>1 && ($3<0.98 || $3>1.02) {print "  A/A outlier:",$0}'
for c in "$@"; do
  gate
  case $c in
    pgo-split) $S/screen.sh $S/build-pgo-split pgo-split-test "$(cat $S/filter-broad-test.txt)" fresh reuse parse render > /dev/null 2>&1; N=pgo-split-test;;
    *) $S/screen.sh $S/build-$c $c-screen "$RENDER" render fresh reuse > /dev/null 2>&1; N=$c-screen;;
  esac
  echo "$c: $(grep -E '^(fresh|reuse|parse|render) ' $S/results/$N.summary | tr '\n' ';')"
done
echo "end $(uptime)"
