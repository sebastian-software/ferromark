"""Reconstruct the frozen independent experiment outside the repository."""
from pathlib import Path
import argparse,gzip,hashlib,json,shutil,subprocess
p=argparse.ArgumentParser();p.add_argument('repository',type=Path);p.add_argument('work',type=Path);a=p.parse_args();A=Path(__file__).parent;W=a.work.resolve();R=a.repository.resolve();W.mkdir(parents=True,exist_ok=False)
revision='a1c308cba63f7bb038699df8560daa8959159b4e'
for name,expected in json.loads((A/'source-hashes.json').read_text()).items():
 data=subprocess.check_output(['git','show',f'{revision}:{name}'],cwd=R)
 assert hashlib.sha256(data).hexdigest()==expected,name
 for folder in ['baseline','source']:
  out=W/folder/name;out.parent.mkdir(parents=True,exist_ok=True);out.write_bytes(data)
for folder in ['baseline','source']:shutil.copyfile(A/'source-Cargo.toml',W/folder/'Cargo.toml')
for name in ['cases.json','verify.json','production-source.json']:(W/name).write_bytes(gzip.decompress((A/(name+'.gz')).read_bytes()))
for name in ['selected.json','variants.py','experiment.py']:shutil.copyfile(A/name,W/name)
(W/'driver/src').mkdir(parents=True)
for name in ['Cargo.toml','Cargo.lock']:shutil.copyfile(A/'driver'/name,W/'driver'/name)
shutil.copyfile(A/'driver/src/main.rs',W/'driver/src/main.rs')
(W/'rust-toolchain.toml').write_text('[toolchain]\nchannel = "1.97.1"\nprofile = "minimal"\n')
print(W)
