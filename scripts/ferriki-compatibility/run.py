#!/usr/bin/env python3
"""Verify a packaged public Rust consumer and optionally its published Node peer."""
import argparse
import base64
import difflib
import hashlib
import io
import json
from pathlib import Path
import shutil
import subprocess
import tarfile
import tomllib
import urllib.request

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
CASES = ROOT / 'tests/fixtures/ferriki/cases.json'
SNAPSHOTS = HERE / 'snapshots.json'


def command(args, output, name, **kwargs):
    result = subprocess.run(args, cwd=ROOT, capture_output=True, text=True, encoding='utf-8', **kwargs)
    (output / f'{name}.log').write_text(result.stderr, encoding='utf-8')
    (output / f'{name}.stdout').write_text(result.stdout, encoding='utf-8')
    if result.returncode:
        raise RuntimeError(f'{name} failed ({result.returncode}):\n{result.stderr}\n{result.stdout}')
    return result.stdout


def prepare_assets(output):
    source = json.loads((HERE / 'asset-source.json').read_text(encoding='utf-8'))
    if source['version'] != json.loads(CASES.read_text(encoding='utf-8'))['ferrikiVersion']:
        raise ValueError('fixture and asset versions differ')
    algorithm, expected = source['integrity'].split('-', 1)
    with urllib.request.urlopen(source['url'], timeout=60) as response:
        payload = response.read()
    actual = base64.b64encode(hashlib.new(algorithm, payload).digest()).decode()
    if actual != expected:
        raise ValueError('pinned npm archive integrity mismatch')
    assets = output / 'assets'
    assets.mkdir()
    # Only regular asset files enter the fresh output; tar paths are never
    # interpreted as links or allowed to escape the selected directory.
    with tarfile.open(fileobj=io.BytesIO(payload), mode='r:gz') as archive:
        for member in archive.getmembers():
            prefix = 'package/assets/shiki/'
            if not member.name.startswith(prefix) or not member.isfile():
                continue
            relative = Path(member.name[len(prefix):])
            if relative.is_absolute() or '..' in relative.parts:
                raise ValueError(f'unsafe archive path: {member.name}')
            target = assets / relative
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(archive.extractfile(member).read())
    release = json.loads((assets / 'release-manifest.json').read_text(encoding='utf-8'))
    cache = assets / 'by-digest'
    cache.mkdir()
    for name, entry in release['assets'].items():
        relative = Path(name)
        if relative.is_absolute() or '..' in relative.parts:
            raise ValueError(f'unsafe payload path: {name}')
        content = (assets / relative).read_bytes()
        if len(content) != entry['size'] or hashlib.sha256(content).hexdigest() != entry['sha256']:
            raise ValueError(f'release-pinned asset mismatch: {name}')
        (cache / entry['sha256']).write_bytes(content)
    return assets


def rust_consumer(output, assets):
    command(['cargo', 'package', '-p', 'ferromark', '--features', 'ferriki', '--locked',
             '--offline', '--allow-dirty', '--no-verify', '--target-dir', str(output / 'target')],
            output, 'package')
    version = tomllib.loads((ROOT / 'Cargo.toml').read_text(encoding='utf-8'))['package']['version']
    archive = output / f'target/package/ferromark-{version}.crate'
    unpacked = output / 'unpacked'
    unpacked.mkdir()
    with tarfile.open(archive) as tar:
        for member in tar.getmembers():
            relative = Path(member.name)
            if relative.is_absolute() or '..' in relative.parts or not member.isfile():
                if member.isdir():
                    continue
                raise ValueError(f'unsafe Cargo archive member: {member.name}')
            target = unpacked / relative
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(tar.extractfile(member).read())
    package = unpacked / f'ferromark-{version}'
    consumer = output / 'consumer'
    (consumer / 'src').mkdir(parents=True)
    original = tomllib.loads((ROOT / 'Cargo.lock').read_text(encoding='utf-8'))['package']
    serde_version = next(p['version'] for p in original if p['name'] == 'serde_json')
    (consumer / 'Cargo.toml').write_text(f'''[workspace]
[package]
name = "ferromark-ferriki-fixture-consumer"
version = "0.0.0"
edition = "2024"
publish = false
[dependencies]
ferromark = {{ path = {json.dumps(str(package))}, features = ["ferriki"] }}
serde_json = "={serde_version}"
''', encoding='utf-8')
    shutil.copyfile(package / 'examples/ferriki_contract.rs', consumer / 'src/main.rs')
    shutil.copyfile(ROOT / 'Cargo.lock', consumer / 'Cargo.lock')
    manifest_args = ['--manifest-path', str(consumer / 'Cargo.toml')]
    command(['cargo', 'metadata', '--offline', '--format-version', '1', *manifest_args], output, 'resolve')
    resolved = tomllib.loads((consumer / 'Cargo.lock').read_text(encoding='utf-8'))['package']
    fixture_version = json.loads(CASES.read_text(encoding='utf-8'))['ferrikiVersion']
    for name in ['ferriki', 'ferriki-asset-gen', 'ferriki-textmate']:
        versions = [p['version'] for p in resolved if p['name'] == name]
        if versions != [fixture_version]:
            raise ValueError(f'{name} versions {versions} differ from fixtures {fixture_version}')
    pinned = {(p['name'], p['version'], p.get('source'), p.get('checksum')) for p in original}
    for package_entry in resolved:
        if 'source' in package_entry:
            identity = tuple(package_entry.get(k) for k in ('name', 'version', 'source', 'checksum'))
            if identity not in pinned:
                raise ValueError(f'unpinned consumer dependency: {identity}')
    tree = command(['cargo', 'tree', '--edges', 'normal', '--locked', '--offline', *manifest_args], output, 'rust-tree')
    if any(f'{name} v' in tree for name in ['napi', 'napi-derive', 'bincode', 'ureq', 'rustls']):
        raise ValueError('network, advisory-blocked or N-API dependency in Rust consumer')
    data = command(['cargo', 'run', '--locked', '--offline', *manifest_args, '--', str(assets), str(CASES)],
                   output, 'rust-consumer')
    return json.loads(data), hashlib.sha256(archive.read_bytes()).hexdigest()


def node_consumer(output):
    command(['cargo', 'build', '-p', 'ferromark-node', '--profile', 'release-node', '--locked', '--offline'],
            output, 'node-addon')
    facade = output / 'node-package'
    facade.mkdir()
    for filename in ['index.mjs', 'native-target.mjs']:
        shutil.copyfile(ROOT / 'node/ferromark' / filename, facade / filename)
    target = command(['node', '--input-type=module', '-e',
        'import {nativeTarget} from "./node/ferromark/native-target.mjs"; '
        'console.log(nativeTarget(process.platform, process.arch, '
        'process.report.getReport().header.glibcVersionRuntime ? "gnu" : "musl"));'], output, 'node-target').strip()
    filename = ('ferromark_node.dll' if target.startswith('win32') else
                'libferromark_node.dylib' if target.startswith('darwin') else 'libferromark_node.so')
    shutil.copyfile(ROOT / 'target/release-node' / filename, facade / f'ferromark.{target}.node')
    data = command(['node', str(HERE / 'node.mjs'), str(CASES), str(facade / 'index.mjs'), str(output)],
                   output, 'node-consumer')
    return json.loads(data)


def check_snapshot(name, actual, expected):
    if actual != expected:
        lines = difflib.unified_diff(json.dumps(expected, indent=2, sort_keys=True).splitlines(),
            json.dumps(actual, indent=2, sort_keys=True).splitlines(), fromfile=f'expected/{name}', tofile=f'actual/{name}')
        raise AssertionError('\n'.join(lines))


def compare_peers(rust, node):
    cases = {case['id']: case for case in json.loads(CASES.read_text(encoding='utf-8'))['tokens']}
    scopes = json.loads(CASES.read_text(encoding='utf-8'))['languageScopes']
    for left, right in zip(rust['tokens'], node['tokens'], strict=True):
        if left['id'] != right['id']:
            raise AssertionError('peer case identity differs')
        if 'error' in left:
            expected = {'UnknownLanguage': 'ERR_UNSUPPORTED', 'UnknownTheme': 'ERR_UNSUPPORTED'}[left['error']]
            check_snapshot(left['id'], right.get('error'), expected)
            continue
        # When no single scope path survives native style-token merging (or
        # for plain text), the Node facade inserts a synthetic root scope.
        # Keep raw peer snapshots and require exactly that documented root;
        # every scope path actually exposed by Rust must agree unchanged.
        left = json.loads(json.dumps(left))
        for line in left['tokens']['tokens']:
            for token in line:
                if 'scopeNames' not in token:
                    token['scopeNames'] = [scopes[cases[left['id']]['lang']]]
        check_snapshot(left['id'], right, left)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output', type=Path, help='fresh directory for archives, logs and results')
    parser.add_argument('--node', action='store_true', help='also test the published Node highlighter')
    parser.add_argument('--update-snapshots', action='store_true', help='review and replace both peer snapshots')
    args = parser.parse_args()
    if args.update_snapshots and not args.node:
        parser.error('updating snapshots requires --node to keep both peers together')
    output = args.output.resolve()
    output.mkdir(parents=True)
    assets = prepare_assets(output)
    rust, archive_hash = rust_consumer(output, assets)
    result = {'rust': rust}
    if args.node:
        result['node'] = node_consumer(output)
        compare_peers(rust, result['node'])
    if args.update_snapshots:
        SNAPSHOTS.write_text(json.dumps(result, indent=2, ensure_ascii=False) + '\n', encoding='utf-8')
    else:
        expected = json.loads(SNAPSHOTS.read_text(encoding='utf-8'))
        for name, actual in result.items():
            check_snapshot(name, actual, expected[name])
    (output / 'results.json').write_text(json.dumps(result, indent=2, ensure_ascii=False) + '\n', encoding='utf-8')
    (output / 'summary.json').write_text(json.dumps({
        'ferriki': json.loads(CASES.read_text(encoding='utf-8'))['ferrikiVersion'], 'cargoArchiveSha256': archive_hash,
        'tokenCases': len(rust['tokens']), 'markdownCases': len(rust['markdown']),
        'packagedRustConsumer': 'passed', 'nodePeer': 'passed' if args.node else 'not requested',
        'assetErrors': rust['assetErrors'], 'reuse': rust['reuse'],
        'limits': 'No runtime performance, allocation or remote CDN measurements.',
    }, indent=2) + '\n', encoding='utf-8')
    print(f'Ferriki contract passed: {output}')


if __name__ == '__main__':
    main()
