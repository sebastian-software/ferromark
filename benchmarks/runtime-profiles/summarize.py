#!/usr/bin/env python3
"""Generate complete feature, profile, and scaling tables from measured pairs."""

import argparse
import json
import math
from pathlib import Path
import statistics


def geomean(values):
    return math.exp(statistics.mean(math.log(v) for v in values))


def percent(ratio):
    return f'{(ratio - 1) * 100:+.1f}%'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('results', type=Path)
    parser.add_argument('output', type=Path)
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=True)
    rows = json.loads((args.results / 'summary.json').read_text())
    features = list(dict.fromkeys(r['feature'] for r in rows if r['group'] == 'feature'))
    index = {(r['feature'], r['workload'], r['target_bytes'], r['mode']): r
             for r in rows if r['group'] == 'feature'}
    lines = ['# Feature timing matrix', '',
             'Every percentage is `(enabled time / disabled time - 1) × 100`, using',
             'the median of paired ratios. Positive values mean more time. The primary',
             'stage is parse-only for parser options and render-only for renderer options.',
             'Plain prose reveals idle checks; active probes repeat feature syntax. Their',
             'different output is intentional and is not an optimization comparison.', '',
             'Sizes name approximate input targets; exact byte counts, absolute times,',
             'pair ranges, and round medians are in `summary.csv` and `summary.json`.',
             'Treat small changes as noise unless confirmed by longer independent runs.', '',
             '| Feature | Plain ~300 B | Plain ~4 KiB | Plain ~64 KiB | Active ~300 B | Active ~4 KiB | Active ~64 KiB | Active ~4 KiB parse + render, reuse |',
             '| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |']
    for feature in features:
        mode = 'parse' if feature.startswith('parser.') else 'render'
        values = [percent(index[feature, workload, size, mode]['on_over_off'])
                  for workload in ('plain', 'active') for size in (300, 4096, 65536)]
        values.append(percent(index[feature, 'active', 4096, 'reuse']['on_over_off']))
        lines.append('| `' + feature + '` | ' + ' | '.join(values) + ' |')
    lines += ['', '`highlight` and `soft_break` are currently ignored by the renderer;',
              'their variation is a control, not evidence of implemented functionality.',
              '`source_spans` changes even plain-prose HTML. All other plain-probe',
              'equality results are recorded rather than assumed.', '']
    (args.output / 'FEATURES.md').write_text('\n'.join(lines))

    lines = ['# Candidate profile timings', '',
             'Ratios compare each recipe with strict CommonMark on the same documents.',
             'Features and HTML policy differ: this table is a cost budget, not a ranking',
             'of equivalent renderers. Category values are unweighted geometric means',
             'of per-document paired median ratios. Exact configurations are in the corpus.', '',
             '| Candidate | Document group | Count | Fresh time ratio | Reused time ratio | Identical HTML / AST |',
             '| --- | --- | ---: | ---: | ---: | ---: |']
    for feature in ('commonmark', 'gfm-spec', 'comments', 'article', 'docs'):
        for group in ('comments', 'technical-docs', 'readme', 'reference', 'encyclopedia', 'plain-prose'):
            subset = [r for r in rows if r['group'] == 'profile' and r['feature'] == feature and r['workload'] == group]
            if not subset:
                continue
            reuse = [r for r in subset if r['mode'] == 'reuse']
            ratios = [geomean([r['on_over_off'] for r in subset if r['mode'] == mode]) for mode in ('fresh', 'reuse')]
            same = sum(r['html_equal'] and r['ast_equal'] for r in reuse)
            lines.append(f'| {feature} | {group} | {len(reuse)} | {ratios[0]:.3f}× | {ratios[1]:.3f}× | {same}/{len(reuse)} |')
    lines += ['', '## Same-output parser ablations', '',
              'Here only unused parser extensions are removed. Renderer policy is fixed,',
              'and both HTML and AST must match exactly. Negative change means the',
              'tailored recipe took less time than broad options on this input.', '',
              '| Recipe | Input bytes | Broad fresh µs | Tailored fresh µs | Fresh change | Broad reuse µs | Tailored reuse µs | Reuse change |',
              '| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |']
    for name in ('comments', 'article', 'docs', 'mdx-docs'):
        fresh = next(r for r in rows if r['group'] == 'ablation' and r['feature'] == name and r['mode'] == 'fresh')
        reuse = next(r for r in rows if r['group'] == 'ablation' and r['feature'] == name and r['mode'] == 'reuse')
        lines.append(f"| {name} | {fresh['byte_count']} | {fresh['off_ns']/1000:.3f} | {fresh['on_ns']/1000:.3f} | {percent(fresh['on_over_off'])} | {reuse['off_ns']/1000:.3f} | {reuse['on_ns']/1000:.3f} | {percent(reuse['on_over_off'])} |")
    (args.output / 'PROFILES.md').write_text('\n'.join(lines) + '\n')

    lines = ['# Active-probe scaling', '',
             'Enabled primary-stage time divided by original input bytes. These are',
             'repeated syntax shapes at three sizes, not proofs of worst-case complexity.',
             'Frontmatter has a fixed prefix and TOC a single marker; those shapes differ',
             'from the features repeated throughout a document. Blank/escaped/near-miss',
             'and adversarial inputs need separate studies before complexity claims.', '',
             '| Feature | ~300 B ns/B | ~4 KiB ns/B | ~64 KiB ns/B | 64 KiB / 4 KiB time per byte |',
             '| --- | ---: | ---: | ---: | ---: |']
    for feature in features:
        mode = 'parse' if feature.startswith('parser.') else 'render'
        values = [index[feature, 'active', size, mode]['on_ns'] / index[feature, 'active', size, mode]['byte_count']
                  for size in (300, 4096, 65536)]
        lines.append(f"| `{feature}` | {values[0]:.3f} | {values[1]:.3f} | {values[2]:.3f} | {values[2]/values[1]:.2f}× |")
    (args.output / 'SCALING.md').write_text('\n'.join(lines) + '\n')
    print(args.output)


if __name__ == '__main__':
    main()
