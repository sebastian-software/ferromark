from pathlib import Path
import subprocess,shutil,os,json,re
from run import W,Worker
env=dict(os.environ);env.pop('RUSTFLAGS',None);env.pop('CARGO_ENCODED_RUSTFLAGS',None)
with (W/'build-counters.log').open('w') as log:subprocess.run(['cargo','build','--release','--offline','--locked','--features','counters','--manifest-path',str(W/'ferro/Cargo.toml')],cwd=W,env=env,stdout=log,stderr=subprocess.STDOUT,check=True)
shutil.copy2(W/'ferro/target/release/corpus-ferro',W/'bin/ferro-counters')
w=Worker('ferro','counters');rows=[]
for i,c in enumerate(json.loads((W/'case-manifest.json').read_text())):
 snap=w.call('counters',i)['snapshot'];rows.append(dict(index=i,case=c['case'],counts={k:int(v) for k,v in re.findall(r'(\w+): (\d+)',snap)}))
w.close();(W/'counters.json').write_text(json.dumps(rows,indent=2)+'\n');print('Counters complete',len(rows),flush=True)
