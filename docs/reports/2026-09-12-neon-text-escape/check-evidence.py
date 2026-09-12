"""Check archive completeness, source reconstruction and timing arithmetic."""
from pathlib import Path
import gzip,hashlib,io,json,subprocess,sys,tarfile,tempfile
D=Path(__file__).resolve().parent;R=D.parents[2]
for line in (D/'SHA256SUMS').read_text().splitlines():
 h,f=line.split('  ',1);assert hashlib.sha256((D/f).read_bytes()).hexdigest()==h,f
meta=json.loads((D/'metadata.json').read_text())
for f,h in meta['driver_sha256'].items():assert hashlib.sha256((D/'driver'/f).read_bytes()).hexdigest()==h,f
archive=subprocess.check_output(['git','archive',meta['baseline_revision'],'src'],cwd=R)
for n in meta['binary_sha256']:
 expected=json.loads((D/'measurements'/n/'source.json').read_text());assert expected,n
 with tempfile.TemporaryDirectory(prefix='ferromark-escape-evidence-') as tmp:
  root=Path(tmp)
  with tarfile.open(fileobj=io.BytesIO(archive)) as tar:tar.extractall(root,filter='data')
  patch=D/'measurements'/n/'patch.diff'
  if patch.stat().st_size:subprocess.run(['patch','-p1','-i',str(patch)],cwd=root,check=True,stdout=subprocess.DEVNULL)
  actual={str(p.relative_to(root)):hashlib.sha256(p.read_bytes()).hexdigest() for p in (root/'src').rglob('*.rs')}
  assert actual==expected,n
for name in ['baseline','single-neon','direct-neon','outlined-neon','tail-neon','bulk-neon','bulk-inline-neon']:
 total=0
 for p in (D/'measurements'/name).glob('*verification.json'):
  v=json.loads(p.read_text());assert not v.get('failures',v.get('failed',[])),p;total+=v['count']
 assert total==(111902 if name in ['baseline','single-neon'] else 2788),(name,total)
sys.dont_write_bytecode=True
sys.path.insert(0,str(D));from results import rows
runs=json.loads((D/'runs.json').read_text());seen=set();count=0
for p in (D/'measurements').rglob('windows.jsonl.gz'):
 g=rows(p.parent);raw=[json.loads(s) for s in gzip.decompress(p.read_bytes()).decode().splitlines()]
 engines=set(x['engine'] for x in raw);assert len(engines)==2 and 'baseline' in engines,p
 key=str(p.parent.relative_to(D/'measurements'));config=runs[key];seen.add(key);rounds=config['rounds']
 assert list(g)==config['indices'],key
 assert engines=={'baseline',config['recorded_engine']},key
 assert all(x['elapsed_ns']>=config['ms']*1000000 for x in raw),key
 if config['operation']=='window':assert all(x['count']%16==0 for x in raw),key
 for index in config['indices']:
  cells=[x for x in raw if x['index']==index]
  for r in range(rounds):
   order=['baseline',config['recorded_engine']]
   if (r+config['order'])%2:order.reverse()
   assert [x['engine'] for x in cells if x['round']==r]==order,(key,index,r)
 for i,row in g.items():
  for engine in engines:
   cells=[x for x in raw if x['index']==i and x['engine']==engine]
   assert sorted(x['round'] for x in cells)==list(range(rounds)),(p,i,engine)
   assert all(x['count']>0 and x['elapsed_ns']>0 and x['case']==row['case'] for x in cells)
 if 'complete.json' in [x.name for x in p.parent.iterdir()]:
  v=json.loads((p.parent/'complete.json').read_text());assert v['expected']==v['measured']==len(g)==27
 count+=len(g)
assert seen==set(runs)
subprocess.run([sys.executable,str(D/'results.py'),'--check'],check=True)
print(f'Archive hashes, source identities, output guards and {count} timing summaries verified.')
