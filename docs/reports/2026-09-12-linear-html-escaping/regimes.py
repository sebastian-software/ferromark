from experiment import *
def run_regimes(name,rounds=5,ms=30,tag='regime-screen', casefile='cases.json', selection='regime-selected.json'):
 dest=W/name/tag;dest.mkdir(parents=True,exist_ok=True);ids=json.loads((W/selection).read_text());cases=json.loads((W/casefile).read_text());workers={n:Worker(W/'bin'/n,W/casefile) for n in ['baseline',name]};raw=[];summary=[]
 for i in ids:
  vals={n:[] for n in workers}
  answers=[w.call('html',i) for w in workers.values()];assert answers[0]==answers[1],i
  for w in workers.values():w.call('window-single',i,20)
  for r in range(rounds):
   for n in (list(workers) if r%2==0 else list(reversed(workers))):
    x=workers[n].call('window-single',i,ms);x.update(engine=n,round=r,index=i);raw.append(x);vals[n].append(x['elapsed_ns']/x['count'])
  a,b=[statistics.median(vals[n]) for n in workers];row=dict(index=i,case=cases[i]['case'],baseline_ns=a,candidate_ns=b,change_pct=(b/a-1)*100);summary.append(row)
  print(name,row['case'],round(a/1e6,4),round(b/1e6,4),round(row['change_pct'],2),flush=True)
  (dest/'windows.jsonl.gz').write_bytes(gzip.compress(('\n'.join(json.dumps(x) for x in raw)+'\n').encode(),mtime=0));(dest/'summary.json').write_text(json.dumps(summary,indent=2)+'\n')
 for w in workers.values():w.close()
 (dest/'complete.json').write_text(json.dumps(dict(expected=len(ids),measured=len(summary),rounds=rounds,window_ms=ms,warmup_ms=20,operation='window-single'))+'\n')
if __name__=='__main__':
 import sys
 for name in sys.argv[1:]:
  assert build(name) and verify(name)
  if name!='baseline':run_regimes(name)
