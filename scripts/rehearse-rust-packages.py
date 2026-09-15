#!/usr/bin/env python3
"""Build the Cargo archive and verify an isolated consumer of its contents."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import tarfile
import tomllib

ROOT = Path(__file__).resolve().parents[1]


def run(args, log):
    with log.open('w') as stream:
        subprocess.run(args, cwd=ROOT, stdout=stream, stderr=subprocess.STDOUT, check=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output', type=Path, help='new directory; existing outputs are never overwritten')
    args = parser.parse_args()
    output = args.output.resolve()
    output.mkdir()
    workspace = tomllib.loads((ROOT / 'Cargo.toml').read_text())['workspace']
    version = workspace['package']['version']
    names = sorted(tomllib.loads((ROOT / member / 'Cargo.toml').read_text())['package']['name']
                   for member in workspace['members'] if member.startswith('crates/'))
    assert names == ['ferromark']
    run(['cargo', 'package', '-p', 'ferromark', '--locked', '--allow-dirty',
         '--target-dir', str(output / 'target')], output / 'package.log')
    unpacked = output / 'unpacked'
    unpacked.mkdir()
    artifacts = []
    for name in names:
        archive = output / 'target/package' / f'{name}-{version}.crate'
        with tarfile.open(archive) as tar:
            tar.extractall(unpacked, filter='data')
        package = unpacked / f'{name}-{version}'
        assert (package / 'LICENSE').read_bytes() == (ROOT / 'LICENSE').read_bytes(), f'{name}: upstream MIT notice'
        assert (package / 'README.md').is_file(), f'{name}: README'
        manifest = tomllib.loads((package / 'Cargo.toml').read_text())
        assert manifest['package']['version'] == version
        assert manifest['package']['repository'] == workspace['package']['repository']
        assert manifest['package']['license'] == 'MIT'
        assert (package / 'src/lib.rs').is_file()
        for section in ('dependencies', 'dev-dependencies', 'build-dependencies'):
            for dependency, spec in manifest.get(section, {}).items():
                assert not dependency.startswith('ferromark_'), f'{name}: internal crate dependency'
                assert 'path' not in spec, f'{name}: leaked workspace path'
        artifacts.append(dict(name=name, version=version, archive=str(archive.relative_to(output)),
            bytes=archive.stat().st_size, sha256=hashlib.sha256(archive.read_bytes()).hexdigest()))
    consumer = output / 'consumer'
    (consumer / 'src').mkdir(parents=True)
    (consumer / 'Cargo.toml').write_text(f'''[workspace]
[package]
name = "ferromark-packaged-consumer"
version = "0.0.0"
edition = "2024"
publish = false
[dependencies]
ferromark = {{ path = {json.dumps(str(unpacked / f"ferromark-{version}"))}, version = "={version}" }}
''')
    (consumer / 'src/main.rs').write_text('''fn main() {
    assert_eq!(ferromark::to_html("Hello, **world**!").unwrap(),
               "<p>Hello, <strong>world</strong>!</p>\\n");
    assert!(ferromark::to_html("[guide]\\n\\n- [guide]: /start\\n").unwrap()
        .contains("<a href=\\"/start\\">guide</a>"));
}
''')
    shutil.copyfile(ROOT / 'Cargo.lock', consumer / 'Cargo.lock')
    run(['cargo', 'run', '--offline', '--manifest-path', str(consumer / 'Cargo.toml')], output / 'consumer.log')
    original = tomllib.loads((ROOT / 'Cargo.lock').read_text())['package']
    resolved = tomllib.loads((consumer / 'Cargo.lock').read_text())['package']
    for package in resolved:
        if 'source' in package:
            assert any(all(candidate.get(key) == package.get(key) for key in ('name', 'version', 'source', 'checksum'))
                       for candidate in original), f'Unexpected registry dependency: {package["name"]}'
        elif package['name'] != 'ferromark-packaged-consumer':
            assert package['name'] in names and package['version'] == version
    result = dict(version=version, packages=artifacts, packaged_consumer='passed',
        method='Full cargo package --locked, including packaged builds; additional isolated consumer uses unpacked archives.',
        limits='No registry upload or publishing credentials were tested.')
    (output / 'results.json').write_text(json.dumps(result, indent=2) + '\n')
    print(f'Verified ferromark Cargo archive and isolated packaged consumer: {output}')


if __name__ == '__main__':
    main()
