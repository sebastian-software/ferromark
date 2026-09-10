#!/usr/bin/env python3
"""Compare retained feature-probe executables with exact HTML and paired timing."""
import argparse
import hashlib
import json
import pathlib
import statistics
import subprocess

p=argparse.ArgumentParser(description=__doc__)
p.add_argument('before',type=pathlib.Path);p.add_argument('after',type=pathlib.Path);p.add_argument('catalog',type=pathlib.Path);p.add_argument('output',type=pathlib.Path)
p.add_argument('--rounds',type=int,default=7);p.add_argument('--window-ms',type=int,default=100)
a=p.parse_args();binaries={'before':a.before.resolve(),'after':a.after.resolve()}
def run(binary,mode,filter='all',lane='all'):
    result=subprocess.run([str(binary),mode,str(a.catalog),filter,lane,str(a.window_ms),'1'],capture_output=True,text=True,check=True)
    return json.loads(result.stdout)
html={key:run(binary,'html') for key,binary in binaries.items()}
assert html['before']==html['after'],'HTML changed; aborting measurement'
filters=['lifecycle/','syntax/heading_ids/medium','syntax/allow_link_refs/medium','syntax/footnotes/medium','syntax/tables/medium','core/emphasis/medium','core/links/medium','core/inline_code/medium']
rows=[]
for round_index in range(a.rounds):
    order=filters[round_index%len(filters):]+filters[:round_index%len(filters)]
    for filter in order:
        labels=['before','after'] if round_index%2==0 else ['after','before']
        pair={label:run(binaries[label],'measure',filter) for label in labels}
        for first,second in zip(pair['before'],pair['after'],strict=True):
            assert (first['id'],first['variant'],first['lane'])==(second['id'],second['variant'],second['lane'])
            rows.append({'id':first['id'],'variant':first['variant'],'lane':first['lane'],'round':round_index,'before':first['ns'],'after':second['ns']})
    print(f'round {round_index+1}/{a.rounds}',flush=True)
summary=[]
for key in dict.fromkeys((r['id'],r['variant'],r['lane']) for r in rows):
    group=[r for r in rows if (r['id'],r['variant'],r['lane'])==key]
    deltas=[(r['after']/r['before']-1)*100 for r in group]
    summary.append({'id':key[0],'variant':key[1],'lane':key[2],'before_ns':statistics.median(r['before'] for r in group),'after_ns':statistics.median(r['after'] for r in group),'paired_change_percent':statistics.median(deltas),'paired_min_percent':min(deltas),'paired_max_percent':max(deltas)})
result={'exact_html_pairs':len(html['before']),'catalog_sha256':hashlib.sha256(a.catalog.read_bytes()).hexdigest(),'binary_sha256':{k:hashlib.sha256(v.read_bytes()).hexdigest() for k,v in binaries.items()},'rounds':a.rounds,'window_ms':a.window_ms,'summary':summary,'samples':rows}
a.output.parent.mkdir(parents=True,exist_ok=True);a.output.write_text(json.dumps(result,indent=2)+'\n')
for r in summary:
    if r['variant'] in ['on','syntax','default','commonmark']:
        print(f"{r['id']:40} {r['variant']:10} {r['lane']:8} {r['paired_change_percent']:+6.2f}%")
