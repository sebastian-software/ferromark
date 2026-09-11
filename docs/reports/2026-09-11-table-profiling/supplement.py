#!/usr/bin/env python3
"""Isolate disabled tilde syntax without changing the measured executable."""
from pathlib import Path
import importlib.util, json, statistics, subprocess, sys
HERE=Path(__file__).resolve().parent
spec=importlib.util.spec_from_file_location('table_runner',HERE/'run.py')
m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
ROOT=m.ROOT; WORK=m.WORK
base='| Item | State |\n| --- | --- |\n| **table** | ~~pending~~ done |\n\n'*80
variants={'original':base,'no-tilde':base.replace('~','x'),'no-emphasis':base.replace('*','x'),'plain':base.replace('~','x').replace('*','x')}
for name,cell in [('tilde-only','~~data~~'),('plain-same-length','xxdataxx')]:
    row='|'+f' {cell} |'*4+'\n'
    variants[name]=row+'|'+' --- |'*4+'\n'+row*100+'\n'
out=Path(sys.argv[1]);out.mkdir(parents=True,exist_ok=True)
lanes=['ferro-reuse','pull-reuse','ferro-retained']
def invoke(binary,mode,name,lane):
    root=WORK/'variants'/name
    return json.loads(subprocess.check_output(list(map(str,[binary,mode,'fixture','tables',lane,root,250])),text=True))
for name,value in variants.items():
    path=WORK/'variants'/name/'benches/fixtures/tables-5k.md';path.parent.mkdir(parents=True,exist_ok=True);path.write_text(value)
    o=invoke(WORK/'timing','verify',name,'all')
    assert m.CanonicalHTML(o['ferromark']).tokens==m.CanonicalHTML(o['pulldown']).tokens
    (out/f'{name}.outputs.json').write_text(json.dumps(o,indent=2)+'\n')
rows=[]
with (out/'timings.jsonl').open('w') as f:
    for r in range(9):
        names=list(variants);names=names[r%6:]+names[:r%6]
        ls=lanes[r%3:]+lanes[:r%3]
        if r%2:ls=ls[::-1]
        for name in names:
            for lane in ls:
                x=invoke(WORK/'timing','time',name,lane);x['variant']=name;x['round']=r;rows.append(x);f.write(json.dumps(x)+'\n');f.flush()
        print('supplement round',r+1,flush=True)
summary=[]
for name in variants:
    for lane in lanes:
        values=[x['elapsed_ns']/x['iterations']/1000 for x in rows if x['variant']==name and x['lane']==lane]
        summary.append({'variant':name,'lane':lane,'median_us':statistics.median(values),'windows_us':values})
(out/'summary.json').write_text(json.dumps(summary,indent=2)+'\n')
counts=[]
for name in variants:
    for lane in lanes:
        x=invoke(WORK/'counts','counts',name,lane);x['variant']=name;counts.append(x)
(out/'counts.json').write_text(json.dumps(counts,indent=2)+'\n')
