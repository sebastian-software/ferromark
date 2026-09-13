from experiment import W, Worker
import gzip, json, statistics, sys
cases=[]
for n in [40,80,160,320,640,1280]:
    cases.append(dict(case=f'scaling/tag-code/{n}',input=('The `<Suspense>` component waits for `pending` work. '*n).rstrip(),flags=15,reuse=False))
    cases.append(dict(case=f'scaling/mixed/{n}',input=('**bold** `<Widget>` `pending` <https://example.test> '*n).rstrip(),flags=15,reuse=False))
p=W/'scaling-cases.json';p.write_text(json.dumps(cases))
name=sys.argv[1]; tag=sys.argv[2]
workers={n:Worker(W/'bin'/n,p) for n in ['baseline',name]}
raw=[];rows=[]
for i,c in enumerate(cases):
    html=[w.call('html',i) for w in workers.values()]
    assert html[0]==html[1], c['case']
    values={n:[] for n in workers}
    for w in workers.values():w.call('window',i,35)
    for r in range(7):
        for n in (list(workers) if r%2==0 else list(reversed(workers))):
            v=workers[n].call('window',i,50);v.update(index=i,engine=n,round=r);raw.append(v)
            values[n].append(v['elapsed_ns']/v['count'])
    a,b=[statistics.median(v) for v in values.values()]
    rows.append(dict(index=i,case=c['case'],baseline_ns=a,candidate_ns=b,change_pct=(b/a-1)*100))
for w in workers.values():w.close()
d=W/name/tag;d.mkdir(exist_ok=True,parents=True)
(d/'windows.jsonl.gz').write_bytes(gzip.compress(('\n'.join(json.dumps(v) for v in raw)+'\n').encode(),mtime=0))
(d/'summary.json').write_text(json.dumps(rows,indent=2))
print(name,tag,rows,flush=True)
