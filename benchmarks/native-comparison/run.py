#!/usr/bin/env python3
"""Measure six native HTML engines on frozen inputs; retain every output and window."""
import argparse
from collections import Counter
import gzip
import hashlib
import json
import os
from pathlib import Path
import platform
import random
import re
import shutil
import statistics
import struct
import subprocess
import time

from verify import classify, groups

HERE = Path(__file__).resolve().parent
ENGINES = ('v2', 'ox-content', 'v1', 'md4c', 'pulldown-cmark', 'bun')
MODES = ('fresh', 'reuse')


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def read_json(path):
    with (gzip.open(path, 'rt') if str(path).endswith('.gz') else open(path)) as f:
        return json.load(f)


def write_json(path, value):
    path.write_text(json.dumps(value, indent=2, ensure_ascii=False) + '\n')


class Worker:
    def __init__(self, binary, engine, profile, mode, paths):
        self.process = subprocess.Popen([str(binary), engine, profile, mode, *map(str, paths)],
            text=True, stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE)

    def command(self, value):
        self.process.stdin.write(value + '\n')
        self.process.stdin.flush()

    def line(self):
        line = self.process.stdout.readline()
        if not line:
            raise RuntimeError(self.process.stderr.read())
        return line.rstrip('\r\n')  # Preserve an empty HTML hex field.

    def verify(self):
        self.command('verify')
        result = []
        while (line := self.line()) != 'done':
            kind, index, html = line.split(' ', 2)
            assert kind == 'html' and int(index) == len(result)
            result.append(bytes.fromhex(html).decode('utf-8'))
        return result

    def bench(self, ns, output_bytes):
        self.command(f'bench {ns}')
        kind, iterations, elapsed, checksum = self.line().split()
        result = dict(iterations=int(iterations), elapsed_ns=int(elapsed), checksum=int(checksum))
        assert kind == 'timing' and result['iterations'] > 0 and result['elapsed_ns'] >= ns
        assert result['checksum'] == result['iterations'] * output_bytes % (1 << (8 * struct.calcsize('P')))
        return result

    def close(self):
        if self.process.poll() is None:
            self.command('quit')
        _, err = self.process.communicate(timeout=10)
        assert self.process.returncode == 0, err


def host():
    def probe(args):
        try:
            return subprocess.check_output(args, text=True, stderr=subprocess.STDOUT, timeout=5).strip()
        except (OSError, subprocess.SubprocessError) as error:
            return str(error)
    return dict(time_utc=time.strftime('%Y-%m-%dT%H:%M:%SZ', time.gmtime()),
        platform=platform.platform(), cpu=os.environ.get('BENCH_CPU', platform.machine()),
        load_average=os.getloadavg(), power=probe(['pmset', '-g', 'batt']), thermal=probe(['pmset', '-g', 'therm']))


def behavior_checks(binary, directory):
    """Exercise effective options in all six native adapters, including md4c's FFI bits."""
    fixtures = {
        'empty': '',
        'common': 'one **two** and `three` &amp; [four](/path)\n\n<div>raw</div>\n\n<https://example.com>\n',
        'extensions': '| A | B |\n| :--- | ---: |\n| x | y |\n\n- [x] Done\n- [ ] Pending\n\n~~old~~\n\n[[page]]\n\nhttps://example.com\n',
        'literal': '```txt\nx  y\n\n z\n```\n',
        'headings': '# Same\n\n# Same\n',
        'extras': '// visible\n\n[[toc]]\n\nTerm\n: meaning\n\n"quotes" -- ...\n\nH~2~O x^2^\n\n[link](target.md)\n',
        'callout': '> [!NOTE]\n> Ordinary quote\n',
        'fence-metadata': '```js{1}\nconst value = 1;\n```\n',
        'raw-tagfilter': '<div>\n<script>raw()</script>\n</div>\n',
        'footnotes': 'Text[^note]\n\n[^note]: ordinary definition\n',
        'frontmatter': '---\ntitle: Ordinary text\n---\n',
        'table-extras': '| A | B |\n| --- | --- |\n| x ||\n',
        'references': '[x][ref]\n\n[ref]: /first\n',
        'undefined': '[x][ref]\n',
    }
    paths = []
    for name, value in fixtures.items():
        path = directory / f'guard-{name}.md'
        path.write_text(value)
        paths.append(path)
    result = {}
    for profile in ('commonmark', 'gfm-shared'):
        result[profile] = {}
        for engine in ENGINES:
            outputs = {}
            for mode in MODES:
                worker = Worker(binary, engine, profile, mode, paths)
                try:
                    first = worker.verify()
                    assert worker.verify() == first, ('state leaked across input cycles', engine, mode)
                    outputs[mode] = dict(zip(fixtures, first))
                finally:
                    worker.close()
            assert outputs['fresh'] == outputs['reuse'], ('lifecycle mismatch', engine)
            out = outputs['fresh']
            assert out['empty'] == ''
            for token in ('<strong>two</strong>', '<code>three</code>', '<div>raw</div>', 'href="https://example.com"'):
                assert token in out['common'], (engine, profile, token)
            assert 'x  y\n\n z\n' in out['literal']
            assert '[[page]]' in out['extensions'] and '<a ' not in out['extensions']
            assert '[x][ref]' in out['undefined'], ('reference state leaked', engine)
            assert '<script>raw()</script>' in out['raw-tagfilter'], (engine, profile, 'tagfilter must be off')
            for token in ('// visible', 'Term', ': meaning', '-- ...', 'x^2^', 'href="target.md"'):
                assert token in out['extras'], (engine, profile, 'disabled extension', token)
            for token in ('<dl>', '<sup>', '<sub>', 'target="_blank"'):
                assert token not in out['extras'], (engine, profile, 'unexpected extension', token)
            assert 'footnote' not in out['footnotes'], (engine, profile, 'footnotes must be off')
            assert 'title: Ordinary text' in out['frontmatter'], (engine, profile, 'frontmatter must be off')
            assert 'colspan' not in out['table-extras'] and '<colgroup' not in out['table-extras']
            if engine != 'ox-content':
                assert ' id=' not in out['headings'], (engine, profile, 'heading IDs must be off')
                assert '[[toc]]' in out['extras'], (engine, profile, 'TOC marker must remain literal')
                assert '[!NOTE]' in out['callout'], (engine, profile, 'callouts must be off')
                assert 'language-js{1}' in out['fence-metadata'], (engine, profile, 'fence metadata must be off')
            else:
                # Original OX has no public switches for these three behaviors.
                # Verify and retain them; do not patch or normalize them away.
                assert 'id="same"' in out['headings'] and 'id="same-1"' in out['headings']
                assert '[[toc]]' not in out['extras']
                assert 'ox-callout' in out['callout']
                assert 'language-js"' in out['fence-metadata']
            extension = profile == 'gfm-shared'
            if not extension:
                assert 'H~2~O' in out['extras'], (engine, profile, 'strikethrough must be off')
            assert ('<table>' in out['extensions']) == extension, (engine, profile, 'tables')
            assert ('<del>old</del>' in out['extensions']) == extension, (engine, profile, 'strike')
            assert out['extensions'].count('type="checkbox"') == (2 if extension else 0), (engine, profile, 'tasks')
            result[profile][engine] = out
    return result


def case_filter(pattern, path):
    """Resolve the measured-set regex; --filter-file keeps the set an auditable input."""
    if pattern and path:
        raise SystemExit('pass only one of --filter and --filter-file')
    if path:
        pattern = Path(path).read_text().strip()
        if not pattern:
            raise SystemExit(f'empty case filter file: {path}')
    if pattern:
        try:
            re.compile(pattern)
        except re.error as error:
            raise SystemExit(f'not a valid case filter: {error}') from error
    return pattern or None


def select_cases(cases, pattern):
    if not pattern:
        return cases
    matcher = re.compile(pattern)
    chosen = [case for case in cases if matcher.search(case['name'])]
    if not chosen:
        raise SystemExit('case filter selected no cases')
    return chosen


def summaries(rows, jobs):
    result = []
    for job in jobs:
        for mode in MODES:
            samples = [r for r in rows if r['case'] == job['name'] and r['mode'] == mode]
            if not samples:
                continue
            engines = {}
            for engine in ENGINES:
                by_round = {}
                for row in samples:
                    by_round.setdefault(row['round'], []).append(row[engine]['elapsed_ns'] / row[engine]['iterations'])
                medians = [statistics.median(v) for _, v in sorted(by_round.items())]
                engines[engine] = {'ns': statistics.median(medians), 'round_medians_ns': medians,
                    'round_min_ns': min(medians), 'round_max_ns': max(medians)}
            result.append({'case': job['name'], 'mode': mode, 'members': job['members'], 'engines': engines})
    return result


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('build', type=Path, help='build.json emitted by prepare.py')
    p.add_argument('corpus', type=Path)
    p.add_argument('output', type=Path)
    p.add_argument('--verify-only', action='store_true')
    p.add_argument('--rounds', type=int, default=3)
    p.add_argument('--samples', type=int, default=6)
    p.add_argument('--window-ms', type=int, default=40)
    p.add_argument('--warmup-ms', type=int, default=60)
    p.add_argument('--filter', dest='case_filter', help='regex selecting the measured documents')
    p.add_argument('--filter-file', type=Path, help='file holding that regex, for example a held-out PGO split')
    args = p.parse_args()
    assert min(args.rounds, args.samples, args.window_ms, args.warmup_ms) > 0
    pattern = case_filter(args.case_filter, args.filter_file)
    args.output.mkdir(parents=True, exist_ok=False)
    build = read_json(args.build)
    binary = Path(build['binary'])
    assert sha(binary) == build['binary_sha256']
    assert sha(HERE / 'worker.rs') == build['adapter_sha256']['worker.rs']
    lock = Path(build['lockfile'])
    assert sha(lock) == build['lock_sha256']
    shutil.copyfile(lock, args.output / 'Cargo.lock')
    write_json(args.output / 'build.json', build)
    corpus = read_json(args.corpus)
    available = len(corpus['cases'])
    cases = select_cases(corpus['cases'], pattern)
    assert len({c['name'] for c in cases}) == len(cases)
    # Retain the measured selection, not the whole corpus, so report.py groups
    # and the archived corpus describe exactly what was timed.
    corpus['cases'] = cases
    corpus['case_filter'] = pattern
    corpus['corpus_case_count'] = available
    write_json(args.output / 'corpus.json', corpus)
    inputs = args.output / 'inputs'
    inputs.mkdir()
    for case in cases:
        value = case['input'].encode()
        assert len(value) == case['byte_count'] and hashlib.sha256(value).hexdigest() == case['sha256']
        (inputs / (case['name'] + '.md')).write_bytes(value)
    write_json(args.output / 'behavior.json', behavior_checks(binary, inputs))
    verification = {}
    for case in cases:
        outputs = {}
        for engine in ENGINES:
            by_mode = {}
            for mode in MODES:
                worker = Worker(binary, engine, case['profile'], mode, [inputs / (case['name'] + '.md')])
                try:
                    by_mode[mode] = worker.verify()[0]
                finally:
                    worker.close()
            assert by_mode['fresh'] == by_mode['reuse'], (case['name'], engine)
            outputs[engine] = by_mode['fresh']
        verification[case['name']] = {
            'outputs': outputs, 'agreement_groups': groups(outputs),
            'versus_v2': {e: classify(outputs['v2'], outputs[e]) for e in ENGINES},
            'strict_all_six': all(classify(outputs['v2'], outputs[e]) in ('exact', 'serialization-equivalent') for e in ENGINES),
        }
    write_json(args.output / 'verification.json', verification)
    print('All-six output agreement:', Counter(v['strict_all_six'] for v in verification.values()), flush=True)
    jobs = [dict(name=c['name'], profile=c['profile'], members=[c['name']]) for c in cases]
    for profile in ('commonmark', 'gfm'):
        members = [c['name'] for c in cases if c['profile'] == profile]
        if members:
            jobs.append(dict(name='rotating-' + profile, profile=profile, members=members))
    for job in jobs:
        if not job['name'].startswith('rotating-'):
            continue
        for engine in ENGINES:
            for mode in MODES:
                worker = Worker(binary, engine, job['profile'], mode, [inputs / (n + '.md') for n in job['members']])
                try:
                    expected = [verification[n]['outputs'][engine] for n in job['members']]
                    for _ in range(2):
                        assert worker.verify() == expected, ('rotating state leakage', job['name'], engine, mode)
                finally:
                    worker.close()
    if args.verify_only:
        return
    config = dict(rounds=args.rounds, samples=args.samples, window_ms=args.window_ms, warmup_ms=args.warmup_ms,
        seed=20260914, jobs=jobs, modes=MODES, engines=ENGINES, host_before=host(), observations=[],
        case_filter=pattern, case_filter_file=str(args.filter_file) if args.filter_file else None,
        cases=[c['name'] for c in cases], corpus_case_count=available,
        runner_sha256=sha(__file__), verifier_sha256=sha(HERE / 'verify.py'), corpus_sha256=sha(args.corpus),
        aggregation='Per-engine median of per-process round medians; equal-document geometric mean of time ratios.',
        lifecycle='Fresh: new parser and owned HTML. Reuse: retained arenas/scratch/output where public API permits; Bun still uses its fresh owned-output API.')
    write_json(args.output / 'run.json', config)
    rows = []
    rng = random.Random(config['seed'])
    for round_index in range(args.rounds):
        order_jobs = [(j, m) for j in jobs for m in MODES]
        rng.shuffle(order_jobs)
        observation = dict(round=round_index, before=host())
        config['observations'].append(observation)
        print(f'Round {round_index+1}/{args.rounds}: {len(order_jobs)} documents/batches and lifecycles', flush=True)
        for index, (job, mode) in enumerate(order_jobs):
            workers = {e: Worker(binary, e, job['profile'], mode, [inputs / (n + '.md') for n in job['members']]) for e in ENGINES}
            expected = {e: [verification[n]['outputs'][e] for n in job['members']] for e in ENGINES}
            lengths = {e: sum(len(h.encode()) for h in values) for e, values in expected.items()}
            try:
                for engine in ENGINES:
                    assert workers[engine].verify() == expected[engine]
                    workers[engine].bench(args.warmup_ms * 1_000_000, lengths[engine])
                for sample in range(args.samples):
                    offset = (sample + round_index + index) % len(ENGINES)
                    engine_order = ENGINES[offset:] + ENGINES[:offset]
                    if round_index % 2:
                        engine_order = tuple(reversed(engine_order))
                    row = dict(round=round_index, sample=sample, case=job['name'], mode=mode, order=engine_order)
                    for engine in engine_order:
                        row[engine] = workers[engine].bench(args.window_ms * 1_000_000, lengths[engine])
                    rows.append(row)
                for engine in ENGINES:
                    assert workers[engine].verify() == expected[engine], ('post-timing output changed', engine, job['name'])
            finally:
                for worker in workers.values():
                    worker.close()
            if (index + 1) % 20 == 0:
                print(f'  {index+1}/{len(order_jobs)} complete', flush=True)
                write_json(args.output / 'samples.json', rows)
        observation['after'] = host()
        write_json(args.output / 'samples.json', rows)
        write_json(args.output / 'run.json', config)
    config['host_after'] = host()
    write_json(args.output / 'run.json', config)
    write_json(args.output / 'summary.json', summaries(rows, jobs))
    print(args.output / 'summary.json', flush=True)


if __name__ == '__main__':
    main()
