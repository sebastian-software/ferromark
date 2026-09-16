#!/usr/bin/env python3
"""Freeze a committed core and build one worker for every runtime option set."""

import argparse
import hashlib
import importlib.util
import io
import json
import os
from pathlib import Path
import shutil
import subprocess
import tarfile

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]


def crate_source(source: Path) -> Path:
    """Return the `ferromark` package directory inside a checkout.

    The crate is the repository root package since the release-blueprint move;
    checkouts of older revisions keep it under `crates/ferromark`.
    """
    nested = source / "crates" / "ferromark"
    return nested if (nested / "Cargo.toml").is_file() else source


def load_module(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output', type=Path)
    parser.add_argument('--revision', default='HEAD')
    parser.add_argument('--working-tree', action='store_true', help='Freeze current core files, including untracked source files')
    parser.add_argument('--worker-source', type=Path, default=HERE / 'worker.rs',
                        help='Alternative worker, hashed and identical for both compared builds')
    args = parser.parse_args()
    out = args.output.resolve()
    out.mkdir(parents=True, exist_ok=False)
    helpers = load_module('optimization_prepare', HERE.parent / 'optimization-rounds/prepare.py')
    revision = subprocess.check_output(['git', 'rev-parse', args.revision], cwd=ROOT, text=True).strip()
    native = subprocess.check_output(['git', 'ls-tree', '--name-only', revision, 'node/native'], cwd=ROOT, text=True).splitlines()
    archive = subprocess.check_output(['git', 'archive', revision, 'Cargo.toml', 'Cargo.lock',
                                       'rust-toolchain.toml', 'crates', *native], cwd=ROOT)
    if args.working_tree:
        buffer = io.BytesIO()
        with tarfile.open(fileobj=buffer, mode='w') as tar:
            for name in ['Cargo.toml', 'Cargo.lock', 'rust-toolchain.toml', 'crates', *native]:
                tar.add(ROOT / name, arcname=name)
        archive = buffer.getvalue()
    source = out / 'source'
    source.mkdir()
    with tarfile.open(fileobj=io.BytesIO(archive)) as tar:
        tar.extractall(source, filter='data')
    worker = args.worker_source.read_bytes()
    build = out / 'worker'
    (build / 'src').mkdir(parents=True)
    (build / 'src/main.rs').write_bytes(worker)
    shutil.copyfile(source / 'Cargo.lock', build / 'Cargo.lock')
    (build / 'Cargo.toml').write_text(
        '[package]\nname = "runtime-profile-worker"\nversion = "0.0.0"\n'
        'edition = "2024"\npublish = false\n\n[workspace]\n\n[dependencies]\n'
        f'ferromark = {{ path = {json.dumps(str(crate_source(source)))} }}\n'
        'serde_json = "1.0"\n\n[features]\noptional-writing = []\n\n[profile.release]\nopt-level = 3\nlto = "fat"\n'
        'codegen-units = 1\npanic = "abort"\nstrip = true\n')
    env = os.environ.copy()
    env.pop('CARGO_ENCODED_RUSTFLAGS', None)
    env['RUSTFLAGS'] = '-C target-cpu=generic'
    env['CARGO_TARGET_DIR'] = str(build / 'target')
    command = ['cargo', '+1.95', 'build', '--release', '--offline']
    if 'pub inline_footnotes:' in next(path for path in [source / 'src/parser/options.rs', source / 'crates/ferromark/src/parser/options.rs', source / 'crates/ferromark_parser/src/parser/options.rs'] if path.is_file()).read_text():
        command += ['--features', 'optional-writing']
    with (out / 'build.log').open('w') as log:
        result = subprocess.run(command, cwd=build, env=env, stdout=log, stderr=log)
    if result.returncode:
        raise SystemExit((out / 'build.log').read_text())
    original = helpers.registry_packages(source / 'Cargo.lock')
    retained = helpers.registry_packages(build / 'Cargo.lock')
    assert all(original.get(key) == value for key, value in retained.items())
    binary = build / 'target/release/runtime-profile-worker'
    data = {'revision': revision, 'working_tree': args.working_tree, 'binary': str(binary),
            'binary_sha256': helpers.sha256(binary),
            'worker_sha256': hashlib.sha256(worker).hexdigest(),
            'source_tree_sha256': helpers.tree_sha(source),
            'source_archive_sha256': hashlib.sha256(archive).hexdigest(),
            'source_lock_sha256': helpers.sha256(source / 'Cargo.lock'),
            'worker_lock_sha256': helpers.sha256(build / 'Cargo.lock'),
            'rustc': subprocess.check_output(['rustc', '+1.95', '-vV'], text=True),
            'command': command, 'rustflags': env['RUSTFLAGS'], 'lto': 'fat',
            'registry_versions_unchanged': True}
    (out / 'build.json').write_text(json.dumps(data, indent=2) + '\n')
    print(out / 'build.json')


if __name__ == '__main__':
    main()
