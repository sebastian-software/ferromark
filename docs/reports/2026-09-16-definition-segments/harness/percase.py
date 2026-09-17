import json,sys
S=sys.argv[1]; names=sys.argv[2:]
data={n:{(r['mode'],r['case']):r for r in json.load(open(f"{S}/results/{n}/summary.json"))} for n in names}
keys=sorted({k for d in data.values() for k in d})
print(f"{'mode':7s} {'case':44s} "+" ".join(f"{n:>9s}" for n in names))
for k in keys:
    row=[]
    for n in names:
        r=data[n].get(k)
        row.append(f"{r['baseline_over_candidate_median']:9.3f}" if r else f"{'-':>9s}")
    print(f"{k[0]:7s} {k[1]:44s} "+" ".join(row))
