#!/usr/bin/env python3
"""Verify and measure native OX/v2 stages or isolated diagnostic variants."""
import argparse
import gzip
import json
import os
from pathlib import Path
import random
import statistics
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / 'benchmarks/native-comparison'))
import run as native


class Worker(native.Worker):
    def __init__(self, binary, engine, profile, mode, paths, env=None):
        self.process = subprocess.Popen([str(binary), engine, profile, mode, *map(str, paths)],
            text=True, stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
            env=os.environ | (env or {}))

    def details(self):
        self.command('details')
        rows = []
        while (line := self.line()) != 'done':
            rows.append(json.loads(line))
        return rows


def save(path, value):
    raw = json.dumps(value, indent=2, ensure_ascii=False).encode()
    if path.suffix == '.gz':
        raw = gzip.compress(raw, mtime=0)
    path.write_bytes(raw)


def verify_stages(verification, same_ast):
    """Reject semantic drift before any candidate is timed."""
    for name, engines in verification.items():
        for label, stages in engines.items():
            assert len({v['html'] for v in stages.values()}) == 1, (name, label, 'stage HTML mismatch')
            assert len({v['ast_debug'] for v in stages.values() if 'ast_debug' in v}) <= 1, (name, label, 'stage AST mismatch')
        if same_ast:
            asts = []
            htmls = []
            for label in same_ast:
                stages = engines[label]
                ast = [v['ast_debug'] for v in stages.values() if 'ast_debug' in v]
                assert ast, (name, label, 'AST stage required')
                asts.append(ast[0])
                htmls.append(next(iter(stages.values()))['html'])
            assert len(set(asts)) == 1, (name, same_ast, 'candidate AST mismatch')
            assert len(set(htmls)) == 1, (name, same_ast, 'candidate HTML mismatch')


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('build', type=Path)
    p.add_argument('output', type=Path)
    p.add_argument('--corpus', type=Path, default=ROOT / 'docs/reports/2026-09-14-native-matched/corpus.json.gz')
    p.add_argument('--variants', type=Path, help='JSON mapping labels to build directories, engine names, and optional env')
    p.add_argument('--modes', nargs='+', default=['fresh', 'reuse', 'init', 'parse', 'render'])
    p.add_argument('--rounds', type=int, default=2)
    p.add_argument('--pairs', type=int, default=3)
    p.add_argument('--window-ms', type=int, default=30)
    p.add_argument('--all-cases', action='store_true')
    p.add_argument('--same-ast', nargs='+', default=[], help='Require identical HTML and full AST Debug across these variant labels before timing')
    args = p.parse_args()
    args.output.mkdir(parents=True, exist_ok=False)
    corpus = native.read_json(args.corpus)
    reference = native.read_json(ROOT / 'docs/reports/2026-09-14-native-matched/verification.json.gz')
    cases = [c for c in corpus['cases'] if args.all_cases or reference[c['name']]['strict_all_six']]
    variants = json.loads(args.variants.read_text()) if args.variants else {
        'ox': {'build': str(args.build), 'engine': 'ox-content'},
        'v2': {'build': str(args.build), 'engine': 'v2'},
    }
    for label, variant in variants.items():
        build = json.loads((Path(variant['build']) / 'build.json').read_text())
        assert native.sha(build['binary']) == build['binary_sha256']
        variant['metadata'] = build
    save(args.output / 'variants.json', variants)
    save(args.output / 'corpus.json.gz', corpus | {'cases': cases})
    paths = {}
    inputs = args.output / 'inputs'
    inputs.mkdir()
    for case in cases:
        path = inputs / (case['name'] + '.md')
        path.write_bytes(case['input'].encode())
        assert native.sha(path) == case['sha256']
        paths[case['name']] = path

    def worker(label, case, mode):
        v = variants[label]
        return Worker(v['metadata']['binary'], v['engine'], case['profile'], mode,
                      [paths[case['name']]], v.get('env'))

    verification = {}
    for case in cases:
        verification[case['name']] = {}
        for label in variants:
            verification[case['name']][label] = {}
            for mode in args.modes:
                w = worker(label, case, mode)
                try:
                    html = w.verify()[0]
                    expected = reference.get(case['name'], {}).get('outputs', {}).get('v2')
                    if not args.all_cases:
                        assert html == expected, (case['name'], label, mode, 'HTML mismatch')
                    detail = w.details()[0] if mode in ('init', 'parse', 'render') else {'html': html}
                    assert detail['html'] == html
                    detail['metric'] = detail.get('metric', len(html.encode()))
                    verification[case['name']][label][mode] = detail
                finally:
                    w.close()
    verify_stages(verification, args.same_ast)
    save(args.output / 'verification.json.gz', verification)
    print(f'Verified {len(cases)} cases × {len(variants)} variants × {len(args.modes)} stages.', flush=True)
    config = {'rounds': args.rounds, 'pairs': args.pairs, 'window_ms': args.window_ms,
              'modes': args.modes, 'same_ast': args.same_ast, 'seed': 20260914, 'host_before': native.host(),
              'runner_sha256': native.sha(__file__)}
    rows = []
    rng = random.Random(config['seed'])
    jobs = [(c, mode) for c in cases for mode in args.modes]
    labels = list(variants)
    for rnd in range(args.rounds):
        rng.shuffle(jobs)
        for case, mode in jobs:
            workers = {label: worker(label, case, mode) for label in labels}
            expected = verification[case['name']]
            try:
                for label, w in workers.items():
                    assert w.verify() == [expected[label][mode]['html']]
                    w.bench(10_000_000, expected[label][mode]['metric'])
                for pair in range(args.pairs):
                    order = labels if (rnd + pair) % 2 == 0 else labels[::-1]
                    row = {'case': case['name'], 'mode': mode, 'round': rnd, 'pair': pair, 'order': order}
                    for label in order:
                        row[label] = workers[label].bench(args.window_ms * 1_000_000, expected[label][mode]['metric'])
                    rows.append(row)
                for label, w in workers.items():
                    assert w.verify() == [expected[label][mode]['html']]
            finally:
                for w in workers.values():
                    w.close()
        save(args.output / 'samples.json.gz', rows)
        print(f'Round {rnd + 1}/{args.rounds} complete.', flush=True)
    config['host_after'] = native.host()
    save(args.output / 'run.json', config)
    summary = []
    for case in cases:
        for mode in args.modes:
            subset = [r for r in rows if r['case'] == case['name'] and r['mode'] == mode]
            times = {label: [r[label]['elapsed_ns'] / r[label]['iterations'] for r in subset] for label in labels}
            summary.append({'case': case['name'], 'mode': mode,
                'engines': {label: {'ns': statistics.median(times[label]),
                    'round_medians_ns': [statistics.median(r[label]['elapsed_ns'] / r[label]['iterations'] for r in subset if r['round'] == rnd) for rnd in range(args.rounds)]} for label in labels}})
    save(args.output / 'summary.json', summary)
    print(args.output / 'summary.json')


if __name__ == '__main__':
    main()
