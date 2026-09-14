#!/usr/bin/env python3
"""Record allocation differences independently of uninstrumented timing runs.

Uses the prior counting worker unchanged; its alloc_bytes includes zeroed bytes,
while reallocations are separate. Counter differences are expected and retained.
The timing harness must verify exact AST/HTML before accepting an optimization.
"""
import argparse
import importlib.util
import json
from pathlib import Path
import sys

PRIOR = Path(__file__).resolve().parents[1] / 'simd-round'
sys.path.insert(0, str(PRIOR))
spec = importlib.util.spec_from_file_location('prior_allocations', PRIOR / 'allocations.py')
prior = importlib.util.module_from_spec(spec)
spec.loader.exec_module(prior)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('baseline', 'candidate', 'build', 'corpus', 'output'):
        parser.add_argument(name, type=Path)
    args = parser.parse_args()
    if prior.registry_packages(args.baseline / 'Cargo.lock') != prior.registry_packages(args.candidate / 'Cargo.lock'):
        raise SystemExit('registry lock mismatch')
    build = prior.build_workers(args.baseline, args.candidate, args.build)
    cases = json.loads(args.corpus.read_text())['cases']
    args.output.mkdir(parents=True, exist_ok=False)
    (args.output / 'build.json').write_text(json.dumps(build, indent=2) + '\n')
    (args.output / 'allocation_worker.rs').write_bytes(prior.WORKER.read_bytes())
    rows = []
    for case in cases:
        data = case['input'].encode()
        import hashlib
        if len(data) != case['byte_count'] or hashlib.sha256(data).hexdigest() != case['sha256']:
            raise AssertionError((case['name'], 'corpus integrity'))
        path = args.output / (case['name'] + '.md')
        path.write_bytes(data)
        for mode in prior.MODES:
            engines = {}
            for engine in prior.ENGINES:
                worker = prior.Worker(Path(build['engines'][engine]['binary']), case['profile'], mode, path)
                try:
                    engines[engine] = worker.measure()
                finally:
                    worker.close()
            for key in ('html_len', 'html_checksum', 'children'):
                if engines['baseline'][key] != engines['candidate'][key]:
                    raise AssertionError((case['name'], mode, key, engines))
            rows.append({'case': case['name'], 'mode': mode, 'engines': engines})
        print(case['name'], flush=True)
    (args.output / 'allocations.json').write_text(json.dumps(rows, indent=2) + '\n')


if __name__ == '__main__':
    main()
