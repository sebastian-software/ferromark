#!/usr/bin/env python3
"""Paired timing of the container-definition correction and unaffected controls."""
import argparse
import gzip
import importlib.util
import json
import random
import statistics
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SPEC = importlib.util.spec_from_file_location('paired', ROOT / 'benchmarks/optimization-rounds/run.py')
paired = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(paired)


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument('before', type=Path)
    ap.add_argument('after', type=Path)
    ap.add_argument('output', type=Path)
    ap.add_argument('--pairs', type=int, default=7)
    ap.add_argument('--window-ms', type=int, default=50)
    args = ap.parse_args()
    out = args.output.resolve()
    out.mkdir(parents=True, exist_ok=False)
    builds = [json.loads((p / 'build.json').read_text()) for p in [args.before, args.after]]
    for build in builds:
        assert paired.digest(Path(build['binary'])) == build['binary_sha256']
    (out / 'builds.json').write_text(json.dumps(builds, indent=2))
    (out / 'run.json').write_text(json.dumps({'pairs': args.pairs, 'window_ms': args.window_ms, 'warmup_ms': 10, 'shuffle_seed': 20260915}, indent=2))
    (out / 'host.json').write_text(json.dumps(paired.host(), indent=2))
    config = out / 'options.json'
    config.write_text(json.dumps({'parser': {}, 'renderer': {}}))
    cases = []
    snippets = {
        'prose': 'Ordinary prose with clear words and another sentence.\n\n',
        'list-prose': '- A simple list item with **strong** text.\n- Another item.\n\n',
        'root-definition': '[target]: /url\n\n[target] and ![image][target].\n\n',
        'nested-definition': '- [target]: /url\n\n[target] and ![image][target].\n\n',
        'code-decoy': '- ```\n  [target]: /wrong\n  ```\n\n',
    }
    for name, snippet in snippets.items():
        for size in [4096, 65536]:
            source = snippet * max(1, size // len(snippet))
            if name == 'code-decoy':
                source = '- ```\n' + '  ordinary code content\n' * (size // 24) + '  [target]: /wrong\n  ```\n\n[target]: /right\n\n[target]\n'
            cases.append((f'{name}-{size}', source, name not in ['nested-definition', 'code-decoy']))
    cases.append(('sparse-container-65536', snippets['prose'] * 1200 + '\n- [target]: /url\n\n[target]\n', False))
    for name in ['UPSTREAM.md', 'CONTRIBUTING.md', 'docs/front-matter.md', 'docs/table-layout.md']:
        cases.append((name.replace('/', '_'), (ROOT / name).read_text(), True))
    jobs = [(name, source, same, stage) for name, source, same in cases for stage in ['parse', 'reuse', 'fresh']]
    random.Random(20260915).shuffle(jobs)
    inputs = out / 'inputs'
    inputs.mkdir()
    rows = []
    for name, source, same, stage in jobs:
        path = inputs / f'{name}.md'
        path.write_text(source)
        workers = [paired.Worker(b['binary'], str(config), stage, [path]) for b in builds]
        try:
            verification = [w.verify()[0] for w in workers]
            same_html = verification[0]['html'] == verification[1]['html']
            same_ast = verification[0]['ast_debug'] == verification[1]['ast_debug']
            if same: assert same_html and same_ast, (name, stage, 'control output changed')
            else: assert not same_html, (name, stage, 'expected semantic correction missing')
            for w in workers: w.bench(10_000_000)
            samples = []
            for index in range(args.pairs):
                sample = [None, None]
                for side in ([0, 1] if index % 2 == 0 else [1, 0]):
                    metric = verification[side]['children'] if stage == 'parse' else len(verification[side]['html'].encode())
                    sample[side] = paired.checked_bench(workers[side], args.window_ms * 1_000_000, metric)
                samples.append(sample)
            for side, w in enumerate(workers): paired.expected_result(w.verify(), verification[side], name)
            ns = [[p['elapsed_ns'] / p['iterations'] for p in sample] for sample in samples]
            ratios = [r / l for l, r in ns]
            row = dict(case=name, stage=stage, bytes=len(source.encode()), same_html=same_html, same_ast=same_ast,
                       ratio=statistics.median(ratios), range=[min(ratios), max(ratios)],
                       before_ns=statistics.median(p[0] for p in ns), after_ns=statistics.median(p[1] for p in ns),
                       samples=samples, verification=verification)
            rows.append(row)
            print(f'{name}/{stage}: {row["ratio"]:.3f}x', flush=True)
        finally:
            for w in workers: w.close()
    (out / 'results.json.gz').write_bytes(gzip.compress(json.dumps(rows).encode()))
    summary = [{k:v for k,v in row.items() if k not in ['samples', 'verification']} for row in rows]
    (out / 'summary.json').write_text(json.dumps(summary, indent=2))
    lines = ['# Container reference timing', '', 'Generated by `benchmarks/container-references/run.py`. Positive changes mean slower. Each row is the median of alternating paired elapsed-time ratios.', '', '| Input | Lifecycle | Same HTML | Change |', '| --- | --- | --- | ---: |']
    for r in sorted(rows, key=lambda r:(r['case'], r['stage'])):
        lines.append(f'| {r["case"]} | {r["stage"]} | {r["same_html"]} | {(r["ratio"]-1)*100:+.2f}% |')
    (out / 'tables.md').write_text('\n'.join(lines)+'\n')
    (out / 'host-after.json').write_text(json.dumps(paired.host(), indent=2))

if __name__ == '__main__':
    main()
