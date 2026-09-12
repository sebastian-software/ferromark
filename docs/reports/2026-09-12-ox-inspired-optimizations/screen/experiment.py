from pathlib import Path
import argparse,json,subprocess,shutil,hashlib,gzip,statistics,difflib,sys
from variants import transform
W=Path(__file__).parent
class Worker:
 def __init__(self,binary,cases):self.p=subprocess.Popen([str(binary),str(cases)],stdin=subprocess.PIPE,stdout=subprocess.PIPE,text=True)
 def call(self,op,index,ms=0):
  self.p.stdin.write(json.dumps(dict(op=op,index=index,ms=ms))+'\n');self.p.stdin.flush();line=self.p.stdout.readline()
  if not line:raise RuntimeError(f'worker exited {self.p.poll()}')
  return json.loads(line)
 def close(self):self.p.stdin.close();self.p.wait()
def digest(x):return hashlib.sha256(json.dumps(x,sort_keys=True).encode()).hexdigest()
def baseline():
 (W/'bin').mkdir(exist_ok=True);shutil.copy2(W/'driver/target/release/ox-experiment-driver',W/'bin/baseline')
 w=Worker(W/'bin/baseline',W/'verify.json');result=[]
 for i in range(len(json.loads((W/'verify.json').read_text()))):result.append(digest(w.call('verify',i)))
 w.close();(W/'baseline-verification.json').write_text(json.dumps(result));print('Baseline verified',len(result),flush=True)
def build(name):
 files={str(p.relative_to(W/'baseline')):p.read_text() for p in (W/'baseline/src').rglob('*.rs')}
 base=files.copy();files=transform(name,files)
 for f,s in files.items():(W/'source'/f).write_text(s)
 dest=W/name;dest.mkdir(exist_ok=True)
 (dest/'patch.diff').write_text(''.join(''.join(difflib.unified_diff(base[f].splitlines(True),s.splitlines(True),fromfile='a/'+f,tofile='b/'+f)) for f,s in files.items() if s!=base[f]))
 with (dest/'build.log').open('w') as out:r=subprocess.run(['cargo','build','--release','--offline','--locked','--manifest-path',str(W/'driver/Cargo.toml')],cwd=W,stdout=out,stderr=subprocess.STDOUT)
 if r.returncode:print('BUILD FAILED',name,flush=True);return False
 shutil.copy2(W/'driver/target/release/ox-experiment-driver',W/'bin'/name)
 (dest/'source.json').write_text(json.dumps({f:hashlib.sha256(s.encode()).hexdigest() for f,s in files.items()},indent=2))
 return True
def run(name,rounds=5,ms=35,select=None,tag=None):
 dest=W/(tag or name);dest.mkdir(exist_ok=True)
 w=Worker(W/'bin'/name,W/'verify.json');expected=json.loads((W/'baseline-verification.json').read_text());checks=[]
 for i,target in enumerate(expected):
  actual=w.call('verify',i);h=digest(actual)
  if h!=target:
   (dest/'failure.json').write_text(json.dumps({'index':i,'input':json.loads((W/'verify.json').read_text())[i],'actual':actual},indent=2));w.close();print('VERIFY FAILED',name,i,flush=True);return False
  checks.append(h)
 w.close();(dest/'verification.json').write_text(json.dumps({'count':len(checks),'sha256':digest(checks)}))
 cases=json.loads((W/'cases.json').read_text());workers={n:Worker(W/'bin'/n,W/'cases.json') for n in ['baseline',name]};raw=[];summary=[]
 for i,c in enumerate(cases):
  if select and c['case'] not in select:continue
  vals={n:[] for n in workers}
  for w in workers.values():w.call('window',i,100)
  for round in range(rounds):
   for n in (list(workers) if round%2==0 else list(reversed(workers))):
    x=workers[n].call('window',i,ms);x.update(engine=n,round=round);raw.append(x);vals[n].append(x['elapsed_ns']/x['count'])
  a,b=[statistics.median(vals[n]) for n in workers];summary.append(dict(case=c['case'],baseline_ns=a,candidate_ns=b,change_pct=(b/a-1)*100))
 for w in workers.values():w.close()
 (dest/'windows.jsonl.gz').write_bytes(gzip.compress(('\n'.join(json.dumps(x) for x in raw)+'\n').encode(),mtime=0));(dest/'summary.json').write_text(json.dumps(summary,indent=2))
 print(name,' '.join(x['case']+'='+str(round_value(x['change_pct']))+'%' for x in summary[:17]),flush=True);return True
def round_value(x):return round(x,1)
if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('names',nargs='+');p.add_argument('--rounds',type=int,default=5);p.add_argument('--ms',type=int,default=35);a=p.parse_args()
 if a.names==['baseline']:baseline()
 else:
  for name in a.names:
   if build(name):run(name,a.rounds,a.ms)
