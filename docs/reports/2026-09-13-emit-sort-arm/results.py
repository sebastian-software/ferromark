"""Generate the report directly from frozen timing and allocation samples."""
from pathlib import Path
import gzip
import json
import math
import statistics
import sys

D = Path(__file__).resolve().parent
names = ['unstable-packed', 'stable-packed', 'stable-tuple']

def read(path):
    data = path.read_bytes()
    return json.loads(gzip.decompress(data) if path.suffix == '.gz' else data)

lines = ['Negative time changes mean less time; positive changes mean more time.', '',
         '| Variant | >1% faster / within ±1% / >1% slower (103 inputs) | Geometric mean time change |',
         '| --- | ---: | ---: |']
for name in names:
    rows = [r for r in read(D / name / 'screen/summary.json')
            if not r['case'].startswith('sort-scaling/')]
    gain = sum(r['change_pct'] < -1 for r in rows)
    loss = sum(r['change_pct'] > 1 for r in rows)
    geo = (math.exp(statistics.mean(math.log(r['candidate_ns'] / r['baseline_ns'])
                                    for r in rows)) - 1) * 100
    lines.append(f'| {name} | {gain} / {len(rows)-gain-loss} / {loss} | {geo:+.2f}% |')
lines += ['', 'The following are the median changes across the three longer pairs,',
          'with the full range of pair results in parentheses. A/A uses identical',
          'baseline binaries and is a single pair, not a confidence interval.', '',
          '| Document | Unstable, packed | Stable, packed | Stable, tuple | A/A |',
          '| --- | ---: | ---: | ---: | ---: |']
aa = {r['index']: r for r in read(D / 'aa-control/summary.json')}
for index, label in [(79, 'Vue Suspense'), (487, 'Rust book ch09-02'),
                     (680, 'TypeScript 6.0'), (3, 'Short control'), (11, 'Heading control')]:
    values = []
    for name in names:
        changes = [next(r['change_pct'] for r in read(D / name / f'confirm-{i}/summary.json')
                        if r['index'] == index) for i in [1, 2, 3]]
        values.append(f'{statistics.median(changes):+.2f}% ({min(changes):+.2f}…{max(changes):+.2f}%)')
    lines.append(f'| {label} | ' + ' | '.join(values) + f" | {aa[index]['change_pct']:+.2f}% |")

lines += ['', 'All per-input results, including improvements, scaling controls and',
          'counterexamples, are retained in each variant directory. The focused set',
          'was selected after screening; its aggregate is not a workload average.', '',
          '| Variant | Inputs with changed allocation statistics / 123 | Maximum extra peak requested bytes |',
          '| --- | ---: | ---: |']
baseline = {r['index']: r for r in read(D / 'baseline/memory.json.gz')}
for name in names:
    rows = read(D / name / 'memory.json.gz')
    changed = sum(r['samples'][0] != baseline[r['index']]['samples'][0] for r in rows)
    peak = max(r['samples'][0]['peak_live_bytes'] -
               baseline[r['index']]['samples'][0]['peak_live_bytes'] for r in rows)
    lines.append(f'| {name} | {changed} / {len(rows)} | {peak:,} |')
lines += ['', 'All four variants produce equal output on all 779 fixed inputs.',
          'The packed unstable variant has identical recorded allocation statistics',
          'on all 123 memory cases. The two stable variants have identical allocation',
          'statistics to each other, including their extra temporary allocations.']

report = D / 'REPORT.md'
text = report.read_text()
start = '<!-- generated-results:start -->'
end = '<!-- generated-results:end -->'
generated = text.split(start)[0] + start + '\n' + '\n'.join(lines) + '\n' + end + text.split(end)[1]
if '--check' in sys.argv:
    assert generated == text, 'generated report results are stale'
else:
    report.write_text(generated)
