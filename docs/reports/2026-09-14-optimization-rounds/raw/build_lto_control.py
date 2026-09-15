"""Rebuild the ORIGINAL worker with LTO disabled; preserve all other settings."""
import json,os,shutil,subprocess,hashlib
from pathlib import Path
old=Path('/private/tmp/ferromark-v2-simd-round/build-once')
new=Path('/private/tmp/ferromark-v2-rounds-2/build-probe-no-lto')
new.mkdir(exist_ok=False)
record=json.loads((old/'build.json').read_text()); record['profile']={'lto':False,'opt-level':3,'codegen-units':1,'panic':'abort','strip':True}
record['control']='Original single-link-probe experiment, only LTO changed from fat to false'
env=os.environ.copy(); env.pop('CARGO_ENCODED_RUSTFLAGS',None); env['RUSTFLAGS']='-C target-cpu=generic'
for name in ('baseline','candidate'):
 root=new/name; (root/'src').mkdir(parents=True)
 for file in ('Cargo.toml','Cargo.lock','src/main.rs'):
  shutil.copyfile(old/name/file,root/file)
 p=root/'Cargo.toml'; p.write_text(p.read_text().replace('lto = "fat"','lto = false'))
 env['CARGO_TARGET_DIR']=str(root/'target')
 with (root/'build.log').open('w') as log:
  subprocess.run(['cargo','+1.95','build','--release','--offline','--locked'],cwd=root,env=env,stdout=log,stderr=log,check=True)
 binary=root/'target/release/simd-round-worker'
 record['engines'][name]['binary']=str(binary)
 record['engines'][name]['binary_sha256']=hashlib.sha256(binary.read_bytes()).hexdigest()
 record['engines'][name]['command']=['cargo','+1.95','build','--release','--offline','--locked']
(new/'build.json').write_text(json.dumps(record,indent=2)+'\n')
print(new/'build.json')
