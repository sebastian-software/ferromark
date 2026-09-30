#!/usr/bin/env python3
"""Measure independent Markdown-to-HTML pairs on the frozen corpus."""
import argparse
from collections import Counter
import importlib.util
import json
import math
from pathlib import Path
import random
import shutil
import statistics
import subprocess
import sys

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[1]
sys.path.insert(0, str(HERE.parent / 'native-comparison'))
spec = importlib.util.spec_from_file_location('native_run', HERE.parent / 'native-comparison/run.py')
native = importlib.util.module_from_spec(spec)
spec.loader.exec_module(native)


def output_units(html, track):
    return len(html.encode('utf-16-le')) // 2 if track == 'node' else len(html.encode())


def aggregate(rows, members, mode, competitor):
    ratios = []
    for name in sorted(members):
        times = {}
        for engine in ('v2', competitor):
            rounds = {}
            for row in rows:
                if row['case'] == name and row['mode'] == mode:
                    rounds.setdefault(row['round'], []).append(row[engine]['elapsed_ns'] / row[engine]['iterations'])
            times[engine] = statistics.median(statistics.median(values) for values in rounds.values())
        ratios.append(times[competitor] / times['v2'])
    return math.exp(statistics.mean(map(math.log, ratios))) if ratios else None


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('track', choices=('native', 'node'))
    p.add_argument('output', type=Path)
    p.add_argument('--competitor', choices=('markdown-rs', 'comrak', 'cmark', 'cmark-gfm', 'micromark', 'marked', 'markdown-it', 'remark', 'showdown', 'commonmark'))
    p.add_argument('--corpus', type=Path, default=REPO / 'docs/reports/2026-09-14-optimization-rounds/broad-corpus.json.gz')
    p.add_argument('--binary', type=Path, default=HERE / 'native/target/release/markdown-ecosystem-worker')
    p.add_argument('--build-metadata', type=Path)
    p.add_argument('--rounds', type=int, default=3)
    p.add_argument('--samples', type=int, default=6)
    p.add_argument('--window-ms', type=int, default=40)
    p.add_argument('--warmup-ms', type=int, default=60)
    p.add_argument('--verify-only', action='store_true')
    args = p.parse_args()
    if min(args.rounds, args.samples, args.window_ms, args.warmup_ms) <= 0:
        p.error('rounds, samples, and timing windows must be positive')
    competitor = args.competitor or ('markdown-rs' if args.track == 'native' else 'micromark')
    native_engines = ('markdown-rs', 'comrak', 'cmark', 'cmark-gfm')
    if (competitor in native_engines) != (args.track == 'native'):
        p.error('competitor does not belong to the selected track')
    engines = ('v2', competitor)
    command = args.binary.resolve() if args.track == 'native' else HERE / 'worker.mjs'
    args.output.mkdir(parents=True, exist_ok=False)
    inputs = args.output / 'inputs'
    inputs.mkdir()
    corpus = native.read_json(args.corpus)
    cases = corpus['cases']
    if competitor in ('commonmark', 'cmark'):
        # commonmark.js has no GFM extensions: keep every input, disable extras in both engines.
        for case in cases:
            case['profile'] = 'commonmark'
    native.write_json(args.output / 'corpus.json', corpus)
    for case in cases:
        path = inputs / (case['name'] + '.md')
        path.write_bytes(case['input'].encode())
        assert path.stat().st_size == case['byte_count'] and native.sha(path) == case['sha256']
    # Same executable option guards as the six-engine harness.
    native.write_json(args.output / 'behavior.json', native.behavior_checks(command, inputs, engines, ('commonmark',) if competitor in ('commonmark', 'cmark') else ('commonmark', 'gfm-shared')))
    verification = {}
    for case in cases:
        outputs = {}
        for engine in engines:
            modes = []
            for mode in native.MODES:
                worker = native.Worker(command, engine, case['profile'], mode, [inputs / (case['name'] + '.md')])
                try:
                    first = worker.verify()
                    assert worker.verify() == first, ('state leaked', engine, case['name'])
                    modes.append(first[0])
                finally:
                    worker.close()
            assert modes[0] == modes[1], ('lifecycle mismatch', engine, case['name'])
            outputs[engine] = modes[0]
        verification[case['name']] = {'outputs': outputs, 'agreement': native.classify(outputs['v2'], outputs[competitor])}
    native.write_json(args.output / 'verification.json', verification)
    print(args.track, Counter(v['agreement'] for v in verification.values()), flush=True)
    matched = {name for name, value in verification.items() if value['agreement'] in ('exact', 'serialization-equivalent')}
    lock = HERE / ('native/Cargo.lock' if args.track == 'native' else 'package-lock.json')
    shutil.copyfile(lock, args.output / lock.name)
    source_paths = list((REPO / 'src').rglob('*.rs')) + list((REPO / 'node/native').rglob('*.rs'))
    source_paths += list((REPO / 'transforms').rglob('*.rs'))
    source_paths += [REPO / path for path in ('Cargo.toml', 'Cargo.lock', 'node/native/Cargo.toml', 'transforms/Cargo.toml', 'node/ferromark/index.mjs', 'node/ferromark/native-target.mjs')]
    config = {
        'track': args.track, 'engines': engines, 'rounds': args.rounds, 'samples': args.samples,
        'window_ms': args.window_ms, 'warmup_ms': args.warmup_ms, 'seed': 20260930,
        'host_before': native.host(), 'corpus_sha256': native.sha(args.corpus),
        'lock_sha256': native.sha(lock), 'worker_sha256': native.sha(command),
        'adapter_sha256': native.sha(HERE / ('worker.rs' if args.track == 'native' else 'worker.mjs')),
        'node_adapters_sha256': native.sha(HERE / 'node-adapters.mjs') if args.track == 'node' else None,
        'benchmark_facade_sha256': native.sha(HERE / 'benchmark-facade.mjs') if args.track == 'node' else None,
        'benchmark_target_sha256': native.sha(HERE / 'benchmark-target.mjs') if args.track == 'node' else None,
        'profile_scope': 'CommonMark only on all inputs' if competitor in ('commonmark', 'cmark') else 'CommonMark or tables/strikethrough/tasks, per frozen input profile',
        'runner_sha256': native.sha(__file__), 'verifier_sha256': native.sha(HERE.parent / 'native-comparison/verify.py'),
        'guards_sha256': native.sha(HERE.parent / 'native-comparison/run.py'),
        'git_head': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=REPO, text=True).strip(),
        'git_status': subprocess.check_output(['git', 'status', '--short'], cwd=REPO, text=True),
        'local_source_sha256': {str(path.relative_to(REPO)): native.sha(path) for path in sorted(source_paths)},
        'native_addon_sha256': {str(path.relative_to(REPO)): native.sha(path) for path in (REPO / 'node/ferromark').glob('*.node')} if args.track == 'node' else None,
        'native_build': 'RUSTFLAGS=-C target-cpu=generic; opt-level=3; fat LTO; codegen-units=1; panic=abort; system allocator' if args.track == 'native' else 'Cargo release-node; no PGO; system allocator; panic=unwind',
        'runtime': subprocess.check_output(['rustc' if args.track == 'native' else 'node', '--version'], text=True).strip(),
        'lifecycle': 'Competitors retain configured public parser/processor objects in both modes; each call parses and returns fresh owned HTML. Ferromark reuse retains arena/renderer storage.',
        'timing_boundary': 'Complete Markdown to owned HTML, consumed by output length; no I/O, startup, or normalization. Native: UTF-8 bytes, system allocator. Node: UTF-16 strings, public binding overhead and JS GC included.',
    }
    if args.build_metadata:
        config['build_metadata'] = json.loads(args.build_metadata.read_text())
        for item in config['build_metadata']['binaries'].values():
            assert native.sha(item['path']) == item['sha256'], 'build binary changed'
    native.write_json(args.output / 'run.json', config)
    if args.verify_only:
        return
    rows = []
    rng = random.Random(config['seed'])
    for round_index in range(args.rounds):
        jobs = [(case, mode) for case in cases for mode in native.MODES]
        rng.shuffle(jobs)
        print(f'Round {round_index + 1}/{args.rounds}: {len(jobs)} jobs', flush=True)
        for index, (case, mode) in enumerate(jobs):
            workers = {}
            try:
                for engine in engines:
                    workers[engine] = native.Worker(command, engine, case['profile'], mode, [inputs / (case['name'] + '.md')])
                    assert workers[engine].verify() == [verification[case['name']]['outputs'][engine]]
                lengths = {engine: output_units(verification[case['name']]['outputs'][engine], args.track) for engine in engines}
                for engine in engines:
                    workers[engine].bench(args.warmup_ms * 1_000_000, lengths[engine])
                for sample in range(args.samples):
                    order = engines if (sample + index + round_index) % 2 == 0 else engines[::-1]
                    row = dict(round=round_index, sample=sample, case=case['name'], mode=mode, order=order)
                    for engine in order:
                        row[engine] = workers[engine].bench(args.window_ms * 1_000_000, lengths[engine])
                    rows.append(row)
                for engine in engines:
                    assert workers[engine].verify() == [verification[case['name']]['outputs'][engine]]
            finally:
                for worker in workers.values():
                    worker.close()
        native.write_json(args.output / 'samples.json', rows)
    config['host_after'] = native.host()
    native.write_json(args.output / 'run.json', config)
    result = {'documents': len(cases), 'agreeing_documents': len(matched), 'agreement_counts': dict(Counter(v['agreement'] for v in verification.values())),
              'v2_relative_throughput': {mode: aggregate(rows, matched, mode, competitor) for mode in native.MODES}}
    native.write_json(args.output / 'summary.json', result)
    print(json.dumps(result, indent=2), flush=True)


if __name__ == '__main__':
    main()
