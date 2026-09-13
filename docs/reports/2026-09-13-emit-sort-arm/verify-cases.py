from experiment import W,Worker,digest
import json
cases=json.loads((W/'cases.json').read_text());expected=None
for name in ['baseline','unstable-packed','stable-packed','stable-tuple']:
    worker=Worker(W/'bin'/name,W/'cases.json')
    actual=[digest(worker.call('verify' if c['flags']&256 or c['reuse'] else 'html',i)) for i,c in enumerate(cases)]
    worker.close()
    if expected is None:expected=actual
    failures=[i for i,(a,b) in enumerate(zip(actual,expected)) if a!=b]
    (W/name/'all-cases-verification.json').write_text(json.dumps(dict(count=len(actual),sha256=digest(actual),failures=failures),indent=2)+'\n')
    assert not failures,(name,failures)
    print(name,len(actual),'outputs match',flush=True)
