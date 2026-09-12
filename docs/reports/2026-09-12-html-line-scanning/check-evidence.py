"""Check archived checksums and recompute paired timing summaries from raw windows."""
from pathlib import Path
import gzip
import hashlib
import json
import statistics

root = Path(__file__).resolve().parent
for line in (root / 'SHA256SUMS').read_text().splitlines():
    expected, name = line.split('  ', 1)
    assert hashlib.sha256((root / name).read_bytes()).hexdigest() == expected, name

checked = 0
for summary_file in root.rglob('summary.json'):
    windows_file = summary_file.with_name('windows.jsonl.gz')
    if not windows_file.exists():
        continue
    rows = [json.loads(line) for line in gzip.decompress(windows_file.read_bytes()).splitlines()]
    engines = {row['engine'] for row in rows}
    assert 'baseline' in engines and len(engines) == 2
    candidate = (engines - {'baseline'}).pop()
    for summary in json.loads(summary_file.read_text()):
        medians = {}
        for engine in engines:
            values = [row['elapsed_ns'] / row['count'] for row in rows
                      if row['index'] == summary['index'] and row['engine'] == engine]
            assert len(values) in [5, 9]
            medians[engine] = statistics.median(values)
        assert summary['baseline_ns'] == medians['baseline']
        assert summary['candidate_ns'] == medians[candidate]
        assert summary['change_pct'] == (medians[candidate] / medians['baseline'] - 1) * 100
        checked += 1
print(f'Checksums and {checked} paired timing summaries passed')

corpus_checked = 0
for summary_file in (root / 'corpus').glob('final-*-summary.json'):
    raw_file = summary_file.with_name(summary_file.name.replace('-summary.json', '-raw.jsonl.gz'))
    raw = [json.loads(line) for line in gzip.decompress(raw_file.read_bytes()).splitlines()]
    for summary in json.loads(summary_file.read_text()):
        for engine in ['ferro', 'ox-grow', 'ox-presize']:
            values = [r['elapsed_ns'] / r['count'] for r in raw
                      if r['index'] == summary['index'] and r['engine'] == engine]
            assert len(values) == 9
            assert summary['median_ns'][engine] == statistics.median(values)
            assert summary['min_ns'][engine] == min(values)
            assert summary['max_ns'][engine] == max(values)
            corpus_checked += 1
print(f'{corpus_checked} corpus engine/input summaries passed')

for source in ['baseline', 'production']:
    folder = root / 'memory' / source
    summaries = json.loads((folder / 'summary.json').read_text())
    raw = [json.loads(line) for line in gzip.decompress((folder / 'raw.jsonl.gz').read_bytes()).splitlines()]
    assert len(summaries) == 51 and len(raw) == 153
    for summary in summaries:
        rows = [r for r in raw if r['case'] == summary['case']]
        assert len(rows) == 3 and {r['process'] for r in rows} == {0, 1, 2}
        for row in rows:
            assert row['input_bytes'] == summary['input_bytes']
            assert row['html_sha256'] == summary['html_sha256']
            assert len(row['samples']) == 10
            for sample in row['samples']:
                assert all(summary[key] == value for key, value in sample.items())
assert json.loads((root / 'memory/baseline/summary.json').read_text()) == json.loads((root / 'memory/production/summary.json').read_text())
print('All raw allocation observations reproduce the unchanged summaries')
