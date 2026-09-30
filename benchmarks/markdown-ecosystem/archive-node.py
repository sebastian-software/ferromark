#!/usr/bin/env python3
"""Verify and retain expanded Node comparisons and their committed adapter sources."""
import argparse
import gzip
import hashlib
import json
import math
from pathlib import Path
import shutil
import subprocess

import run

REPO = Path(__file__).resolve().parents[2]
ENGINES = ('marked', 'markdown-it', 'remark', 'showdown', 'commonmark')
SOURCES = {
    'worker.mjs': ('benchmarks/markdown-ecosystem/worker.mjs', 'worker_sha256'),
    'node-adapters.mjs': ('benchmarks/markdown-ecosystem/node-adapters.mjs', 'node_adapters_sha256'),
    'run.py': ('benchmarks/markdown-ecosystem/run.py', 'runner_sha256'),
    'verify.py': ('benchmarks/native-comparison/verify.py', 'verifier_sha256'),
    'native-run.py': ('benchmarks/native-comparison/run.py', 'guards_sha256'),
}


def read_json(path):
    return json.loads(path.read_text())


def validate(source, baseline, engine):
    config = read_json(source / 'run.json')
    summary = read_json(source / 'summary.json')
    rows = read_json(source / 'samples.json')
    verification = read_json(source / 'verification.json')
    corpus = read_json(source / 'corpus.json')
    assert config['engines'] == ['v2', engine]
    assert config['local_source_sha256'] == baseline['local_source_sha256'], engine
    assert config['native_addon_sha256'] == baseline['native_addon_sha256'], engine
    assert config['corpus_sha256'] == baseline['corpus_sha256'], engine
    assert run.native.sha(source / 'package-lock.json') == config['lock_sha256']
    expected = config['rounds'] * config['samples'] * len(corpus['cases']) * 2
    assert len(rows) == expected == 2052
    keys = {(row['round'], row['sample'], row['case'], row['mode']) for row in rows}
    assert keys == {(r, s, case['name'], mode) for r in range(config['rounds'])
                    for s in range(config['samples']) for case in corpus['cases'] for mode in run.native.MODES}
    matched = {name for name, result in verification.items()
               if result['agreement'] in ('exact', 'serialization-equivalent')}
    assert len(matched) == summary['agreeing_documents']
    assert set(verification) == {case['name'] for case in corpus['cases']}
    for case in corpus['cases']:
        data = case['input'].encode()
        assert len(data) == case['byte_count'] and hashlib.sha256(data).hexdigest() == case['sha256']
        result = verification[case['name']]
        assert run.native.classify(result['outputs']['v2'], result['outputs'][engine]) == result['agreement']
    for mode in run.native.MODES:
        assert math.isclose(run.aggregate(rows, matched, mode, engine),
                            summary['v2_relative_throughput'][mode], rel_tol=1e-12)
    for row in rows:
        for measured_engine in ('v2', engine):
            timing = row[measured_engine]
            assert timing['iterations'] > 0 and timing['elapsed_ns'] >= config['window_ms'] * 1_000_000
            units = run.output_units(verification[row['case']]['outputs'][measured_engine], 'node')
            assert timing['checksum'] == timing['iterations'] * units
    for path, digest in config['native_addon_sha256'].items():
        assert run.native.sha(REPO / path) == digest
    sources = {}
    for name, (relative, key) in SOURCES.items():
        data = subprocess.check_output(['git', 'show', config['git_head'] + ':' + relative], cwd=REPO)
        assert hashlib.sha256(data).hexdigest() == config[key], (engine, key)
        sources[name] = data
    return summary, sources, len(rows)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--input-prefix', required=True,
                        help='Runner output directory prefix, followed by each engine name')
    parser.add_argument('--report', type=Path,
                        default=REPO / 'docs/reports/2026-09-30-markdown-ecosystem-main')
    args = parser.parse_args()
    baseline = read_json(args.report / 'node/run.json')
    validated = []
    for engine in ENGINES:
        source = Path(args.input_prefix + engine)
        summary, sources, count = validate(source, baseline, engine)
        validated.append((engine, source, summary, sources, count))
    for engine, source, summary, sources, _ in validated:
        destination = args.report / ('node-' + engine)
        destination.mkdir(exist_ok=False)
        for name in ('corpus', 'verification', 'samples', 'behavior'):
            data = (source / (name + '.json')).read_bytes()
            (destination / (name + '.json.gz')).write_bytes(gzip.compress(data, mtime=0))
        for name in ('run.json', 'summary.json', 'package-lock.json'):
            shutil.copyfile(source / name, destination / name)
        revision = read_json(source / 'run.json')['git_head']
        adapters = args.report / 'node-adapters' / revision
        adapters.mkdir(parents=True, exist_ok=True)
        for name, data in sources.items():
            path = adapters / name
            if path.exists():
                assert path.read_bytes() == data, ('adapter snapshot changed', name)
            else:
                path.write_bytes(data)
        print(engine, summary['agreeing_documents'], summary['v2_relative_throughput'], flush=True)
    print(f'Verified and retained {sum(item[4] for item in validated):,} timing windows and source/addon provenance.')


if __name__ == '__main__':
    main()
