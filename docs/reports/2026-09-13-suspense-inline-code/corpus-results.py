"""Describe the distribution across all individual corpus documents."""
from pathlib import Path
import json
import math
import statistics
import sys

D = Path(__file__).resolve().parent
candidate = json.loads((D/'metadata.json').read_text())['candidate_variant']
M = D/'measurements'/candidate
runs = [{r['index']: r for r in json.loads((M/f'corpus-{i}/summary.json').read_text())} for i in range(1, 4)]
assert all(set(r) == set(runs[0]) for r in runs)
assert len(runs[0]) == 638
rows = []
for i, r in runs[0].items():
    delta = [run[i]['change_pct'] for run in runs]
    rows.append(dict(index=i, case=r['case'].removeprefix('corpus/'), project=r['case'].split('/')[1], changes=delta, median_change=statistics.median(delta), before_ns=statistics.median(run[i]['baseline_ns'] for run in runs), after_ns=statistics.median(run[i]['candidate_ns'] for run in runs)))


def stats(rs):
    return dict(count=len(rs), lower_median=sum(r['median_change']<0 for r in rs), faster_1pct=sum(r['median_change']<-1 for r in rs), within_1pct=sum(abs(r['median_change'])<=1 for r in rs), slower_1pct=sum(r['median_change']>1 for r in rs), slower_2pct=sum(r['median_change']>2 for r in rs), lower_in_all_three=sum(max(r['changes'])<0 for r in rs), slower_1pct_in_all_three=sum(min(r['changes'])>1 for r in rs), geometric_mean_change_pct=100*(math.exp(statistics.mean(math.log1p(r['median_change']/100) for r in rs))-1), summed_median_time_change_pct=100*(sum(r['after_ns'] for r in rs)/sum(r['before_ns'] for r in rs)-1), summed_before_ms=sum(r['before_ns'] for r in rs)/1e6, summed_after_ms=sum(r['after_ns'] for r in rs)/1e6)

summary = {p: stats([r for r in rows if r['project']==p]) for p in sorted({r['project'] for r in rows})}
summary['all'] = stats(rows)
out = ['# Distribution across 638 individual documents', '', 'Three fresh process pairs per document, five alternating 20 ms windows after 20 ms warmup. Same native before/after operation and frozen inputs as the focused comparisons. This includes every individual document in the corpus, regardless of cross-engine output equality; concatenations and synthetic Ox controls are excluded. Before/after output equality is checked separately.', '', 'Changes are the median of the three paired percentage changes. The ±1% band is descriptive, not a confidence interval or significance test. Small changes must not be presented as reliably faster rendering.', '', '| Project | Documents | Lower median | >1% faster | Within ±1% | >1% slower | >2% slower |', '| --- | ---: | ---: | ---: | ---: | ---: | ---: |']
for name, s in summary.items():
    out.append(f"| {name} | {s['count']} | {s['lower_median']} | {s['faster_1pct']} | {s['within_1pct']} | {s['slower_1pct']} | {s['slower_2pct']} |")
out += ['', '| Project | Lower in all three pairs | >1% slower in all three | Geometric mean change | Sum of before medians ms | Sum of after medians ms | Change in sum |', '| --- | ---: | ---: | ---: | ---: | ---: | ---: |']
for name, s in summary.items():
    out.append(f"| {name} | {s['lower_in_all_three']} | {s['slower_1pct_in_all_three']} | {s['geometric_mean_change_pct']:+.2f}% | {s['summed_before_ms']:.3f} | {s['summed_after_ms']:.3f} | {s['summed_median_time_change_pct']:+.2f}% |")
out += ['', 'The geometric mean weights documents equally. The sum describes one render of each document using measured per-document medians; it is not a separately timed end-to-end build and excludes loading and setup. Neither assumes that this corpus matches a user’s workload.', '', '## Longer checks of repeated costs', '', 'Every document over 1% slower in all three broad pairs receives three fresh seven-window comparisons at 100 ms per window, after 60 ms warmup. These retain the initial observations and do not replace them.', '', '| Document | Broad changes | Longer changes |', '| --- | --- | --- |']
followups = [{r['index']: r for r in json.loads((M/f'corpus-followup-{i}/summary.json').read_text())} for i in range(1, 4)]
expected = {r['index'] for r in rows if min(r['changes'])>1}
assert all(set(r) == expected for r in followups)
for r in rows:
    if r['index'] in expected:
        out.append('| '+r['case']+' | '+', '.join(f'{d:+.2f}%' for d in r['changes'])+' | '+', '.join(f"{run[r['index']]['change_pct']:+.2f}%" for run in followups)+' |')
if not expected:
    out.append('| None | — | — |')
out += ['', '## All individual results', '', '| Document | Before µs | After µs | Three paired changes |', '| --- | ---: | ---: | --- |']
for r in rows:
    out.append(f"| {r['case']} | {r['before_ns']/1000:.3f} | {r['after_ns']/1000:.3f} | "+', '.join(f'{d:+.2f}%' for d in r['changes'])+' |')
out.append('')
text = '\n'.join(out)
report_path = D/'REPORT.md'
report = report_path.read_text()
a = report.index('<!-- measured-summary:start -->')
b = report.index('<!-- measured-summary:end -->', a) + len('<!-- measured-summary:end -->')
all_stats = summary['all']
measured = f"""<!-- measured-summary:start -->
Of 638 individual documents, {all_stats['lower_median']} have a lower median
runtime and {all_stats['lower_in_all_three']} are lower in all three broad pairs.
Using a descriptive ±1% band, {all_stats['faster_1pct']} are faster,
{all_stats['within_1pct']} are within the band and {all_stats['slower_1pct']} are
slower. The sum of per-document medians changes by
{all_stats['summed_median_time_change_pct']:+.2f}%; this is not a separately timed
site build. The geometric mean change is
{all_stats['geometric_mean_change_pct']:+.2f}% with equal document weighting.

Small changes do not establish a universal speedup. The original Suspense
input is practically unchanged; long code/tag paragraphs have the substantial
improvement. The focused TypeScript 6.0 loss and heading outliers remain in
[RESULTS.md](RESULTS.md), alongside the broader follow-ups in
[CORPUS.md](CORPUS.md). No source or timing observation is dropped to improve the
reported distribution.
<!-- measured-summary:end -->"""
expected_report = report[:a] + measured + report[b:]
if '--check' in sys.argv:
    assert report == expected_report
    assert (D/'CORPUS.md').read_text() == text
    assert json.loads((D/'corpus-summary.json').read_text()) == summary
else:
    report_path.write_text(expected_report)
    (D/'CORPUS.md').write_text(text)
    (D/'corpus-summary.json').write_text(json.dumps(summary, indent=2)+'\n')
print(json.dumps(summary['all']))
