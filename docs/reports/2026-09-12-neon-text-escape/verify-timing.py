from experiment import W,Worker,digest
import json,sys
cases=json.loads((W/'cases.json').read_text());indices=json.loads((W/'selected.json').read_text())
for name in sys.argv[1:]:
 worker=Worker(W/'bin'/name,W/'cases.json');hashes=[]
 for i in indices:hashes.append(digest(worker.call('verify' if cases[i]['flags']&256 else 'html',i)))
 worker.close()
 if name=='baseline':(W/'baseline/timing-hashes.json').write_text(json.dumps(hashes))
 expected=json.loads((W/'baseline/timing-hashes.json').read_text());failed=[i for i,a,b in zip(indices,hashes,expected) if a!=b]
 (W/name/'timing-verification.json').write_text(json.dumps(dict(count=len(hashes),failures=failed,sha256=digest(hashes))))
 print(name,'timing guards',len(hashes),'failures',len(failed),flush=True);assert not failed
