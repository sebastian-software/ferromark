from experiment import W,Worker,digest
import json,sys
for name in sys.argv[1:]:
 cases=json.loads((W/'extra.json').read_text());w=Worker(W/'bin'/name,W/'extra.json');hashes=[];failed=[]
 expected=json.loads((W/'baseline/extra-hashes.json').read_text()) if name!='baseline' else None
 for i,c in enumerate(cases):
  actual=w.call('events',i);h=digest(actual);hashes.append(h)
  if expected is not None and h!=expected[i]:failed.append(dict(index=i,case=c,actual=actual))
 w.close()
 if name=='baseline':(W/'baseline/extra-hashes.json').write_text(json.dumps(hashes))
 (W/name/'extra-verification.json').write_text(json.dumps(dict(count=len(hashes),hash=digest(hashes),failures=failed),indent=2))
 print(name,len(hashes),'extra event/HTML guards; failures',len(failed),flush=True)
