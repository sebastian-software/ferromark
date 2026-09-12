from pathlib import Path
import subprocess,json,gzip,os,shutil,sys
from variants import variant
from experiment import Worker,W,verify
name=sys.argv[1] if len(sys.argv)>1 else 'baseline'
sample=W/'bin'/f'{name}-sample'
if not sample.exists():
 files=variant(name,{str(p.relative_to(W/'baseline')):p.read_text() for p in (W/'baseline/src').rglob('*.rs')})
 for f,s in files.items():(W/'source'/f).write_text(s)
 env=dict(os.environ);env.pop('CARGO_ENCODED_RUSTFLAGS',None);env['RUSTFLAGS']='-C force-frame-pointers=yes'
 with (W/name/'sample-build.log').open('w') as log:
  subprocess.run(['cargo','build','--profile','sample','--offline','--locked','--manifest-path',str(W/'driver/Cargo.toml')],cwd=W,env=env,stdout=log,stderr=subprocess.STDOUT,check=True)
 shutil.copy2(W/'driver/target/sample/ox-experiment-driver',sample)
 print('Sample build complete',flush=True)
(W/(name+'-sample')).mkdir(exist_ok=True)
assert verify(name+'-sample')
(W/'profiles').mkdir(exist_ok=True)
for capture in range(1,3):
 for i in [79,487,108,613,7]:
  dest=W/'profiles'/f'{name}-{i}-{capture}.txt';w=Worker(sample,W/'cases.json');w.call('window',i,100)
  w.p.stdin.write(json.dumps(dict(op='window',index=i,ms=8500))+'\n');w.p.stdin.flush()
  with dest.with_suffix('.log').open('w') as log:
   subprocess.run(['/usr/bin/sample',str(w.p.pid),'6','1','-file',str(dest)],stdout=log,stderr=subprocess.STDOUT,check=True)
  row=json.loads(w.p.stdout.readline());w.close();dest.with_suffix('.txt.gz').write_bytes(gzip.compress(dest.read_bytes(),mtime=0));dest.unlink()
  dest.with_suffix('.json').write_text(json.dumps(dict(index=i,capture=capture,seconds=6,interval_ms=1,window=row),indent=2))
  print('Sampled',i,capture,flush=True)
