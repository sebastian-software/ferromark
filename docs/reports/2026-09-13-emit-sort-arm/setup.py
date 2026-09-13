"""Reconstruct the frozen benchmark in an isolated workspace."""
from pathlib import Path
import argparse
import gzip
import hashlib
import io
import json
import shutil
import subprocess
import tarfile

D = Path(__file__).resolve().parent
R = D.parents[2]
p = argparse.ArgumentParser()
p.add_argument('workspace', type=Path)
a = p.parse_args()
W = a.workspace.resolve()
assert not W.exists(), 'choose a new workspace'
W.mkdir(parents=True)
metadata = json.loads((D / 'metadata.json').read_text())
archive = subprocess.check_output(['git', 'archive', metadata['baseline_revision'], 'src'], cwd=R)
(W / 'baseline').mkdir()
with tarfile.open(fileobj=io.BytesIO(archive)) as tar:
    tar.extractall(W / 'baseline', filter='data')
shutil.copyfile(D / 'source-Cargo.toml', W / 'baseline/Cargo.toml')
shutil.copytree(W / 'baseline', W / 'source')
for name in ['driver', 'memory-driver', 'snapshots']:
    shutil.copytree(D / name, W / name)
for name in ['experiment.py', 'variants.py', 'screen.py', 'confirm.py', 'memory.py',
             'verify-cases.py', 'selected.json', 'followup-selected.json']:
    shutil.copyfile(D / name, W / name)
inputs = metadata['inputs']
base = (D / inputs['base']).read_bytes()
assert hashlib.sha256(base).hexdigest() == inputs['base_sha256']
cases = json.loads(gzip.decompress(base))
cases += json.loads(gzip.decompress((D / inputs['additional']).read_bytes()))
(W / 'cases.json').write_text(json.dumps(cases))
print(W)
