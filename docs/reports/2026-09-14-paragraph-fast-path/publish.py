"""Generate timing tables from the frozen optimization results."""
import gzip
import json
import lzma
from pathlib import Path
import statistics

HERE = Path(__file__).resolve().parent


def read(path):
    for suffix, decode in [('', lambda x: x), ('.gz', gzip.decompress), ('.xz', lzma.decompress)]:
        candidate = Path(str(path) + suffix)
        if candidate.exists():
            return json.loads(decode(candidate.read_bytes()))
    raise FileNotFoundError(path)


def native_ratio(rows, mode, candidate, baseline, names=None, rnd=None):
    def ns(value):
        return value['ns'] if rnd is None else value['round_medians_ns'][rnd]
    return statistics.geometric_mean(
        ns(row['engines'][candidate]) / ns(row['engines'][baseline])
        for row in rows if row['mode'] == mode and (names is None or row['case'] in names))


def main():
    lines = ['# Timing tables', '', 'Time ratios: **lower is faster**. Aggregates give each document equal weight.', '']
    for name, candidates in [('A-native', ['A']), ('B-native', ['A', 'B'])]:
        rows = read(HERE / 'runs' / name / 'summary.json')
        lines += [f'## {name}: 14 agreeing documents', '',
                  '| Candidate | Mode | Candidate / baseline | Candidate / OX | Round 1 vs baseline | Round 2 vs baseline |',
                  '| --- | --- | ---: | ---: | ---: | ---: |']
        for candidate in candidates:
            for mode in ['fresh', 'reuse', 'init', 'parse', 'render']:
                values = [native_ratio(rows, mode, candidate, 'baseline'),
                          native_ratio(rows, mode, candidate, 'ox'),
                          *[native_ratio(rows, mode, candidate, 'baseline', rnd=r) for r in range(2)]]
                lines.append(f'| {candidate} | {mode} | ' + ' | '.join(f'{v:.4f}' for v in values) + ' |')
        lines.append('')
    feature_runs = ['A-features', 'B-features']
    if (HERE / 'runs/accepted-features/summary.json').exists():
        feature_runs.append('accepted-features')
    for name in feature_runs:
        rows = read(HERE / 'runs' / name / 'summary.json')
        groups = {
            'Comments off, plain prose': lambda r: r['case'].startswith('control--line_comments'),
            'Comments on, no comments present': lambda r: r['case'].startswith('feature--parser.line_comments-plain'),
            'Comments on, active comments': lambda r: r['case'].startswith('feature--parser.line_comments-active'),
            'Mixed CommonMark documents': lambda r: r['category'] == 'mixed-docs-control',
            'Mixed docs plus definition lists': lambda r: r['category'] == 'mixed-docs-definition-lists',
        }
        lines += [f'## {name}: runtime profiles', '',
                  '| Group | Documents | Fresh / baseline | Reuse / baseline | Parse / baseline |',
                  '| --- | ---: | ---: | ---: | ---: |']
        for label, predicate in groups.items():
            subset = [r for r in rows if predicate(r)]
            n = len({r['case'] for r in subset})
            values = [statistics.geometric_mean(r['candidate_ns_per_document'] / r['baseline_ns_per_document']
                      for r in subset if r['mode'] == mode) for mode in ['fresh', 'reuse', 'parse']]
            lines.append(f'| {label} | {n} | ' + ' | '.join(f'{v:.4f}' for v in values) + ' |')
        lines.append('')
    final = HERE / 'runs/native-final/summary.json'
    if final.exists():
        rows = read(final)
        lines += ['## Prototype confirmation: all 57 native-comparison documents', '',
                  '| Mode | A / baseline | B / baseline | B / A |', '| --- | ---: | ---: | ---: |']
        for mode in ['fresh', 'reuse', 'parse']:
            values = [native_ratio(rows, mode, a, b) for a, b in [('A', 'baseline'), ('B', 'baseline'), ('B', 'A')]]
            lines.append(f'| {mode} | ' + ' | '.join(f'{v:.4f}' for v in values) + ' |')
        lines.append('')
    accepted = HERE / 'runs/accepted-native/summary.json'
    if accepted.exists():
        rows = read(accepted)
        names = {row['case'] for row in read(HERE / 'runs/A-native/summary.json')}
        lines += ['## Accepted source: all 57 inputs and the agreeing OX subset', '',
                  '| Mode | Candidate / baseline, all 57 | Candidate / baseline, agreeing 14 | Candidate / OX, agreeing 14 |',
                  '| --- | ---: | ---: | ---: |']
        for mode in ['fresh', 'reuse', 'parse']:
            values = [native_ratio(rows, mode, 'candidate', 'baseline'),
                      native_ratio(rows, mode, 'candidate', 'baseline', names),
                      native_ratio(rows, mode, 'candidate', 'ox', names)]
            lines.append(f'| {mode} | ' + ' | '.join(f'{v:.4f}' for v in values) + ' |')
        lines.append('')
    if (HERE / 'runs/accepted-flag/summary.json').exists():
        lines += ['## Same-binary flag cost on three plain-prose sizes', '',
                  '| Core | Fresh on/off | Reuse on/off | Parse on/off |', '| --- | ---: | ---: | ---: |']
        for name in ['flag-baseline', 'flag-B', 'accepted-flag']:
            rows = read(HERE / 'runs' / name / 'summary.json')
            values = [statistics.geometric_mean(r['on_over_off'] for r in rows if r['mode'] == mode)
                      for mode in ['fresh', 'reuse', 'parse']]
            lines.append(f'| {name} | ' + ' | '.join(f'{v:.4f}' for v in values) + ' |')
        lines.append('')
    (HERE / 'TABLES.md').write_text('\n'.join(lines))


if __name__ == '__main__':
    main()
