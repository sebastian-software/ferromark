"""Reconstruct the frozen baseline and measured text-escape candidate."""
from pathlib import Path
import argparse,gzip,hashlib,io,json,shutil,subprocess,tarfile
p=argparse.ArgumentParser();p.add_argument('destination',type=Path);a=p.parse_args()
D=Path(__file__).resolve().parent;R=D.parents[2];W=a.destination.resolve()
W.mkdir(parents=True,exist_ok=True)
meta=json.loads((D/'metadata.json').read_text())
def copy(src,dst):
 dst.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(src,dst)
old=R/'docs/reports/2026-09-12-block-render-hotspots'
for name in ['cases','verify','blocks','extra','extended','fences']:
 (W/(name+'.json')).write_bytes(gzip.decompress((old/(name+'.json.gz')).read_bytes()))
cases=json.loads((W/'cases.json').read_text());assert len(cases)==718
cases.extend(json.loads(gzip.decompress((D/'regime-cases.json.gz').read_bytes())))
(W/'cases.json').write_text(json.dumps(cases))
# Preserve the recorded corpus serialization, as well as its semantic content.
for f,h in meta['input_sha256'].items():
 assert hashlib.sha256((W/f).read_bytes()).hexdigest()==h,f
for f in ['experiment.py','run-recorded.py','runs.json','variants.py','regimes.py','verify-all.py','verify-timing.py','verify-blocks.py','verify-extra.py','verify-extended.py','verify-fences.py','selected.json','regime-selected.json']:copy(D/f,W/f)
shutil.copytree(D/'driver',W/'driver',dirs_exist_ok=True)
(W/'baseline').mkdir(exist_ok=True)
archive=subprocess.check_output(['git','archive',meta['baseline_revision'],'src'],cwd=R)
with tarfile.open(fileobj=io.BytesIO(archive)) as tar:tar.extractall(W/'baseline',filter='data')
copy(D/'source-Cargo.toml',W/'baseline/Cargo.toml')
for name in ['source','direct-neon-source']:shutil.copytree(W/'baseline',W/name,dirs_exist_ok=True)
subprocess.run(['patch','-p1','-i',str(D/'measurements/direct-neon/patch.diff')],cwd=W/'direct-neon-source',check=True)
for name,source in [('baseline','baseline'),('direct-neon','direct-neon-source')]:
 for f,h in json.loads((D/'measurements'/name/'source.json').read_text()).items():
  assert hashlib.sha256((W/source/f).read_bytes()).hexdigest()==h,(name,f)
print(W)
