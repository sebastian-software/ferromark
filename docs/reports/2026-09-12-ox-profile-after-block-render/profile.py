from run import *
import shutil
selected=json.loads((W/'selected.json').read_text());(W/'profiles').mkdir(exist_ok=True)
checks=[]
for engine in ['ferro','ox-grow']:
 release=Worker(engine);sample=Worker(engine,'sample')
 for i in range(len(json.loads((W/'cases.json').read_text()))):
  a=release.call('verify',i);b=sample.call('verify',i);assert a==b,(engine,i)
  checks.append(dict(engine=engine,index=i,html_sha256=sha(a['html'])))
 release.close();sample.close()
(W/'sample-verification.json').write_text(json.dumps(checks,indent=2)+'\n');print('Verified sample/release outputs',len(checks),flush=True)
for capture in range(1,3):
 for i in selected:
  for engine in (['ferro','ox-grow'] if capture==1 else ['ox-grow','ferro']):
   dest=W/'profiles'/f'{engine}-{i}-{capture}.txt';w=Worker(engine,'sample');w.call('window',i,100)
   w.p.stdin.write(json.dumps(dict(op='window',index=i,ms=8500))+'\n');w.p.stdin.flush()
   with dest.with_suffix('.log').open('w') as log:
    subprocess.run(['/usr/bin/sample',str(w.p.pid),'6','1','-file',str(dest)],stdout=log,stderr=subprocess.STDOUT,check=True)
   row=json.loads(w.p.stdout.readline());w.close();dest.with_suffix('.txt.gz').write_bytes(gzip.compress(dest.read_bytes(),mtime=0));dest.unlink()
   dest.with_suffix('.json').write_text(json.dumps(dict(index=i,engine=engine,capture=capture,seconds=6,interval_ms=1,window=row),indent=2)+'\n')
   print('Sampled',engine,i,capture,flush=True)
