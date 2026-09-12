from pathlib import Path
import subprocess,shutil,os,json,gzip,hashlib,sys
from variants import variant
W=Path(__file__).parent
OLD=Path('/private/tmp/ferromark-ox-implementation/memory')
M=W/'memory';M.mkdir(exist_ok=True)
if not (M/'driver/Cargo.toml').exists():
 (M/'driver/src').mkdir(parents=True,exist_ok=True)
 shutil.copytree(OLD/'ferro/src',M/'driver/src',dirs_exist_ok=True)
 for f in ['Cargo.toml','Cargo.lock']:shutil.copyfile(OLD/'ferro'/f,M/'driver'/f)
if not (M/'cases.json').exists():
 cases=json.loads((OLD/'cases.json').read_text())
 for size in [65536,8388608]:
  for label,opening,body,closing in [('root','<div>\n','  <p>x & text</p>\n','\n'),('comment','<!--\n','text and more text\n','-->\n'),('script','<script>\n','const x = 1;\n','</script>\n')]:
   content=(body*((size-len(opening)-len(closing))//len(body)+1))[:size-len(opening)-len(closing)]
   cases.append(dict(case=f'html/{label}-{size}',input=opening+content+closing,flags=8,reuse=False))
 (M/'cases.json').write_text(json.dumps(cases))
(M/'source').mkdir(exist_ok=True);shutil.copyfile(W/'baseline/Cargo.toml',M/'source/Cargo.toml')
base={str(p.relative_to(W/'baseline')):p.read_text() for p in (W/'baseline/src').rglob('*.rs')}
env=dict(os.environ);env.pop('RUSTFLAGS',None);env.pop('CARGO_ENCODED_RUSTFLAGS',None)
for name in sys.argv[1:]:
 d=M/name;d.mkdir(exist_ok=True)
 files=variant(name,base.copy())
 for f,s in files.items():
  p=M/'source'/f;p.parent.mkdir(parents=True,exist_ok=True);p.write_text(s)
 with (d/'build.log').open('w') as out:subprocess.run(['cargo','build','--release','--offline','--locked','--manifest-path',str(M/'driver/Cargo.toml')],cwd=M,env=env,stdout=out,stderr=subprocess.STDOUT,check=True)
 binary=M/'driver/target/release/ox-experiment-driver';raw=[]
 for process in range(3):
  worker=subprocess.Popen([str(binary),str(M/'cases.json')],stdin=subprocess.PIPE,stdout=subprocess.PIPE,text=True)
  for i,c in enumerate(json.loads((M/'cases.json').read_text())):
   worker.stdin.write(json.dumps(dict(index=i))+'\n');worker.stdin.flush();r=json.loads(worker.stdout.readline())
   r['html_sha256']=hashlib.sha256(r.pop('html').encode()).hexdigest();r.pop('normalized');r.update(process=process,engine=name);raw.append(r)
  worker.stdin.close();assert worker.wait()==0
 (d/'raw.jsonl.gz').write_bytes(gzip.compress(('\n'.join(json.dumps(r) for r in raw)+'\n').encode(),mtime=0))
 summary=[]
 for i,c in enumerate(json.loads((M/'cases.json').read_text())):
  rows=[r for r in raw if r['case']==c['case']];samples=[s for r in rows for s in r['samples']];assert all(s==samples[0] for s in samples)
  summary.append(dict(case=c['case'],input_bytes=rows[0]['input_bytes'],html_sha256=rows[0]['html_sha256'],**samples[0]))
 (d/'summary.json').write_text(json.dumps(summary,indent=2));print('Memory complete',name,flush=True)
