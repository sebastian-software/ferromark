"""Reconstruct this report's native harnesses and frozen sources."""
from pathlib import Path
import argparse,gzip,subprocess,shutil,tarfile,io,json,hashlib
p=argparse.ArgumentParser();p.add_argument('destination',type=Path);p.add_argument('--ox-source',type=Path,required=True);a=p.parse_args()
D=Path(__file__).resolve().parent;R=D.parents[2];W=a.destination.resolve();W.mkdir(parents=True,exist_ok=True)
meta=json.loads((D/'metadata.json').read_text());ox=a.ox_source.resolve()
assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=ox,text=True).strip()==meta['ox_revision']
subprocess.run(['git','diff','--exit-code','HEAD','--','crates','Cargo.toml'],cwd=ox,check=True,stdout=subprocess.DEVNULL)
if not (W/'ox-source').exists():(W/'ox-source').symlink_to(ox,target_is_directory=True)
def copy(src,dst):dst.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(src,dst)
old=R/'docs/reports/2026-09-12-block-render-hotspots';original=R/'docs/reports/2026-09-12-ox-corpus-profiling'
(W/'cases.json').write_bytes(gzip.decompress((old/'corpus/cases.json.gz').read_bytes()))
assert hashlib.sha256((W/'cases.json').read_bytes()).hexdigest()==meta['cases_sha256']
for f in ['case-manifest.json','corpus-manifest.json']:copy(old/'corpus'/f,W/f)
for f in ['selected.json','build.py','run.py','profile.py','prepare-attribution.py','attribution-profile.py','analyze-profiles.py']:copy(D/f,W/f)
for e in ['ferro','ox']:shutil.copytree(D/'harnesses'/e,W/e,dirs_exist_ok=True)
(W/'source').mkdir(exist_ok=True);copy(D/'source-Cargo.toml',W/'source/Cargo.toml')
archive=subprocess.check_output(['git','archive',meta['ferromark_revision'],'src'],cwd=R)
with tarfile.open(fileobj=io.BytesIO(archive)) as tar:tar.extractall(W/'source',filter='data')
for f,h in meta['source_sha256'].items():assert hashlib.sha256((W/'source'/f).read_bytes()).hexdigest()==h,f
shutil.copytree(original/'demangle',W/'demangle',dirs_exist_ok=True)
E=W/'experiments';E.mkdir(exist_ok=True)
for f in ['experiment.py','variants.py','verify-timing.py','verify-blocks.py','verify-extra.py','verify-extended.py','verify-fences.py','confirmation-selected.json','confirm.py','screen.py','continue-screen.py','guarded-screen.py','autolink-screen.py']:copy(D/'experiments'/f,E/f)
for name in ['cases','verify','blocks','extra','extended','fences']:(E/(name+'.json')).write_bytes(gzip.decompress((old/(name+'.json.gz')).read_bytes()))
copy(old/'selected.json',E/'selected.json')
shutil.copytree(old/'driver',E/'driver',dirs_exist_ok=True)
for name in ['baseline','source']:shutil.copytree(W/'source',E/name,dirs_exist_ok=True)
print(W)
