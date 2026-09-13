from experiment import *
from regimes import run_regimes
# Include every ordinary case that is slower in all three pairs, or exceeds
# two percent in any pair, together with the previous code/table controls.
groups=[{r['index']:r for r in json.loads((W/f'linear/confirm-{i}/summary.json').read_text())} for i in [1,2,3]]
ids=[i for i in groups[0] if all(g[i]['change_pct']>0 for g in groups) or any(g[i]['change_pct']>2 for g in groups)]
ids=sorted(set(ids+[7,9,10,21,42,44,79,108,487,612,613]))
(W/'follow-selected.json').write_text(json.dumps(ids));print('Follow-up indices',ids,flush=True)
for i in range(2):
 run('aa-control',rounds=11,ms=80,tag=f'aa-control/follow-{i+1}',order=i,warm=80,indices=ids)
 run('linear',rounds=11,ms=80,tag=f'linear/follow-{i+1}',order=i,warm=80,indices=ids)
for prefix,casefile,selection in [('regime','cases.json','regime-selected.json'),('attribute','attributes.json','attribute-selected.json')]:
 groups=[{r['index']:r for r in json.loads((W/f'linear/{prefix}-confirm-{i}/summary.json').read_text())} for i in [1,2,3]]
 ids=[i for i in groups[0] if all(g[i]['change_pct']>0 for g in groups) or any(g[i]['change_pct']>2 for g in groups)]
 if not ids:continue
 path=f'{prefix}-follow-selected.json';(W/path).write_text(json.dumps(ids));print(prefix,'follow-up indices',ids,flush=True)
 for i in range(2):run_regimes('linear',rounds=11,ms=80,tag=f'{prefix}-follow-{i+1}',casefile=casefile,selection=path)
