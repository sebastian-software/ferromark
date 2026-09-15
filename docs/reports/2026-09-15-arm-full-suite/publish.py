#!/usr/bin/env python3
"""Publish the complete optimization-suite rerun without pooling its input sets."""
import gzip
import json
from pathlib import Path
import statistics

HERE = Path(__file__).resolve().parent


def read(name):
    path = HERE / name
    if path.exists():
        return json.loads(path.read_text())
    return json.loads(gzip.decompress(path.with_suffix(path.suffix + '.gz').read_bytes()))


def group(rows):
    result = {}
    for mode in ('fresh', 'reuse', 'parse', 'render'):
        selected = [r for r in rows if r['mode'] == mode]
        result[mode] = {
            'cases': len(selected),
            'geomean': statistics.geometric_mean(r['baseline_over_candidate_median'] for r in selected),
            'round_geomeans': [statistics.geometric_mean(r['round_medians'][i] for r in selected) for i in range(3)],
            'below_one': sum(r['baseline_over_candidate_median'] < 1 for r in selected),
            'worst': sorted(selected, key=lambda r: r['baseline_over_candidate_median'])[:5],
        }
    return result


def main():
    corpus = {c['name']: c for c in read('corpus.json')['cases']}
    rows = read('summary.json')
    run = read('run.json')
    assert len(corpus) == 207 and len(rows) == 207 * 4
    groups = {suite: group([r for r in rows if corpus[r['case']]['suite'] == suite])
              for suite in ('broad', 'autolink-broad', 'diagnostic')}
    labels = {'broad': 'Original broad documents', 'autolink-broad': 'Renderer-autolink replays', 'diagnostic': 'Authored diagnostics'}
    table = ['| Input set | N | Fresh | Reuse | Parse | Render |', '| --- | ---: | ---: | ---: | ---: | ---: |']
    for suite, modes in groups.items():
        table.append(f"| {labels[suite]} | {modes['fresh']['cases']} | " +
                     ' | '.join(f"{modes[m]['geomean']:.3f}×" for m in modes) + ' |')
    all_tables = ['# Complete optimization-suite results', '', '\n'.join(table), '',
                  'Baseline time / candidate time; higher is faster. The three input sets remain separate.', '',
                  '## Round agreement and measured losses', '']
    for suite, modes in groups.items():
        for mode, value in modes.items():
            rounds = ', '.join(f'{x:.3f}×' for x in value['round_geomeans'])
            worst = value['worst'][0]
            all_tables.append(f"- {labels[suite]}, {mode}: rounds {rounds}; {value['below_one']} case medians below 1.000×; "
                              f"lowest `{worst['case']}` at {worst['baseline_over_candidate_median']:.3f}×.")
    all_tables += ['', '## Every case and stage', '', '| Case | Set | Profile | Stage | Baseline ns | Candidate ns | Speedup |',
                   '| --- | --- | --- | --- | ---: | ---: | ---: |']
    for row in rows:
        all_tables.append(f"| {row['case']} | {corpus[row['case']]['suite']} | {row['profile']} | {row['mode']} | "
                          f"{row['baseline_ns_per_document']:.1f} | {row['candidate_ns_per_document']:.1f} | "
                          f"{row['baseline_over_candidate_median']:.3f}× |")
    (HERE / 'TABLES.md').write_text('\n'.join(all_tables) + '\n')
    (HERE / 'aggregates.json').write_text(json.dumps(groups, indent=2) + '\n')
    windows = len(rows) * run['rounds'] * run['pairs_per_round'] * 2
    broad = groups['broad']
    worst = min(rows, key=lambda r: r['baseline_over_candidate_median'])
    report = f'''# Complete Apple Silicon optimization-suite rerun — 2026-09-15

The finished performance branch at **`e93394e`**, including both dense-escape
fixes, is compared with main **`a7f0a00`** on all **207 cases and four stages**.
No production code changed during this rerun. The 57 broad documents, their
57 renderer-autolink replays, and 93 authored diagnostics are summarized
separately; overlapping views are not independent population samples.

**Speedup = baseline time / candidate time; higher is faster.**

{chr(10).join(table)}

On the original broad set, the finished branch runs at
{broad['fresh']['geomean']:.3f}× baseline throughput fresh,
{broad['reuse']['geomean']:.3f}× with reuse,
{broad['parse']['geomean']:.3f}× in parsing, and
{broad['render']['geomean']:.3f}× in rendering.
[Every case, round summary, and measured loss](TABLES.md) remains visible;
the aggregate is not a claim of a win on every diagnostic.

All 57 original documents improve in fresh, reuse, and parse medians. In
render-only measurements, {broad['render']['below_one']} original documents have
medians below 1.000×; the weakest is
`{broad['render']['worst'][0]['case']}` at
{broad['render']['worst'][0]['baseline_over_candidate_median']:.3f}×.
Across all sets and stages, the lowest median is `{worst['case']}`
in {worst['mode']} at {worst['baseline_over_candidate_median']:.3f}×
({(1 / worst['baseline_over_candidate_median'] - 1) * 100:.1f}% more time).
These small measured losses are retained, without asserting statistical
significance or attributing them to a proven mechanism.

## Method and verification

The unchanged optimization worker uses Rust 1.95, generic AArch64, fat LTO,
one codegen unit, and its standalone allocator environment. The preserved
`build-final2` binaries were verified against their hashes and frozen source
trees before reuse. The candidate was built at `639c79b`; all production and
test sources exactly match `e93394e`, whose only subsequent change was the
original iteration report. The baseline exactly matches `a7f0a00`.

This is a fresh timing run of those audited artifacts, not a new compiler build
or an extrapolation from the earlier 102-case review. It used three process
rounds, three alternating pairs per round, 40 ms timing windows and 40 ms
warmups. No native benchmark, compiler, tests, or archive compression ran
concurrently. Host, power, load, and thermal probe output is in `run.json`.
The macOS thermal probe returned unavailable telemetry; no absence of
throttling is inferred from that result.

Exact HTML, full AST Debug including source spans, and child counts agree
between implementations and stages on all 207 inputs. Every before/after
verification and all **{windows:,} timing-window checksums** passed.
The [shared validation logs](../2026-09-15-native-arm/checks/commands.json)
record 788 passing workspace tests, formatting, Clippy, benchmark compilation,
and both harness test suites before timing.

Profiles retain this harness's original semantics: CommonMark parsing, GFM
with footnotes off, MDX, selected extensions, option-bit diagnostics, and
renderer-autolink diagnostics. Its HTML renderer retains heading IDs and
inline TOC defaults. These settings differ from the separately matched
[six-engine native comparison](../2026-09-15-native-arm/README.md).
Do not combine ratios or absolute timings between those two build environments.
Line comments/frontmatter are not enabled by this worker's profiles; their
correctness remains covered by the workspace suite.

The [original iteration report](../2026-09-15-arm-iterations/README.md) retains
the individual attempts and rejected variants. This report supplies the full
confirmation after both final fixes. `ParseError`'s representation change is
an accepted v2 API change; no compatibility adapter changes timed behavior.

## Artifacts and reproduction

- [Full tables](TABLES.md), [aggregates](aggregates.json), [run configuration](run.json).
- [Inputs and attribution](corpus.json.gz), [all equality checks](verification.json.gz),
  [raw timing windows](samples.json.gz), and [arena capacities](arena-capacities.json.gz).
- [Build provenance](build.json), [source audit](source-audit.json), and the exact
  [worker](harness/worker.rs). `build/` retains Cargo manifests, locks, and logs.
- [Archived-data audit](artifact-audit.json); `python3 audit.py` rechecks all input
  hashes, exact equality, timing checksums, and complete case/stage coverage.

For a new build, export `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, and
`crates/` from `a7f0a00` and `e93394e` to separate clean temporary directories.
The archived `harness/prepare.py` pins its baseline to `a7f0a00`; this is the
only adjustment from the existing preparation script. The worker and runner
are unchanged. Run from this report directory:

```sh
python3 harness/prepare.py --baseline-path /private/tmp/arm-replay-main \\
  --candidate-path /private/tmp/arm-replay-branch --out /private/tmp/arm-replay-build
python3 harness/run.py /private/tmp/arm-replay-build corpus.json.gz \\
  /private/tmp/arm-replay-results --rounds 3 --pairs 3 --window-ms 40
```

The output directory must be new. `python3 publish.py` regenerates this report's
tables and summary from its archived data. `SHA256SUMS` covers the report;
input attribution and licenses remain unchanged.
'''
    (HERE / 'README.md').write_text(report)
    print(json.dumps({s: {m: v['geomean'] for m, v in values.items()} for s, values in groups.items()}, indent=2))


if __name__ == '__main__':
    main()
