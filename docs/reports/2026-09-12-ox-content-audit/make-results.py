import gzip,json,statistics
from pathlib import Path
root=Path(__file__).parent

def rows(name):
 p=root/name
 data=gzip.decompress(p.read_bytes()).decode() if p.suffix=='.gz' else p.read_text()
 return [json.loads(l) for l in data.splitlines()]
def locate(name):
 return name if (root/name).exists() else name+'.gz'
measurements={}
for n in range(1,4):
 r=rows(locate(f'matrix-{n}.jsonl'));assert len(r)==99
 for x in r:
  assert len(x['windows_ns'])==11 and all(t>0 for t in x['windows_ns'])
  measurements.setdefault((x['case'],x['mode']),[]).append(statistics.median(x['windows_ns'])/1000)
def time(c,m):return statistics.median(measurements[c,m])
lines=['# Measured results','', 'Generated from the archived measurement windows by `make-results.py`. Times are microseconds; lower is better. Ratios above 1 mean Ox is faster.','', '## Same upstream text, different versions and settings','', '| Input | Ferro 0.7 defaults | Ferro 0.9 defaults | Ox defaults | Ferro 0.9 tables | Ox tables | Matched ratio |','|---|---:|---:|---:|---:|---:|---:|']
for c in ['upstream-large','upstream-huge']:
 vals=[time(c,m) for m in ['old-default','ferro-default','ox-default','ferro-tables','ox-tables']]
 lines.append('| '+c+' | '+' | '.join(f'{x:.2f}' for x in vals)+f' | {vals[-2]/vals[-1]:.2f}x |')
lines+=['','## Workload localization','', 'Both use the matched CommonMark settings with heading IDs, except the table workload which enables tables in both. All these pairs passed Ox-normalized output equality.','', '| Input | Ferromark | Ox | Ferro / Ox |','|---|---:|---:|---:|']
for c in ['short','plain','inline','lists','code','tables','headings']:
 suffix='tables' if c=='tables' else 'cm';a,b=time(c,'ferro-'+suffix),time(c,'ox-'+suffix)
 lines.append(f'| {c} | {a:.3f} | {b:.3f} | {a/b:.2f}x |')
lines+=['','## One-variable controls','', '| Input | Ox presized, fresh renderer | Ox growing arena | Ox reused renderer | Ferro heading IDs on | Ferro heading IDs off |','|---|---:|---:|---:|---:|---:|']
for c in ['upstream-large','upstream-huge','short','headings']:
 lines.append('| '+c+' | '+' | '.join(f'{time(c,m):.3f}' for m in ['ox-cm','ox-cm-grow','ox-cm-reuse','ferro-cm','ferro-cm-noids'])+' |')
lines+=['','## Run stability','', 'Median per process run, in microseconds.','', '| Mode, large input | Run 1 | Run 2 | Run 3 |','|---|---:|---:|---:|']
for m in ['ferro-tables','ox-tables','ferro-cm','ox-cm']:
 lines.append('| '+m+' | '+' | '.join(f'{x:.2f}' for x in measurements['upstream-large',m])+' |')
scans={}
for n in range(1,4):
 for x in rows(locate(f'scans-{n}.jsonl')):scans.setdefault((x['bytes'],x['shape'],x['engine']),[]).append(statistics.median(x['windows_ns']))
lines+=['','## Isolated scanner comparison','', 'Nanoseconds per search. Actual upstream scanner implementations, with the same 11-byte set for both; this is not Ferromark’s different Markdown marker set. All 256 byte values at every offset in lengths 1–64 were checked for equal results before each run. Three runs, eleven 20 ms windows per variant; fixed per-case engine order. These are diagnostic kernel timings, not end-to-end speedups.','', '| Bytes | Match | Ferromark ByteSet | Ox scanner | Ferro / Ox |','|---|---|---:|---:|---:|']
for n in [8,16,32,128,1024,65536]:
 for shape in ['absent','last','first']:
  a,b=[statistics.median(scans[n,shape,e]) for e in ['ferro','ox']]
  lines.append(f'| {n} | {shape} | {a:.2f} | {b:.2f} | {a/b:.2f}x |')
if (root/locate('allocations.jsonl')).exists():
 lines+=['','## Allocation counts','', 'Mean over ten calls after one warmup. Global allocator calls include allocations and reallocations. Requested bytes count every allocation request (including full replacement size on realloc), not peak live memory or RSS. Instrumented runs are excluded from timing. Renderer construction is inside each fresh call and outside each reused call.','', '| Case | Mode | Allocation calls | Requested bytes |','|---|---|---:|---:|']
 for x in rows(locate('allocations.jsonl')):
  if x['case'] in ['upstream-large','short','tables'] and x['mode'] in ['ferro-cm','ox-cm','ox-cm-grow','ox-cm-reuse','ferro-tables','ox-tables']:
   lines.append(f"| {x['case']} | {x['mode']} | {x['allocation_calls']} | {x['requested_bytes']} |")
reuse={}
for n in range(1,4):
 for x in rows(locate(f'reuse-{n}.jsonl')):
  reuse.setdefault((x['case'],x['mode']),[]).append(statistics.median(x['windows_ns'])/1000)
lines+=['','## Reuse on both sides','', 'Supplementary three-run comparison, same protocol. Both still return a fresh owned HTML string; Ferromark retains its parser buffers and Ox retains renderer scratch, with a fresh Ox arena per document. Heading IDs are enabled on both.','', '| Input | Ferromark fresh | Ox fresh | Ferromark reused | Ox reused |','|---|---:|---:|---:|---:|']
for c in ['short','upstream-large']:
 lines.append('| '+c+' | '+' | '.join(f'{statistics.median(reuse[c,m]):.3f}' for m in ['ferro-cm','ox-cm','ferro-cm-reuse','ox-cm-reuse'])+' |')
v=rows(locate('output-verification.jsonl'));assert len(v)==18 and all(x['normalized_equal'] for x in v)
(root/'RESULTS.md').write_text('\n'.join(lines)+'\n')
