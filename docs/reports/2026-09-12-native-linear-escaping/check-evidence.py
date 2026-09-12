"""Check recorded sources, output guards, metadata and timing arithmetic."""
from pathlib import Path
import gzip,hashlib,json,subprocess,sys
D=Path(__file__).resolve().parent;R=D.parents[2]
for line in (D/'SHA256SUMS').read_text().splitlines():
 h,f=line.split('  ',1);assert hashlib.sha256((D/f).read_bytes()).hexdigest()==h,f
baseline='e731dc8162828e0ac2df30b7567107f560159ff8'
basefiles={p:subprocess.check_output(['git','show',baseline+':'+p],cwd=R) for p in subprocess.check_output(['git','ls-tree','-r','--name-only',baseline,'src'],cwd=R,text=True).splitlines() if p.endswith('.rs')}
for rounddir in sorted((D/'measurements').iterdir()):
 meta=json.loads((rounddir/'native-metadata.json').read_text());assert hashlib.sha256((rounddir/'driver/src/main.rs').read_bytes()).hexdigest()==meta['driver_sha256']
 if 'binary_sha256' in meta:assert meta['binary_sha256']['baseline']==meta['binary_sha256']['aa-control']
 for sources in rounddir.glob('*/source.json'):
  expected=json.loads(sources.read_text());actual={p:hashlib.sha256(b).hexdigest() for p,b in basefiles.items()}
  name=sources.parent.name
  if name!='baseline':actual['src/escape.rs']=hashlib.sha256(gzip.decompress((rounddir/'native-sources'/f'{name}.rs.gz').read_bytes())).hexdigest()
  assert actual==expected,(rounddir,name)
  total=0
  for path in sources.parent.glob('*verification.json'):
   v=json.loads(path.read_text());assert not v.get('failures',v.get('failed',[])),path;total+=v['count']
  assert total in [2695,111902],(sources,total)
  if name=='baseline' or name.startswith('bulk-') or name=='current' or (rounddir.name.startswith('round-3') and name.startswith('masked-')) or (rounddir.name.startswith('round-4') and name in ['probe-attr-outlined','writers-outlined']):assert total==111902,(sources,total)
sys.dont_write_bytecode=True
subprocess.run([sys.executable,str(D/'results.py'),'--check'],check=True)
print('Native archive hashes, source identities and guards verified.')
