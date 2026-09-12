"""Repeat one archived selection after building the baseline and candidate."""
from experiment import W,run,verify
from regimes import run_regimes
import argparse,json
runs=json.loads((W/'runs.json').read_text())
p=argparse.ArgumentParser();p.add_argument('run',choices=list(runs));a=p.parse_args();r=runs[a.run]
assert verify(r['name'])
if r['operation']=='window-single':
 assert r['indices']==json.loads((W/'regime-selected.json').read_text()) and r['warm']==20 and r['order']==0
 run_regimes(r['name'],rounds=r['rounds'],ms=r['ms'],tag=r['tag'].split('/')[1])
else:run(r['name'],rounds=r['rounds'],ms=r['ms'],warm=r['warm'],order=r['order'],tag=r['tag'],indices=r['indices'])
