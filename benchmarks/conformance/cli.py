#!/usr/bin/env python3
"""Measure frozen CommonMark and GFM extension output agreement; never time calls."""
import argparse
from collections import Counter
from datetime import datetime, timezone
import hashlib
from html.parser import HTMLParser
import importlib.util
import json
from pathlib import Path
import platform
import shutil
import subprocess
import sys
import tempfile
import tomllib

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[1]
REPORT = 'docs/reports/2026-10-10-comparison-conformance'
DATA = REPO / 'homepage/app/data/comparison-conformance.json'
SUITES = {'commonmark': ('CommonMark 0.31.2', 652), 'gfm': ('GFM extensions', 28)}


def module(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    value = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(value)
    return value


audit = module('comparison_fixture_audit', HERE.parent / 'compatibility-audit/run.py')
verify = module('comparison_html_verify', HERE.parent / 'native-comparison/verify.py')


def read(path):
    return json.loads(path.read_text())


def write(path, value):
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2) + '\n')


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def tracked_inputs():
    paths = [*HERE.glob('*.py'), *HERE.glob('*worker*'), HERE / 'profiles.json',
             HERE.parent / 'markdown-ecosystem/conformance-adapters.mjs',
             HERE.parent / 'markdown-ecosystem/node-adapters.mjs',
             HERE.parent / 'markdown-ecosystem/benchmark-facade.mjs',
             HERE.parent / 'markdown-ecosystem/benchmark-target.mjs',
             HERE.parent / 'markdown-ecosystem/package.json',
             HERE.parent / 'markdown-ecosystem/package-lock.json',
             HERE.parent / 'markdown-ecosystem/native/Cargo.toml',
             HERE.parent / 'markdown-ecosystem/native/Cargo.lock',
             HERE.parent / 'markdown-ecosystem/prepare-native.py',
             HERE.parent / 'markdown-ecosystem/goldmark/go.mod',
             HERE.parent / 'markdown-ecosystem/goldmark/go.sum',
             HERE.parent / 'native-comparison/Cargo.lock',
             HERE.parent / 'native-comparison/prepare.py',
             HERE.parent / 'native-comparison/restore.py',
             HERE.parent / 'native-comparison/native.h',
             HERE.parent / 'native-comparison/stack.c',
             HERE.parent / 'native-comparison/md4c_alloc.h',
             HERE.parent / 'native-comparison/verify.py',
             HERE.parent / 'compatibility-audit/run.py',
             HERE.parent / 'compatibility-audit/fixtures/gfm-0.29-2026-09-14.html',
             REPO / 'tests/spec_fixtures/commonmark-0.31.2-spec.txt',
             REPO / 'homepage/app/data/benchmark-projects.json']
    return {str(p.relative_to(REPO)): sha(p) for p in sorted(set(paths))}


def fixtures():
    cm = audit.parse_txt((REPO / 'tests/spec_fixtures/commonmark-0.31.2-spec.txt').read_text())
    gfm = [e for e in audit.GFMExamples((HERE.parent / 'compatibility-audit/fixtures/gfm-0.29-2026-09-14.html').read_text()).examples if '(extension)' in e['section']]
    if (len(cm), len(gfm)) != (652, 28):
        raise ValueError('Frozen fixture counts changed')
    return {'commonmark': cm, 'gfm': gfm}


class UnsafeNormalization(HTMLParser):
    """Keep known limits of the campaign comparator from admitting differences."""
    def __init__(self, html):
        super().__init__(convert_charrefs=False)
        self.unsafe = False
        self.feed(html)
        self.close()

    def handle_starttag(self, tag, attrs):
        names = [key for key, _ in attrs]
        self.unsafe |= tag in ('svg', 'math') or len(names) != len(set(names))

    handle_startendtag = handle_starttag


def classify(expected, actual):
    if expected == actual:
        return 'exact'
    if UnsafeNormalization(expected).unsafe or UnsafeNormalization(actual).unsafe:
        return 'other'
    return verify.classify(expected, actual)


def summarize(cases):
    counts = Counter(case['status'] for case in cases)
    passed = counts['exact'] + counts['serialization-equivalent']
    return {'status': 'measured', 'passed': passed, 'total': len(cases),
            'failed': len(cases) - passed, 'errors': counts['error'],
            'percent': round(passed / len(cases) * 100, 2), 'classifications': dict(sorted(counts.items()))}


def node_results(engine, suite, examples):
    command = ['node', str(HERE / 'node-worker.mjs'), engine, suite]
    payload = ''.join(json.dumps({'markdown': e['markdown']}) + '\n' for e in examples)
    process = subprocess.run(command, input=payload, text=True, capture_output=True, timeout=120)
    if process.returncode == 2 and not process.stdout:
        return None, process.stderr
    if process.returncode != 0:
        raise ValueError(f'Node worker failed unexpectedly: {engine}: {process.stderr}')
    results = [json.loads(line) for line in process.stdout.splitlines()]
    if len(results) != len(examples):
        raise ValueError(f'Incomplete Node worker output: {engine}')
    return results, None


def parse_native(process, count):
    results = {}
    done = False
    for line in process.stdout.splitlines():
        if line == 'done':
            if done:
                raise ValueError('Duplicate completion')
            done = True
            continue
        prefix, index, html = line.split(' ', 2)
        index = int(index)
        if prefix != 'html' or index in results or not 0 <= index < count or done:
            raise ValueError('Invalid native protocol output')
        results[index] = {'html': bytes.fromhex(html).decode('utf-8'), 'error': None}
    if process.returncode == 0 and (not done or len(results) != count):
        raise ValueError('Native worker succeeded with incomplete output')
    return results


def native_results(command, paths):
    process = subprocess.run([*command, *map(str, paths)], input='verify\nquit\n', text=True, capture_output=True, timeout=120)
    results = parse_native(process, len(paths))
    # A crashing render must not hide the remaining cases. Re-run each missing
    # input in its own process and retain the exit code/stderr as a measured error.
    for index in range(len(paths)):
        if index in results:
            continue
        try:
            one = subprocess.run([*command, str(paths[index])], input='verify\nquit\n', text=True, capture_output=True, timeout=10)
            rendered = parse_native(one, 1)
            results[index] = rendered.get(0, {'html': None, 'error': f'exit {one.returncode}: {one.stderr}'})
        except subprocess.TimeoutExpired:
            results[index] = {'html': None, 'error': 'Render timeout after 10 seconds'}
    return [results[index] for index in range(len(paths))]


def measure(build, output):
    output.mkdir(parents=True, exist_ok=False)
    metadata = read(build / 'build.json')
    if metadata['inputs'] != tracked_inputs():
        raise ValueError('Prepared build inputs changed; rebuild before measuring')
    for binary in [*metadata['binaries'].values(), metadata['addon']]:
        if sha(Path(binary['path'])) != binary['sha256']:
            raise ValueError('Built adapter hash changed')
    profiles = read(HERE / 'profiles.json')
    for reference in ('ferromark-native', 'ferromark-node'):
        if profiles[reference]['version'] != metadata['ferromark_version']:
            raise ValueError('Reference profile version differs from built Ferromark')
    examples = fixtures()
    projects = read(REPO / 'homepage/app/data/benchmark-projects.json')
    projects = [{'id': 'ferromark-native', 'label': 'Ferromark', 'runtime': 'Native'},
                {'id': 'ferromark-node', 'label': 'Ferromark', 'runtime': 'Node.js'}, *projects]
    if set(profiles) != {p['id'] for p in projects}:
        raise ValueError('Profiles must cover exactly all displayed variants and references')
    rows = []
    for project in projects:
        engine = project['id']
        print('Measuring', engine, flush=True)
        suites, raw = {}, {}
        for suite, (_, total) in SUITES.items():
            if profiles[engine][suite] is None:
                suites[suite] = {'status': 'unsupported', 'reason': profiles[engine]['notes']}
                raw[suite] = []
                continue
            if project['runtime'] == 'Node.js':
                results, error = node_results(engine, suite, examples[suite])
                if results is None:
                    suites[suite] = {'status': 'unmeasured', 'reason': error}
                    raw[suite] = []
                    continue
            else:
                group = 'native' if engine in ('ferromark-native', 'pulldown-cmark', 'md4c', 'bun', 'ox-content') else ('rust' if engine in ('markdown-rs', 'comrak') else engine)
                name = 'v2' if engine == 'ferromark-native' else engine
                with tempfile.TemporaryDirectory(prefix='ferromark-spec-') as directory:
                    paths = []
                    for i, example in enumerate(examples[suite]):
                        path = Path(directory) / f'{i}.md'
                        path.write_bytes(example['markdown'].encode())
                        paths.append(path)
                    results = native_results([metadata['binaries'][group]['path'], name, suite, 'fresh'], paths)
            cases = []
            for example, result in zip(examples[suite], results, strict=True):
                if (result['error'] is None) != isinstance(result['html'], str):
                    raise ValueError('Invalid render result')
                status = 'error' if result['error'] else classify(example['html'], result['html'])
                cases.append({**example, 'actual': result['html'], 'error': result['error'], 'status': status})
            raw[suite] = cases
            suites[suite] = summarize(cases)
            assert len(cases) == total
        # One compact JSON line per example keeps raw evidence readable without
        # inflating checked-in spec fixtures into tens of thousands of lines.
        evidence = engine + '.json'
        (output / evidence).write_text('{\n"id": ' + json.dumps(engine) + ',\n"profiles": ' + json.dumps(profiles[engine]) + ',\n"suites": {\n' + ',\n'.join(json.dumps(suite) + ': [\n' + ',\n'.join(json.dumps(case, ensure_ascii=False) for case in cases) + '\n]' for suite, cases in raw.items()) + '\n}\n}\n')
        rows.append({**project, **profiles[engine], 'suites': suites, 'evidence': evidence, 'sha256': sha(output / evidence)})
    write(output / 'build.json', metadata)
    summary = {'buildSha256': sha(output / 'build.json'), 'schema': 1, 'measured': datetime.now(timezone.utc).isoformat(), 'revision': metadata['revision'],
               'machine': platform.platform(), 'suiteLabels': {k: v[0] for k, v in SUITES.items()},
               'inputs': tracked_inputs(), 'rows': rows}
    write(output / 'summary.json', summary)
    lines = ['# Comparison conformance', '',
             'Finite output agreement with 652 CommonMark 0.31.2 examples and 28 **GFM extension** examples. This is not a full GFM conformance score or a performance eligibility filter.', '',
             f"Measured {summary['measured']} on {summary['machine']}; Ferromark source `{summary['revision']}`.", '',
             '[Method, reproduction command, and comparison rules](../../../benchmarks/conformance/README.md). [Build provenance](build.json). [Machine-readable summary](summary.json). Each library link below retains every input, expected/actual HTML, error, and classification.', '',
             '| Variant | Version | CommonMark | GFM extensions |', '| --- | --- | ---: | ---: |']
    for row in rows:
        def label(suite):
            value = row['suites'][suite]
            return f"{value['passed']}/{value['total']} ({value['percent']:.2f}%)" if value['status'] == 'measured' else value['status'].capitalize()
        lines.append(f"| [{row['label']} ({row['runtime']})]({row['evidence']}) | {row['version']} | {label('commonmark')} | {label('gfm')} |")
    lines += ['', '## Profiles and limitations', '']
    for row in rows:
        lines += [f"### {row['label']} ({row['runtime']})", '', f"CommonMark: `{row['commonmark']}`.", '', f"GFM extensions: `{row['gfm']}`." if row['gfm'] else 'GFM extensions: unsupported.', '', row['notes'], '']
    lines += ['## Specification attribution', '', 'CommonMark examples are from John MacFarlane’s CommonMark 0.31.2 specification. GFM extension examples are from the frozen official 0.29-gfm website response retrieved on September 14, 2026, based on CommonMark. The inputs and expected outputs reproduced in the JSON files retain the [CC BY-SA 4.0](https://creativecommons.org/licenses/by-sa/4.0/) license. They are not covered by the repository’s MIT license. [Fixture provenance](../../../benchmarks/compatibility-audit/README.md#fixture-provenance).', '']
    (output / 'README.md').write_text('\n'.join(lines))
    return summary


def check(output, published=None):
    summary = read(output / 'summary.json')
    if summary['schema'] != 1 or summary['inputs'] != tracked_inputs():
        raise ValueError('Report inputs are stale; remeasure after changing adapters or fixtures')
    build = read(output / 'build.json')
    if summary['buildSha256'] != sha(output / 'build.json') or build['revision'] != summary['revision'] or build['inputs'] != summary['inputs']:
        raise ValueError('Build provenance differs from measurement')
    profiles = read(HERE / 'profiles.json')
    for row in summary['rows']:
        if any(row.get(key) != value for key, value in profiles[row['id']].items()):
            raise ValueError('Summary profile differs')
    source = fixtures()
    ids = [row['id'] for row in summary['rows']]
    if len(ids) != len(set(ids)) or set(ids) != set(profiles):
        raise ValueError('Missing or duplicate comparison variants')
    for row in summary['rows']:
        path = output / (row['id'] + '.json')
        if row['evidence'] != path.name or row['sha256'] != sha(path):
            raise ValueError('Evidence hash changed')
        raw = read(path)
        if raw['id'] != row['id'] or raw['profiles'] != profiles[row['id']]:
            raise ValueError('Evidence profile changed')
        for suite in SUITES:
            result, cases = row['suites'][suite], raw['suites'][suite]
            if result['status'] in ('unsupported', 'unmeasured'):
                if cases or not result.get('reason') or (result['status'] == 'unsupported') != (profiles[row['id']][suite] is None):
                    raise ValueError('Invalid unsupported/unmeasured result')
                continue
            if profiles[row['id']][suite] is None or len(cases) != len(source[suite]):
                raise ValueError('Invalid measured suite')
            for expected, case in zip(source[suite], cases, strict=True):
                if any(case[k] != v for k, v in expected.items()):
                    raise ValueError('Specification example changed')
                if (case['error'] is None) != isinstance(case['actual'], str):
                    raise ValueError('Invalid retained render result')
                if case['status'] != ('error' if case['error'] else classify(case['html'], case['actual'])):
                    raise ValueError('Stored classification differs')
            if result != summarize(cases):
                raise ValueError('Counts do not match raw results')
    if published is not None and read(published) != {'report': REPORT, **summary}:
        raise ValueError('Homepage values differ from retained evidence')
    return summary


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('command', choices=('run', 'measure', 'publish', 'check'))
    parser.add_argument('directory', type=Path, nargs='?')
    parser.add_argument('--build', type=Path)
    args = parser.parse_args()
    if args.command == 'check':
        check(args.directory or REPO / REPORT, None if args.directory else DATA)
        print('Conformance evidence, counts, inputs, and published values agree.')
    elif args.command == 'run':
        if not args.directory or args.build:
            parser.error('run DIRECTORY builds and measures into a new directory')
        args.directory.mkdir(parents=True, exist_ok=False)
        builder = module('comparison_build', HERE / 'build.py')
        if subprocess.check_output(['git', 'status', '--porcelain'], cwd=REPO, text=True):
            raise ValueError('Commit measurement inputs before a retained run')
        builder.prepare(args.directory.resolve() / 'build', tracked_inputs())
        measure(args.directory.resolve() / 'build', args.directory.resolve() / 'results')
        check(args.directory.resolve() / 'results')
    elif args.command == 'measure':
        if not args.directory or not args.build:
            parser.error('measure DIRECTORY --build BUILD')
        measure(args.build.resolve(), args.directory.resolve())
        check(args.directory.resolve())
    elif args.command == 'publish':
        if not args.directory:
            parser.error('publish RESULTS')
        summary = check(args.directory.resolve())
        dest = REPO / REPORT
        dest.mkdir(parents=True, exist_ok=False)
        for path in args.directory.iterdir():
            if path.is_file():
                shutil.copyfile(path, dest / path.name)
        write(DATA, {'report': REPORT, **summary})
        check(dest, DATA)


if __name__ == '__main__':
    main()
