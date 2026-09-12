from pathlib import Path
import json,statistics,sys
W=Path(__file__).parent;R=Path('/Users/sebastian/Workspace/.codex/15d0/ferromark');D=R/'docs/reports/2026-09-12-ox-profile-after-block-render';E=W/'experiments'
C=W/'comparison' if (W/'comparison').exists() else W
if (W/'comparison').exists():D=W
labels={568:'Compiler Options',567:'MSBuild',63:'Vue render function',34:'Vue Suspense',442:'Rust recoverable errors'}
a=['# Measured results','','Generated from the archived raw windows and profiles. Negative experiment changes mean less elapsed time.','','## Current native comparison','','Medians across three fresh process groups; each group has nine rotating 60 ms windows and a 35 ms warmup. All five inputs pass the limited normalized-output comparison.','','| Document | Ferromark µs | Ox growing µs | Ox presized µs | Ferromark extra time vs growing, three groups |','| --- | ---: | ---: | ---: | --- |']
for i,label in labels.items():
 rs=[next(x for x in json.loads((C/f'baseline-{n}-summary.json').read_text()) if x['index']==i) for n in range(1,4)]
 med=[statistics.median(x['median_ns'][name] for x in rs)/1000 for name in ['ferro','ox-grow','ox-presize']]
 ratios=', '.join(f"{(x['median_ns']['ferro']/x['median_ns']['ox-grow']-1)*100:+.2f}%" for x in rs)
 a.append(f'| {label} | {med[0]:.3f} | {med[1]:.3f} | {med[2]:.3f} | {ratios} |')
a+=['','## Visible profile attribution','','Each entry shows two captures. These are sampled stack shares, not exact phase durations. Inlining and the diagnostic call boundaries change attribution. Allocation, conversion and sort shares overlap other groups.','','| Document | Ferromark visible block % | Ferromark visible inline % | Ferromark allocation stacks % | Ox allocation stacks % |','| --- | ---: | ---: | ---: | ---: |']
profiles=json.loads((W/'profile-summary.json').read_text())
for i,label in labels.items():
 fs=[p for p in profiles if p['file'].startswith(f'ferro-{i}-')];os=[p for p in profiles if p['file'].startswith(f'ox-grow-{i}-')]
 vals=[', '.join(f"{p['stages'][key]['pct']:.2f}" for p in fs) for key in ['visible block frames','visible inline frames']]
 vals += [', '.join(f"{p['allocation_stack_pct']:.2f}" for p in ps) for ps in [fs,os]]
 a.append('| '+label+' | '+' | '.join(vals)+' |')
a+=['','### Separate attribution probe','','Only three functions are forced out of line. Its 649 outputs match the regular release build. These percentages describe the probe, not a performance improvement.','','| Document | Mark collection % | Event construction and sorting % | Autolink search % |','| --- | ---: | ---: | ---: |']
for i in [34,442]:
 ps=[p for p in profiles if p['file'].startswith(f'attribution-{i}-')];values=[]
 for part in ['collect_marks_impl','>::emit_events','find_autolinks_into']:
  vals=[]
  for p in ps:
   found=[x['pct'] for x in p['inclusive'] if part in x['symbol'] and 'closure' not in x['symbol']]
   vals.append(f'{found[0]:.2f}' if found else 'below top 25')
  values.append(', '.join(vals))
 a.append('| '+labels[i]+' | '+' | '.join(values)+' |')
a+=['','## Independent experiments','','Each correct source passed 111,902 exact before/after output comparisons before its screen. Initial screening covers 93 inputs with five alternating 30 ms windows and 60 ms warmups. Incorrect sources are excluded from timing.','','| Prototype | Output result | Compiler Options % | MSBuild % | Vue render function % | Vue Suspense % | Rust chapter % |','| --- | --- | ---: | ---: | ---: | ---: | ---: |']
names=['html-blank-run','softbreak-ranges','softbreak-no-code','softbreak-guarded','escape-short-copy','escape-single-scan','autolink-prefilter'];idx=[613,612,108,79,487]
for n in names:
 directory=E/n
 if not directory.exists():continue
 f=directory/'summary.json'
 if f.exists():
  rows=json.loads(f.read_text());v=[next(x['change_pct'] for x in rows if x['index']==i) for i in idx]
  a.append('| '+n+' | exact output preserved | '+' | '.join(f'{x:+.2f}' for x in v)+' |')
 else:
  failures=[]
  for p in directory.glob('*verification.json'):
   d=json.loads(p.read_text());bad=d.get('failures',d.get('failed',[]))
   if bad:failures.append(f'{len(bad)} in {p.name}')
  a.append('| '+n+' | '+('; '.join(failures) or 'not completed')+' | — | — | — | — | — |')
a+=['','### Longer confirmation','','Two additional fresh process pairs per correct prototype, seven alternating 50 ms windows per input and 60 ms warmup. The 37 selected inputs include the five target documents, baseline controls, and every input over 2% slower in any of the first four correct screens. These are repeat ranges, not confidence intervals.','','| Prototype | Compiler Options % | MSBuild % | Vue render function % | Vue Suspense % | Rust chapter % |','| --- | ---: | ---: | ---: | ---: | ---: |']
for n in names:
 paths=[E/n/f'confirm-{r}/summary.json' for r in [1,2]]
 if not all(p.exists() for p in paths):continue
 rs=[json.loads(p.read_text()) for p in paths];vals=[', '.join(f"{next(x['change_pct'] for x in r if x['index']==i):+.2f}" for r in rs) for i in idx]
 a.append('| '+n+' | '+' | '.join(vals)+' |')
extra=[E/'autolink-prefilter'/f'heading-check-{r}/summary.json' for r in [1,2]]
if all(p.exists() for p in extra):
 vals=[json.loads(p.read_text())[0]['change_pct'] for p in extra]
 a += ['', 'The later autolink screen also flagged the headings control outside the original selection. Two additional fresh pairs measure '+', '.join(f'{x:+.2f}%' for x in vals)+'.']
a+=['','### Repeated control regressions','','Cases more than 2% slower in both longer pairs. This prevents target-document wins from hiding repeatable losses.','','| Prototype | Input | Changes % |','| --- | --- | ---: |']
for n in names:
 paths=[E/n/f'confirm-{r}/summary.json' for r in [1,2]]
 if not all(p.exists() for p in paths):continue
 left,right=[json.loads(p.read_text()) for p in paths]
 for x,y in zip(left,right):
  assert x['index']==y['index']
  if min(x['change_pct'],y['change_pct'])>2:a.append(f"| {n} | {x['case']} | {x['change_pct']:+.2f}, {y['change_pct']:+.2f} |")
result='\n'.join(a)+'\n'
if '--check' in sys.argv:assert (D/'RESULTS.md').read_text()==result, 'Generated results differ'
else:(D/'RESULTS.md').write_text(result)
