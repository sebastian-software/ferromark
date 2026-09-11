#!/usr/bin/env python3
"""Summarize macOS sample leaf symbols; percentages are not phase timings."""
from pathlib import Path
import json,re,sys
root=Path(sys.argv[1]); result=[]
for file in sorted(root.glob('*.sample.txt')):
    s=file.read_text()
    total=int(re.search(r'Call graph:\n\s+(\d+) Thread_',s)[1])
    tail=s.split('Sort by top of stack, same collapsed (when >= 5):\n')[1].split('\nBinary Images:')[0]
    entries=[]
    for line in tail.splitlines():
        match=re.match(r'\s+(.+?)\s+\(in (.+?)\)\s+(\d+)\s*$',line)
        if match:
            symbol,image,count=match.groups();count=int(count)
            entries.append({'symbol':symbol,'image':image,'samples':count,'percent':100*count/total})
    result.append({'file':file.name,'samples':total,'leaf_symbols':entries,'unlisted_samples':total-sum(e['samples'] for e in entries)})
(root/'profile-summary.json').write_text(json.dumps(result,indent=2)+'\n')
for p in result:
    print(p['file'],p['samples'])
    for e in p['leaf_symbols'][:10]:print(f"  {e['percent']:5.1f}% {e['symbol']}")
