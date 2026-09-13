from experiment import W, Worker, digest
import json, sys
cases = json.loads((W/'precedence.json').read_text())
for name in sys.argv[1:]:
    w = Worker(W/'bin'/name, W/'precedence.json')
    hashes = []
    expected = json.loads((W/'baseline/precedence-hashes.json').read_text()) if name != 'baseline' else None
    failures = []
    for i, case in enumerate(cases):
        v = w.call('events', i)
        hashes.append(digest(v))
        if expected is not None and hashes[-1] != expected[i]:
            failures.append(dict(index=i, case=case, actual=v))
    w.close()
    if expected is None:
        (W/'baseline/precedence-hashes.json').write_text(json.dumps(hashes))
    (W/name/'precedence-verification.json').write_text(json.dumps(dict(count=len(cases), failures=failures, sha256=digest(hashes))))
    print(name, 'precedence', len(cases), 'failures', len(failures), flush=True)
    assert not failures
