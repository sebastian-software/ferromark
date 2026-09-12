"""Validate native timing summaries and render every paired observation."""
from pathlib import Path
import argparse,gzip,json,statistics,math
D=Path(__file__).resolve().parent

def rows(directory):
 raw=[json.loads(x) for x in gzip.decompress((directory/'windows.jsonl.gz').read_bytes()).decode().splitlines()]
 engines=list(dict.fromkeys(x['engine'] for x in raw));assert len(engines)==2 and engines[0] in ['baseline',directory.parent.name]
 candidate=next(e for e in engines if e!='baseline');out={}
 for entry in json.loads((directory/'summary.json').read_text()):
  cells=[x for x in raw if x['index']==entry['index']];a,b=[statistics.median(x['elapsed_ns']/x['count'] for x in cells if x['engine']==e) for e in ['baseline',candidate]]
  assert all(x['elapsed_ns']>=50_000_000 and x['count']>0 and x['case']==entry['case'] for x in cells)
  assert len(cells)==10,(directory,len(cells))
  for key,value in [('baseline_ns',a),('candidate_ns',b),('change_pct',(b/a-1)*100)]:assert math.isclose(entry[key],value,rel_tol=1e-10,abs_tol=1e-10),(directory,key)
  out[entry['case']]=entry
 assert len(raw)==10*len(out)
 return out

def generate():
 text=['# Native Linux paired results','','Generated from the archived windows. Negative values are faster; positive values are slower. Each cell preserves one fresh process pair. No workload weighting or A/A subtraction is applied.','']
 total=0
 for rounddir in sorted((D/'measurements').iterdir()):
  meta=json.loads((rounddir/'native-metadata.json').read_text());cpu=next(s.split(':',1)[1].strip() for s in meta['cpu'].splitlines() if s.startswith('Model name:'))
  text += ['## '+rounddir.name,'',cpu+'; '+meta['rustc'].splitlines()[0]+'.','']
  for candidate in sorted(p for p in rounddir.iterdir() if p.is_dir() and any(p.glob('*/windows.jsonl.gz'))):
   groups={}
   for path in sorted(candidate.glob('*/windows.jsonl.gz')):
    suite=path.parent.name.rsplit('-',1)[0];groups.setdefault(suite,[]).append((path.parent.name,rows(path.parent)))
   for suite,runs in groups.items():
    text+=['### '+candidate.name+' / '+suite,'','| Case | Before µs¹ | After µs¹ | '+ ' | '.join(name for name,_ in runs)+' |','|---|---:|---:|'+ '---:|'*len(runs)]
    names=list(dict.fromkeys(case for _,r in runs for case in r))
    for case in names:
     cells=[r[case] for _,r in runs if case in r];a=statistics.median(c['baseline_ns'] for c in cells)/1000;b=statistics.median(c['candidate_ns'] for c in cells)/1000
     text.append('| '+case.replace('|','\\|')+f' | {a:.3f} | {b:.3f} | '+' | '.join(f"{r[case]['change_pct']:+.2f}%" if case in r else '—' for _,r in runs)+' |')
     total+=len(cells)
    text+=['']
 text+=['¹ Median of the pair medians; the per-pair percentage columns are the primary observations.','']
 return '\n'.join(text),total
if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('--check',action='store_true');a=p.parse_args();content,total=generate();dest=D/'RESULTS.md'
 if a.check:assert dest.read_text()==content,'Regenerate RESULTS.md'
 else:dest.write_text(content)
 print('Validated',total,'native timing summaries.')
