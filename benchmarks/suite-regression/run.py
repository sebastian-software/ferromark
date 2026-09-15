#!/usr/bin/env python3
"""Measure frozen before/after workers on the complete native comparison corpus."""
import argparse
import gzip
import importlib.util
import json
from pathlib import Path
import random
import statistics

ROOT = Path(__file__).resolve().parents[2]
SPEC = importlib.util.spec_from_file_location('paired', ROOT / 'benchmarks/optimization-rounds/run.py')
paired = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(paired)


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument('before', type=Path)
    ap.add_argument('after', type=Path)
    ap.add_argument('corpus', type=Path)
    ap.add_argument('output', type=Path)
    ap.add_argument('--batches-only', action='store_true', help='Verify every document and time only full-profile batches')
    ap.add_argument('--rounds', type=int, default=3)
    ap.add_argument('--pairs', type=int, default=5)
    ap.add_argument('--window-ms', type=int, default=40)
    args = ap.parse_args()
    assert min(args.rounds, args.pairs, args.window_ms) > 0
    out = args.output.resolve()
    out.mkdir(parents=True, exist_ok=False)
    builds = [json.loads((p / 'build.json').read_text()) for p in (args.before, args.after)]
    for build in builds:
        assert paired.digest(Path(build['binary'])) == build['binary_sha256']
    for key in ('worker_sha256', 'worker_lock_sha256', 'rustc', 'rustflags', 'lto'):
        assert builds[0][key] == builds[1][key], key
    corpus = paired.load_corpus(args.corpus)
    cases = corpus['cases']
    assert len({c['name'] for c in cases}) == len(cases)
    (out / 'corpus.json.gz').write_bytes(gzip.compress(json.dumps(corpus).encode()))
    (out / 'builds.json').write_text(json.dumps(builds, indent=2))
    (out / 'run.json').write_text(json.dumps(dict(rounds=args.rounds, pairs=args.pairs,
        window_ms=args.window_ms, warmup_ms=20, seed=20260915, cases=len(cases), batches_only=args.batches_only), indent=2))
    (out / 'host.json').write_text(json.dumps(paired.host(), indent=2))
    inputs = out / 'inputs'
    inputs.mkdir()
    configs = {}
    for profile in ('commonmark', 'gfm'):
        config = out / f'{profile}.json'
        config.write_text(json.dumps({'parser': {k: profile == 'gfm' for k in
            ('gfm', 'tables', 'task_lists', 'strikethrough')}, 'renderer': {}}))
        configs[profile] = str(config)
    for profile, options in corpus.get('profiles', {}).items():
        assert profile not in configs, ('cannot replace standard profile', profile)
        assert profile.replace('-', '').isalnum(), ('invalid profile name', profile)
        config = out / f'{profile}.json'
        config.write_text(json.dumps(options))
        configs[profile] = str(config)
    for c in cases:
        assert c['profile'] in configs
        path = inputs / (c['name'] + '.md')
        path.write_bytes(c['input'].encode())
        assert paired.digest(path) == c['sha256']
        assert path.stat().st_size == c['byte_count']
    jobs = [] if args.batches_only else [(c['name'], c['profile'], [c['name']]) for c in cases]
    for profile in configs:
        members = [c['name'] for c in cases if c['profile'] == profile]
        if members:
            jobs.append(('rotating-' + profile, profile, members))
    rows, verified = [], {}
    for round_index in range(args.rounds):
        shuffled = [(job, stage) for job in jobs for stage in ('fresh', 'reuse')]
        random.Random(20260915 + round_index).shuffle(shuffled)
        for (name, profile, members), stage in shuffled:
            workers = [paired.Worker(b['binary'], configs[profile], stage,
                [inputs / (m + '.md') for m in members]) for b in builds]
            try:
                verification = [w.verify() for w in workers]
                assert all(len(v) == len(members) for v in verification)
                for index, member in enumerate(members):
                    value = [{key: v[index][key] for key in ('html', 'ast_debug', 'children')} for v in verification]
                    if member in verified:
                        assert verified[member] == value, (member, 'lifecycle/state mismatch')
                    else:
                        verified[member] = value
                    assert value[0] == value[1], (member, 'output changed: review before timing')
                metrics = [sum(len(v['html'].encode()) for v in side) for side in verification]
                for side, worker in enumerate(workers):
                    paired.checked_bench(worker, 20_000_000, metrics[side])
                samples = []
                for pair in range(args.pairs):
                    sample = [None, None]
                    for side in ([0, 1] if (pair + round_index) % 2 == 0 else [1, 0]):
                        sample[side] = paired.checked_bench(workers[side], args.window_ms * 1_000_000, metrics[side])
                    samples.append(sample)
                for side, worker in enumerate(workers):
                    for index, value in enumerate(worker.verify()):
                        for key in ('html', 'ast_debug', 'children'):
                            assert value[key] == verification[side][index][key], (name, 'post-timing mutation')
                ns = [[s['elapsed_ns'] / s['iterations'] for s in sample] for sample in samples]
                row = dict(round=round_index + 1, case=name, stage=stage, members=members,
                    ratio=statistics.median(b / a for a, b in ns),
                    before_ns=statistics.median(a for a, b in ns), after_ns=statistics.median(b for a, b in ns), samples=samples)
                rows.append(row)
                print(f'round {round_index + 1} {name}/{stage}: {100 * (row["ratio"] - 1):+.2f}%', flush=True)
            finally:
                for worker in workers:
                    worker.close()
        (out / 'samples.json.gz').write_bytes(gzip.compress(json.dumps(rows).encode()))
    (out / 'verification.json.gz').write_bytes(gzip.compress(json.dumps(verified).encode()))
    summary = []
    for (name, profile, members) in jobs:
        for stage in ('fresh', 'reuse'):
            subset = [r for r in rows if r['case'] == name and r['stage'] == stage]
            summary.append(dict(case=name, stage=stage, members=members,
                ratio=statistics.median(r['ratio'] for r in subset), round_ratios=[r['ratio'] for r in subset],
                before_ns=statistics.median(r['before_ns'] for r in subset), after_ns=statistics.median(r['after_ns'] for r in subset)))
    for stage in ('fresh', 'reuse'):
        totals = []
        for round_index in range(1, args.rounds + 1):
            subset = [r for r in rows if r['round'] == round_index and r['stage'] == stage and r['case'].startswith('rotating-')]
            before, after = (sum(r[key] for r in subset) for key in ('before_ns', 'after_ns'))
            totals.append(dict(before_ns=before, after_ns=after, ratio=after / before))
        summary.append(dict(case='complete-suite', stage=stage, ratio=statistics.median(r['ratio'] for r in totals), rounds=totals))
    (out / 'summary.json').write_text(json.dumps(summary, indent=2))
    (out / 'host-after.json').write_text(json.dumps(paired.host(), indent=2))


if __name__ == '__main__':
    main()
