#!/usr/bin/env python3
"""Audit raw, conservative, and spec HTML agreement for CM and full GFM."""
import argparse
from collections import Counter
import difflib
import hashlib
from html.parser import HTMLParser
import importlib.util
import json
from pathlib import Path
import subprocess

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
SPEC = importlib.util.spec_from_file_location('native_verify', HERE.parent / 'native-comparison/verify.py')
VERIFY = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(VERIFY)


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def parse_txt(text):
    lines = iter(text.splitlines())
    examples, section = [], ''
    fence = '`' * 32
    for line in lines:
        if line.startswith('#'):
            section = line.lstrip('#').strip()
        if not line.startswith(fence + ' example'):
            continue
        fields = [[], []]
        side = 0
        for line in lines:
            if line.startswith(fence):
                break
            if line == '.' and side == 0:
                side = 1
                continue
            fields[side].append(line.replace('→', '\t') + '\n')
        examples.append(dict(example=len(examples)+1, section=section, markdown=''.join(fields[0]), html=''.join(fields[1])))
    return examples


class GFMExamples(HTMLParser):
    def __init__(self, html):
        super().__init__(convert_charrefs=True)
        self.examples = []
        self.div_depth = 0
        self.current = None
        self.capture = None
        self.heading = None
        self.section = ''
        self.feed(html)
        self.close()

    def handle_starttag(self, tag, attributes):
        attrs = dict(attributes)
        if tag in ('h1', 'h2', 'h3') and self.current is None:
            self.heading = []
        if tag == 'div':
            if self.current is not None:
                self.div_depth += 1
            elif 'example' in attrs.get('class', '').split():
                number = int(attrs['id'].removeprefix('example-'))
                self.current = dict(example=number, section=self.section, markdown='', html='')
                self.div_depth = 1
        if tag == 'code' and self.current is not None:
            classes = attrs.get('class', '').split()
            if 'language-markdown' in classes:
                self.capture = 'markdown'
            elif 'language-html' in classes:
                self.capture = 'html'

    def handle_endtag(self, tag):
        if tag in ('h1', 'h2', 'h3') and self.heading is not None:
            self.section = ''.join(self.heading).strip()
            self.heading = None
        if tag == 'code':
            self.capture = None
        if tag == 'div' and self.current is not None:
            self.div_depth -= 1
            if self.div_depth == 0:
                self.examples.append(self.current)
                self.current = None

    def handle_data(self, data):
        if self.capture is not None:
            self.current[self.capture] += data.replace('→', '\t')
        elif self.heading is not None:
            self.heading.append(data)


class Worker:
    def __init__(self, binary):
        self.process = subprocess.Popen([str(binary)], text=True, stdin=subprocess.PIPE, stdout=subprocess.PIPE)

    def render(self, markdown, profile, html=''):
        self.process.stdin.write(json.dumps(dict(markdown=markdown, profile=profile, html=html)) + '\n')
        self.process.stdin.flush()
        line = self.process.stdout.readline()
        if not line:
            raise RuntimeError('audit worker exited')
        return json.loads(line)

    def close(self):
        self.process.stdin.close()
        assert self.process.wait(timeout=10) == 0


def audit(worker, examples, profile):
    result = []
    for example in examples:
        response = worker.render(example['markdown'], profile, example['html'])
        status = VERIFY.classify(example['html'], response['html']) if not response['error'] else 'error'
        row = dict(example, profile=profile, actual=response['html'], error=response['error'],
            status=status, spec_equal=response['spec_equal'])
        if status not in ('exact', 'serialization-equivalent', 'heading-id-only'):
            row['token_diff'] = list(difflib.unified_diff(
                [repr(t) for t in VERIFY.without_heading_ids(VERIFY.canonical(example['html']))],
                [repr(t) for t in VERIFY.without_heading_ids(VERIFY.canonical(response['html']))],
                fromfile='expected', tofile='actual', n=1))
        result.append(row)
    return result


def line_endings(worker, examples):
    """Change only source line endings; normalize raw output CR/LF for comparison."""
    failures = []
    for example in examples:
        original = worker.render(example['markdown'], 'commonmark')
        assert original['error'] is None
        for kind, ending in [('crlf', '\r\n'), ('cr', '\r')]:
            source = example['markdown'].replace('\n', ending)
            actual = worker.render(source, 'commonmark')
            normalized = actual['html'].replace('\r\n', '\n').replace('\r', '\n')
            if normalized != original['html'] or actual['error']:
                failures.append(dict(example=example['example'], section=example['section'], kind=kind,
                    markdown=source, expected=original['html'], actual=actual['html'], error=actual['error']))
    return dict(count=2*len(examples), failures=failures)


def compare_upstream(binary, endings, probes, folder):
    """Reuse the previously pinned native OX/V2 worker for inheritance evidence."""
    cases = [dict(id=f"cm-{x['example']}-{x['kind']}", profile='commonmark',
        markdown=x['markdown'], current=x['actual']) for x in endings['failures']]
    for x in probes:
        if x['id'].startswith('nul-') or x['id'] in ('url-backslash-path', 'gfm-single-tilde', 'gfm-triple-tilde'):
            cases.append(dict(id=x['id'], profile='gfm-shared' if x['profile']=='gfm' else 'commonmark',
                markdown=x['markdown'], current=x['html']))
    folder.mkdir()
    for x in cases:
        (folder/(x['id']+'.md')).write_bytes(x['markdown'].encode())
    for profile in ('commonmark', 'gfm-shared'):
        selected = [x for x in cases if x['profile']==profile]
        for engine in ('v2', 'ox-content'):
            output = subprocess.run([str(binary), engine, profile, 'fresh',
                *[str(folder/(x['id']+'.md')) for x in selected]], input='verify\nquit\n',
                text=True, capture_output=True, check=True).stdout
            html = [bytes.fromhex(line.split()[2]).decode() for line in output.splitlines() if line.startswith('html ')]
            assert len(html) == len(selected)
            for x, value in zip(selected, html):
                x[engine] = value
    for x in cases:
        x['equal'] = x['current'] == x['v2'] == x['ox-content']
    return dict(worker_sha256=sha(binary), count=len(cases), results=cases)


def has_gating_failures(results, endings, probes):
    return bool(endings['failures'] or
        any(x['error'] for rows in results.values() for x in rows) or
        any(x['error'] for x in probes) or
        any(not x['spec_equal'] or x['status'] in ('other', 'error')
            for name in ('commonmark-configured', 'gfm-extensions-configured')
            for x in results[name]) or
        any(x['kind'] == 'normative' and x['status'] in ('other', 'error') for x in probes) or
        any(x['kind'] == 'normalizer-counterexample' and x['spec_equal'] for x in probes))


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('worker', type=Path)
    p.add_argument('gfm_html', type=Path)
    p.add_argument('output', type=Path)
    p.add_argument('--upstream-worker', type=Path,
        help='optional native-comparison worker pinned in the 2026-09-14 engine report')
    p.add_argument('--fail-on-differences', action='store_true',
        help='exit 1 for configured CommonMark/GFM-extension, line-ending, or normative probe differences')
    args = p.parse_args()
    args.output.mkdir(parents=True, exist_ok=False)
    fixtures = ROOT / 'tests/spec_fixtures'
    cm = parse_txt((fixtures / 'commonmark-0.31.2-spec.txt').read_text())
    ext = parse_txt((fixtures / 'gfm-extensions-spec.txt').read_text())
    gfm = GFMExamples(args.gfm_html.read_text()).examples
    assert len(cm) == 652 and len(ext) == 24
    assert [x['example'] for x in gfm] == list(range(1, len(gfm)+1))
    # Compare independently vendored examples; do not silently replace historical
    # fixtures when the live page changes without changing its version label.
    extension_mapping = [dict(local_example=x['example'], official_examples=[y['example'] for y in gfm
        if (x['markdown'], x['html']) == (y['markdown'], y['html'])]) for x in ext]
    assert sum(bool(x['official_examples']) for x in extension_mapping) >= 22, 'unexpected extraction or source drift'
    (args.output / 'extension-mapping.json').write_text(json.dumps(extension_mapping, indent=2)+'\n')
    (args.output / 'gfm-examples.json').write_text(json.dumps(gfm, indent=2, ensure_ascii=False)+'\n')
    worker = Worker(args.worker)
    results = {}
    try:
        for name, examples, profile in (
            ('commonmark-configured', cm, 'commonmark'),
            ('commonmark-default', cm, 'default'),
            ('gfm-extensions-configured', [x for x in gfm if '(extension)' in x['section']], 'gfm'),
            ('gfm-configured', gfm, 'gfm'),
            ('gfm-without-tagfilter', gfm, 'gfm-no-tagfilter'),
            ('gfm-public-preset', gfm, 'gfm-preset'),
        ):
            results[name] = audit(worker, examples, profile)
            print(name, len(results[name]), dict(Counter(x['status'] for x in results[name])),
                'spec failures', sum(not x['spec_equal'] for x in results[name]), flush=True)
        endings = line_endings(worker, cm)
        probes = [dict(case, **worker.render(case['markdown'], case['profile'], case['expected']))
            for case in json.loads((HERE/'probes.json').read_text())]
        for row in probes:
            row['status'] = VERIFY.classify(row['expected'], row['html']) if row['error'] is None else 'error'
    finally:
        worker.close()
    print('line-ending variants', endings['count'], 'differences', len(endings['failures']), flush=True)
    (args.output / 'line-endings.json').write_text(json.dumps(endings, indent=2, ensure_ascii=False)+'\n')
    (args.output / 'probes.json').write_text(json.dumps(probes, indent=2, ensure_ascii=False)+'\n')
    if args.upstream_worker:
        upstream = compare_upstream(args.upstream_worker, endings, probes, args.output/'upstream-inputs')
        (args.output/'upstream-comparison.json').write_text(json.dumps(upstream, indent=2, ensure_ascii=False)+'\n')
    (args.output / 'results.json').write_text(json.dumps(results, indent=2, ensure_ascii=False)+'\n')
    core_files = sorted(p for p in (ROOT/'crates').glob('*/src/**/*.rs'))
    core_hash = hashlib.sha256()
    for path in core_files:
        core_hash.update(str(path.relative_to(ROOT)).encode() + b'\0' + path.read_bytes() + b'\0')
    metadata = dict(core_sha256=core_hash.hexdigest(), source_revision=subprocess.check_output(['git', '-C', str(ROOT), 'rev-parse', 'HEAD'],text=True).strip(),
        worker_sha256=sha(args.worker), worker_source_sha256=sha(HERE/'worker.rs'), runner_sha256=sha(Path(__file__)),
        gfm_html_sha256=sha(args.gfm_html), gfm_url='https://github.github.com/gfm/',
        probes_sha256=sha(HERE/'probes.json'), comparator_sha256=sha(HERE.parent/'native-comparison/verify.py'),
        spec_normalizer_sha256=sha(fixtures.parent/'spec_support/normalize.rs'),
        spec_text_codec_sha256=sha(fixtures.parent/'spec_support/text_codec.rs'),
        root_lock_sha256=sha(ROOT/'Cargo.lock'),
        rustc=subprocess.check_output(['rustc', '-Vv'], cwd=ROOT, text=True).strip(),
        commonmark_spec_sha256=sha(fixtures/'commonmark-0.31.2-spec.txt'),
        extension_spec_sha256=sha(fixtures/'gfm-extensions-spec.txt'),
        summary={name:dict(count=len(rows), statuses=dict(Counter(x['status'] for x in rows)),
             spec_failures=sum(not x['spec_equal'] for x in rows)) for name,rows in results.items()})
    (args.output / 'metadata.json').write_text(json.dumps(metadata, indent=2)+'\n')
    if args.fail_on_differences and has_gating_failures(results, endings, probes):
        raise SystemExit(1)


if __name__ == '__main__':
    main()
