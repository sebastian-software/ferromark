from experiment import W,Worker,digest
import json,sys
for name in sys.argv[1:]:
 cases=json.loads((W/'blocks.json').read_text());w=Worker(W/'bin'/name,W/'blocks.json');hashes=[];failed=[]
 expected=json.loads((W/'baseline/block-hashes.json').read_text()) if name!='baseline' else None
 for i,c in enumerate(cases):
  actual=w.call('blocks',i);h=digest(actual);hashes.append(h)
  if expected is not None and h!=expected[i]:failed.append(dict(index=i,case=c,actual=actual))
 w.close()
 if name=='baseline':(W/'baseline/block-hashes.json').write_text(json.dumps(hashes))
 (W/name/'block-verification.json').write_text(json.dumps(dict(count=len(hashes),hash=digest(hashes),failures=failed),indent=2))
 print(name,len(hashes),'block/MDX/HTML guards; failures',len(failed),flush=True)
 assert not failed
