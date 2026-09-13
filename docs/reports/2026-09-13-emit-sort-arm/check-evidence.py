"""Validate source identities, raw timing medians and allocation observations."""
from pathlib import Path
import collections
import gzip
import hashlib
import io
import json
import math
import statistics
import subprocess
import tarfile
import tempfile

D = Path(__file__).resolve().parent
R = D.parents[2]

def sha(data):
    return hashlib.sha256(data).hexdigest()

def read(path):
    data = path.read_bytes()
    return json.loads(gzip.decompress(data) if path.suffix == '.gz' else data)

for line in (D / 'SHA256SUMS').read_text().splitlines():
    digest, name = line.split('  ', 1)
    assert sha((D / name).read_bytes()) == digest, name
metadata = read(D / 'metadata.json')
assert metadata['binaries']['baseline'] == metadata['binaries']['baseline-copy']
archive = subprocess.check_output(['git', 'archive', metadata['baseline_revision'], 'src'], cwd=R)
names = ['baseline', 'unstable-packed', 'stable-packed', 'stable-tuple']
for name in names:
    with tempfile.TemporaryDirectory() as temporary:
        root = Path(temporary)
        with tarfile.open(fileobj=io.BytesIO(archive)) as tar:
            tar.extractall(root, filter='data')
        patch = gzip.decompress((D / name / 'patch.diff.gz').read_bytes())
        if patch:
            subprocess.run(['patch', '-s', '-p1', '-d', str(root)], input=patch, check=True)
        for relative, digest in read(D / name / 'source.json').items():
            assert sha((root / relative).read_bytes()) == digest, (name, relative)
        if name != 'baseline':
            assert (root / 'src/inline/mod.rs').read_bytes() == gzip.decompress(
                (D / 'snapshots' / (name + '.rs.gz')).read_bytes())

inputs = metadata['inputs']
assert sha((D / inputs['base']).read_bytes()) == inputs['base_sha256']
cases = read(D / inputs['base']) + read(D / inputs['additional'])
selected = read(D / 'selected.json')
followup = read(D / 'followup-selected.json')
assert len(cases) == 779 and len(selected) == 127 and len(followup) == 17
checks = [read(D / name / 'all-cases-verification.json') for name in names]
assert all(r['count'] == len(cases) and not r['failures'] for r in checks)
assert len({r['sha256'] for r in checks}) == 1

timings = 0
for path in sorted(D.glob('**/windows.jsonl.gz')):
    rows = [json.loads(line) for line in gzip.decompress(path.read_bytes()).decode().splitlines()]
    summary = read(path.parent / 'summary.json')
    groups = collections.defaultdict(lambda: collections.defaultdict(list))
    for row in rows:
        assert row['count'] > 0 and row['count'] % 16 == 0 and row['elapsed_ns'] > 0
        assert row['case'] == cases[row['index']]['case']
        groups[row['index']][row['engine']].append(row)
    screen = path.parent.name == 'screen'
    assert set(groups) == set(selected if screen else followup)
    assert {r['index'] for r in summary} == set(groups)
    for row in summary:
        values = {}
        for engine, windows in groups[row['index']].items():
            assert sorted(v['round'] for v in windows) == list(range(5 if screen else 7))
            values[engine] = statistics.median(v['elapsed_ns'] / v['count'] for v in windows)
        assert len(values) == 2 and 'baseline' in values
        before = values.pop('baseline')
        after = next(iter(values.values()))
        for actual, expected in [(before, row['baseline_ns']), (after, row['candidate_ns']),
                                 ((after / before - 1) * 100, row['change_pct'])]:
            assert math.isclose(actual, expected, rel_tol=1e-12, abs_tol=1e-9)
        timings += 1
assert timings == 551, timings

memory = {name: read(D / name / 'memory.json.gz') for name in names}
baseline = {r['index']: r for r in memory['baseline']}
for name, rows in memory.items():
    assert len(rows) == 123
    assert {r['index'] for r in rows} == {
        i for i in selected if not cases[i]['reuse'] and not cases[i]['flags'] & 256}
    for row in rows:
        assert len(row['samples']) == 5 and all(s == row['samples'][0] for s in row['samples'])
        assert row['html_sha256'] == baseline[row['index']]['html_sha256']
        assert row['html_bytes'] == baseline[row['index']]['html_bytes']
        sample = row['samples'][0]
        assert sample['peak_live_bytes'] >= sample['live_after_render'] >= row['html_bytes']
        if name == 'unstable-packed' or cases[row['index']]['case'].startswith('corpus/'):
            assert sample == baseline[row['index']]['samples'][0]
assert memory['stable-packed'] == memory['stable-tuple']
subprocess.run(['python3', str(D / 'results.py'), '--check'], check=True)
print(f'Checked source identities, {timings} timing summaries, 779 matching outputs and 123 allocation cases per variant.')
