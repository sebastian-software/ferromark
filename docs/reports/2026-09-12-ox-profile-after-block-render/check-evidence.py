"""Verify archived hashes, timing arithmetic, source identities and output guards."""
from pathlib import Path
import hashlib,json,gzip,statistics,sys,subprocess,io,tarfile
D=Path(__file__).resolve().parent;R=D.parents[2]
def read(p):return json.loads(gzip.decompress(p.read_bytes()) if p.suffix==".gz" else p.read_text())
for line in (D/'SHA256SUMS').read_text().splitlines():
 h,f=line.split('  ',1);assert hashlib.sha256((D/f).read_bytes()).hexdigest()==h,f
meta=read(D/'metadata.json');archive=subprocess.check_output(['git','archive',meta['ferromark_revision'],'src'],cwd=R)
with tarfile.open(fileobj=io.BytesIO(archive)) as tar:files={m.name:tar.extractfile(m).read().decode() for m in tar.getmembers() if m.isfile()}
assert {f:hashlib.sha256(s.encode()).hexdigest() for f,s in files.items()}==meta['source_sha256']
sys.dont_write_bytecode=True
sys.path.insert(0,str(D/'experiments'));from variants import variant
for name in meta['experiment_binary_sha256']:
 expected=read(D/'experiments'/name/'source.json');actual=variant(name,files.copy());assert {f:hashlib.sha256(s.encode()).hexdigest() for f,s in actual.items()}==expected,name
probe=files.copy()
for f,needle in {'src/inline/marks.rs':'fn collect_marks_impl<','src/inline/mod.rs':'    fn emit_events(','src/inline/links.rs':'pub fn find_autolinks_into('}.items():
 assert probe[f].count(needle)==1
 probe[f]=probe[f].replace(needle,('    #[inline(never)]\n' if needle.startswith('    ') else '#[inline(never)]\n')+needle)
assert {f:hashlib.sha256(s.encode()).hexdigest() for f,s in probe.items()}==read(D/'attribution/source.json')
assert read(D/'attribution/verification.json')['count']==649
count=0
for p in (D/'comparison').glob('*-raw.jsonl.gz'):
 rows=[json.loads(x) for x in gzip.decompress(p.read_bytes()).decode().splitlines()];summary=read(p.with_name(p.name.replace('-raw.jsonl.gz','-summary.json')))
 for row in summary:
  for n in ['ferro','ox-grow','ox-presize']:
   vals=[r['elapsed_ns']/r['count'] for r in rows if r['index']==row['index'] and r['engine']==n]
   assert len(vals)==9
   assert statistics.median(vals)==row['median_ns'][n]
   assert min(vals)==row['min_ns'][n] and max(vals)==row['max_ns'][n];count+=1
for p in (D/'experiments').rglob('windows.jsonl.gz'):
 rows=[json.loads(x) for x in gzip.decompress(p.read_bytes()).decode().splitlines()];summary=read(p.with_name('summary.json'))
 for row in summary:
  vals={n:[r['elapsed_ns']/r['count'] for r in rows if r['index']==row['index'] and r['engine']==n] for n in set(r['engine'] for r in rows)}
  base=statistics.median(vals.pop('baseline'));assert len(vals)==1;candidate=statistics.median(next(iter(vals.values())))
  assert base==row['baseline_ns'] and candidate==row['candidate_ns'];assert abs((candidate/base-1)*100-row['change_pct'])<1e-10;count+=1
 for row in rows:assert row['elapsed_ns']>0 and row['count']>0
for name in ['html-blank-run','escape-short-copy','escape-single-scan','softbreak-guarded','autolink-prefilter']:
 total=0
 for p in (D/'experiments'/name).glob('*verification.json'):
  v=read(p);assert not v.get('failures',v.get('failed',[])),p;total+=v['count']
 assert total==111902,(name,total)
v=read(D/'verification.json');assert len(v)==649 and sum(r['equal'] for r in v)==594
old=read(R/'docs/reports/2026-09-12-block-render-hotspots/corpus/verification.json.gz');assert old==v
assert len(read(D/'sample-verification.json'))==1298
profiles=read(D/'profile-summary.json');assert len(profiles)==24
for engine in ['ferro','ox-grow']:
 for i in meta['inputs']:
  for cap in [1,2]:assert sum(p['file']==f'{engine}-{i}-{cap}.txt.gz' for p in profiles)==1
for p in profiles:assert sum(v['samples'] for v in p['stages'].values())==p['total_samples']
print(f'All archived hashes, {count} timing summaries, source identities, 24 captures and exact-output records verified.')
