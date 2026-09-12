from experiment import W,Worker
import json,gzip,statistics,sys
cases=json.loads((W/'memory/cases.json').read_text())
for case in cases:case['reuse']=False
(W/'large').mkdir(exist_ok=True)
cases_file=W/'large/cases.json';cases_file.write_text(json.dumps(cases))
indices=[i for i,c in enumerate(cases) if len(c['input'].encode())>=8000000]
for process in range(1,4):
 dest=W/'large'/f'confirm-{process}';dest.mkdir(parents=True,exist_ok=True)
 if '--resume' in sys.argv and (dest/'summary.json').exists():continue
 workers={name:Worker(W/'bin'/name,cases_file) for name in ['baseline','production']};raw=[];summary=[]
 for i in indices:
  vals={name:[] for name in workers}
  for worker in workers.values():worker.call('window',i,60)
  for window_index in range(5):
   for name in (list(workers) if (window_index+process)%2 else list(reversed(workers))):
    row=workers[name].call('window',i,30);row.update(index=i,round=window_index,engine=name);raw.append(row);vals[name].append(row['elapsed_ns']/row['count'])
  a,b=[statistics.median(vals[name]) for name in workers]
  summary.append(dict(index=i,case=cases[i]['case'],baseline_ns=a,candidate_ns=b,change_pct=(b/a-1)*100))
 for worker in workers.values():worker.close()
 (dest/'windows.jsonl.gz').write_bytes(gzip.compress(('\n'.join(json.dumps(x) for x in raw)+'\n').encode(),mtime=0));(dest/'summary.json').write_text(json.dumps(summary,indent=2))
 print('Large confirmation',process,[(r['case'],round(r['change_pct'],2)) for r in summary],flush=True)
