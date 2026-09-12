"""Check archived bytes, source reconstruction, guards and timing arithmetic."""
from pathlib import Path
import gzip,hashlib,io,json,subprocess,sys,tarfile,tempfile
D=Path(__file__).resolve().parent;R=D.parents[2]
for line in (D/'SHA256SUMS').read_text().splitlines():
 h,f=line.split('  ',1);assert hashlib.sha256((D/f).read_bytes()).hexdigest()==h,f
meta=json.loads((D/'metadata.json').read_text())
assert meta['binary_sha256']['aa-control']==meta['binary_sha256']['baseline']
for f,h in meta['driver_sha256'].items():assert hashlib.sha256((D/'driver'/f).read_bytes()).hexdigest()==h,f
archive=subprocess.check_output(['git','archive',meta['baseline_revision'],'src'],cwd=R)
for name in meta['binary_sha256']:
 expected=json.loads((D/'measurements'/name/'source.json').read_text());assert expected,name
 with tempfile.TemporaryDirectory(prefix='ferromark-linear-evidence-') as tmp:
  root=Path(tmp)
  with tarfile.open(fileobj=io.BytesIO(archive)) as tar:tar.extractall(root,filter='data')
  patch=D/'measurements'/name/'patch.diff'
  if patch.stat().st_size:subprocess.run(['patch','-p1','-i',str(patch)],cwd=root,check=True,stdout=subprocess.DEVNULL)
  actual={str(p.relative_to(root)):hashlib.sha256(p.read_bytes()).hexdigest() for p in (root/'src').rglob('*.rs')}
  assert actual==expected,name
  if name in ['production','final','linear']:
   assert (root/'src/escape.rs').read_bytes()==gzip.decompress((D/'snapshots'/f'{name}-escape.rs.gz').read_bytes())
 total=0
 for p in (D/'measurements'/name).glob('*verification.json'):
  v=json.loads(p.read_text());assert not v.get('failures',v.get('failed',[])),p;total+=v['count']
 if name in ['baseline','production','final','linear']:assert total==111902,(name,total)
 elif name!='aa-control':assert total>=2695,(name,total)
scope=json.loads((D/'arm-scope.json').read_text())
assert scope['recorded_linear_text_sha256']==scope['arm_scoped_text_sha256']
with tempfile.TemporaryDirectory(prefix='ferromark-arm-scope-') as tmp:
 root=Path(tmp)
 with tarfile.open(fileobj=io.BytesIO(archive)) as tar:tar.extractall(root,filter='data')
 subprocess.run(['patch','-p1','-i',str(D/'arm-scoped.patch')],cwd=root,check=True,stdout=subprocess.DEVNULL)
 actual={str(p.relative_to(root)):hashlib.sha256(p.read_bytes()).hexdigest() for p in (root/'src').rglob('*.rs')}
 assert actual==scope['source_sha256']
 assert (root/'src/escape.rs').read_bytes()==gzip.decompress((D/'snapshots/arm-scoped-escape.rs.gz').read_bytes())
sys.dont_write_bytecode=True;sys.path.insert(0,str(D));from results import rows
runs=json.loads((D/'runs.json').read_text());seen=set();count=0
for p in (D/'measurements').rglob('windows.jsonl.gz'):
 g=rows(p.parent);raw=[json.loads(s) for s in gzip.decompress(p.read_bytes()).decode().splitlines()]
 key=str(p.parent.relative_to(D/'measurements'));config=runs[key];seen.add(key)
 assert list(g)==config['indices'],key
 assert set(x['engine'] for x in raw)=={'baseline',config['candidate']},key
 assert all(x['elapsed_ns']>=config['ms']*1000000 for x in raw),key
 if config['operation']=='window':assert all(x['count']%16==0 for x in raw),key
 for i in config['indices']:
  cells=[x for x in raw if x['index']==i]
  assert len(cells)==2*config['rounds'],(key,i)
  for r in range(config['rounds']):
   order=['baseline',config['candidate']]
   if (r+config['order'])%2:order.reverse()
   assert [x['engine'] for x in cells if x['round']==r]==order,(key,i,r)
  assert all(x['count']>0 and x['case']==g[i]['case'] for x in cells),(key,i)
 complete=p.parent/'complete.json'
 if config['operation']=='window-single':
  v=json.loads(complete.read_text());assert v['expected']==v['measured']==len(g),(key,v)
 count+=len(g)
assert seen==set(runs)
subprocess.run([sys.executable,str(D/'results.py'),'--check'],check=True)
print(f'Archive hashes, source identities, output guards and {count} timing summaries verified.')
