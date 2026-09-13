from run import *
checks=[];release=Worker('ferro');probe=Worker('attribution','sample')
for i in range(len(json.loads((W/'cases.json').read_text()))):
 a=release.call('verify',i);b=probe.call('verify',i);assert a==b,i;checks.append(sha(a['html']))
release.close();probe.close();(W/'attribution/verification.json').write_text(json.dumps(dict(count=len(checks),hashes=checks),indent=2)+'\n')
for cap in range(1,3):
 for i in [34,442]:
  w=Worker('attribution','sample');w.call('window',i,100)
  w.p.stdin.write(json.dumps(dict(op='window',index=i,ms=8500))+'\n');w.p.stdin.flush()
  dest=W/'profiles'/f'attribution-{i}-{cap}.txt'
  with dest.with_suffix('.log').open('w') as log:subprocess.run(['/usr/bin/sample',str(w.p.pid),'6','1','-file',str(dest)],stdout=log,stderr=subprocess.STDOUT,check=True)
  row=json.loads(w.p.stdout.readline());w.close();dest.with_suffix('.txt.gz').write_bytes(gzip.compress(dest.read_bytes(),mtime=0));dest.unlink();dest.with_suffix('.json').write_text(json.dumps(row,indent=2)+'\n');print('Attribution sampled',i,cap,flush=True)
