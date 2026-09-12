from pathlib import Path
import json,gzip,statistics
W=Path(__file__).parent
P='code-safe+compact-all+compact-code+html-search'
def read(p):
 p=W/p
 return json.loads(gzip.decompress(p.with_suffix(p.suffix+'.gz').read_bytes()) if not p.exists() else p.read_text())
def write_table(out,headers,rows):
 out.extend(['| '+' | '.join(headers)+' |','| '+' | '.join(['---']*len(headers))+' |'])
 out.extend('| '+' | '.join(map(str,row))+' |' for row in rows);out.append('')
lines=['# Inline emission measurements','','Generated from the archived raw samples. Negative changes mean less elapsed time. Screening is exploratory; production confirmation is independent.','', '## Fresh baseline profiles','', 'Each input has two six-second native sample captures at a one-millisecond interval. Percentages are visible stack shares, not additive bottleneck budgets or elapsed-time improvements.','']
profiles=read('profile-summary.json');cases=read('cases.json');rows=[]
for i in [79,487,108,613,7]:
 ps=[p for p in profiles if p['file'].startswith(f'baseline-{i}-')]
 def spread(f):
  vals=[f(p) for p in ps];return f'{min(vals):.1f}–{max(vals):.1f}%'
 rows.append([cases[i]['case'],spread(lambda p:p['stages'].get('visible inline frames',{}).get('pct',0)),spread(lambda p:p['emit_sort_stack_pct'])])
write_table(lines,['Input','Visible inline frames','Visible emit-point sorting'],rows)
after=[p for p in profiles if p['file'].startswith('production-')]
if after:
 lines+=['## Production profiles','','These are new diagnostic captures of the final source, with the same sampling settings. A smaller sample share is not by itself an absolute timing improvement; use the paired timing results below.','']
 rows=[]
 for i in [79,487,108,613,7]:
  ps=[p for p in after if p['file'].startswith(f'production-{i}-')]
  rows.append([cases[i]['case'],f"{min(p['stages'].get('visible inline frames',{}).get('pct',0) for p in ps):.1f}–{max(p['stages'].get('visible inline frames',{}).get('pct',0) for p in ps):.1f}%",f"{min(p['emit_sort_stack_pct'] for p in ps):.1f}–{max(p['emit_sort_stack_pct'] for p in ps):.1f}%"])
 write_table(lines,['Input','Visible inline frames','Visible emit-point sorting'],rows)
lines+=['## Independent screens','','Five alternating 30 ms windows per implementation/input after 60 ms warmup. The median covers 69 selected inputs and is not a weighted application workload. Failed output guards exclude a variant from production regardless of its timing.','']
rows=[]
for d in sorted(W.iterdir()):
 if not (d/'patch.diff').exists() or d.name in ['baseline','production']:continue
 v=read(d.name+'/verification.json');extra=read(d.name+'/extra-verification.json') if (d/'extra-verification.json').exists() else None
 timed=read(d.name+'/summary.json') if (d/'summary.json').exists() else None
 rows.append([d.name,f"{statistics.median(r['change_pct'] for r in timed):+.1f}%" if timed else 'not admitted',len(v['failed']),len(extra['failures']) if extra else 'not run'])
write_table(lines,['Variant','Median time change','Original guard failures','Additional event guard failures'],rows)
if (W/'layout/sizes.json').exists():
 lines+=['## Private point layout','','The layout probe compiles the actual declarations with the recorded Rust compiler. Public event types are unchanged.','']
 write_table(lines,['Variant','EmitPoint bytes','EmitKind bytes'],[[r['variant'],r['emit_point_bytes'],r['emit_kind_bytes']] for r in read('layout/sizes.json')])
if (W/'confirmation-3/summary.json').exists():
 lines+=['## Combination confirmation','','Three fresh three-engine process groups, nine rotating 75 ms windows per engine/input after 100 ms warmup. These are prototype combinations before the final shared-padding cleanup. The additional compact-payload benefit is workload-dependent; complete results remain in the archived JSON.','']
 runs=[read(f'confirmation-{i}/summary.json') for i in [1,2,3]];rows=[]
 for j,x in enumerate(runs[0]):
  if x['index'] not in [6,7,41,79,487,613]:continue
  rows.append([x['case'],', '.join(f"{r[j]['changes_pct']['code-safe+html-search']:+.1f}%" for r in runs),', '.join(f"{r[j]['changes_pct'][P]:+.1f}%" for r in runs)])
 write_table(lines,['Input','Code/HTML vs baseline','With compact payloads vs baseline'],rows)
if (W/'production/confirm-1/summary.json').exists():
 lines+=['## Production confirmation','','The formatted production source is rebuilt separately. Three fresh process pairs, nine alternating 75 ms windows per implementation/input after 60 ms warmup. Latencies summarize the three per-process medians; each paired change remains visible.','']
 runs=[read(f'production/confirm-{i}/summary.json') for i in [1,2,3]];rows=[]
 for j,x in enumerate(runs[0]):
  rows.append([x['case'],f"{statistics.median(r[j]['baseline_ns'] for r in runs)/1000:.3f}",f"{statistics.median(r[j]['candidate_ns'] for r in runs)/1000:.3f}",', '.join(f"{r[j]['change_pct']:+.1f}%" for r in runs)])
 write_table(lines,['Input','Baseline µs','Production µs','Paired changes'],rows)
 reg=[x['case'] for j,x in enumerate(runs[0]) if all(r[j]['change_pct']>2 for r in runs)]
 lines+=['Controls exceeding a 2% regression in all three runs: '+(', '.join(reg) if reg else 'none')+'.','']
if (W/'memory/production/summary.json').exists():
 lines+=['## Requested live heap','','The existing counting allocator measures 45 inputs, three fresh processes per implementation, and ten identical observations per process/input. HTML remains live at the observation boundary. This is not RSS.','']
 a=read('memory/baseline/summary.json');b=read('memory/production/summary.json');changed=sum(x['peak_live_bytes']!=y['peak_live_bytes'] for x,y in zip(a,b));higher=sum(y['peak_live_bytes']>x['peak_live_bytes'] for x,y in zip(a,b))
 lines+=[f'Peak requested live bytes change in {changed} of {len(a)} cases; {higher} increase. All corresponding HTML hashes match.',''];rows=[]
 for x,y in zip(a,b):
  assert x['html_sha256']==y['html_sha256']
  rows.append([x['case'],x['input_bytes'],x['peak_live_bytes'],y['peak_live_bytes'],f"{(y['peak_live_bytes']/x['peak_live_bytes']-1)*100:+.4f}%"])
 write_table(lines,['Input','Input bytes','Baseline peak','Production peak','Change'],rows)
if (W/'corpus/final-3-summary.json').exists():
 lines+=['## Fresh Ox corpus comparison','','This is a separate native harness: do not subtract its times from the optimization experiment. Three process groups, nine 60 ms windows per engine/input. Both engines create and destroy fresh state and owned HTML with heading IDs enabled. Ox is pinned to `026d1859d1c35e5fb1ea65e7e855b428a918b9bb`; its growing arena is primary. Only equal normalized outputs support a ranking.','']
 runs=[read(f'corpus/final-{i}-summary.json') for i in [1,2,3]];rows=[]
 for j,x in enumerate(runs[0]):
  if not x['equal']:continue
  f,o,p=[statistics.median(r[j]['median_ns'][n] for r in runs)/1000 for n in ['ferro','ox-grow','ox-presize']]
  rows.append([x['case'],f'{f:.2f}',f'{o:.2f}',f'{p:.2f}',f'{f/o:.2f}×'])
 write_table(lines,['Input','Ferromark µs','Ox growing µs','Ox presized µs','F / Ox growing'],rows)
 verify=read('corpus/verification.json');lines+=[f"Normalized equality: {sum(r['equal'] for r in verify)} / {len(verify)} corpus cases. Unequal concatenations remain in the raw diagnostics and are excluded from this ranking table.",'']
if (W/'production/extended-verification.json').exists():
 lines+=['## Exact-output verification',''];rows=[]
 for file,label in [('verification.json','Specification, extensions, renderer reuse, and Ox corpus'),('extra-verification.json','Initial deterministic HTML / inline / MDX event guards'),('extended-verification.json','Independent deterministic HTML / inline / MDX event guards')]:
  r=read('production/'+file);rows.append([label,r['count'],len(r.get('failed',r.get('failures',[])))])
 write_table(lines,['Guard set','Cases','Failures'],rows)
(W/'RESULTS.md').write_text('\n'.join(lines)+'\n')
print('Generated RESULTS.md')
