from experiment import W,Worker,digest
import json,sys
for name in sys.argv[1:]:
 cases=json.loads((W/'fences.json').read_text());w=Worker(W/'bin'/name,W/'fences.json');hashes=[];failed=[]
 expected=json.loads((W/'baseline/fence-hashes.json').read_text()) if name!='baseline' else None
 for i,c in enumerate(cases):
  actual=w.call('blocks',i);h=digest(actual);hashes.append(h)
  if expected is not None and h!=expected[i]:failed.append(dict(index=i,case=c,actual=actual))
 w.close()
 if name=='baseline':(W/'baseline/fence-hashes.json').write_text(json.dumps(hashes))
 (W/name/'fence-verification.json').write_text(json.dumps(dict(count=len(hashes),hash=digest(hashes),failures=failed),indent=2))
 print(name,len(hashes),'fence block/MDX/HTML guards; failures',len(failed),flush=True)
 assert not failed
