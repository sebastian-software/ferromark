#!/usr/bin/env python3
"""Describe paired run results without treating corpus views as independent trials."""
import argparse
import json
from pathlib import Path
import statistics

p=argparse.ArgumentParser(description=__doc__)
p.add_argument('run',type=Path)
p.add_argument('output',type=Path)
a=p.parse_args()
rows=json.loads((a.run/'summary.json').read_text())
corpus={c['name']:c for c in json.loads((a.run/'corpus.json').read_text())['cases']}

def group(selected):
 result={}
 for mode in dict.fromkeys(r['mode'] for r in selected):
  items=[r for r in selected if r['mode']==mode]
  ratios=[r['baseline_over_candidate_median'] for r in items]
  result[mode]={
   'cases':len(items),'geometric_mean':statistics.geometric_mean(ratios),
   'round_geometric_means':[statistics.geometric_mean(r['round_medians'][i] for r in items)
                             for i in range(len(items[0]['round_medians']))],
   'measured_above_1':sum(x>1 for x in ratios),'measured_below_1':sum(x<1 for x in ratios),
   'within_one_percent_ratio':sum(.99<=x<=1.01 for x in ratios),
   'worst':sorted(items,key=lambda r:r['baseline_over_candidate_median'])[:5],
   'best':sorted(items,key=lambda r:r['baseline_over_candidate_median'],reverse=True)[:5],
  }
 return result
result={'all':group(rows),'categories':{},'sizes':{}}
for category in sorted({corpus[r['case']]['category'] for r in rows}):
 result['categories'][category]=group([r for r in rows if corpus[r['case']]['category']==category])
for low,high in ((0,512),(513,2048),(2049,16384),(16385,65536),(65537,2**64)):
 items=[r for r in rows if low<=corpus[r['case']]['byte_count']<=high]
 if items:result['sizes'][f'{low}..{high}']=group(items)
a.output.write_text(json.dumps(result,indent=2)+'\n')
for mode,g in result['all'].items():print(mode,g['cases'],round(g['geometric_mean'],4))
