#!/usr/bin/env python3
"""Recreate the pinned native audit harness in a separate working directory."""
import argparse
import shutil
import subprocess
from pathlib import Path

OX = '026d1859d1c35e5fb1ea65e7e855b428a918b9bb'
FERRO = '66892cfa2639c3ecb3e049583db746c75354fbda'
p = argparse.ArgumentParser(description=__doc__)
p.add_argument('destination', type=Path)
p.add_argument('--ferromark', type=Path, help='Optional existing Ferromark checkout with the same source tree')
a = p.parse_args()
out = a.destination.resolve()
out.mkdir(parents=True, exist_ok=True)
archive = Path(__file__).resolve().parent

def checkout(url, revision, dest):
    if dest.exists():
        raise SystemExit(f'Refusing to overwrite {dest}')
    subprocess.run(['git', 'init', '-q', str(dest)], check=True)
    subprocess.run(['git', '-C', str(dest), 'fetch', '--depth', '1', url, revision], check=True)
    subprocess.run(['git', '-C', str(dest), 'checkout', '--detach', 'FETCH_HEAD'], check=True)

ferro = a.ferromark.resolve() if a.ferromark else out / 'ferromark'
if not a.ferromark:
    checkout('https://github.com/sebastian-software/ferromark.git', FERRO, ferro)
else:
    subprocess.run(['git', '-C', str(ferro), 'diff', '--exit-code', FERRO, '--', 'src', 'crates', 'Cargo.toml', 'Cargo.lock'], check=True)
checkout('https://github.com/ubugeeei-prod/ox-content.git', OX, out / 'ox-current')
shutil.copytree(archive / 'harness', out / 'harness')
manifest = out / 'harness/Cargo.toml'
manifest.write_text(manifest.read_text().replace('@FERROMARK@', ferro.as_posix()))
scanner = out / 'harness/src/scan.rs'
scanner.write_text(scanner.read_text().replace('@FERROMARK@', ferro.as_posix()))
subprocess.run(['cargo', 'build', '--release', '--locked', '--manifest-path', str(manifest)], check=True)
print(f'Ready: {out}/harness/target/release/ox-audit')
