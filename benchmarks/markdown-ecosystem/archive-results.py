#!/usr/bin/env python3
"""Validate a complete ecosystem pair and retain its raw evidence and source snapshot."""
import argparse
from collections import Counter
import gzip
import hashlib
import json
import math
from pathlib import Path
import shutil
import subprocess
import run

REPO = run.REPO


def validate(source):
    read = lambda name: json.loads((source / (name + '.json')).read_text())
    config, summary, corpus, outputs, rows = (read(name) for name in ('run', 'summary', 'corpus', 'verification', 'samples'))
    engine = config['engines'][1]
    assert config['rounds'] == 3 and config['samples'] == 6
    assert config['window_ms'] == 40 and config['warmup_ms'] == 60
    assert len(corpus['cases']) == summary['documents'] == 57
    expected = {(r, s, case['name'], mode) for r in range(3) for s in range(6)
                for case in corpus['cases'] for mode in run.native.MODES}
    assert len(rows) == len(expected) == 2052
    assert {(row['round'], row['sample'], row['case'], row['mode']) for row in rows} == expected
    assert set(outputs) == {case['name'] for case in corpus['cases']}
    matched = set()
    for case in corpus['cases']:
        data = case['input'].encode()
        assert len(data) == case['byte_count'] and hashlib.sha256(data).hexdigest() == case['sha256']
        result = outputs[case['name']]
        assert run.native.classify(result['outputs']['v2'], result['outputs'][engine]) == result['agreement']
        if result['agreement'] in ('exact', 'serialization-equivalent'):
            matched.add(case['name'])
    assert len(matched) == summary['agreeing_documents']
    assert dict(Counter(result['agreement'] for result in outputs.values())) == summary['agreement_counts']
    for mode in run.native.MODES:
        assert math.isclose(run.aggregate(rows, matched, mode, engine), summary['v2_relative_throughput'][mode], rel_tol=1e-12)
    for row in rows:
        assert set(row['order']) == {'v2', engine}
        for name in ('v2', engine):
            timing = row[name]
            units = run.output_units(outputs[row['case']]['outputs'][name], config['track'])
            assert timing['iterations'] > 0 and timing['elapsed_ns'] >= 40_000_000
            assert timing['checksum'] == timing['iterations'] * units
    lock = 'Cargo.lock' if config['track'] == 'native' else 'package-lock.json'
    assert run.native.sha(source / lock) == config['lock_sha256']
    # The measured core must match the earlier clean-core run, across both platforms.
    baseline = json.loads((REPO / 'docs/reports/2026-09-30-markdown-ecosystem-main/node/run.json').read_text())
    assert config['local_source_sha256'] == baseline['local_source_sha256']
    assert config['corpus_sha256'] == baseline['corpus_sha256']
    frozen = run.native.read_json(REPO / 'docs/reports/2026-09-14-optimization-rounds/broad-corpus.json.gz')
    if engine in ('cmark', 'commonmark'):
        for case in frozen['cases']:
            case['profile'] = 'commonmark'
    assert corpus['cases'] == frozen['cases'], 'frozen inputs or profiles changed'
    for path, digest in config['local_source_sha256'].items():
        data = subprocess.check_output(['git', 'show', config['git_head'] + ':' + path], cwd=REPO)
        assert hashlib.sha256(data).hexdigest() == digest, path
    sources = {}
    lock_path = 'benchmarks/markdown-ecosystem/' + ('native/Cargo.lock' if config['track'] == 'native' else 'package-lock.json')
    lock_data = subprocess.check_output(['git', 'show', config['git_head'] + ':' + lock_path], cwd=REPO)
    assert hashlib.sha256(lock_data).hexdigest() == config['lock_sha256'], 'committed lock changed'
    files = {'run.py': ('benchmarks/markdown-ecosystem/run.py', 'runner_sha256'),
             'verify.py': ('benchmarks/native-comparison/verify.py', 'verifier_sha256'),
             'native-run.py': ('benchmarks/native-comparison/run.py', 'guards_sha256')}
    if config['track'] == 'node':
        files.update({'worker.mjs': ('benchmarks/markdown-ecosystem/worker.mjs', 'worker_sha256'),
                      'node-adapters.mjs': ('benchmarks/markdown-ecosystem/node-adapters.mjs', 'node_adapters_sha256')})
    else:
        files['worker.rs'] = ('benchmarks/markdown-ecosystem/worker.rs', 'adapter_sha256')
        build = config['build_metadata']
        assert config['worker_sha256'] == build['dispatcher_sha256']
        for name, digest in build['adapter_sha256'].items():
            data = subprocess.check_output(['git', 'show', config['git_head'] + ':benchmarks/markdown-ecosystem/' + name], cwd=REPO)
            assert hashlib.sha256(data).hexdigest() == digest
            sources[name] = data
    for name, (path, key) in files.items():
        data = subprocess.check_output(['git', 'show', config['git_head'] + ':' + path], cwd=REPO)
        assert hashlib.sha256(data).hexdigest() == config[key], name
        sources[name] = data
    return config, sources


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('source', type=Path)
    p.add_argument('destination', type=Path)
    args = p.parse_args()
    config, sources = validate(args.source)
    args.destination.mkdir(parents=True, exist_ok=False)
    for name in ('corpus', 'verification', 'samples', 'behavior'):
        (args.destination / (name + '.json.gz')).write_bytes(gzip.compress((args.source / (name + '.json')).read_bytes(), mtime=0))
    for name in ('run.json', 'summary.json', 'Cargo.lock' if config['track'] == 'native' else 'package-lock.json', 'host.txt', 'build.log'):
        if (args.source / name).exists():
            shutil.copyfile(args.source / name, args.destination / name)
    snapshot = args.destination / 'adapters'
    snapshot.mkdir()
    for name, data in sources.items():
        (snapshot / name).write_bytes(data)
    print('Verified and retained 2,052 windows:', args.destination)


if __name__ == '__main__':
    main()
