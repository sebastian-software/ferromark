#!/usr/bin/env python3
"""Generate comparison tables from retained native benchmark evidence."""
import argparse
import csv
import gzip
import json
import math
from pathlib import Path
import statistics

ENGINES = ('v2', 'ox-content', 'v1', 'md4c', 'pulldown-cmark', 'bun')
LABELS = {'v2': 'Ferromark v2', 'ox-content': 'OX-Content original', 'v1': 'Ferromark v1',
          'md4c': 'md4c', 'pulldown-cmark': 'pulldown-cmark', 'bun': 'Bun native bun_md'}


def read(directory, name):
    path = directory / name
    if path.exists():
        return json.loads(path.read_text())
    return json.loads(gzip.open(str(path) + '.gz', 'rt').read())


def geomean(values):
    return math.exp(statistics.mean(math.log(x) for x in values))


def aggregate(summary, names, mode):
    rows = [r for r in summary if r['case'] in names and r['mode'] == mode]
    assert len(rows) == len(names), ('incomplete measurement group', mode, len(rows), len(names))
    return {e: geomean(r['engines']['v2']['ns'] / r['engines'][e]['ns'] for r in rows) for e in ENGINES} if rows else {}


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('results', type=Path)
    args = p.parse_args()
    directory = args.results
    cases = read(directory, 'corpus.json')['cases']
    verification = read(directory, 'verification.json')
    summary = read(directory, 'summary.json')
    by_name = {c['name']: c for c in cases}
    all_names = set(by_name)
    strict = {n for n, v in verification.items() if v['strict_all_six']}
    groups = [('All 57 native workloads (output differences included)', all_names),
              (f'All-six HTML agreement ({len(strict)} cases)', strict)]
    for category in sorted({c['category'] for c in cases}):
        groups.append((category, {c['name'] for c in cases if c['category'] == category}))
    for profile in sorted({c['profile'] for c in cases}):
        groups.append((profile + (' (shared subset)' if profile == 'gfm' else ''), {c['name'] for c in cases if c['profile'] == profile}))
    for label, lower, upper in (('<512 B', 0, 512), ('512 B–2 KiB', 512, 2048), ('2–8 KiB', 2048, 8192), ('8–32 KiB', 8192, 32768), ('32–128 KiB', 32768, 131072)):
        groups.append((label, {c['name'] for c in cases if lower <= c['byte_count'] < upper}))
    lines = ['# Native benchmark tables', '',
        'Speed relative to Ferromark v2 in the same lifecycle: **higher is faster**; v2 = 1.00×.',
        'Geometric mean of per-document time ratios; median of three process-round medians per engine/document.',
        'All-workload aggregates include different HTML and are diagnostic. Strict agreement excludes heading-ID differences.',
        'Bun uses its fresh owned-output API in both lifecycle schedules.', '']
    aggregate_data = []
    for mode in ('fresh', 'reuse'):
        lines += [f'## {mode}', '', '| Group | N | ' + ' | '.join(LABELS[e] for e in ENGINES) + ' |',
                  '| --- | ---: | ' + ' | '.join('---:' for _ in ENGINES) + ' |']
        for label, names in groups:
            values = aggregate(summary, names, mode)
            if not values:
                continue
            aggregate_data.append(dict(group=label, count=len(names), mode=mode, speed_relative_to_v2=values))
            lines.append(f'| {label} | {len(names)} | ' + ' | '.join(f'{values[e]:.2f}×' for e in ENGINES) + ' |')
        lines += ['']
    lines += ['## HTML agreement with v2', '', '| Engine | Exact | Serialization equivalent | Heading IDs only | Other |',
              '| --- | ---: | ---: | ---: | ---: |']
    for engine in ENGINES:
        counts = {s: sum(v['versus_v2'][engine] == s for v in verification.values()) for s in ('exact', 'serialization-equivalent', 'heading-id-only', 'other')}
        lines.append('| ' + LABELS[engine] + ' | ' + ' | '.join(str(n) for n in counts.values()) + ' |')
    lines += ['', '## Per-document timings', '',
              'Microseconds per complete Markdown→HTML operation; lower is faster. “Agree” requires all six engines.',
              'Original input sizes are UTF-8 bytes. “gfm” is the shared subset described in the harness README.', '']
    csv_rows = []
    for mode in ('fresh', 'reuse'):
        lines += [f'### {mode}', '', '| Document | Bytes | Profile | Agree | ' + ' | '.join(LABELS[e] for e in ENGINES) + ' |',
            '| --- | ---: | --- | --- | ' + ' | '.join('---:' for _ in ENGINES) + ' |']
        for row in summary:
            if row['mode'] != mode or row['case'] not in all_names:
                continue
            name = row['case']
            case = by_name[name]
            agreement = 'yes' if name in strict else 'no'
            lines.append(f'| {name} | {case["byte_count"]} | {case["profile"]} | {agreement} | ' +
                ' | '.join(f'{row["engines"][e]["ns"] / 1000:.3f}' for e in ENGINES) + ' |')
            for engine in ENGINES:
                values = row['engines'][engine]
                csv_rows.append(dict(case=name, bytes=case['byte_count'], category=case['category'], profile=case['profile'],
                    mode=mode, engine=engine, strict_all_six=agreement, ns=values['ns'],
                    round_min_ns=values['round_min_ns'], round_max_ns=values['round_max_ns'],
                    speed_relative_to_v2=row['engines']['v2']['ns'] / values['ns']))
        lines += ['']
    lines += ['## Rotating batches', '', 'Microseconds for a complete batch of all inputs in the named profile; lower is faster.',
        'This total weights large documents more heavily and is not included in per-document geometric means.', '',
        '| Batch | Mode | Documents | ' + ' | '.join(LABELS[e] for e in ENGINES) + ' |',
        '| --- | --- | ---: | ' + ' | '.join('---:' for _ in ENGINES) + ' |']
    for row in summary:
        if row['case'] in all_names:
            continue
        lines.append(f'| {row["case"]} | {row["mode"]} | {len(row["members"])} | ' +
            ' | '.join(f'{row["engines"][e]["ns"] / 1000:.2f}' for e in ENGINES) + ' |')
    (directory / 'TABLES.md').write_text('\n'.join(lines) + '\n')
    (directory / 'aggregates.json').write_text(json.dumps(aggregate_data, indent=2) + '\n')
    with (directory / 'timings.csv').open('w') as f:
        writer = csv.DictWriter(f, fieldnames=list(csv_rows[0]))
        writer.writeheader()
        writer.writerows(csv_rows)
    print(directory / 'TABLES.md')


if __name__ == '__main__':
    main()
