from pathlib import Path
import shutil,subprocess,os,json,hashlib
from experiment import W
from variants import variant
old=Path('/private/tmp/ferromark-ox-implementation/corpus');c=W/'corpus';c.mkdir(exist_ok=True)
for f in ['cases.json','case-manifest.json','corpus-manifest.json','selected.json','run.py']:shutil.copyfile(old/f,c/f)
(c/'ferro').mkdir(exist_ok=True);shutil.copytree(old/'ferro/src',c/'ferro/src',dirs_exist_ok=True)
for f in ['Cargo.toml','Cargo.lock']:shutil.copyfile(old/'ferro'/f,c/'ferro'/f)
(c/'source').mkdir(exist_ok=True);shutil.copyfile(W/'baseline/Cargo.toml',c/'source/Cargo.toml')
files=variant('production',{str(p.relative_to(W/'baseline')):p.read_text() for p in (W/'baseline/src').rglob('*.rs')})
for f,s in files.items():
 p=c/'source'/f;p.parent.mkdir(parents=True,exist_ok=True);p.write_text(s)
env=dict(os.environ);env.pop('RUSTFLAGS',None);env.pop('CARGO_ENCODED_RUSTFLAGS',None)
with (c/'build.log').open('w') as out:subprocess.run(['cargo','build','--release','--offline','--locked','--manifest-path',str(c/'ferro/Cargo.toml')],cwd=c,env=env,stdout=out,stderr=subprocess.STDOUT,check=True)
(c/'bin').mkdir(exist_ok=True);shutil.copy2(c/'ferro/target/release/corpus-ferro',c/'bin/ferro-release');shutil.copy2(old/'bin/ox-release',c/'bin/ox-release')
(c/'metadata.json').write_text(json.dumps(dict(ox_revision='026d1859d1c35e5fb1ea65e7e855b428a918b9bb',source_hashes={f:hashlib.sha256(s.encode()).hexdigest() for f,s in files.items()},binary_hashes={p.name:hashlib.sha256(p.read_bytes()).hexdigest() for p in (c/'bin').iterdir()}),indent=2))
print('Corpus driver built',flush=True)
