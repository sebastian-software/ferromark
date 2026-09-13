"""Check identities, reconstruct measured sources, and recompute timing tables."""
from pathlib import Path
import collections
import gzip
import hashlib
import io
import json
import math
import statistics
import subprocess
import sys
import tarfile
import tempfile

D = Path(__file__).resolve().parent
R = D.parents[2]
meta = json.loads((D/'metadata.json').read_text())


def sha(data):
    return hashlib.sha256(data).hexdigest()


def same(a, b):
    assert math.isclose(a, b, rel_tol=1e-12, abs_tol=1e-9), (a, b)


for line in (D/'SHA256SUMS').read_text().splitlines():
    digest, name = line.split('  ', 1)
    assert sha((D/name).read_bytes()) == digest, name

archive = subprocess.check_output(['git', 'archive', meta['baseline_revision'], 'src'], cwd=R)
for directory in sorted((D/'measurements').iterdir()):
    if not (directory/'source.json').exists():
        continue
    with tempfile.TemporaryDirectory() as temporary:
        root = Path(temporary)
        with tarfile.open(fileobj=io.BytesIO(archive)) as tar:
            tar.extractall(root, filter='data')
        patch = (directory/'patch.diff').read_bytes()
        if patch:
            subprocess.run(['patch', '-s', '-p1', '-d', str(root)], input=patch, check=True)
        for name, digest in json.loads((directory/'source.json').read_text()).items():
            assert sha((root/name).read_bytes()) == digest, (directory.name, name)

checked = 0
for path in sorted((D/'measurements').glob('**/windows.jsonl.gz')):
    raw = [json.loads(s) for s in gzip.decompress(path.read_bytes()).decode().splitlines()]
    summary = json.loads((path.parent/'summary.json').read_text())
    groups = collections.defaultdict(lambda: collections.defaultdict(list))
    for row in raw:
        assert row['count'] > 0 and row['count'] % 16 == 0 and row['elapsed_ns'] > 0
        groups[row['index']][row['engine']].append(row)
    assert set(groups) == {r['index'] for r in summary}
    expected_rounds = 5 if path.parent.name in ['screen', 'targets', 'corpus-1', 'corpus-2', 'corpus-3'] else 7
    for row in summary:
        engines = groups[row['index']]
        assert len(engines) == 2 and 'baseline' in engines
        values = {}
        for engine, windows in engines.items():
            assert sorted(v['round'] for v in windows) == list(range(expected_rounds))
            assert all(v['case'] == row['case'] for v in windows)
            values[engine] = statistics.median(v['elapsed_ns']/v['count'] for v in windows)
        before = values.pop('baseline'); after = next(iter(values.values()))
        same(before, row['baseline_ns']); same(after, row['candidate_ns'])
        same((after/before-1)*100, row['change_pct'])
        checked += 1

for path in sorted((D/'native').glob('**/*-raw.jsonl.gz')):
    raw = [json.loads(s) for s in gzip.decompress(path.read_bytes()).decode().splitlines()]
    summary = json.loads(path.with_name(path.name.replace('-raw.jsonl.gz', '-summary.json')).read_text())
    groups = collections.defaultdict(lambda: collections.defaultdict(list))
    for row in raw:
        assert row['count'] > 0 and row['count'] % 8 == 0
        groups[row['index']][row['engine']].append(row)
    assert set(groups) == {r['index'] for r in summary}
    for row in summary:
        for engine, windows in groups[row['index']].items():
            assert sorted(v['round'] for v in windows) == list(range(9))
            assert all(v['case'] == row['case'] for v in windows)
            values = [v['elapsed_ns']/v['count'] for v in windows]
            same(statistics.median(values), row['median_ns'][engine])
            same(min(values), row['min_ns'][engine]); same(max(values), row['max_ns'][engine])
        checked += 1

adopted = D/'measurements'/meta['candidate_variant']
current_source = json.loads((adopted/'source.json').read_text())
if review := meta.get('review_test_update'):
    with tempfile.TemporaryDirectory() as temporary:
        root = Path(temporary)
        with tarfile.open(fileobj=io.BytesIO(archive)) as tar:
            tar.extractall(root, filter='data')
        for patch in [adopted/'patch.diff', D/review['patch']]:
            subprocess.run(['patch', '-s', '-p1', '-d', str(root)],
                           input=patch.read_bytes(), check=True)
        current_source = json.loads((D/review['source']).read_text())
        for name, digest in current_source.items():
            assert sha((root/name).read_bytes()) == digest, name
    controls = json.loads((D/review['negative_controls']).read_text())
    assert len(controls) == 7 and controls[0]['exit_code'] == 0
    for row in controls[1:]:
        assert row['exit_code'] == 101 and 'running 1 test' in row['output']
        assert 'must exercise' in row['output'] or 'probes grew from' in row['output']
    assert all(r['exit_code'] == 0 for r in json.loads((D/review['preflight']).read_text()))
if '--check-current' in sys.argv:
    for name, digest in current_source.items():
        assert sha((R/name).read_bytes()) == digest, name
total = 0
for path in adopted.glob('*verification.json'):
    record = json.loads(path.read_text())
    assert not record.get('failures', record.get('failed', [])), path.name
    total += record['count']
assert total == 131912, total
assert all(r['exit_code'] == 0 for r in json.loads((D/'validation/preflight.json').read_text()))
subprocess.run(['python3', str(D/'results.py'), '--check'], check=True)
subprocess.run(['python3', str(D/'corpus-results.py'), '--check'], check=True)
print(f'Checked all hashes, reconstructed sources, {checked} timing summaries and {total} exact-output comparisons.')
