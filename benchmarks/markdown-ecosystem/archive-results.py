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


def validate(source, revision=None):
    read = lambda name: json.loads((source / (name + '.json')).read_text()) if (source / (name + '.json')).exists() else json.loads(gzip.decompress((source / (name + '.json.gz')).read_bytes()))
    config, summary, corpus, outputs, rows = (read(name) for name in ('run', 'summary', 'corpus', 'verification', 'samples'))
    engine = config['engines'][1]
    assert config.get('schema', 1) in (1, 2, 3), 'unknown scoring schema'
    policy_path = 'benchmarks/markdown-ecosystem/scoring-policy.json'
    has_policy = subprocess.run(['git', 'cat-file', '-e', config['git_head'] + ':' + policy_path], cwd=REPO, capture_output=True).returncode == 0
    policy_data = None
    if has_policy:
        policy_data = subprocess.check_output(['git', 'show', config['git_head'] + ':' + policy_path], cwd=REPO)
        policy = json.loads(policy_data)
        assert config['schema'] == policy['pair_schema'], 'scoring policy cannot be downgraded'
        assert config['scoring_scope'] == policy['scoring_scope']
        assert hashlib.sha256(policy_data).hexdigest() == config['scoring_policy_sha256']
    else:
        assert config.get('schema', 1) < 3, 'full-corpus campaigns require their committed scoring policy'
    new_contract = subprocess.run(['git', 'cat-file', '-e', config['git_head'] + ':benchmarks/markdown-ecosystem/comparisons.json'], cwd=REPO, capture_output=True).returncode == 0
    assert not new_contract or config.get('schema', 1) >= 2, 'new campaigns cannot omit the rotating-control contract'
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
    validate_scores(config, summary, outputs, rows, matched)
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
    if revision is None:
        assert config['local_source_sha256'] == baseline['local_source_sha256']
    else:
        assert config['git_head'] == revision, 'unexpected measured revision'
        assert not config['git_status'], 'dirty measured checkout'
        tracked = subprocess.check_output(['git', 'ls-tree', '-r', '--name-only', revision], cwd=REPO, text=True).splitlines()
        paths = {path for path in tracked if path.endswith('.rs') and path.startswith(('src/', 'node/native/', 'transforms/'))}
        paths.update(('Cargo.toml', 'Cargo.lock', 'node/native/Cargo.toml', 'transforms/Cargo.toml', 'node/ferromark/index.mjs', 'node/ferromark/native-target.mjs'))
        assert set(config['local_source_sha256']) == paths, 'incomplete measured source hashes'
    assert config['corpus_sha256'] == baseline['corpus_sha256']
    frozen = run.native.read_json(REPO / 'docs/reports/2026-09-14-optimization-rounds/broad-corpus.json.gz')
    if (config['comparison']['profile'] == 'commonmark-only' if config.get('schema', 1) >= 2 else engine in ('cmark', 'commonmark')):
        for case in frozen['cases']:
            case['profile'] = 'commonmark'
    assert corpus['cases'] == frozen['cases'], 'frozen inputs or profiles changed'
    for path, digest in config['local_source_sha256'].items():
        data = subprocess.check_output(['git', 'show', config['git_head'] + ':' + path], cwd=REPO)
        assert hashlib.sha256(data).hexdigest() == digest, path
    sources = {'scoring-policy.json': policy_data} if policy_data is not None else {}
    lock_path = 'benchmarks/markdown-ecosystem/' + ('native/Cargo.lock' if config['track'] == 'native' else 'package-lock.json')
    lock_data = subprocess.check_output(['git', 'show', config['git_head'] + ':' + lock_path], cwd=REPO)
    assert hashlib.sha256(lock_data).hexdigest() == config['lock_sha256'], 'committed lock changed'
    if config.get('schema', 1) >= 2:
        assert config['rotating_controls'] == 1, 'rotating-document controls required'
        validate_rotation(config, corpus['cases'], outputs, read('rotating-controls'))
        for name, key in [('comparisons.json', 'manifest_sha256'), ('contracts.py', 'contracts_sha256')]:
            data = subprocess.check_output(['git', 'show', config['git_head'] + ':benchmarks/markdown-ecosystem/' + name], cwd=REPO)
            assert hashlib.sha256(data).hexdigest() == config[key], name
            sources[name] = data
        manifest = json.loads(sources['comparisons.json'])
        assert config['comparison'] == next(p for p in manifest if p['id'] == engine)
        assert config['comparison']['track'] == config['track']
        if engine == 'goldmark':
            assert config['build_metadata']['go'] == config['runtime']
            assert config['build_metadata']['binaries']['goldmark']['version'] == '2.1.6'
    files = {'run.py': ('benchmarks/markdown-ecosystem/run.py', 'runner_sha256'),
             'verify.py': ('benchmarks/native-comparison/verify.py', 'verifier_sha256'),
             'native-run.py': ('benchmarks/native-comparison/run.py', 'guards_sha256')}
    if config['track'] == 'node':
        files.update({'worker.mjs': ('benchmarks/markdown-ecosystem/worker.mjs', 'worker_sha256'),
                      'node-adapters.mjs': ('benchmarks/markdown-ecosystem/node-adapters.mjs', 'node_adapters_sha256')})
        for name, key in [('benchmark-facade.mjs', 'benchmark_facade_sha256'), ('benchmark-target.mjs', 'benchmark_target_sha256')]:
            if key in config:
                files[name] = ('benchmarks/markdown-ecosystem/' + name, key)
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


def validate_scores(config, summary, outputs, rows, matched):
    scored = set(outputs) if config.get('schema') == 3 else matched
    if config.get('schema') == 3:
        assert config['scoring_scope'] == 'all-documents'
        assert summary['scored_documents'] == len(scored) == 57
    for mode in run.native.MODES:
        assert math.isclose(run.aggregate(rows, scored, mode, config['engines'][1]), summary['v2_relative_throughput'][mode], rel_tol=1e-12)


def validate_rotation(config, cases, outputs, rows):
    engine = config['engines'][1]
    groups = run.contracts.groups(cases)
    expected = {(r, s, profile, mode) for r in range(3) for s in range(6)
                for profile in groups for mode in run.native.MODES}
    assert len(rows) == len(expected), 'incomplete rotating controls'
    assert {(row['round'], row['sample'], row['profile'], row['mode']) for row in rows} == expected
    for row in rows:
        members = [case['name'] for case in groups[row['profile']]]
        assert row['members'] == members and set(row['order']) == {'v2', engine}
        for name in ('v2', engine):
            timing = row[name]
            units = sum(run.output_units(outputs[member]['outputs'][name], config['track']) for member in members)
            assert timing['iterations'] > 0 and timing['elapsed_ns'] >= 40_000_000
            assert timing['checksum'] == timing['iterations'] * units


def retain(source, destination, revision=None):
    config, sources = validate(source, revision)
    destination.mkdir(parents=True, exist_ok=False)
    for name in ('corpus', 'verification', 'samples', 'behavior', *(('rotating-controls',) if config.get('schema', 1) >= 2 else ())):
        path = source / (name + '.json')
        data = path.read_bytes() if path.exists() else gzip.decompress(Path(str(path) + '.gz').read_bytes())
        (destination / (name + '.json.gz')).write_bytes(gzip.compress(data, mtime=0))
    for name in ('run.json', 'summary.json', 'Cargo.lock' if config['track'] == 'native' else 'package-lock.json', 'host.txt', 'build.log'):
        if (source / name).exists():
            shutil.copyfile(source / name, destination / name)
    snapshot = destination / 'adapters'
    snapshot.mkdir()
    for name, data in sources.items():
        (snapshot / name).parent.mkdir(parents=True, exist_ok=True)
        (snapshot / name).write_bytes(data)


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('source', type=Path)
    p.add_argument('destination', type=Path)
    p.add_argument('--revision', help='full committed revision for a new measurement; default requires the historical clean core')
    args = p.parse_args()
    retain(args.source, args.destination, args.revision)
    print('Verified and retained 2,052 windows:', args.destination)


if __name__ == '__main__':
    main()
