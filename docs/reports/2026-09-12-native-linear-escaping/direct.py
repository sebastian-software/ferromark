"""Replay the direct API controls with the recorded allocation policies."""
from experiment import *
direct=json.loads((W/'direct.json').read_text())
def direct_run(name,pair):
 dest=W/name/f'direct-{pair}';dest.mkdir(parents=True,exist_ok=True);workers={n:Worker(W/'bin'/n,W/'direct.json') for n in ['baseline',name]};raw=[];summary=[]
 for i,c in enumerate(direct):
  actual=[w.call('escape',i) for w in workers.values()];assert actual[0]==actual[1],(name,i)
  op='window-escape-single' if len(c['input'])>=65536 and 'quotes-' in c['case'] else 'window-escape'
  vals={n:[] for n in workers}
  for w in workers.values():w.call(op,i,40)
  for r in range(5):
   for n in (list(workers) if (r+pair)%2==0 else list(reversed(workers))):
    x=workers[n].call(op,i,50);x.update(engine=n,round=r,index=i,operation=op);raw.append(x);vals[n].append(x['elapsed_ns']/x['count'])
  a,b=[statistics.median(vals[n]) for n in workers];summary.append(dict(index=i,case=c['case'],baseline_ns=a,candidate_ns=b,change_pct=(b/a-1)*100))
 for w in workers.values():w.close()
 (dest/'windows.jsonl.gz').write_bytes(gzip.compress(('\n'.join(json.dumps(x) for x in raw)+'\n').encode(),mtime=0));(dest/'summary.json').write_text(json.dumps(summary,indent=2))
 print(name,'direct',pair,' '.join(f"{r['case']}={r['change_pct']:+.1f}%" for r in summary),flush=True)

if __name__=='__main__':
 import argparse
 p=argparse.ArgumentParser();p.add_argument('candidate');p.add_argument('--pair',type=int,default=1);a=p.parse_args();direct_run(a.candidate,a.pair)
