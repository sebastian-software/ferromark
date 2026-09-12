#!/usr/bin/env python3
"""Restore the measured native heap audit outside Cargo repositories."""
from pathlib import Path
import sys,tarfile,subprocess,shutil,json,hashlib,gzip
D=Path(__file__).parent;OLD=D.parent/'2026-09-12-ox-inspired-optimizations'
if len(sys.argv)!=3:raise SystemExit('Usage: prepare.py NEW_WORKDIR PINNED_OX_CHECKOUT')
W=Path(sys.argv[1]).resolve();ox=Path(sys.argv[2]).resolve()
if W.exists():raise SystemExit('Choose a new workspace directory.')
revision=subprocess.check_output(['git','rev-parse','HEAD'],cwd=ox,text=True).strip()
if revision!='026d1859d1c35e5fb1ea65e7e855b428a918b9bb':raise SystemExit('Ox checkout must match the recorded revision.')
if subprocess.check_output(['git','status','--porcelain','--untracked-files=no','--','crates'],cwd=ox):raise SystemExit('Ox crates must be unmodified.')
for p in [W,*W.parents,Path.home()]:
 if any((p/'.cargo'/name).exists() for name in ['config','config.toml']):raise SystemExit(f'Cargo configuration under {p} may alter the build.')
W.mkdir(parents=True);(W/'source').mkdir()
with tarfile.open(OLD/'baseline-src.tar.gz') as tar:tar.extractall(W/'source',filter='data')
shutil.copy2(OLD/'library-Cargo.toml',W/'source/Cargo.toml')
subprocess.run(['git','apply',str(OLD/'retained.patch')],cwd=W/'source',check=True)
expected=json.loads((OLD/'confirmation/source-hashes.json').read_text())
for f,h in expected.items():assert hashlib.sha256((W/'source'/f).read_bytes()).hexdigest()==h,f
for role in ['ferro','ox']:
 shutil.copytree(D/role,W/role)
 p=W/role/'Cargo.toml';text=p.read_text()
 if role=='ox':
  for crate in ['allocator','parser','renderer']:
   old='"../../ferromark-ox-audit/ox-current/crates/ox_content_'+crate+'"'
   text=text.replace(old,json.dumps(str(ox/'crates'/('ox_content_'+crate))))
  p.write_text(text)
(W/'cases.json').write_bytes(gzip.decompress((D/'cases.json.gz').read_bytes()))
shutil.copy2(D/'run.py',W/'run.py')
print(f'Restored {W}; Ferromark source hashes and Ox revision match.')
