#!/usr/bin/env python3
"""Confirm the retained implementation using builds of the real repository."""
from pathlib import Path
import hashlib,importlib.util,json,statistics
HERE=Path(__file__).resolve().parent
spec=importlib.util.spec_from_file_location('variants',HERE/'experiment.py')
e=importlib.util.module_from_spec(spec);spec.loader.exec_module(e)
out=HERE/'production-confirmation';out.mkdir(exist_ok=False)
roots={}
for case,content in e.fixtures().items():
 if case not in ['plain','mixed','links']:continue
 root=e.WORK/'inputs'/case;roots[case]=root
 old=e.invoke(e.WORK/'bin/production-before','verify',root,'all',250)
 new=e.invoke(e.WORK/'bin/production-after','verify',root,'all',250)
 assert old==new,case
 (out/(case+'.outputs.json')).write_text(json.dumps(new,indent=2)+'\n')
(out/'protocol.json').write_text(json.dumps({'rounds':9,'window_ms':250,'warmup_renders':32,'clock_batch':16,'order':'rotate cases; rotate/reverse before-after-pulldown triplets within each lifecycle','builds':'real repository; same original probe and dependency lock'},indent=2)+'\n')
# Warm the machine and all three paths before the recorded sequence.
for version,lane in [('production-before','ferro-reuse'),('production-after','ferro-reuse'),('production-after','pull-reuse')]:
 e.invoke(e.WORK/'bin'/version,'time',roots['plain'],lane,1000)
rows=[];names=list(roots)
with (out/'timings.jsonl').open('w') as stream:
 for r in range(9):
  for i,case in enumerate(names[r%3:]+names[:r%3]):
   for lifecycle in (['reuse','owned'] if r%2==0 else ['owned','reuse']):
    paths=[('before','production-before','ferro-'+lifecycle),('after','production-after','ferro-'+lifecycle),('pulldown','production-after','pull-'+lifecycle)]
    offset=(r+i)%3;paths=paths[offset:]+paths[:offset]
    if r%2:paths=paths[::-1]
    for name,binary,lane in paths:
     x=e.invoke(e.WORK/'bin'/binary,'time',roots[case],lane,250)
     x.update(case=case,version=name,lifecycle=lifecycle,round=r)
     rows.append(x);stream.write(json.dumps(x)+'\n');stream.flush()
  print(f'production confirmation {r+1}/9',flush=True)
summary=[]
for case in names:
 for lifecycle in ['reuse','owned']:
  values={name:[x['elapsed_ns']/x['iterations']/1000 for x in rows if x['case']==case and x['lifecycle']==lifecycle and x['version']==name] for name in ['before','after','pulldown']}
  change=[100*(b/a-1) for a,b in zip(values['before'],values['after'])]
  versus=[100*(b/a-1) for a,b in zip(values['pulldown'],values['after'])]
  record={'case':case,'lifecycle':lifecycle,'median_us':{name:statistics.median(v) for name,v in values.items()},'windows_us':values,'change_percent':change,'versus_pulldown_percent':versus,'median_change_percent':statistics.median(change),'median_versus_pulldown_percent':statistics.median(versus)}
  summary.append(record);print(case,lifecycle,record['median_us'],round(record['median_change_percent'],2),round(record['median_versus_pulldown_percent'],2),flush=True)
(out/'summary.json').write_text(json.dumps(summary,indent=2)+'\n')
