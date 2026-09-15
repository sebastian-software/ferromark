#!/usr/bin/env python3
"""Recheck complete archived four-stage coverage, equality, and timed checksums."""
import gzip
import hashlib
import json
import statistics
from collections import defaultdict
from pathlib import Path

HERE = Path(__file__).resolve().parent


def raw(name):
    path = HERE / name
    return path.read_bytes() if path.exists() else gzip.decompress(path.with_suffix(path.suffix + '.gz').read_bytes())


def read(name):
    return json.loads(raw(name))


def main():
    run = read('run.json')
    build = read('build.json')
    cases = read('corpus.json')['cases']
    verification = read('verification.json')
    samples = read('samples.json')
    assert len(cases) == 207 and len(run['modes']) == 4
    assert set(run['cases']) == {c['name'] for c in cases} == set(verification)
    assert hashlib.sha256(raw('corpus.json')).hexdigest() == run['corpus_sha256']
    for name, key in [('run.py', 'runner_sha256'), ('worker.rs', 'worker_sha256')]:
        assert hashlib.sha256((HERE / 'harness' / name).read_bytes()).hexdigest() == run[key]
    assert run['worker_sha256'] == build['worker_sha256']
    for engine in ('baseline', 'candidate'):
        for filename, key in [('Cargo.lock', 'lock_sha256'), ('source-Cargo.lock', 'source_lock_sha256'),
                              ('source-Cargo.toml', 'source_manifest_sha256')]:
            assert hashlib.sha256((HERE / 'build' / engine / filename).read_bytes()).hexdigest() == build['engines'][engine][key]
    for case in cases:
        source = case['input'].encode()
        assert len(source) == case['byte_count'] and hashlib.sha256(source).hexdigest() == case['sha256']
        value = verification[case['name']]
        assert value['status'] == 'exact'
        expected = value['modes']['fresh']['baseline']
        for mode in run['modes']:
            for engine in ('baseline', 'candidate'):
                actual = value['modes'][mode][engine]
                assert all(actual[k] == expected[k] for k in ('html', 'ast_debug', 'children'))
    seen = set()
    measured = defaultdict(list)
    for sample in samples:
        identity = (sample['round'], sample['pair'], sample['case'], sample['mode'])
        assert identity not in seen
        seen.add(identity)
        measured[sample['case'], sample['mode']].append(sample)
        assert sorted(sample['order']) == ['baseline', 'candidate']
        for engine in ('baseline', 'candidate'):
            timing = sample[engine]
            output = verification[sample['case']]['modes'][sample['mode']][engine]
            count = output['children'] if sample['mode'] == 'parse' else len(output['html'].encode())
            assert timing['iterations'] > 0 and timing['elapsed_ns'] >= run['window_ms'] * 1_000_000
            assert timing['checksum'] == timing['iterations'] * count % (1 << 64)
    expected = {(r, p, c, m) for r in range(run['rounds']) for p in range(run['pairs_per_round'])
                for c in run['cases'] for m in run['modes']}
    assert seen == expected
    summary = read('summary.json')
    assert len(summary) == len(cases) * len(run['modes'])
    for row in summary:
        selected = measured[row['case'], row['mode']]
        ratios = [s['baseline']['elapsed_ns'] / s['baseline']['iterations'] /
                  (s['candidate']['elapsed_ns'] / s['candidate']['iterations']) for s in selected]
        assert row['baseline_over_candidate_median'] == statistics.median(ratios)
        assert row['round_medians'] == [statistics.median(v for v, s in zip(ratios, selected) if s['round'] == i)
                                        for i in range(run['rounds'])]
        for engine in ('baseline', 'candidate'):
            assert row[engine + '_ns_per_document'] == statistics.median(
                s[engine]['elapsed_ns'] / s[engine]['iterations'] for s in selected)
    result = dict(status='passed', cases=len(cases), stages=len(run['modes']), timed_windows=len(samples) * 2,
                  checks=['input lengths and hashes', 'HTML/AST/spans/child-count equality across both engines and all stages',
                          'all timed checksums', 'complete unique window coverage', 'archived runner and worker hashes',
                          'archived build lock and source manifest hashes', 'summary medians recomputed from raw windows'])
    (HERE / 'artifact-audit.json').write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps(result, indent=2))


if __name__ == '__main__':
    main()
