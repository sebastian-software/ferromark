#!/usr/bin/env python3
"""Publish the completed ecosystem platform results from validated raw archives."""
import argparse
import hashlib
import json
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
REPORT = REPO / 'docs/reports/2026-09-30-ecosystem-platforms'
FIGURES = REPO / 'homepage/app/data/benchmark-platform-values.json'
PAIRS = [('native', 'comrak', 'Comrak'), ('native', 'cmark', 'cmark'), ('native', 'cmark-gfm', 'cmark-gfm'),
         ('native', 'markdown-rs', 'markdown-rs'), ('node', 'micromark', 'micromark'),
         ('node', 'marked', 'marked'), ('node', 'markdown-it', 'markdown-it'),
         ('node', 'remark', 'remark / unified'), ('node', 'showdown', 'Showdown'), ('node', 'commonmark', 'commonmark.js')]


def figures():
    values = []
    for platform, label in [('macos-arm64', 'macOS arm64'), ('linux-x86-64', 'Linux x86-64')]:
        for track, engine, name in PAIRS:
            if platform == 'macos-arm64' and engine not in ('comrak', 'cmark', 'cmark-gfm'):
                continue
            folder = REPORT / platform / (track + '-' + engine)
            result = json.loads((folder / 'summary.json').read_text())
            config = json.loads((folder / 'run.json').read_text())
            values.append({'id': engine, 'label': name, 'runtime': 'Native' if track == 'native' else 'Node.js',
                           'platform': platform, 'platformLabel': label, 'machine': (REPORT / 'macos-host.txt').read_text().strip() if platform == 'macos-arm64' else config['host_before']['cpu'],
                           'measured': config['host_before']['time_utc'][:10], 'revision': config['git_head'][:8],
                           'report': str(folder.relative_to(REPO)), 'profileScope': config['profile_scope'],
                           'documents': result['agreeing_documents'], 'corpusDocuments': result['documents'],
                           'fresh': result['v2_relative_throughput']['fresh'], 'reuse': result['v2_relative_throughput']['reuse']})
    return values


def manual_workflow():
    import importlib.util
    path = REPO / 'benchmarks/manual-comparison/cli.py'
    spec = importlib.util.spec_from_file_location('manual_comparison', path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def current_figures():
    historical = figures()
    pointer = json.loads((REPO / 'benchmarks/manual-comparison/current.json').read_text())
    current = manual_workflow().active() if pointer['reports'] else []
    return merge_figures(historical, current)


def merge_figures(historical, current):
    keys = {(value['platform'], value['id']) for value in current}
    assert len(keys) == len(current), 'duplicate current platform values'
    return [value for value in historical if (value['platform'], value['id']) not in keys] + current


def current_content():
    pointer = json.loads((REPO / 'benchmarks/manual-comparison/current.json').read_text())
    if not pointer['reports']:
        return ''
    workflow = manual_workflow()
    values = workflow.active()
    if not values:
        return ''
    hosts = []
    for name in pointer['reports'].values():
        suite = workflow.read(workflow.report_path(name), 'suite')
        context = suite.get('runner_context')
        if context:
            observed = context['observed']
            hosts.append(f"{suite['platform_label']}: {context['provider']} VM, "
                         f"{observed['cpus']} vCPU, {observed['memory_bytes'] / 2**30:.1f} GiB observed memory "
                         f"([workflow]({context['run_url']})).")
        else:
            hosts.append(f"{suite['platform_label']}: {suite['machine']}.")
    return '\n'.join(hosts) + '\n\n' + workflow.content(values)


def content(values=None):
    values = figures() if values is None else values
    text = '\n## Completed native and Linux comparisons\n\n'
    text += 'These independently measured pairs fill the remaining homepage cells. The\n'
    text += 'Ferromark core source hashes match the earlier clean-core ecosystem runs.\n'
    text += '**Factors mean Ferromark throughput relative to the library.** Each row uses\n'
    text += 'its own equivalent-HTML set; rows do not establish a shared-set ranking.\n\n'
    text += '| Platform | Project | Equivalent documents | Fresh | Reuse |\n| --- | --- | ---: | ---: | ---: |\n'
    for value in values:
        text += f"| {value['platformLabel']} | {value['label']} | {value['documents']}/57 | {value['fresh']:.2f}× | {value['reuse']:.2f}× |\n"
    text += '''
cmark 0.31.2 uses CommonMark only in **both** engines for every document, as
commonmark.js does. Comrak 0.55.0 and cmark-gfm 0.29.0.gfm.13 enable only tables,
strikethrough, and task lists on the extension inputs. All use their public
Markdown-to-HTML APIs and preserve raw HTML. The C adapters call native library
functions directly; no CLI startup or IPC is timed. cmark's convenience call
and cmark-gfm's parser/feed/finish/render sequence own fresh parser/AST/output
state in both schedules. C HTML consumption includes `strlen` before freeing
the returned allocation. Comrak uses `markdown_to_html`, also with fresh owned
AST/output. Reuse retains configuration in these competitors; Ferromark retains
its arena and renderer. AST and output destruction remain within native timing.

The native pairs use system malloc, Rust 1.95.0, generic CPU, optimization level
3, fat LTO, one codegen unit, and panic abort. C libraries/adapters use clang
`-O3`, system malloc, and no PGO; no allocator redirection or cross-language LTO.
The Node pairs use Node 24.21.0 and the release-node addon without PGO, including
public binding/string conversion costs and JavaScript GC.

Every pair has three process rounds, six rotating 40 ms windows and 60 ms warmup,
with verification before timing and around each round. All 57 inputs, exact
outputs, option guards, 2,052 paired windows, locks, source hashes, and committed
adapter snapshots are retained. Only exact or conservative serialization-
equivalent HTML contributes to factors; code whitespace, attributes, task-list
markup, and text differences remain significant.

macOS runs use a shared Apple M1 Ultra workstation. Linux pairs each run on their
own GitHub-hosted Ubuntu 24.04 x86-64 VM; the pair alternates on one VM, but the
CPU/host can differ between pairs. Metadata records CPU, load, clocks, thermal
observations and hypervisor steal counters. No other workflow step runs during
timing. These are separate steady-state measurements, with no significance
claim or comparison of absolute durations between hosts.

[Raw evidence and per-pair host metadata](https://github.com/sebastian-software/ferromark/tree/main/docs/reports/2026-09-30-ecosystem-platforms),
[reproduction](https://github.com/sebastian-software/ferromark/blob/main/benchmarks/markdown-ecosystem/README.md).
'''
    return text


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--check', action='store_true')
    args = p.parse_args()
    values = figures()
    readme = '# Completed ecosystem comparisons — 2026-09-30\n' + content(values) + '\n## Evidence\n\n'
    for value in values:
        folder = str(Path(value['report']).relative_to(REPORT.relative_to(REPO)))
        readme += f"- {value['platformLabel']} / {value['label']}: [metadata]({folder}/run.json), [aggregate]({folder}/summary.json), [outputs]({folder}/verification.json.gz), [windows]({folder}/samples.json.gz), [guards]({folder}/behavior.json.gz), [corpus]({folder}/corpus.json.gz), [adapters]({folder}/adapters/).\n"
    readme += '\nLinux source: [successful workflow](https://github.com/sebastian-software/ferromark/actions/runs/36702822137); [origin metadata](origin.json). The macOS hardware identifier is retained in [macos-host.txt](macos-host.txt). [SHA256SUMS](SHA256SUMS) covers the complete report archive.\n'
    readme += '\nInput attribution remains in the [frozen broad corpus](../../../benchmarks/broad-comparison/README.md). Original measurements are preserved separately. Regenerate with `python3 benchmarks/markdown-ecosystem/publish_values.py`.\n'
    for path, expected in [(REPORT / 'README.md', readme), (FIGURES, json.dumps({'figures': current_figures()}, indent=2) + '\n')]:
        if args.check:
            assert path.read_text() == expected, f'stale generated content: {path}'
        else:
            path.write_text(expected)
    manifest = ''.join(f'{hashlib.sha256(path.read_bytes()).hexdigest()}  {path.relative_to(REPORT)}\n'
                       for path in sorted(REPORT.rglob('*')) if path.is_file() and path.name != 'SHA256SUMS')
    checksums = REPORT / 'SHA256SUMS'
    if args.check:
        assert checksums.read_text() == manifest, 'report checksum drift'
    else:
        checksums.write_text(manifest)


if __name__ == '__main__':
    main()
