#!/usr/bin/env python3
"""Recheck archived inputs, timed checksums, coverage, and prior HTML agreement."""
import gzip
import hashlib
import json
import statistics
from collections import defaultdict
from pathlib import Path

HERE = Path(__file__).resolve().parent


def read(root, name):
    path = root / name
    raw = path.read_bytes() if path.exists() else gzip.decompress(path.with_suffix(path.suffix + '.gz').read_bytes())
    return json.loads(raw)


def main():
    run = read(HERE, 'run.json')
    cases = read(HERE, 'corpus.json')['cases']
    verification = read(HERE, 'verification.json')
    previous = read(HERE.parent / '2026-09-14-native-matched', 'verification.json')
    samples = read(HERE, 'samples.json')
    jobs = {j['name']: j for j in run['jobs']}
    assert len(cases) == 57 and len(jobs) == 59
    for case in cases:
        raw = case['input'].encode()
        assert len(raw) == case['byte_count']
        assert hashlib.sha256(raw).hexdigest() == case['sha256']
    assert verification == previous, 'HTML or agreement classifications changed'
    for name, key in [('run.py', 'runner_sha256'), ('verify.py', 'verifier_sha256')]:
        assert hashlib.sha256((HERE / 'harness' / name).read_bytes()).hexdigest() == run[key]
    seen = set()
    measured = defaultdict(list)
    for sample in samples:
        identity = (sample['round'], sample['sample'], sample['case'], sample['mode'])
        assert identity not in seen
        seen.add(identity)
        measured[sample['case'], sample['mode']].append(sample)
        assert sorted(sample['order']) == sorted(run['engines'])
        for engine in run['engines']:
            timing = sample[engine]
            count = sum(len(verification[n]['outputs'][engine].encode()) for n in jobs[sample['case']]['members'])
            assert timing['iterations'] > 0 and timing['elapsed_ns'] >= run['window_ms'] * 1_000_000
            assert timing['checksum'] == timing['iterations'] * count % (1 << 64)
    expected = {(r, s, j, m) for r in range(run['rounds']) for s in range(run['samples'])
                for j in jobs for m in run['modes']}
    assert seen == expected
    summary = read(HERE, 'summary.json')
    assert len(summary) == len(jobs) * len(run['modes'])
    for row in summary:
        selected = measured[row['case'], row['mode']]
        for engine in run['engines']:
            rounds = [statistics.median(s[engine]['elapsed_ns'] / s[engine]['iterations']
                                         for s in selected if s['round'] == i)
                      for i in range(run['rounds'])]
            assert row['engines'][engine]['round_medians_ns'] == rounds
            assert row['engines'][engine]['ns'] == statistics.median(rounds)
    result = dict(status='passed', documents=len(cases), workloads=len(jobs),
                  timed_windows=len(samples) * len(run['engines']),
                  previous_html_outputs_unchanged=len(cases) * len(run['engines']),
                  checks=['input lengths and hashes', 'all timed checksums', 'complete unique window coverage',
                          'unchanged HTML and agreement classifications', 'archived runner and comparator hashes',
                          'summary medians recomputed from raw windows'])
    (HERE / 'artifact-audit.json').write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps(result, indent=2))


if __name__ == '__main__':
    main()
