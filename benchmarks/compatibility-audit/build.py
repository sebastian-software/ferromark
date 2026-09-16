#!/usr/bin/env python3
"""Build the audit worker outside the production Cargo workspace, offline."""
import argparse
import json
from pathlib import Path
import shutil
import subprocess
import tomllib

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]


def registry_packages(lockfile):
    return sorted((p['name'], p['version'], p['source'], p.get('checksum'))
        for p in tomllib.loads(lockfile.read_text())['package'] if 'source' in p)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('directory', type=Path)
    args = parser.parse_args()
    build = args.directory.resolve()
    build.mkdir(parents=True, exist_ok=False)
    (build / 'Cargo.toml').write_text('''[package]
name = "compatibility-audit-worker"
version = "0.0.0"
edition = "2024"
publish = false
[workspace]
[[bin]]
name = "compatibility-audit-worker"
path = WORKER
[dependencies]
ferromark = { path = FACADE }
serde_json = "1"
[profile.release]
opt-level = 3
'''.replace('WORKER', json.dumps(str(HERE / 'worker.rs')))
        .replace('FACADE', json.dumps(str(ROOT))))
    shutil.copyfile(ROOT / 'Cargo.lock', build / 'Cargo.lock')
    # Cargo only removes unused root dev dependencies and adds the worker entry.
    subprocess.run(['cargo', 'build', '--offline', '--release', '--manifest-path', str(build/'Cargo.toml')],
        cwd=ROOT, check=True)
    root_packages = set(registry_packages(ROOT/'Cargo.lock'))
    assert set(registry_packages(build/'Cargo.lock')) <= root_packages, 'dependency resolution drift'
    print(build / 'target/release/compatibility-audit-worker')


if __name__ == '__main__':
    main()
