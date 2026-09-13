"""Reconstruct an isolated workspace from this report's frozen evidence."""
from pathlib import Path
import argparse
import gzip
import io
import json
import shutil
import subprocess
import tarfile

p = argparse.ArgumentParser()
p.add_argument('destination', type=Path)
p.add_argument('--ox-source', type=Path)
a = p.parse_args()
D = Path(__file__).resolve().parent
R = D.parents[2]
W = a.destination.resolve()
W.mkdir(parents=True, exist_ok=True)
meta = json.loads((D/'metadata.json').read_text())
old = R/'docs/reports/2026-09-12-block-render-hotspots'
for name in ['verify', 'blocks', 'extra', 'extended', 'fences']:
    (W/(name+'.json')).write_bytes(gzip.decompress((old/(name+'.json.gz')).read_bytes()))
for name in ['cases', 'precedence']:
    (W/(name+'.json')).write_bytes(gzip.decompress((D/'inputs'/(name+'.json.gz')).read_bytes()))
for path in (D/'harness').glob('*.py'):
    shutil.copyfile(path, W/path.name)
for name in ['selected.json', 'confirmation-selected.json', 'target-selected.json', 'final-followup-selected.json','corpus-selected.json','corpus-followup-selected.json']:
    shutil.copyfile(D/'inputs'/name, W/name)
shutil.copytree(D/'driver', W/'driver', dirs_exist_ok=True)
(W/'baseline').mkdir(exist_ok=True)
archive = subprocess.check_output(['git', 'archive', meta['baseline_revision'], 'src'], cwd=R)
with tarfile.open(fileobj=io.BytesIO(archive)) as tar:
    tar.extractall(W/'baseline', filter='data')
shutil.copyfile(D/'source-Cargo.toml', W/'baseline/Cargo.toml')
shutil.copytree(W/'baseline', W/'source', dirs_exist_ok=True)
for name in ['production', 'linear-formatted', 'final']:
    if not (D/'snapshots'/name).exists():
        continue
    shutil.copytree(W/'baseline', W/(name+'-source'), dirs_exist_ok=True)
    for path in (D/'snapshots'/name).glob('*.gz'):
        relative = path.name.removesuffix('.gz').replace('__', '/')
        (W/(name+'-source')/relative).write_bytes(gzip.decompress(path.read_bytes()))
if a.ox_source:
    ox = a.ox_source.resolve()
    assert subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ox, text=True).strip() == meta['ox_revision']
    subprocess.run(['git', 'diff', '--exit-code', 'HEAD', '--', 'crates', 'Cargo.toml'], cwd=ox, check=True)
    n = W/'native'; n.mkdir(exist_ok=True)
    for engine in ['ferro', 'ox']:
        shutil.copytree(D/'native'/engine, n/engine, dirs_exist_ok=True)
    (n/'ox-source').symlink_to(ox, target_is_directory=True)
    (n/'source').symlink_to(W/'source', target_is_directory=True)
    (n/'cases.json').write_bytes(gzip.decompress((old/'corpus/cases.json.gz').read_bytes()))
    for name in ['case-manifest.json', 'corpus-manifest.json']:
        shutil.copyfile(old/'corpus'/name, n/name)
    for name in ['run.py', 'selected.json']:
        shutil.copyfile(D/'native'/name, n/name)
print(W)
