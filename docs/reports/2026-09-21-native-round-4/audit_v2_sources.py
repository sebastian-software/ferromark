#!/usr/bin/env python3
"""Audit the measured v2 tree, which `harness/audit_sources.py` under-covers here.

`audit_sources.py` looks for v2 sources under `crates/`. At the measured
revision `39b1f0b7` the Rust components are consolidated into the repository
root package (`23a212d8`, `885e5b1c`), so that path list matches only
`Cargo.toml` and `Cargo.lock` and the parser/renderer sources go unchecked.

This supplement runs the harness's own `audit_git` over the paths v2 actually
uses at this revision and writes `source-audit-v2.json`. It is additive: the
harness script is unmodified and its output is archived as `source-audit.json.gz`.
"""

import argparse
import importlib.util
import json
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
HARNESS = HERE / 'harness'


def load(name):
    spec = importlib.util.spec_from_file_location(name, HARNESS / (name + '.py'))
    module = importlib.util.module_from_spec(spec)
    sys.modules[name] = module
    spec.loader.exec_module(module)
    return module


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('build', type=Path, help='prepare.py build directory')
    parser.add_argument('output', type=Path)
    parser.add_argument('--v2', type=Path, default=HERE.parents[2])
    args = parser.parse_args()

    sys.path.insert(0, str(HARNESS))
    load('prepare')
    audit_sources = load('audit_sources')

    metadata = json.loads((args.build / 'build.json').read_text())
    revision = metadata['engines']['ferromark_v2']['revision']
    root = args.build / 'sources/ferromark_v2'
    result = audit_sources.audit_git(args.v2, revision, root, ['src', 'Cargo.toml', 'Cargo.lock'])
    result['paths'] = ['src', 'Cargo.toml', 'Cargo.lock']
    result['note'] = (
        'Supplements harness/audit_sources.py, whose v2 path list is "crates" and '
        'therefore covers only the manifests at this revision.'
    )
    args.output.write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps({'v2': {'revision': revision,
                             'checked_files': result['checked_files'],
                             'mismatches': result['mismatches']}}))


if __name__ == '__main__':
    main()
