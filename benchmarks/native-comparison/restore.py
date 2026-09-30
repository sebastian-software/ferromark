#!/usr/bin/env python3
"""Restore the current pinned competitors, or an archived report's own pins."""
from __future__ import annotations

import argparse
import hashlib
import importlib.util
import json
from pathlib import Path
import shutil
import subprocess

HERE = Path(__file__).resolve().parent
# Reports retain prepare.py under harness/, whereas source/cache copies have
# it beside this script. Never import an installed or unrelated prepare module.
PREPARE = HERE / 'harness/prepare.py' if (HERE / 'harness/prepare.py').is_file() else HERE / 'prepare.py'
spec = importlib.util.spec_from_file_location('pinned_prepare', PREPARE)
pins = importlib.util.module_from_spec(spec)
spec.loader.exec_module(pins)


def run(args, cwd=None):
    subprocess.run(list(map(str, args)), cwd=cwd, check=True)


def restore(root):
    root.mkdir(parents=True, exist_ok=True)
    repositories = [
        ('bun', 'https://github.com/oven-sh/bun.git', pins.BUN_REVISION, pins.BUN_VERSION),
        ('md4c', 'https://github.com/mity/md4c.git', pins.MD4C_REVISION, pins.MD4C_VERSION),
    ]
    archives = [
        ('ox.tar.gz', f'https://api.github.com/repos/ubugeeei-prod/ox-content/tarball/{pins.OX_REVISION}', pins.OX_ARCHIVE_SHA256),
        ('native/mimalloc.tar.gz', 'https://codeload.github.com/oven-sh/mimalloc/tar.gz/6a64e1ba7f5b2130d4efccb67ec87fd0003f0f6a', pins.MI_ARCHIVE_SHA256),
        ('native/highway.tar.gz', 'https://codeload.github.com/google/highway/tar.gz/2607d3b5b0113992fe84d3848859eae13b3b52c1', pins.HWY_ARCHIVE_SHA256),
    ]
    # Refuse before downloading anything, retaining failed attempts for diagnosis.
    for name in [*[row[0] for row in repositories], *[row[0] for row in archives], 'restore.json']:
        if (root / name).exists():
            raise SystemExit(f'refusing to replace existing cache input: {root / name}')
    for source, name in [(Path(__file__), 'restore.py'), (PREPARE, 'prepare.py')]:
        destination = root / name
        if source.resolve() != destination.resolve():
            if destination.exists():
                raise SystemExit(f'refusing to replace existing harness: {destination}')
            shutil.copyfile(source, destination)
    (root / 'native').mkdir(exist_ok=True)
    records = {}
    for name, url, revision, version in repositories:
        destination = root / name
        run(['git', 'init', '-q', destination])
        run(['git', 'remote', 'add', 'origin', url], destination)
        fetch = ['git', 'fetch', '--depth=1']
        if name == 'bun':
            run(['git', 'config', 'remote.origin.promisor', 'true'], destination)
            run(['git', 'config', 'remote.origin.partialclonefilter', 'blob:none'], destination)
            fetch += ['--filter=blob:none']
        run([*fetch, 'origin', revision], destination)
        if name == 'bun':
            run(['git', 'sparse-checkout', 'init', '--no-cone'], destination)
            run(['git', 'sparse-checkout', 'set', '--no-cone', '/src/', '/scripts/build/', '/Cargo.toml', '/Cargo.lock', '/rust-toolchain.toml', '/.cargo/', '/LICENSE*'], destination)
        run(['git', 'checkout', '--detach', revision], destination)
        records[name] = {'url': url, 'revision': revision, 'version': version}
    for name, url, expected in archives:
        destination = root / name
        run(['curl', '-fL', '--retry', '2', '--max-time', '180', '-o', destination, url])
        actual = hashlib.sha256(destination.read_bytes()).hexdigest()
        if actual != expected:
            raise SystemExit(f'{name} has sha256 {actual}, expected {expected}')
        records[name] = {'url': url, 'sha256': actual}
    records['ox.tar.gz'].update(revision=pins.OX_REVISION, version=pins.OX_VERSION)
    (root / 'restore.json').write_text(json.dumps(records, indent=2) + '\n')
    print('Pinned competitor sources and verified native archives restored.', flush=True)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('cache', nargs='?', type=Path, default=HERE, help='new cache directory (default: beside this script)')
    restore(parser.parse_args().cache.resolve())
