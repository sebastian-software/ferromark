from pathlib import Path
import argparse,json,subprocess,shutil,hashlib,gzip,statistics,difflib,os
from variants import variant
W=Path(__file__).parent
class Worker:
 def __init__(self,binary,cases):self.p=subprocess.Popen([str(binary),str(cases)],stdin=subprocess.PIPE,stdout=subprocess.PIPE,text=True)
 def call(self,op,index,ms=0):
  self.p.stdin.write(json.dumps(dict(op=op,index=index,ms=ms))+'\n');self.p.stdin.flush();line=self.p.stdout.readline()
  if not line:raise RuntimeError(f'worker exited {self.p.poll()}')
  return json.loads(line)
 def close(self):self.p.stdin.close();self.p.wait()
def digest(x):return hashlib.sha256(json.dumps(x,sort_keys=True).encode()).hexdigest()
def build(name):
 files={str(p.relative_to(W/'baseline')):p.read_text() for p in (W/'baseline/src').rglob('*.rs')};base=files.copy();files=variant(name,files)
 for f,s in files.items():(W/'source'/f).write_text(s)
 dest=W/name;dest.mkdir(exist_ok=True);(W/'bin').mkdir(exist_ok=True)
 (dest/'patch.diff').write_text(''.join(''.join(difflib.unified_diff(base[f].splitlines(True),s.splitlines(True),fromfile='a/'+f,tofile='b/'+f)) for f,s in files.items() if s!=base[f]))
 env=os.environ.copy();env.pop('RUSTFLAGS',None);env.pop('CARGO_ENCODED_RUSTFLAGS',None)
 with (dest/'build.log').open('w') as out:r=subprocess.run(['cargo','build','--release','--offline','--locked','--manifest-path',str(W/'driver/Cargo.toml')],cwd=W,env=env,stdout=out,stderr=subprocess.STDOUT)
 if r.returncode:print('BUILD FAILED',name,flush=True);return False
 shutil.copy2(W/'driver/target/release/ox-experiment-driver',W/'bin'/name)
 (dest/'source.json').write_text(json.dumps({f:hashlib.sha256(s.encode()).hexdigest() for f,s in files.items()},indent=2));print('Built',name,flush=True);return True
def verify(name):
 hashes=[]
 for casefile,op,indices in [('verify.json','verify',range(2046)),('cases.json','html',range(45,694))]:
  w=Worker(W/'bin'/name,W/casefile)
  for i in indices:hashes.append(digest(w.call(op,i)))
  w.close()
 if name=='baseline':(W/'baseline-verification.json').write_text(json.dumps(hashes))
 expected=json.loads((W/'baseline-verification.json').read_text());failed=[i for i,(a,b) in enumerate(zip(expected,hashes)) if a!=b]
 (W/name/'verification.json').write_text(json.dumps(dict(count=len(hashes),sha256=digest(hashes),failed=failed)))
 print('Verified',name,len(hashes),'failures',len(failed),flush=True);return not failed
def run(name,rounds=5,ms=30,tag=None,order=0,warm=60,indices=None):
 dest=W/(tag or name);dest.mkdir(exist_ok=True)
 selected=json.loads((W/'selected.json').read_text()) if indices is None else indices;cases=json.loads((W/'cases.json').read_text());workers={n:Worker(W/'bin'/n,W/'cases.json') for n in ['baseline',name]};raw=[];summary=[]
 for i in selected:
  vals={n:[] for n in workers}
  for w in workers.values():w.call('window',i,warm)
  for round in range(rounds):
   for n in (list(workers) if (round+order)%2==0 else list(reversed(workers))):
    x=workers[n].call('window',i,ms);x.update(engine=n,round=round,index=i);raw.append(x);vals[n].append(x['elapsed_ns']/x['count'])
  a,b=[statistics.median(vals[n]) for n in workers];summary.append(dict(index=i,case=cases[i]['case'],baseline_ns=a,candidate_ns=b,change_pct=(b/a-1)*100))
 for w in workers.values():w.close()
 (dest/'windows.jsonl.gz').write_bytes(gzip.compress(('\n'.join(json.dumps(x) for x in raw)+'\n').encode(),mtime=0));(dest/'summary.json').write_text(json.dumps(summary,indent=2))
 print(name,'median',round_value(statistics.median(x['change_pct'] for x in summary)),' '.join(x['case']+'='+str(round_value(x['change_pct']))+'%' for x in summary if x['index'] in [0,2,3,6,7,9,10,45,79,108,487,612,613]),flush=True)
def round_value(x):return round(x,1)
if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('names',nargs='+');p.add_argument('--rounds',type=int,default=5);p.add_argument('--ms',type=int,default=30);p.add_argument('--runs',type=int,default=1);p.add_argument('--no-build',action='store_true');a=p.parse_args()
 for name in a.names:
  if (a.no_build or build(name)) and verify(name) and name!='baseline':
   for r in range(a.runs):run(name,a.rounds,a.ms,None if a.runs==1 else name+'/confirm-'+str(r+1),r)
