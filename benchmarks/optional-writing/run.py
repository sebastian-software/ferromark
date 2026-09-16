#!/usr/bin/env python3
"""Paired baseline, disabled-option, and active-syntax measurements."""
import argparse
import gzip
import hashlib
import importlib.util
import json
import random
import re
import statistics
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location('paired', ROOT / 'benchmarks/optimization-rounds/run.py')
paired = importlib.util.module_from_spec(spec)
spec.loader.exec_module(paired)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('baseline', type=Path)
    parser.add_argument('candidate', type=Path)
    parser.add_argument('output', type=Path)
    parser.add_argument('--pairs', type=int, default=7)
    parser.add_argument('--window-ms', type=int, default=50)
    parser.add_argument('--filter', default='', help='Regular expression over case/stage/comparison')
    parser.add_argument('--profile', choices=['gfm', 'commonmark'], default='gfm')
    args = parser.parse_args()
    out = args.output.resolve()
    out.mkdir(parents=True, exist_ok=False)
    builds = {name: json.loads((path / 'build.json').read_text()) for name, path in [('baseline', args.baseline), ('candidate', args.candidate)]}
    for build in builds.values():
        assert paired.digest(Path(build['binary'])) == build['binary_sha256']
    (out / 'run.json').write_text(json.dumps({k: str(v) if isinstance(v, Path) else v for k, v in vars(args).items()}, indent=2) + '\n')
    (out / 'builds.json').write_text(json.dumps(builds, indent=2) + '\n')
    (out / 'host.json').write_text(json.dumps(paired.host(), indent=2) + '\n')
    gfm = dict(tables=True, task_lists=True, strikethrough=True, autolinks=True, footnotes=True)
    if args.profile == 'commonmark':
        gfm = {}
    configs = {'off': {}, 'highlight': {'highlight': True}, 'notes': {'inline_footnotes': True}, 'both': {'highlight': True, 'inline_footnotes': True}, 'no-refs': {'allow_link_refs': False}}
    for name, config in configs.items():
        (out / f'{name}.json').write_text(json.dumps({'parser': gfm | config, 'renderer': {'semantic_footnotes': True}}))
    cases = []
    for name in ['UPSTREAM.md', 'CONTRIBUTING.md', 'docs/front-matter.md', 'docs/table-layout.md', 'benches/fixtures/upstream-changelog.md']:
        cases.append((name.replace('/', '_'), (ROOT / name).read_text(), 'document'))
    snippets = {
        'prose': 'The parser renders ordinary prose with clear sentences and useful words.\nA second line continues the paragraph without any special syntax.\n\n',
        'mixed': '# Heading\n\nSome **strong** and *soft* words, [inline](/url) and `code`.\n\n- Item one\n- Item two\n\n',
        'equals-caret': 'A = B and x^2 are literal expressions. No paired extension delimiters appear.\n\n',
        'highlight-active': 'Some ==marked **important** text== and ordinary text.\n\n',
        'notes-active': 'A statement^[An explanatory *note* with [link](/target).] follows.\n\n',
        'reference-active': '[label]: /target\n\nA [label] and an ![image][label].\n\n',
        'malformed': 'A == and ^[ are unclosed extension markers with ordinary text.\n\n',
    }
    for name, snippet in snippets.items():
        for size in [300, 4096, 65536]:
            cases.append((f'{name}-{size}', snippet * max(1, size // len(snippet)), name))
    for size in [300, 4096, 65536]:
        cases.append((f'sparse-active-{size}', snippets['prose'] * max(1, size // len(snippets['prose'])) + 'One ==mark== and one note^[An explanatory note.].\n', 'sparse-active'))
    inputs = out / 'inputs'
    inputs.mkdir()
    jobs = []
    for name, source, kind in cases:
        path = inputs / (name + '.md')
        path.write_text(source)
        for stage in ['parse', 'reuse', 'fresh']:
            jobs.append((name, kind, stage, 'baseline', 'off', 'candidate', 'off', True))
            if kind in ['prose', 'mixed', 'equals-caret', 'document', 'malformed']:
                for profile in ['highlight', 'notes', 'both', 'no-refs']:
                    # Real docs may contain definitions. Measure the policy change
                    # separately, without claiming equal-output acceleration.
                    equal = not (profile == 'no-refs' and kind == 'document')
                    jobs.append((name, kind, stage, 'candidate', 'off', 'candidate', profile, equal))
            elif kind == 'highlight-active':
                jobs.append((name, kind, stage, 'candidate', 'off', 'candidate', 'highlight', False))
            elif kind == 'notes-active':
                jobs.append((name, kind, stage, 'candidate', 'off', 'candidate', 'notes', False))
            elif kind == 'sparse-active':
                jobs.append((name, kind, stage, 'candidate', 'off', 'candidate', 'both', False))
            elif kind == 'reference-active':
                jobs.append((name, kind, stage, 'candidate', 'off', 'candidate', 'no-refs', False))
    random.Random(20260915).shuffle(jobs)
    rows = []
    for name, kind, stage, left_engine, left_profile, right_engine, right_profile, equal in jobs:
        comparison = 'baseline' if left_engine == 'baseline' else right_profile
        key = f'{name}/{stage}/{comparison}'
        if args.filter and not re.search(args.filter, key):
            continue
        workers = [paired.Worker(builds[engine]['binary'], str(out / f'{profile}.json'), stage, [inputs / (name + '.md')]) for engine, profile in [(left_engine, left_profile), (right_engine, right_profile)]]
        try:
            before = [w.verify()[0] for w in workers]
            actual_equal = before[0]['html'] == before[1]['html']
            ast_equal = before[0]['ast_debug'] == before[1]['ast_debug']
            if equal:
                assert actual_equal, (key, 'HTML changed')
            if comparison == 'baseline':
                assert ast_equal, (key, 'baseline AST changed')
            if kind.endswith('-active') and comparison != 'baseline':
                assert before[0]['html'] != before[1]['html'], (key, 'feature did not change HTML')
            for w in workers: w.bench(10_000_000)
            samples = []
            for pair in range(args.pairs):
                sample = [None, None]
                for side in ([0, 1] if pair % 2 == 0 else [1, 0]):
                    metric = before[side]['children'] if stage == 'parse' else len(before[side]['html'].encode())
                    sample[side] = paired.checked_bench(workers[side], args.window_ms * 1_000_000, metric)
                samples.append(sample)
            for side, worker in enumerate(workers):
                paired.expected_result(worker.verify(), before[side], key + '/post-timing')
            ns = [[v['elapsed_ns'] / v['iterations'] for v in pair] for pair in samples]
            ratios = [r / l for l, r in ns]
            row = dict(case=name, kind=kind, stage=stage, comparison=comparison, same_output=actual_equal, same_ast=ast_equal, bytes=(inputs / (name + '.md')).stat().st_size, ratio=statistics.median(ratios), ratio_range=[min(ratios), max(ratios)], left_ns=statistics.median(v[0] for v in ns), right_ns=statistics.median(v[1] for v in ns), samples=samples, verification=before)
            rows.append(row)
            (out / 'checkpoint.json').write_text(json.dumps([{k:v for k,v in r.items() if k not in ['samples', 'verification']} for r in rows]))
            print(f'{key}: {row["ratio"]:.3f}x', flush=True)
        finally:
            for w in workers: w.close()
    (out / 'results.json.gz').write_bytes(gzip.compress(json.dumps(rows).encode()))
    summary = [{k:v for k,v in row.items() if k not in ['samples', 'verification']} for row in rows]
    (out / 'summary.json').write_text(json.dumps(summary, indent=2) + '\n')
    (out / 'host-after.json').write_text(json.dumps(paired.host(), indent=2) + '\n')

if __name__ == '__main__':
    main()
