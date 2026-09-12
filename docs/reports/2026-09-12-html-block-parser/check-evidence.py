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
