from pathlib import Path
import subprocess,json,time,gzip,sys
from run import W,Worker
(W/'profiles').mkdir(exist_ok=True)
indices=[97,132,509,644,34,63,114,442,568,625,648]
if len(sys.argv)>1:indices=[int(x) for x in sys.argv[1:]]
for i in indices:
 for engine in ['ferro','ox-grow']:
  stem=f'{engine}-{i}';n=1
  while (W/'profiles'/f'{stem}-{n}.txt.gz').exists():n+=1
  dest=W/'profiles'/f'{stem}-{n}.txt';w=Worker(engine,'sample')
  w.call('window',i,100)
  w.p.stdin.write(json.dumps(dict(op='window',index=i,ms=8500))+'\n');w.p.stdin.flush()
  with (W/'profiles'/f'{stem}-{n}.log').open('w') as log:
   r=subprocess.run(['/usr/bin/sample',str(w.p.pid),'6','1','-file',str(dest)],stdout=log,stderr=subprocess.STDOUT)
  row=json.loads(w.p.stdout.readline());w.close()
  if r.returncode:raise RuntimeError(dest)
  raw=dest.read_bytes();dest.with_suffix('.txt.gz').write_bytes(gzip.compress(raw,mtime=0));dest.unlink()
  (W/'profiles'/f'{stem}-{n}.json').write_text(json.dumps(dict(engine=engine,index=i,pid=w.p.pid,duration_seconds=6,interval_ms=1,window=row),indent=2)+'\n')
  print('sampled',engine,i,'capture',n,flush=True)
