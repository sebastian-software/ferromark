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
    previous = read(HERE.parent / '2026-09-15-native-arm', 'verification.json')
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
    diagnosis = read(HERE, 'diagnosis.json')
    assert diagnosis['metadata']['binaries'][0]['sha256'] == read(HERE.parent / '2026-09-15-native-arm', 'build.json')['binary_sha256']
    assert diagnosis['metadata']['binaries'][1]['sha256'] == read(HERE, 'build.json')['binary_sha256']
    assert diagnosis['metadata']['corpus_sha256'] == hashlib.sha256((HERE / 'corpus.json.gz').read_bytes()).hexdigest()
    expected_diagnosis = {(name, variant, mode)
        for name in ('comment-incident', 'rust-book-ch00-00-introduction', 'comment-review')
        for variant in ('original', 'no-definitions') for mode in ('fresh', 'reuse')}
    assert len(diagnosis['rows']) == len(expected_diagnosis)
    assert {(r['case'], r['variant'], r['mode']) for r in diagnosis['rows']} == expected_diagnosis
    for row in diagnosis['rows']:
        source = next(c['input'] for c in cases if c['name'] == row['case'])
        if row['variant'] == 'no-definitions':
            source = source.replace(']:', ']=')
        else:
            assert row['output'] == verification[row['case']]['outputs']['v2']
        assert hashlib.sha256(source.encode()).hexdigest() == row['input_sha256']
        assert len(row['samples']) == diagnosis['metadata']['samples']
        for sample in row['samples']:
            for side in ('before', 'after'):
                timing = sample[side]
                assert timing['iterations'] > 0 and timing['elapsed_ns'] >= diagnosis['metadata']['window_ms'] * 1_000_000
                assert timing['checksum'] == timing['iterations'] * len(row['output'].encode()) % (1 << 64)
            assert sample['ratio'] == ((sample['after']['elapsed_ns'] / sample['after']['iterations']) /
                                       (sample['before']['elapsed_ns'] / sample['before']['iterations']))
        assert row['delta_percent'] == 100 * (statistics.median(s['ratio'] for s in row['samples']) - 1)
    result = dict(status='passed', documents=len(cases), workloads=len(jobs),
                  timed_windows=len(samples) * len(run['engines']),
                  diagnosis_windows=sum(len(r['samples']) * 2 for r in diagnosis['rows']),
                  previous_html_outputs_unchanged=len(cases) * len(run['engines']),
                  checks=['input lengths and hashes', 'all timed checksums', 'complete unique window coverage',
                          'unchanged HTML and agreement classifications', 'archived runner and comparator hashes',
                          'summary medians recomputed from raw windows',
                          'paired diagnosis binary hashes, inputs, checksums and medians'])
    (HERE / 'artifact-audit.json').write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps(result, indent=2))


if __name__ == '__main__':
    main()
