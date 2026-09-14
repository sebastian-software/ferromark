"""Regenerate descriptive tables from the archived timing windows and medians."""
import gzip
import json
import lzma
from pathlib import Path
import statistics

HERE = Path(__file__).resolve().parent
MODES = ['fresh', 'reuse', 'init', 'parse', 'render']


def read(path):
    raw = path.read_bytes()
    if path.suffix == '.gz':
        raw = gzip.decompress(raw)
    elif path.suffix == '.xz':
        raw = lzma.decompress(raw)
    return json.loads(raw)


def ratio(rows, mode, numerator, denominator, rnd=None):
    def ns(value):
        return value['ns'] if rnd is None else value['round_medians_ns'][rnd]
    return statistics.geometric_mean(
        ns(row['engines'][numerator]) / ns(row['engines'][denominator])
        for row in rows if row['mode'] == mode)


def main():
    lines = ['# Diagnostic tables', '',
        'Time ratio against the stated control; **lower is faster**. Each aggregate',
        'weights the 14 documents equally. These are descriptive local measurements.', '']
    for title, name, labels in [
        ('Stage screen', 'stages', ['v2']),
        ('Coarse history', 'history', ['pre', 'fix1', 'fix2', 'features', 'current']),
        ('Fine history and paragraph ablation', 'history-fine', ['fix2', 'typo', 'tables', 'columns', 'comments', 'features', 'current', 'paragraph']),
    ]:
        rows = read(HERE / 'runs' / name / 'summary.json')
        variants = read(HERE / 'runs' / name / 'variants.json')
        lines += [f'## {title}', '', '| Variant | Core | Fresh / OX | Reuse / OX | Init / OX | Parse / OX | Render / OX |',
                  '| --- | --- | ---: | ---: | ---: | ---: | ---: |']
        for label in labels:
            rev = variants[label]['metadata']['revision'][:7]
            values = ' | '.join(f'{ratio(rows, mode, label, "ox"):.4f}' for mode in MODES)
            lines.append(f'| {label} | `{rev}` | {values} |')
        lines.append('')
    lines += ['## Ablations against their own controls', '',
              '| Experiment | Mode | Candidate / control | Round 1 | Round 2 |',
              '| --- | --- | ---: | ---: | ---: |']
    for name, candidate, control in [('normalization', 'skip', 'control'),
                                     ('normalization', 'control', 'current'),
                                     ('history-fine', 'paragraph', 'current')]:
        rows = read(HERE / 'runs' / name / 'summary.json')
        for mode in MODES:
            values = ' | '.join(f'{ratio(rows, mode, candidate, control, rnd):.4f}' for rnd in [None, 0, 1])
            lines.append(f'| {name}: {candidate}/{control} | {mode} | {values} |')
    lines += ['', '## Selected absolute stage times', '',
              'From the independent stage screen, nanoseconds per document.', '',
              '| Input | Mode | OX, ns | v2, ns |', '| --- | --- | ---: | ---: |']
    for row in read(HERE / 'runs/stages/summary.json'):
        if row['case'] in ['comment-ack', 'comment-review', 'comment-table', 'wiki-chess-plain-prose']:
            lines.append(f'| {row["case"]} | {row["mode"]} | {row["engines"]["ox"]["ns"]:.2f} | {row["engines"]["v2"]["ns"]:.2f} |')
    lines += ['', '## Structure sizes', '',
              'Native bytes from `size_of`, using the first node of `comment-ack`.', '',
              '| Variant | Parser | Parser options | Renderer | Document | Node enum |',
              '| --- | ---: | ---: | ---: | ---: | ---: |']
    coarse = read(HERE / 'runs/history/verification.json.xz')['comment-ack']
    fine = read(HERE / 'runs/history-fine/verification.json.xz')['comment-ack']
    for label in ['ox', 'pre', 'fix1', 'fix2', 'typo', 'tables', 'columns', 'comments', 'features', 'current']:
        record = (fine if label in fine else coarse)[label]['parse']
        values = ' | '.join(str(record[k]) for k in ['parser_size', 'parser_options_size', 'renderer_size', 'document_size', 'node_size'])
        lines.append(f'| {label} | {values} |')
    lines.append('')
    (HERE / 'TABLES.md').write_text('\n'.join(lines))


if __name__ == '__main__':
    main()
