"""Reconstruct the pinned native corpus experiment outside repository Cargo config."""
from pathlib import Path
import argparse
import hashlib
import io
import json
import re
import shutil
import subprocess
import tarfile

REPORT = Path(__file__).resolve().parent


def run(args, cwd=None):
    subprocess.run(args, cwd=cwd, check=True)


def checkout(destination, repository, revision, paths=None):
    if not destination.exists():
        run(['git', 'init', '-q', str(destination)])
        run(['git', '-C', str(destination), 'remote', 'add', 'origin', repository])
    if paths is not None:
        run(['git', '-C', str(destination), 'sparse-checkout', 'set', '--no-cone',
             *paths, 'LICENSE', 'LICENSE.md', 'LICENSE.txt', 'LICENSE-MIT', 'LICENSE-APACHE'])
    run(['git', '-C', str(destination), 'fetch', '--depth=1', 'origin', revision])
    run(['git', '-C', str(destination), 'checkout', '--detach', 'FETCH_HEAD'])
    actual = subprocess.check_output(['git', '-C', str(destination), 'rev-parse', 'HEAD'], text=True).strip()
    assert actual == revision


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('workdir', type=Path)
    parser.add_argument('--ferromark-repo', type=Path, default=REPORT.parents[2])
    args = parser.parse_args()
    work = args.workdir.resolve()
    repo = args.ferromark_repo.resolve()
    if work == repo or repo in work.parents:
        parser.error('workdir must be outside the repository Cargo configuration')
    work.mkdir(parents=True, exist_ok=True)
    metadata = json.loads((REPORT / 'metadata.json').read_text())
    manifest = json.loads((REPORT / 'corpus-manifest.json').read_text())
    checkout(work / 'ox-source', 'https://github.com/ubugeeei-prod/ox-content.git', metadata['ox_revision'])
    (work / 'source').mkdir(exist_ok=True)
    archive = subprocess.check_output(['git', 'archive', metadata['ferromark_revision'], 'src'], cwd=repo)
    with tarfile.open(fileobj=io.BytesIO(archive)) as tar:
        tar.extractall(work / 'source', filter='data')
    for relative, expected in metadata['source_hashes'].items():
        path = work / 'source' / relative
        assert hashlib.sha256(path.read_bytes()).hexdigest() == expected, relative
        path.touch()  # Never let restored mtimes hide a source change from Cargo.
    shutil.copyfile(REPORT / 'source-Cargo.toml', work / 'source/Cargo.toml')
    for engine in ['ferro', 'ox']:
        shutil.copytree(REPORT / 'harnesses' / engine, work / engine, dirs_exist_ok=True)
    shutil.copytree(REPORT / 'demangle', work / 'demangle', dirs_exist_ok=True)
    for name in ['build.py', 'run.py', 'profile.py', 'analyze-profiles.py', 'counters.py', 'probe-lines.py',
                 'make-results.py', 'selected.json', 'case-manifest.json', 'corpus-manifest.json', 'metadata.json']:
        shutil.copyfile(REPORT / name, work / name)
    cases = []
    for project in manifest:
        directory = work / 'corpora' / project['project']
        checkout(directory, project['repository'], project['revision'], project['paths'])
        actual_paths = sorted(str(p.relative_to(directory)) for p in directory.rglob('*.md') if '.git' not in p.parts)
        assert actual_paths == [f['path'] for f in project['files']], project['project']
        chunks = []
        for record in project['files']:
            data = (directory / record['path']).read_bytes()
            assert hashlib.sha256(data).hexdigest() == record['sha256'], record['path']
            source = data.decode('utf8')
            chunks.append(source + '\n')
            cases.append(dict(case=project['project'] + '/' + record['path'], project=project['project'],
                              kind='file', profile='matched', input=source))
        for profile in ['matched', 'upstream']:
            cases.append(dict(case='concat/' + project['project'] + '/' + profile, project=project['project'],
                              kind='concat', profile=profile, input=''.join(chunks)))
    source = (work / 'ox-source/crates/ox_content_parser/benches/parser.rs').read_text()
    for name, count in [('SIMPLE_MD', 1), ('LARGE_MD', 1), ('LARGE_MD', 100)]:
        value = re.search(r'const ' + name + r': &str = r#"(.*?)"#;', source, re.S)[1] * count
        cases.append(dict(case=f'ox-parser/{name}/{count}', project='controls', kind='control',
                          profile='matched', input=value))
    expected = json.loads((REPORT / 'case-manifest.json').read_text())
    assert len(cases) == len(expected)
    for case, record in zip(cases, expected):
        assert case['case'] == record['case']
        assert hashlib.sha256(case['input'].encode()).hexdigest() == record['sha256']
    (work / 'cases.json').write_text(json.dumps(cases))
    print(f'Reconstructed {len(cases)} cases in {work}')


if __name__ == '__main__':
    main()
