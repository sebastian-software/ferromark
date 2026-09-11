#!/usr/bin/env python3
"""Measure CommonMark table contents and controlled table/cell topology."""
from pathlib import Path
import argparse
import hashlib
import importlib.util
import json
import platform
import shutil
import statistics
import subprocess
import time

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
BASE = HERE.parent / '2026-09-11-table-profiling'
spec = importlib.util.spec_from_file_location('table_probe', BASE / 'run.py')
probe = importlib.util.module_from_spec(spec)
spec.loader.exec_module(probe)
LANES = ['ferro-owned', 'pull-owned', 'ferro-reuse', 'pull-reuse', 'ferro-retained', 'ferro-block']


def inputs():
    result = {}
    for kind, fixture in [('plain', 'tables-plain'), ('mixed', 'tables-commonmark-inline')]:
        text = (ROOT / f'benches/fixtures/{fixture}.md').read_text()
        rows = [line for line in text.splitlines() if line and line != '| --- | --- |']
        assert len(rows) == 160
        result[f'{kind}-many'] = text
        result[f'{kind}-one'] = '\n'.join([rows[0], '| --- | --- |', *rows[1:]]) + '\n'
    for name, rows, repeats in [('short-cells', 160, 2), ('long-cells', 16, 20)]:
        cell = 'data' * repeats
        lines = [f'| {cell} | {cell} |'] * rows
        result[name] = '\n'.join([lines[0], '| --- | --- |', *lines[1:]]) + '\n'
    result["links-many"] = (ROOT / "benches/fixtures/tables-links.md").read_text()
    return result


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('mode', choices=['time', 'counts', 'sample'])
    ap.add_argument('--out', type=Path, required=True)
    ap.add_argument("--window-ms", type=int, default=250)
    ap.add_argument("--case", action="append", help="Select cases; omitted means all")
    args = ap.parse_args()
    out = args.out.resolve()
    out.mkdir(parents=True, exist_ok=True)
    cases = inputs()
    if args.case:
        cases = {name: cases[name] for name in args.case}
    roots = {}
    for name, text in cases.items():
        root = ROOT / 'target/table-probe/commonmark-inputs' / name
        path = root / 'benches/fixtures/tables-5k.md'
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text)
        roots[name] = root
    binary = probe.build(args.mode == 'counts')

    def command(mode, case, lane):
        return list(map(str, [binary, mode, 'fixture', 'tables', lane, roots[case], args.window_ms]))

    def invoke(mode, case, lane):
        result = json.loads(subprocess.check_output(command(mode, case, lane), text=True))
        result['case'] = case
        return result

    if args.mode == 'time':
        assert not (out / 'timings.jsonl').exists(), 'Use a new output directory'
        verification = []
        for case, content in cases.items():
            result = invoke('verify', case, 'all')
            assert probe.CanonicalHTML(result['ferromark']).tokens == probe.CanonicalHTML(result['pulldown']).tokens, case
            (out / f'{case}.outputs.json').write_text(json.dumps(result, indent=2) + '\n')
            (out / f'{case}.md').write_text(content)
            verification.append({'case': case, 'bytes': len(content.encode()), 'sha256': hashlib.sha256(content.encode()).hexdigest(), 'normalized_equal': True, 'exact_equal': result['ferromark'] == result['pulldown']})
        (out / 'verification.json').write_text(json.dumps(verification, indent=2) + '\n')
        metadata = {'revision': probe.command('git', 'rev-parse', 'HEAD').strip(), 'rustc': probe.command('rustc', '-Vv'), 'platform': platform.platform(), 'machine': platform.machine(), 'flags': (ROOT / '.cargo/config.toml').read_text(), 'allocator': 'System', 'rounds': 9, 'window_ms': args.window_ms, 'warmup_renders': 32, 'clock_batch': 16, 'features': 'none', 'binary_sha256': hashlib.sha256(binary.read_bytes()).hexdigest(), 'probe_sha256': hashlib.sha256((BASE / 'probe.rs').read_bytes()).hexdigest(), 'production_source_sha256': {str(p.relative_to(ROOT)): hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted((ROOT / 'src').rglob('*.rs'))}}
        (out / 'metadata.json').write_text(json.dumps(metadata, indent=2) + '\n')
        shutil.copyfile(probe.WORK / 'Cargo.lock', out / 'Cargo.lock')
        rows = []
        names = list(cases)
        with (out / 'timings.jsonl').open('w') as stream:
            for r in range(9):
                order = names[r % len(names):] + names[:r % len(names)]
                lanes = LANES[r % len(LANES):] + LANES[:r % len(LANES)]
                if r % 2:
                    lanes = lanes[::-1]
                for case in order:
                    for lane in lanes:
                        result = invoke('time', case, lane)
                        result['round'] = r
                        rows.append(result)
                        stream.write(json.dumps(result) + '\n')
                        stream.flush()
                print(f'round {r + 1}/9 complete', flush=True)
        summary = []
        for case in cases:
            for lane in LANES:
                values = [x['elapsed_ns'] / x['iterations'] / 1000 for x in rows if x['case'] == case and x['lane'] == lane]
                summary.append({'case': case, 'lane': lane, 'median_us': statistics.median(values), 'min_us': min(values), 'max_us': max(values), 'windows_us': values})
        (out / 'summary.json').write_text(json.dumps(summary, indent=2) + '\n')
    elif args.mode == 'counts':
        results = [invoke('counts', case, lane) for case in cases for lane in LANES]
        (out / 'counts.json').write_text(json.dumps(results, indent=2) + '\n')
        (out / 'counts-binary.sha256').write_text(hashlib.sha256(binary.read_bytes()).hexdigest() + '\n')
    else:
        profiles = []
        for case, lane in [('plain-many', 'ferro-reuse'), ('plain-one', 'ferro-reuse'), ('mixed-many', 'ferro-reuse'), ('plain-one', 'pull-reuse'), ('links-many', 'ferro-reuse')]:
            if case not in cases:
                continue
            label = f'{case}-{lane}'
            with (out / f'{label}.process.log').open('w') as log:
                p = subprocess.Popen(command('forever', case, lane), stdout=log, stderr=log)
                try:
                    time.sleep(.25)
                    assert p.poll() is None
                    result = subprocess.run(['/usr/bin/sample', str(p.pid), '5', '1', '-mayDie', '-fullPaths', '-file', str(out / f'{label}.sample.txt')], capture_output=True, text=True, timeout=20)
                    (out / f'{label}.sample.log').write_text(result.stdout + result.stderr)
                    assert result.returncode == 0, result.stderr
                    assert p.poll() is None
                    profiles.append({'case': case, 'lane': lane, 'seconds': 5, 'interval_ms': 1, 'file': f'{label}.sample.txt'})
                    print(label, 'profile complete', flush=True)
                finally:
                    p.terminate()
                    p.wait(timeout=5)
        (out / 'profiles.json').write_text(json.dumps(profiles, indent=2) + '\n')


if __name__ == '__main__':
    main()
