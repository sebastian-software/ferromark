#!/usr/bin/env python3
"""Restore self-contained experiment workspaces; no builds or network writes."""
from pathlib import Path
import sys,shutil,gzip,tarfile,subprocess,json,hashlib
D=Path(__file__).parent
if len(sys.argv)!=2:raise SystemExit('Usage: prepare.py /tmp/ferromark-ox-rerun')
W=Path(sys.argv[1]).resolve()
if W.exists():raise SystemExit('Choose a new empty directory outside Cargo repositories.')
W.mkdir(parents=True)
for p in [W,*W.parents,Path.home()]:
 for q in [p/'.cargo/config',p/'.cargo/config.toml']:
  if q.exists():raise SystemExit(f'Cargo configuration could change the target flags: {q}')
def expand(src,dst):
 shutil.copytree(src,dst,dirs_exist_ok=True)
 for p in dst.glob('*.json.gz'):p.with_suffix('').write_bytes(gzip.decompress(p.read_bytes()))
expand(D/'screen',W)
for n in ['baseline','source']:
 (W/n).mkdir(exist_ok=True)
 with tarfile.open(D/'baseline-src.tar.gz') as tar:tar.extractall(W/n,filter='data')
 shutil.copy2(D/'library-Cargo.toml',W/n/'Cargo.toml')
expand(D/'confirmation',W/'confirmation')
f=W/'confirmation'
for n in ['baseline','source','retained']:shutil.copytree(W/'baseline',f/n,dirs_exist_ok=True)
subprocess.run(['git','apply',str(D/'retained.patch')],cwd=f/'retained',check=True)
for rel,sha in json.loads((D/'confirmation/source-hashes.json').read_text()).items():
 assert hashlib.sha256((f/'retained'/rel).read_bytes()).hexdigest()==sha,rel
shutil.copy2(D/'screen/variants.py',f/'variants.py')
shutil.copy2(D/'confirmation/confirm.py',f/'confirm.py')
shutil.copytree(D/'allocation',W/'allocation',dirs_exist_ok=True)
(W/'allocation/source').mkdir(exist_ok=True);shutil.copy2(D/'library-Cargo.toml',W/'allocation/source/Cargo.toml');(W/'allocation/bin').mkdir(exist_ok=True)
shutil.copytree(D/'ox-driver',W/'ox-driver',dirs_exist_ok=True)
p=W/'ox-driver/Cargo.toml';p.write_text(p.read_text().replace('../../ferromark-ox-audit/ox-current','../ox-current'))
shutil.copy2(D/'extra.py',W/'extra.py')
print(f'Restored sources and harnesses to {W}; retained source hashes match.')
