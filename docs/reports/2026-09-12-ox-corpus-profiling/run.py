from pathlib import Path
import json,subprocess,gzip,hashlib,statistics,sys
W=Path(__file__).parent
class Worker:
 def __init__(self,engine,profile='release'):
  self.engine=engine
  self.p=subprocess.Popen([str(W/'bin'/f'{engine.split("-")[0]}-{profile}'),str(W/'cases.json'), 'presize' if engine.endswith('presize') else 'grow'],stdin=subprocess.PIPE,stdout=subprocess.PIPE,text=True)
 def call(self,op,index,ms=0):
  self.p.stdin.write(json.dumps(dict(op=op,index=index,ms=ms))+'\n');self.p.stdin.flush();line=self.p.stdout.readline()
  if not line:raise RuntimeError(f'{self.engine} exited {self.p.poll()} on {index}')
  return json.loads(line)
 def close(self):self.p.stdin.close();self.p.wait()
def sha(s):return hashlib.sha256(s.encode()).hexdigest()
def verify():
 cases=json.loads((W/'case-manifest.json').read_text());rows=[]
 workers=[Worker(n) for n in ['ferro','ox-grow','ox-presize']]
 with gzip.open(W/'mismatches.jsonl.gz','wt') as mismatch:
  for i,c in enumerate(cases):
   f,o,p=[w.call('verify',i) for w in workers]
   assert o==p,'Arena configuration changed output'
   equal=f['normalized']==o['normalized']
   row=dict(index=i,case=c['case'],equal=equal,exact_equal=f['html']==o['html'],ferro_html_bytes=len(f['html'].encode()),ox_html_bytes=len(o['html'].encode()),ferro_sha256=sha(f['normalized']),ox_sha256=sha(o['normalized']))
   if not equal:
    a,b=f['normalized'],o['normalized'];prefix=next((j for j in range(min(len(a),len(b))) if a[j]!=b[j]),min(len(a),len(b)))
    row['first_difference']={'ferro':a[max(0,prefix-100):prefix+180],'ox':b[max(0,prefix-100):prefix+180]}
    if c['kind'] == 'file':
     mismatch.write(json.dumps(dict(index=i,case=c['case'],ferro=f,ox=o))+'\n')
   rows.append(row)
 for w in workers:w.close()
 (W/'verification.json').write_text(json.dumps(rows,indent=2)+'\n')
 print('Output verification:',sum(x['equal'] for x in rows),'/',len(rows),'normalized equal',flush=True)
 for project in sorted(set(c['project'] for c in cases)):
  selected=[r for r,c in zip(rows,cases) if c['project']==project and c['kind']=='file'];print(project,sum(r['equal'] for r in selected),'/',len(selected),flush=True)
def timing(tag,selected=None,rounds=3,ms=8):
 cases=json.loads((W/'case-manifest.json').read_text());verified=json.loads((W/'verification.json').read_text());workers={n:Worker(n) for n in ['ferro','ox-grow','ox-presize']};raw=[];summary=[]
 indices=range(len(cases)) if selected is None else selected
 for i in indices:
  c=cases[i];vals={n:[] for n in workers}
  for w in workers.values():w.call('window',i,3 if tag=='screen' else 35)
  for r in range(rounds):
   names=list(workers);names=names[r%3:]+names[:r%3]
   for n in names:
    v=workers[n].call('window',i,ms);v.update(index=i,engine=n,round=r);raw.append(v);vals[n].append(v['elapsed_ns']/v['count'])
  row=dict(index=i,case=c['case'],bytes=c['bytes'],equal=verified[i]['equal'],median_ns={n:statistics.median(v) for n,v in vals.items()},min_ns={n:min(v) for n,v in vals.items()},max_ns={n:max(v) for n,v in vals.items()})
  summary.append(row)
  if i%100==0:print(tag,'completed index',i,flush=True)
 for w in workers.values():w.close()
 (W/f'{tag}-raw.jsonl.gz').write_bytes(gzip.compress(('\n'.join(json.dumps(x) for x in raw)+'\n').encode(),mtime=0));(W/f'{tag}-summary.json').write_text(json.dumps(summary,indent=2)+'\n')
 print(tag,'complete',len(summary),flush=True)
if __name__=='__main__':
 if sys.argv[1]=='verify':verify()
 elif sys.argv[1]=='screen':timing('screen')
 else:timing(sys.argv[1],json.loads((W/'selected.json').read_text()),rounds=9,ms=60)
