#!/usr/bin/env python3
"""Recheck retained diagnostic coverage, duration and consumed output lengths."""
import gzip
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent


def read(path):
    return json.loads(gzip.decompress(path.read_bytes()))


record = json.loads((ROOT / 'readiness.json').read_text())
assert record['publishable'] is False and len(record['verification']) == 14
windows = 0
for lane in record['diagnostic_timing']['lanes']:
    name = lane['lane']
    config = read(ROOT / 'diagnostic' / name / 'run.json.gz')
    rows = read(ROOT / 'diagnostic' / name / 'samples.json.gz')
    descriptors = read(ROOT / 'verification' / ('native' if name == 'native-shared' else name) / 'output-descriptors.json.gz')
    assert len(descriptors) == 57
    assert (config['rounds'], config['samples'], config['window_ms'], config['warmup_ms']) == (1, 1, 10, 10)
    jobs = ({job['name']: job['members'] for job in config['jobs']} if name == 'native-shared'
            else {case: [case] for case in descriptors})
    expected = {(0, 0, case, mode) for case in jobs for mode in ('fresh', 'reuse')}
    seen = set()
    for row in rows:
        identity = (row['round'], row['sample'], row['case'], row['mode'])
        assert identity not in seen, ('duplicate window', name, identity)
        seen.add(identity)
        assert sorted(row['order']) == sorted(config['engines'])
        for engine in config['engines']:
            timing = row[engine]
            field = 'utf16_units' if config.get('track') == 'node' else 'utf8_bytes'
            units = sum(descriptors[case]['outputs'][engine][field] for case in jobs[row['case']])
            checksum = timing['iterations'] * units
            if name == 'native-shared':
                checksum %= 1 << 64
            assert timing['iterations'] > 0 and timing['elapsed_ns'] >= 10_000_000
            assert timing['checksum'] == checksum, ('checksum', name, identity, engine)
            windows += 1
    assert seen == expected and len(rows) == lane['timing_rows']
assert windows == record['executed_timing_windows'] == 2988
print(f'{windows} diagnostic engine windows passed; this report is not publishable performance evidence.')
