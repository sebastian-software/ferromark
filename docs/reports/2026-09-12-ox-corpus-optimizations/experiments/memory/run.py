from pathlib import Path
import json,subprocess,gzip,hashlib
W=Path(__file__).parent
bins={'ferro':W/'ferro/target/release/ox-experiment-driver','ox-grow':Path('/private/tmp/ferromark-memory-audit/ox/target/release/ox-comparison-driver'),'ox-presize':Path('/private/tmp/ferromark-memory-audit/ox/target/release/ox-comparison-driver')}
cases=json.loads((W/'cases.json').read_text());raw=[];checks=[]
for process in range(3):
 workers={n:subprocess.Popen([str(b),str(W/'cases.json')],stdin=subprocess.PIPE,stdout=subprocess.PIPE,text=True) for n,b in bins.items()}
 for i,c in enumerate(cases):
  records={}
  for name,p in workers.items():
   p.stdin.write(json.dumps(dict(index=i,presize=name=='ox-presize'))+'\n');p.stdin.flush();line=p.stdout.readline()
   if not line:raise RuntimeError((name,p.poll(),c['case']))
   record=json.loads(line);records[name]=record
  equal=len({r['normalized'] for r in records.values()})==1
  checks.append(dict(process=process,case=c['case'],normalized_equal=equal,html_sha256={n:hashlib.sha256(r['html'].encode()).hexdigest() for n,r in records.items()}))
  if not equal:
   (W/'mismatches').mkdir(exist_ok=True)
   (W/'mismatches'/f'{i}.json.gz').write_bytes(gzip.compress(json.dumps({'case':c,'outputs':{n:{'html':r['html'],'normalized':r['normalized']} for n,r in records.items()}},indent=2).encode(),mtime=0))
  for name,r in records.items():
   r.update(engine=name,process=process,normalized_equal=equal)
   del r['html'];del r['normalized'];raw.append(r)
 for p in workers.values():
  p.stdin.close();assert p.wait()==0
 print('Process group',process+1,'complete',flush=True)
(W/'raw.jsonl.gz').write_bytes(gzip.compress(('\n'.join(json.dumps(x) for x in raw)+'\n').encode(),mtime=0));(W/'verification.json').write_text(json.dumps(checks,indent=2)+'\n')
# All ten samples per process and all three processes must agree exactly.
summary=[]
for c in cases:
 rows=[r for r in raw if r['case']==c['case']];peaks={}
 for n in bins:
  group=[r for r in rows if r['engine']==n];samples=[s for r in group for s in r['samples']]
  assert all(s==samples[0] for s in samples),(c['case'],n)
  peaks[n]=samples[0]
 summary.append(dict(case=c['case'],input_bytes=rows[0]['input_bytes'],normalized_equal=rows[0]['normalized_equal'],**peaks))
(W/'summary.json').write_text(json.dumps(summary,indent=2)+'\n')
print('\n'.join(f"{r['case']}: Ferro={r['ferro']['peak_live_bytes']} Ox grow={r['ox-grow']['peak_live_bytes']} equal={r['normalized_equal']}" for r in summary))
