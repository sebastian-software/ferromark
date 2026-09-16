#!/usr/bin/env python3
"""Build isolated diagnostic workers using the frozen native dependency setup."""
import argparse
import hashlib
import io
import json
import os
from pathlib import Path
import shutil
import subprocess
import tarfile

ROOT = Path(__file__).resolve().parents[2]
HERE = Path(__file__).resolve().parent


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('output', type=Path)
    p.add_argument('--native-build', type=Path, required=True)
    p.add_argument('--revision', default='33c216b')
    p.add_argument('--patch', type=Path)
    p.add_argument('--worker', type=Path, default=HERE / 'worker.rs')
    args = p.parse_args()
    base = args.native_build.resolve()
    original = json.loads((base / 'build.json').read_text())
    assert sha(original['binary']) == original['binary_sha256']
    out = args.output.resolve()
    out.mkdir(parents=True, exist_ok=False)
    sources = out / 'sources'
    sources.mkdir()
    for name in ('ferromark_v1', 'ox_content', 'md4c', 'bun'):
        (sources / name).symlink_to(base / 'sources' / name, target_is_directory=True)
    source = sources / 'ferromark_v2'
    source.mkdir()
    revision = subprocess.check_output(['git', '-C', str(ROOT), 'rev-parse', args.revision], text=True).strip()
    native = subprocess.check_output(['git', '-C', str(ROOT), 'ls-tree', '--name-only', revision, 'node/native'], text=True).splitlines()
    raw = subprocess.check_output(['git', '-C', str(ROOT), 'archive', revision,
                                   'Cargo.toml', 'Cargo.lock', 'rust-toolchain.toml', 'crates', *native])
    with tarfile.open(fileobj=io.BytesIO(raw)) as archive:
        archive.extractall(source, filter='data')
    if args.patch:
        subprocess.run(['git', 'apply', str(args.patch.resolve())], cwd=source, check=True)
        shutil.copyfile(args.patch, out / 'diagnostic.patch')
    bun = out / 'bun'
    bun.mkdir()
    for filename in ('Cargo.toml', 'Cargo.lock'):
        shutil.copyfile(base / 'bun' / filename, bun / filename)
    (bun / 'src').symlink_to(base / 'bun' / 'src', target_is_directory=True)
    if (base / 'bun' / '.cargo').exists():
        (bun / '.cargo').symlink_to(base / 'bun' / '.cargo', target_is_directory=True)
    worker = bun / 'comparison-worker'
    worker.mkdir()
    for filename in ('Cargo.toml', 'build.rs'):
        shutil.copyfile(base / 'bun' / 'comparison-worker' / filename, worker / filename)
    worker_text = args.worker.read_text()
    adaptations = []
    if 'fn commonmark()' not in next(path for path in [source / 'src/parser/options.rs', source / 'crates/ferromark/src/parser/options.rs', source / 'crates/ferromark_parser/src/parser/options.rs'] if path.is_file()).read_text():
        worker_text = worker_text.replace('..v2::ParserOptions::commonmark()', '..v2::ParserOptions::default()')
        adaptations.append('Older parser: default is the same all-extensions-off option set.')
    if 'fn commonmark()' not in next(path for path in [source / 'src/renderer/html/options.rs', source / 'crates/ferromark/src/renderer/html/options.rs', source / 'crates/ferromark_renderer/src/html/options.rs'] if path.is_file()).read_text():
        worker_text = worker_text.replace('v2::HtmlRendererOptions::commonmark(),', '''v2::HtmlRendererOptions {
            autolink_urls: false,
            autolink_target_blank: false,
            link_target_blank: false,
            ..v2::HtmlRendererOptions::new()
        },''')
        adaptations.append('Older renderer has no strict profile; original OX-like settings remain. Exact-HTML input gate required.')
    (worker / 'worker.rs').write_text(worker_text)
    env = os.environ.copy()
    env.pop('CARGO_ENCODED_RUSTFLAGS', None)
    for key in list(env):
        if key.startswith('CARGO_PROFILE_'):
            env.pop(key)
    env['RUSTFLAGS'] = '-C target-cpu=generic'
    env['BUN_CODEGEN_DIR'] = str(base / 'codegen')
    env['CARGO_TARGET_DIR'] = str(out / 'target')
    command = ['cargo', '+nightly-2026-07-20', 'build', '--release', '--offline', '--locked', '-p', 'native-comparison-worker']
    with (out / 'build.log').open('w') as log:
        result = subprocess.run(command, cwd=bun, env=env, stdout=log, stderr=subprocess.STDOUT)
    if result.returncode:
        print((out / 'build.log').read_text()[-12000:])
        raise SystemExit(result.returncode)
    binary = out / 'target/release/native-comparison-worker'
    assert sha(bun / 'Cargo.lock') == original['lock_sha256']
    files = {str(f.relative_to(source)): sha(f) for f in source.rglob('*') if f.is_file()}
    record = {'native_reference_build': original, 'revision': revision,
              'diagnostic_patch_sha256': sha(args.patch) if args.patch else None,
              'binary': str(binary), 'binary_sha256': sha(binary),
              'worker': str(worker / 'worker.rs'), 'worker_sha256': sha(worker / 'worker.rs'),
              'api_adaptations': adaptations,
              'source': str(source), 'source_files_sha256': files,
              'command': command, 'rustflags': env['RUSTFLAGS'],
              'lock_sha256': sha(bun / 'Cargo.lock'),
              'native_support': 'Unchanged frozen native libraries and Bun source from reference build.'}
    (out / 'build.json').write_text(json.dumps(record, indent=2) + '\n')
    print(out / 'build.json')


if __name__ == '__main__':
    main()
