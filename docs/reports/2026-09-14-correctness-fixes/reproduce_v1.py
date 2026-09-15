#!/usr/bin/env python3
"""Recheck the recorded v1 cases with the pinned native-comparison worker."""
import argparse
import hashlib
import importlib.util
import json
from pathlib import Path
import subprocess
import tempfile

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
SPEC = importlib.util.spec_from_file_location('verify', ROOT/'benchmarks/native-comparison/verify.py')
VERIFY = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(VERIFY)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('worker', type=Path)
    args = parser.parse_args()
    recorded = json.loads((HERE/'raw/v1-comparison.json').read_text())
    assert hashlib.sha256(args.worker.read_bytes()).hexdigest() == recorded['worker_sha256']
    with tempfile.TemporaryDirectory(prefix='ferromark-v1-repros-') as directory:
        folder = Path(directory)
        for profile in ('commonmark', 'gfm-shared'):
            cases = [x for x in recorded['results'] if x['profile'] == profile]
            paths = [folder/(x['id']+'.md') for x in cases]
            for case, path in zip(cases, paths):
                path.write_bytes(case['markdown'].encode())
            output = subprocess.run([str(args.worker.resolve()), 'v1', profile, 'fresh',
                *map(str, paths)], input='verify\nquit\n', text=True,
                capture_output=True, check=True).stdout
            rendered = [bytes.fromhex(line.split()[2]).decode() for line in output.splitlines()
                if line.startswith('html ')]
            assert len(rendered) == len(cases)
            for case, html in zip(cases, rendered):
                assert html == case['v1'], case['id']
                normalized = html.replace('\r\n', '\n').replace('\r', '\n')
                assert VERIFY.classify(case['expected'], normalized) == case['v1_status'], case['id']
    print(f"Verified {len(recorded['results'])} recorded v1 outputs and classifications.")


if __name__ == '__main__':
    main()
