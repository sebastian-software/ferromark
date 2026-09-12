from experiment import W,Worker
import statistics,json,gzip
names=['baseline','code-safe+html-search','code-safe+compact-all+compact-code+html-search']
cases=json.loads((W/'cases.json').read_text());selected=json.loads((W/'selected.json').read_text())
for repeat in range(3):
 workers={n:Worker(W/'bin'/n,W/'cases.json') for n in names};raw=[];summary=[]
 for i in selected:
  vals={n:[] for n in names}
  for n in names:workers[n].call('window',i,100)
  for window in range(9):
   offset=(window+repeat)%len(names);order=names[offset:]+names[:offset]
   if (window+repeat)%2:order=list(reversed(order))
   for n in order:
    row=workers[n].call('window',i,75);row.update(engine=n,index=i,window=window,repeat=repeat);raw.append(row);vals[n].append(row['elapsed_ns']/row['count'])
  medians={n:statistics.median(x) for n,x in vals.items()};summary.append(dict(index=i,case=cases[i]['case'],medians_ns=medians,changes_pct={n:(x/medians['baseline']-1)*100 for n,x in medians.items()}))
  if i in [79,108,487,613]:print('Repeat',repeat+1,cases[i]['case'],{n:round(v,1) for n,v in summary[-1]['changes_pct'].items()},flush=True)
 for p in workers.values():p.close()
 d=W/f'confirmation-{repeat+1}';d.mkdir(exist_ok=True)
 (d/'summary.json').write_text(json.dumps(summary,indent=2));(d/'windows.jsonl.gz').write_bytes(gzip.compress(('\n'.join(json.dumps(x) for x in raw)+'\n').encode(),mtime=0))
 print('Confirmation',repeat+1,'complete',flush=True)
