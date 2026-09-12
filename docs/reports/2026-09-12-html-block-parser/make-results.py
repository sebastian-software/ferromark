from pathlib import Path
import json,gzip,statistics
W=Path(__file__).parent
def read(p):
 p=W/p
 return json.loads(p.read_text() if p.exists() else gzip.decompress(p.with_suffix(p.suffix+'.gz').read_bytes()))
def table(headers,rows):
 lines.extend(['| '+' | '.join(headers)+' |','| '+' | '.join(['---']*len(headers))+' |'])
 lines.extend('| '+' | '.join(map(str,r))+' |' for r in rows);lines.append('')
lines=['# HTML block parser measurements','','Generated from the archived measurement records. Negative changes mean less elapsed time. Exploratory screens and independent production confirmations are reported separately.','','## Profiles','','Two six-second native sample captures per document/implementation at one-millisecond intervals. Visible stack shares are diagnostic, not elapsed-time improvements.','']
ps=read('profile-summary.json');rows=[]
for name in ['baseline','production']:
 for index in [613,108]:
  group=[p for p in ps if p['file'].startswith(f'{name}-{index}-')]
  if group:rows.append([name,read('cases.json')[index]['case'],', '.join(f"{p['stages'].get('visible block frames',{}).get('pct',0):.1f}%" for p in group)])
table(['Source','Input','Visible block frames'],rows)
lines+=['## Exploratory screens','','Five alternating 30 ms windows per implementation/input, following 60 ms warmup. The median is over 81 selected inputs and is not a weighted application workload. All original and block/MDX output guards must pass before timing. Complete per-input results remain in each variant directory.','']
rows=[]
for d in sorted(W.iterdir()):
 if d.name in ['production','baseline'] or not (d/'summary.json').exists() or not (d/'patch.diff').exists():continue
 r=read(d.name+'/summary.json');v=read(d.name+'/verification.json');b=read(d.name+'/block-verification.json')
 def val(i):return next(x['change_pct'] for x in r if x['index']==i)
 rows.append([d.name,f"{statistics.median(x['change_pct'] for x in r):+.1f}%",f'{val(613):+.1f}%',f'{val(694):+.1f}%',f'{val(697):+.1f}%',len(v['failed'])+len(b['failures'])])
table(['Variant','Median change','TS compiler options','Root short lines','Long comment lines','Guard failures'],rows)
if (W/'dispatch-after-line/confirm-3/summary.json').exists():
 lines+=['## Dispatch correction','','The first production integration entered the continuation loop from general block dispatch. Three full process pairs exposed a repeated regression on many short escaped code fences. The final integration enters from HTML-block recognition instead. A targeted three-pair confirmation preserves the HTML gain while removing this regression; the complete final confirmation follows below.','']
 before=[read(f'dispatch-after-line/confirm-{i}/summary.json') for i in [1,2,3]]
 after=[read(f'scan+markers-lite+tags+entry/targeted-{i}/summary.json') for i in [1,2,3]]
 rows=[]
 for index in [16,2,612,613]:
  def changes(runs):return ', '.join(f"{next(x['change_pct'] for x in r if x['index']==index):+.1f}%" for r in runs)
  rows.append([read('cases.json')[index]['case'],changes(before),changes(after)])
 table(['Input','General-dispatch integration','HTML-start integration'],rows)
if (W/'production/confirm-3/summary.json').exists():
 lines+=['## Production confirmation','','The formatted production source is rebuilt separately. Three fresh process pairs, nine alternating 75 ms windows per implementation/input, following 60 ms warmup. Latencies summarize the three process medians; all paired changes remain visible.','']
 runs=[read(f'production/confirm-{i}/summary.json') for i in [1,2,3]]
 table(['Input','Baseline µs','Production µs','Three paired changes'],[[x['case'],f"{statistics.median(r[j]['baseline_ns'] for r in runs)/1000:.3f}",f"{statistics.median(r[j]['candidate_ns'] for r in runs)/1000:.3f}",', '.join(f"{r[j]['change_pct']:+.1f}%" for r in runs)] for j,x in enumerate(runs[0])])
 reg=[x['case'] for j,x in enumerate(runs[0]) if all(r[j]['change_pct']>2 for r in runs)]
 lines+=['Inputs exceeding a 2% regression in all three process pairs: '+(', '.join(reg) if reg else 'none')+'.','']
if (W/'memory/production/summary.json').exists():
 lines+=['## Requested live heap','','Three fresh processes per implementation, ten observations per process/input. This measures allocator requests during rendering with owned output still live, not RSS. The dataset includes 64 KiB and 8 MiB HTML inputs.','']
 a=read('memory/baseline/summary.json');b=read('memory/production/summary.json');rows=[]
 for x,y in zip(a,b):
  assert x['case']==y['case'] and x['html_sha256']==y['html_sha256']
  rows.append([x['case'],x['input_bytes'],x['peak_live_bytes'],y['peak_live_bytes'],x['allocation_calls'],y['allocation_calls']])
 table(['Input','Input bytes','Baseline peak bytes','Production peak bytes','Baseline allocations','Production allocations'],rows)
if (W/'corpus/final-3-summary.json').exists():
 lines+=['## Fresh Ox corpus comparison','','This separate native harness uses fresh state and owned HTML with heading IDs enabled. Three process groups, nine rotating 60 ms windows per engine/input, following 35 ms warmup. Ox is pinned to `026d1859d1c35e5fb1ea65e7e855b428a918b9bb`; the growing arena is primary. Only equal normalized output supports ranking. Do not subtract these latencies from the optimization harness above.','']
 runs=[read(f'corpus/final-{i}-summary.json') for i in [1,2,3]];rows=[]
 for j,x in enumerate(runs[0]):
  if not x['equal']:continue
  f,o,p=[statistics.median(r[j]['median_ns'][n] for r in runs)/1000 for n in ['ferro','ox-grow','ox-presize']]
  rows.append([x['case'],f'{f:.2f}',f'{o:.2f}',f'{p:.2f}',f'{f/o:.2f}×'])
 table(['Input','Ferromark µs','Ox growing µs','Ox presized µs','F / Ox growing'],rows)
 v=read('corpus/verification.json');lines+=[f"Normalized equality: {sum(r['equal'] for r in v)} / {len(v)} corpus cases. All unequal cases remain diagnostics.",'']
if (W/'production/extended-verification.json').exists():
 lines+=['## Exact-output checks',''];rows=[]
 for file,label in [('verification.json','Specification, extensions, reuse, all corpus files'),('extra-verification.json','Initial inline / MDX events and HTML'),('extended-verification.json','Independent inline / MDX events and HTML'),('block-verification.json','Public block / MDX streams, segments, HTML and metadata'),('timing-verification.json','Every timed input')]:
  r=read('production/'+file);rows.append([label,r['count'],len(r.get('failed',r.get('failures',[])))])
 table(['Guard set','Cases','Failures'],rows)
(W/'RESULTS.md').write_text('\n'.join(lines).rstrip()+'\n');print('Generated RESULTS.md')
