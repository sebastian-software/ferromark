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
    previous = read(HERE.parent / '2026-09-16-native-segments', 'verification.json')
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

    # The two held-out PGO runs share the frozen inputs and the same parameters.
    # Their HTML is not archived a second time: every timed window's checksum is
    # recomputed here from this report's own verification.json, which only holds
    # if both executables produced exactly the outputs archived at the root.
    by_name = {c['name']: c for c in cases}
    held_out = {}
    for label, directory in (('default', HERE / 'pgo/default'), ('pgo', HERE / 'pgo/pgo')):
        held = read(directory, 'run.json')
        held_cases = read(directory, 'corpus.json')['cases']
        assert len(held_cases) == 28, label
        assert (held['rounds'], held['samples'], held['window_ms'], held['warmup_ms']) == \
               (run['rounds'], run['samples'], run['window_ms'], run['warmup_ms']), label
        for case in held_cases:
            assert case['sha256'] == by_name[case['name']]['sha256'], (label, case['name'])
        held_jobs = {j['name']: j for j in held['jobs']}
        held_samples = read(directory, 'samples.json')
        held_seen = set()
        for sample in held_samples:
            identity = (sample['round'], sample['sample'], sample['case'], sample['mode'])
            assert identity not in held_seen, (label, identity)
            held_seen.add(identity)
            assert sorted(sample['order']) == sorted(held['engines']), label
            for engine in held['engines']:
                timing = sample[engine]
                count = sum(len(verification[n]['outputs'][engine].encode())
                            for n in held_jobs[sample['case']]['members'])
                assert timing['iterations'] > 0
                assert timing['elapsed_ns'] >= held['window_ms'] * 1_000_000
                assert timing['checksum'] == timing['iterations'] * count % (1 << 64), \
                    (label, sample['case'], engine)
        expected_held = {(r, s, j, m) for r in range(held['rounds'])
                         for s in range(held['samples']) for j in held_jobs
                         for m in held['modes']}
        assert held_seen == expected_held, label
        held_out[label] = dict(documents=len(held_cases),
                               timed_windows=len(held_samples) * len(held['engines']))
    build_default = read(HERE, 'build.json')
    build_pgo = read(HERE, 'build-pgo.json')
    assert 'pgo' not in build_default
    assert build_pgo['pgo']['applied'] is True
    assert build_pgo['pgo']['training']['disjoint_from_measured_set'] is True
    assert build_default['lock_sha256'] == build_pgo['lock_sha256']
    assert build_default['binary_sha256'] != build_pgo['binary_sha256']
    for build in (build_default, build_pgo):
        assert build['engines']['ferromark_v2']['revision'].startswith('bffc89f6')

    result = dict(status='passed', documents=len(cases), workloads=len(jobs),
                  timed_windows=len(samples) * len(run['engines']),
                  held_out_runs=held_out,
                  previous_html_outputs_unchanged=len(cases) * len(run['engines']),
                  checks=['input lengths and hashes', 'all timed checksums', 'complete unique window coverage',
                          'unchanged HTML and agreement classifications against 2026-09-16-native-segments',
                          'archived runner and comparator hashes',
                          'summary medians recomputed from raw windows',
                          'held-out runs share the frozen inputs, parameters and HTML',
                          'default build carries no PGO; PGO build trained disjointly from the measured set'])
    (HERE / 'artifact-audit.json').write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps(result, indent=2))


if __name__ == '__main__':
    main()
