"""Replay one archived pair, rebuilding both native libraries first."""
from experiment import *
from regimes import run_regimes
import argparse,sys
p=argparse.ArgumentParser();p.add_argument('run');a=p.parse_args()
r=json.loads((W/'runs.json').read_text())[a.run];name=r['candidate']
assert build('baseline') and verify('baseline')
if name=='aa-control':
 (W/name).mkdir(exist_ok=True);shutil.copy2(W/'bin/baseline',W/'bin'/name)
else:assert build(name) and verify(name)
subprocess.run([sys.executable,str(W/'verify-all.py'),'baseline',name],check=True)
if r['operation']=='window':
 run(name,rounds=r['rounds'],ms=r['ms'],tag=a.run,order=r['order'],warm=r['warmup_ms'],indices=r['indices'])
else:
 selection='replay-selected.json';(W/selection).write_text(json.dumps(r['indices']))
 run_regimes(name,rounds=r['rounds'],ms=r['ms'],tag=a.run.split('/',1)[1],casefile=r['casefile'],selection=selection)
