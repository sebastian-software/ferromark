#!/usr/bin/env python3
"""Verify each configuration's output, then time interleaved runtime pairs."""

import argparse
import csv
import gzip
import hashlib
import importlib.util
import json
from pathlib import Path
import random
import re
import statistics

HERE = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location('optimization_runner', HERE.parent / 'optimization-rounds/run.py')
shared = importlib.util.module_from_spec(spec)
spec.loader.exec_module(shared)
SIDES = ('off', 'on')
MODES = ('parse', 'render', 'fresh', 'reuse')


def save_gzip(path, data):
    path.write_bytes(gzip.compress(json.dumps(data, ensure_ascii=False).encode(), mtime=0))


def check_effect(case, values):
    html_equal = values['off']['html'] == values['on']['html']
    ast_equal = values['off']['ast_debug'] == values['on']['ast_debug']
    effect = case['expected_effect']
    if effect == 'html' and html_equal:
        raise AssertionError(f"{case['name']}: active probe did not change HTML")
    if effect == 'none' and (not html_equal or not ast_equal):
        raise AssertionError(f"{case['name']}: expected exact HTML and AST equality")
    return {'html_equal': html_equal, 'ast_equal': ast_equal}


def matches(actual, expected):
    return all(actual[key] == expected[key] for key in ('html', 'ast_debug', 'children'))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('build', type=Path)
    parser.add_argument('corpus', type=Path)
    parser.add_argument('output', type=Path)
    parser.add_argument('--filter', default='.*')
    parser.add_argument('--rounds', type=int, default=2)
    parser.add_argument('--pairs', type=int, default=3)
    parser.add_argument('--window-ms', type=int, default=20)
    parser.add_argument('--verify-only', action='store_true')
    args = parser.parse_args()
    if min(args.rounds, args.pairs, args.window_ms) < 1:
        parser.error('all measurement dimensions must be positive')
    args.output.mkdir(parents=True, exist_ok=False)
    build = json.loads((args.build / 'build.json').read_text())
    binary = Path(build['binary'])
    assert shared.digest(binary) == build['binary_sha256'], 'binary changed'
    assert shared.digest(args.build / 'worker/src/main.rs') == build['worker_sha256'], 'worker changed'
    corpus = json.loads(args.corpus.read_text())
    cases = [case for case in corpus['cases'] if re.search(args.filter, case['name'])]
    assert cases, 'no cases selected'
    save_gzip(args.output / 'corpus.json.gz', corpus | {'cases': cases})
    (args.output / 'build.json').write_text(json.dumps(build, indent=2) + '\n')
    inputs = args.output / 'inputs'
    inputs.mkdir()
    configs = args.output / 'configs'
    configs.mkdir()
    paths = {}
    config_paths = {}
    for case in cases:
        source = case['input'].encode()
        assert len(source) == case['byte_count']
        assert hashlib.sha256(source).hexdigest() == case['sha256']
        paths[case['name']] = inputs / (case['name'] + '.md')
        paths[case['name']].write_bytes(source)
        for side in SIDES:
            serialized = json.dumps(case[side], sort_keys=True)
            key = hashlib.sha256(serialized.encode()).hexdigest()
            path = configs / (key + '.json')
            path.write_text(serialized)
            config_paths[case['name'], side] = path

    def worker(case, mode, side):
        return shared.Worker(binary, str(config_paths[case['name'], side]), mode, [paths[case['name']]])

    # All semantic gates precede all measurements. Feature-off outputs are
    # deliberately not treated as an oracle for feature-on semantics.
    verification = {}
    for i, case in enumerate(cases, 1):
        values = {}
        capacities = {}
        for mode in MODES:
            capacities[mode] = {}
            for side in SIDES:
                w = worker(case, mode, side)
                try:
                    value = w.verify()[0]
                finally:
                    w.close()
                capacities[mode][side] = value['arena_capacity_bytes']
                if side in values:
                    assert matches(value, values[side]), (case['name'], side, 'lifecycle mismatch')
                else:
                    values[side] = value
        verification[case['name']] = {'values': values, 'capacities': capacities,
                                     **check_effect(case, values)}
        if i % 25 == 0:
            print(f'Verified {i}/{len(cases)}', flush=True)
    save_gzip(args.output / 'verification.json.gz', verification)
    print(f'All {len(cases)} cases verified across four lifecycles.', flush=True)
    if args.verify_only:
        return

    run = {'rounds': args.rounds, 'pairs': args.pairs, 'window_ms': args.window_ms,
           'warmup_ms': 5, 'seed': 20260914, 'host_before': shared.host(),
           'runner_sha256': shared.digest(Path(__file__)),
           'shared_runner_sha256': shared.digest(HERE.parent / 'optimization-rounds/run.py'),
           'case_generator_sha256': shared.digest(HERE / 'make_cases.py'),
           'corpus_sha256': shared.digest(args.output / 'corpus.json.gz'), 'filter': args.filter}
    rows = []
    jobs = [(case, mode) for case in cases for mode in MODES
            # Unchanged stages are measured on the 300-byte probe as controls;
            # larger toggles only measure stages they can affect.
            if case['group'] != 'feature' or case['target_bytes'] == 300
            or not ((case['feature'].startswith('renderer.') and mode == 'parse'))]
    rng = random.Random(run['seed'])
    for round_index in range(args.rounds):
        rng.shuffle(jobs)
        for i, (case, mode) in enumerate(jobs, 1):
            workers = {side: worker(case, mode, side) for side in SIDES}
            expected = verification[case['name']]['values']
            metric = {side: expected[side]['children'] if mode == 'parse'
                      else len(expected[side]['html'].encode()) for side in SIDES}
            try:
                for side in SIDES:
                    assert matches(workers[side].verify()[0], expected[side])
                    shared.checked_bench(workers[side], run['warmup_ms'] * 1_000_000, metric[side])
                for pair in range(args.pairs):
                    order = SIDES if (pair + round_index) % 2 == 0 else SIDES[::-1]
                    row = {'case': case['name'], 'mode': mode, 'round': round_index,
                           'pair': pair, 'order': list(order)}
                    for side in order:
                        row[side] = shared.checked_bench(workers[side], args.window_ms * 1_000_000, metric[side])
                    rows.append(row)
                for side in SIDES:
                    assert matches(workers[side].verify()[0], expected[side]), 'post-timing drift'
            finally:
                for w in workers.values():
                    w.close()
            if i % 50 == 0:
                print(f'Round {round_index + 1}: {i}/{len(jobs)} jobs', flush=True)
        save_gzip(args.output / 'samples.json.gz', rows)
    run['host_after'] = shared.host()
    (args.output / 'run.json').write_text(json.dumps(run, indent=2) + '\n')
    summary = summarize(cases, rows, verification)
    (args.output / 'summary.json').write_text(json.dumps(summary, indent=2) + '\n')
    with (args.output / 'summary.csv').open('w') as f:
        writer = csv.DictWriter(f, fieldnames=summary[0].keys())
        writer.writeheader()
        writer.writerows(summary)
    print(args.output / 'summary.json', flush=True)


def summarize(cases, rows, verification):
    result = []
    for case in cases:
        for mode in MODES:
            subset = [r for r in rows if r['case'] == case['name'] and r['mode'] == mode]
            if not subset:
                continue
            times = {side: [r[side]['elapsed_ns'] / r[side]['iterations'] for r in subset] for side in SIDES}
            ratios = [on / off for off, on in zip(times['off'], times['on'])]
            v = verification[case['name']]
            result.append({key: case.get(key) for key in
                           ('name', 'group', 'feature', 'workload', 'document', 'target_bytes', 'byte_count')} |
                          {'mode': mode, 'off_ns': statistics.median(times['off']),
                           'on_ns': statistics.median(times['on']),
                           'on_over_off': statistics.median(ratios), 'ratio_min': min(ratios),
                           'ratio_max': max(ratios), 'pairs': len(ratios),
                           'round_ratios': [statistics.median(ratios[i] for i, r in enumerate(subset)
                                                             if r['round'] == rnd)
                                            for rnd in sorted({r['round'] for r in subset})],
                           'html_equal': v['html_equal'], 'ast_equal': v['ast_equal'],
                           'off_html_bytes': len(v['values']['off']['html'].encode()),
                           'on_html_bytes': len(v['values']['on']['html'].encode())})
    return result


if __name__ == '__main__':
    main()
