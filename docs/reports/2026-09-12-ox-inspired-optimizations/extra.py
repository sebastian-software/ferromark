from pathlib import Path
import sys,shutil,subprocess,json,gzip,statistics,hashlib
W=Path(__file__).parent;F=W/'confirmation';sys.path.insert(0,str(F));from experiment import Worker

def allocations():
 a=W/'allocation'
 for n,src in [('baseline',F/'baseline/src'),('candidate',F/'source/src')]:
  shutil.copytree(src,a/'source/src',dirs_exist_ok=True)
  for source in (a/'source/src').rglob('*.rs'):source.touch()
  with (a/(n+'-build.log')).open('w') as out:subprocess.run(['cargo','build','--release','--offline','--locked','--features','allocations','--manifest-path','driver/Cargo.toml'],cwd=a,stdout=out,stderr=subprocess.STDOUT,check=True)
  shutil.copy2(a/'driver/target/release/ox-experiment-driver',a/'bin'/n)
  w=Worker(a/'bin'/n,F/'cases.json');rows=[]
  for i,c in enumerate(json.loads((F/'cases.json').read_text())):
   if c['reuse']:w.call('alloc',i)
   r=w.call('alloc',i);r['engine']=n;rows.append(r)
  w.close();(a/(n+'.json')).write_text(json.dumps(rows,indent=2))
 print('Allocation measurements completed',flush=True)

def oxbuild():
 o=W/'ox-driver'
 with (o/'build.log').open('w') as out:subprocess.run(['cargo','build','--release','--offline','--locked','--manifest-path','Cargo.toml'],cwd=o,stdout=out,stderr=subprocess.STDOUT,check=True)
 shutil.copy2(o/'target/release/ox-comparison-driver',F/'bin/ox')
 print('Ox comparison built',flush=True)

def oxrun(run):
 o=W/'ox-driver';workers={n:Worker(F/'bin'/n,F/'cases.json') for n in ['baseline','candidate','ox']};cases=json.loads((F/'cases.json').read_text())[:17];verification=[];raw=[];summary=[]
 for i,c in enumerate(cases):
  b=workers['baseline'].call('verify',i);f=workers['candidate'].call('verify',i);assert b==f
  p=workers['ox'].p;p.stdin.write(json.dumps({'op':'verify','index':i,'html':f['html']})+'\n');p.stdin.flush();x=json.loads(p.stdout.readline())
  verification.append(dict(case=c['case'],normalized_equal=x['normalized_equal'],ferro_sha256=hashlib.sha256(f['html'].encode()).hexdigest(),ox_sha256=hashlib.sha256(x['html'].encode()).hexdigest()))
  if not x['normalized_equal']:
   print('Ox semantic mismatch; skip timing',c['case'],flush=True);continue
  vals={n:[] for n in workers}
  for w in workers.values():w.call('window',i,250)
  names=list(workers)
  for r in range(9):
   order=names[(r+run)%3:]+names[:(r+run)%3]
   if r%2:order.reverse()
   for n in order:
    x=workers[n].call('window',i,75);x.update(engine=n,round=r);raw.append(x);vals[n].append(x['elapsed_ns']/x['count'])
  summary.append(dict(case=c['case'],**{n:statistics.median(v) for n,v in vals.items()}))
 for w in workers.values():w.close()
 (o/f'run-{run+1}.jsonl.gz').write_bytes(gzip.compress(('\n'.join(json.dumps(x) for x in raw)+'\n').encode(),mtime=0));(o/f'run-{run+1}-summary.json').write_text(json.dumps(summary,indent=2));(o/'verification.json').write_text(json.dumps(verification,indent=2))
 print('Ox run',run+1,' '.join(f"{x['case']}={x['candidate']/x['ox']:.3f}x" for x in summary),flush=True)
if __name__=='__main__':
 if sys.argv[1]=='build':allocations();oxbuild()
 else:
  for i in range(3):oxrun(i)
