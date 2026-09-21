#!/usr/bin/env python3
"""Compare relative engine positions on identical, agreeing document subsets."""
import importlib.util
import json
from pathlib import Path

HERE = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location('native_tables', HERE / 'harness/report.py')
tables = importlib.util.module_from_spec(spec)
spec.loader.exec_module(tables)
previous = HERE.parent / '2026-09-16-native-segments'
old = tables.read(previous, 'summary.json')
new = tables.read(HERE, 'summary.json')
old_sets = tables.agreement_sets(tables.read(previous, 'verification.json'))
new_sets = tables.agreement_sets(tables.read(HERE, 'verification.json'))
assert old_sets == new_sets, 'Do not compare different agreeing input sets'
cases = tables.read(HERE, 'corpus.json')['cases']
six, five = new_sets
groups = [('all-six', six, tables.ENGINES), ('configurable-five', five, tables.CONFIGURABLE)]
for category in sorted({c['category'] for c in cases}):
    members = {c['name'] for c in cases if c['category'] == category} & five
    if members:
        groups.append((category, members, tables.CONFIGURABLE))
rows = []
for group, names, engines in groups:
    for mode in ('fresh', 'reuse'):
        before = tables.aggregate(old, names, mode, engines)
        after = tables.aggregate(new, names, mode, engines)
        for engine in engines:
            if engine == 'v2':
                continue
            rows.append(dict(group=group, documents=len(names), mode=mode, engine=engine,
                previous_v2_relative_throughput=1 / before[engine],
                current_v2_relative_throughput=1 / after[engine],
                relative_position_change_percent=100 * (before[engine] / after[engine] - 1)))
(HERE / 'previous-comparison.json').write_text(json.dumps(rows, indent=2) + '\n')
lines = ['# Position against the preceding native comparison', '',
    'Identical library pins, options, and agreeing input sets. Positive change means',
    'v2 improved its relative throughput against that engine; negative means it lost',
    'ground. These are ratios from separate runs, not isolated causal measurements',
    'of one optimization. Inspect process-round variation before interpreting small changes.', '',
    'The preceding run is [2026-09-16-native-segments](../2026-09-16-native-segments/README.md)',
    'with v2 at `7c887a2b`; this run measures v2 at `bffc89f6`. The other five engine',
    'pins, the flags, the timed loops and the 57 frozen inputs are unchanged.', '',
    '| Group | N | Lifecycle | Against | Previous v2 throughput | Current | Position change |',
    '| --- | ---: | --- | --- | ---: | ---: | ---: |']
for r in rows:
    lines.append(f"| {r['group']} | {r['documents']} | {r['mode']} | {tables.LABELS[r['engine']]} | "
        f"{r['previous_v2_relative_throughput']:.3f}× | {r['current_v2_relative_throughput']:.3f}× | "
        f"{r['relative_position_change_percent']:+.2f}% |")
(HERE / 'previous-comparison.md').write_text('\n'.join(lines) + '\n')

# Keep every input visible, including those outside equal-output score sets.
old_by_case = {(r['case'], r['mode']): r for r in old}
new_by_case = {(r['case'], r['mode']): r for r in new}
old_outputs = tables.read(previous, 'verification.json')
new_outputs = tables.read(HERE, 'verification.json')
documents = []
for case in cases:
    name = case['name']
    assert old_outputs[name]['outputs'] == new_outputs[name]['outputs']
    for mode in ('fresh', 'reuse'):
        before = old_by_case[name, mode]['engines']
        after = new_by_case[name, mode]['engines']
        peers = {}
        for engine in tables.ENGINES:
            if engine == 'v2':
                continue
            old_relative_time = before['v2']['ns'] / before[engine]['ns']
            new_relative_time = after['v2']['ns'] / after[engine]['ns']
            peers[engine] = dict(
                relative_time_change_percent=100 * (new_relative_time / old_relative_time - 1),
                current_v2_relative_throughput=1 / new_relative_time,
                output_agreement=new_outputs[name]['versus_v2'][engine])
        documents.append(dict(case=name, mode=mode, bytes=case['byte_count'], peers=peers))
(HERE / 'previous-documents.json').write_text(json.dumps(documents, indent=2) + '\n')
lines = ['# Every document against the preceding native comparison', '',
    'Positive percentages mean an increase in v2 processing time relative to the',
    'peer, compared with the previous run. These are relative time changes, whereas',
    '`previous-comparison.md` reports throughput changes; they are reciprocals,',
    'not interchangeable percentages. Separate runs do not establish causality.', '',
    'Every one of the 57 documents is listed in both lifecycles, ordered by the',
    'change relative to v1. A dagger marks differing HTML: that cell is diagnostic,',
    'not an equivalent-output competitive score. JSON retains exact classifications.', '',
    '| Document | Lifecycle | Bytes | vs OX | vs v1 | vs md4c | vs pulldown | vs Bun |',
    '| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |']
for row in sorted(documents, key=lambda r: r['peers']['v1']['relative_time_change_percent'], reverse=True):
    cells = []
    for engine in ('ox-content', 'v1', 'md4c', 'pulldown-cmark', 'bun'):
        peer = row['peers'][engine]
        marker = '' if peer['output_agreement'] in ('exact', 'serialization-equivalent') else '†'
        cells.append(f"{peer['relative_time_change_percent']:+.2f}%{marker}")
    lines.append(f"| {row['case']} | {row['mode']} | {row['bytes']} | " + ' | '.join(cells) + ' |')
(HERE / 'previous-documents.md').write_text('\n'.join(lines) + '\n')
print('previous-comparison and previous-documents written')
