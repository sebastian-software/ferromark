#!/usr/bin/env python3
"""Verify a packaged public Rust consumer and optionally its published Node peer."""
import argparse
import base64
from concurrent.futures import ThreadPoolExecutor, as_completed
import difflib
import hashlib
from html.parser import HTMLParser
import io
import json
from pathlib import Path
import re
import shutil
import subprocess
import tarfile
import tomllib
from urllib.parse import quote
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
    # Ferriki publishes the catalog and release manifest in npm, while
    # grammar/theme payloads live on the release CDN. Only regular archive
    # files enter this fresh output; archive paths cannot escape it.
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
    commit = release.get('commit')
    if not isinstance(commit, str) or len(commit) != 40 or any(c not in '0123456789abcdef' for c in commit):
        raise ValueError('release manifest has an invalid asset commit')
    if commit != source['assetCommit']:
        raise ValueError('pinned CDN commit differs from the release manifest')
    if not isinstance(release.get('assets'), dict) or not release['assets']:
        raise ValueError('release manifest has no assets')
    base_url = source['cdnBaseUrl'].rstrip('/')
    if not base_url.startswith('https://'):
        raise ValueError('asset CDN must use HTTPS')
    cache = assets / 'by-digest'
    cache.mkdir()

    def download(name, entry):
        relative = Path(name)
        if relative.is_absolute() or '..' in relative.parts:
            raise ValueError(f'unsafe payload path: {name}')
        url = f'{base_url}/{commit}/assets/shiki/{quote(name, safe="/")}'
        with urllib.request.urlopen(url, timeout=60) as response:
            content = response.read()
        if len(content) != entry['size'] or hashlib.sha256(content).hexdigest() != entry['sha256']:
            raise ValueError(f'release-pinned asset mismatch: {name}')
        return relative, entry['sha256'], content

    # Keep the complete published catalog usable by both consumers. Downloads
    # are bounded in parallel and verified before entering either local source.
    with ThreadPoolExecutor(max_workers=6) as workers:
        pending = [workers.submit(download, name, entry) for name, entry in release['assets'].items()]
        for future in as_completed(pending):
            relative, digest, content = future.result()
            target = assets / relative
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(content)
            (cache / digest).write_bytes(content)
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


def node_consumer(output, assets):
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
    data = command(['node', str(HERE / 'node.mjs'), str(CASES), str(facade / 'index.mjs'), str(output), str(assets)],
                   output, 'node-consumer')
    return json.loads(data)


def check_snapshot(name, actual, expected):
    if actual != expected:
        lines = difflib.unified_diff(json.dumps(expected, indent=2, sort_keys=True).splitlines(),
            json.dumps(actual, indent=2, sort_keys=True).splitlines(), fromfile=f'expected/{name}', tofile=f'actual/{name}')
        raise AssertionError('\n'.join(lines))


def replace_node_snapshot(node):
    # Preserve the reviewed Rust snapshot byte-for-byte while changing the
    # Node peer's representation as its public API evolves.
    current = SNAPSHOTS.read_text(encoding='utf-8')
    key = '  "node": '
    start = current.index(key)
    value_start = start + len(key)
    _, end = json.JSONDecoder().raw_decode(current, value_start)
    lines = json.dumps(node, indent=2, ensure_ascii=False).splitlines()
    formatted = lines[0] + '\n' + '\n'.join('  ' + line for line in lines[1:])
    SNAPSHOTS.write_text(current[:start] + key + formatted + current[end:], encoding='utf-8')


class _VisibleText(HTMLParser):
    def __init__(self):
        super().__init__(convert_charrefs=True)
        self.parts = []

    def handle_data(self, data):
        self.parts.append(data)


def visible_text(html):
    parser = _VisibleText()
    parser.feed(html)
    parser.close()
    return ''.join(parser.parts).replace('\r\n', '\n').replace('\r', '\n')


def markdown_text(html):
    # The adapters intentionally add different blank-line separators around
    # blocks. Exact output stays frozen per peer; this compares visible words.
    return re.sub(r'\n{2,}', '\n', visible_text(html)).strip('\n')


def compare_peers(rust, node):
    corpus = json.loads(CASES.read_text(encoding='utf-8'))
    token_ids = [case['id'] for case in corpus['tokens']]
    if len(token_ids) != len(set(token_ids)):
        raise ValueError('authored highlighting case identities are not unique')
    if [case['id'] for case in rust['tokens']] != token_ids:
        raise ValueError('Rust highlighting cases differ from the complete authored corpus')
    if [case['id'] for case in node['highlighting']] != token_ids:
        raise ValueError('Node highlighting cases differ from the complete authored corpus')
    cases = {case['id']: case for case in corpus['tokens']}
    rust_tokens = {case['id']: case for case in rust['tokens']}
    node_highlighting = {case['id']: case for case in node['highlighting']}
    error_codes = {'UnknownLanguage': 'ERR_UNSUPPORTED', 'UnknownTheme': 'ERR_UNSUPPORTED'}
    for case_id, case in cases.items():
        left = rust_tokens[case_id]
        right = node_highlighting[case_id]
        if 'error' in left:
            check_snapshot(case_id, right.get('error'), error_codes[left['error']])
            continue
        if 'error' in right:
            raise AssertionError(f'{case_id}: Node errored while Rust returned HTML')
        rust_text = visible_text(left['html'])
        node_text = visible_text(right['html'])
        expected_text = case['code'].replace('\r\n', '\n').replace('\r', '\n')
        check_snapshot(f'{case_id}/Rust HTML text', rust_text, expected_text)
        check_snapshot(f'{case_id}/Node HTML text', node_text, expected_text)

    markdown_ids = [case['id'] for case in corpus['markdown']]
    if len(markdown_ids) != len(set(markdown_ids)):
        raise ValueError('authored Markdown case identities are not unique')
    if [case['id'] for case in rust['markdown']] != markdown_ids:
        raise ValueError('Rust Markdown cases differ from the complete authored corpus')
    if [case['id'] for case in node['markdown']] != markdown_ids:
        raise ValueError('Node Markdown cases differ from the complete authored corpus')
    rust_markdown = {case['id']: case for case in rust['markdown']}
    node_markdown = {case['id']: case for case in node['markdown']}
    # The legacy Node Markdown callback forwards unsupported fence metadata and
    # unknown-language failures. The Rust adapter deliberately suppresses both
    # so Ferromark can preserve its own annotation/fallback behavior.
    node_only_errors = {'unknown-language', 'vitepress'}
    for case_id, left in rust_markdown.items():
        right = node_markdown[case_id]
        rust_errors = [error_codes.get(error, 'ERR_ASSET') for error in left['errors']]
        if case_id in node_only_errors:
            if left['errors']:
                raise AssertionError(f'{case_id}: Rust unexpectedly reported {left["errors"]}')
            if right['errors'] != ['ERR_UNSUPPORTED']:
                raise AssertionError(f'{case_id}: unexpected Node Markdown errors {right["errors"]}')
        elif right['errors'] != rust_errors:
            raise AssertionError(f'{case_id}: Markdown errors differ: Rust {rust_errors}, Node {right["errors"]}')
        if case_id == 'vitepress':
            check_snapshot('vitepress/Rust HTML text', markdown_text(left['html']), 'let x = 1;')
            check_snapshot(
                'vitepress/Node HTML text',
                markdown_text(right['html']),
                '// [!code warning]\nlet x = 1;',
            )
        else:
            check_snapshot(
                f'{case_id}/Markdown HTML text',
                markdown_text(right['html']),
                markdown_text(left['html']),
            )


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output', type=Path, help='fresh directory for archives, logs and results')
    parser.add_argument('--node', action='store_true', help='also test the published Node highlighter')
    parser.add_argument('--update-node-snapshot', action='store_true', help='review and replace only the Node HTML snapshot')
    args = parser.parse_args()
    if args.update_node_snapshot and not args.node:
        parser.error('updating the Node snapshot requires --node')
    output = args.output.resolve()
    output.mkdir(parents=True)
    assets = prepare_assets(output)
    rust, archive_hash = rust_consumer(output, assets)
    result = {'rust': rust}
    if args.node:
        result['node'] = node_consumer(output, assets)
        compare_peers(rust, result['node'])
    expected = json.loads(SNAPSHOTS.read_text(encoding='utf-8'))
    if args.update_node_snapshot:
        check_snapshot('frozen Rust output', rust, expected['rust'])
        replace_node_snapshot(result['node'])
    else:
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
