#!/usr/bin/env python3
"""Pair two native v2 binaries on real outliers and marker-only diagnostic variants."""
import argparse
import hashlib
import json
from pathlib import Path
import statistics
import sys
import tempfile

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE / 'harness'))
from run import Worker, host, read_json


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('before', type=Path)
    p.add_argument('after', type=Path)
    p.add_argument('output', type=Path)
    args = p.parse_args()
    assert not args.output.exists(), 'Preserve existing measurements'
    binaries = [args.before.resolve(), args.after.resolve()]
    corpus = read_json(HERE / 'corpus.json.gz')
    rows = []
    metadata = dict(host=host(), binaries=[dict(path=str(b), sha256=hashlib.sha256(b.read_bytes()).hexdigest()) for b in binaries],
        samples=12, window_ms=40, warmup_ms=60, corpus_sha256=hashlib.sha256((HERE / 'corpus.json.gz').read_bytes()).hexdigest())
    with tempfile.TemporaryDirectory(prefix='ferromark-native-probe-') as temporary:
        path = Path(temporary) / 'input.md'
        for name in ('comment-incident', 'rust-book-ch00-00-introduction', 'comment-review'):
            case = next(c for c in corpus['cases'] if c['name'] == name)
            for variant in ('original', 'no-definitions'):
                source = case['input'] if variant == 'original' else case['input'].replace(']:', ']=')
                path.write_text(source)
                for mode in ('fresh', 'reuse'):
                    workers = [Worker(b, 'v2', case['profile'], mode, [path]) for b in binaries]
                    try:
                        outputs = [w.verify() for w in workers]
                        assert outputs[0] == outputs[1], 'Each variant must agree across versions'
                        size = len(outputs[0][0].encode())
                        for worker in workers:
                            worker.bench(60_000_000, size)
                        samples = []
                        for i in range(12):
                            timings = {}
                            for k in ([0, 1] if i % 2 == 0 else [1, 0]):
                                timings[k] = workers[k].bench(40_000_000, size)
                            before, after = [timings[k]['elapsed_ns'] / timings[k]['iterations'] for k in (0, 1)]
                            samples.append(dict(before=timings[0], after=timings[1], ratio=after / before))
                        for worker in workers:
                            assert worker.verify() == outputs[0]
                        result = dict(case=name, variant=variant, mode=mode,
                            input_sha256=hashlib.sha256(source.encode()).hexdigest(), output=outputs[0][0],
                            delta_percent=100 * (statistics.median(s['ratio'] for s in samples) - 1), samples=samples)
                        rows.append(result)
                        print(name, variant, mode, f"{result['delta_percent']:+.2f}%", flush=True)
                    finally:
                        for worker in workers:
                            worker.close()
    args.output.write_text(json.dumps(dict(metadata=metadata, rows=rows), indent=2) + '\n')


if __name__ == '__main__':
    main()
