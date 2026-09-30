#!/usr/bin/env python3
"""Generate the report and website section from the archived measurements."""
import argparse
import json
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
REPORT = REPO / 'docs/reports/2026-09-30-markdown-ecosystem-main'
FIGURES = REPO / 'homepage/app/data/markdown-ecosystem-benchmarks.json'
GUIDE = REPO / 'homepage/app/routes/guide/benchmarks.mdx'
MARKER = '\n## markdown-rs and micromark\n'
NODE_PAIRS = (('marked', 'marked'), ('markdown-it', 'markdown-it'),
              ('remark', 'remark / unified'), ('showdown', 'Showdown'),
              ('commonmark', 'commonmark.js'))


def original_content():
    results = {track: json.loads((REPORT / track / 'summary.json').read_text()) for track in ('native', 'node')}
    records = {track: json.loads((REPORT / track / 'run.json').read_text()) for track in results}
    text = MARKER + '''
Two separate pairs extend the frozen 57-document corpus comparison. These are
new measurements of the recorded main revision, with no core changes. Their runtime and
allocator settings differ from the archived six-engine comparison; the results are not pooled. **Values below are Ferromark throughput
relative to the competitor: higher is faster.**

| Track | Competitor | Agreeing documents | Fresh | Reuse |
| --- | --- | ---: | ---: | ---: |
'''
    for track, competitor in (('native', 'markdown-rs 1.0.0'), ('node', 'micromark 4.0.2')):
        result = results[track]
        speed = result['v2_relative_throughput']
        text += f"| {track.title()} | {competitor} | {result['agreeing_documents']}/{result['documents']} | {speed['fresh']:.2f}× | {speed['reuse']:.2f}× |\n"
    text += f'''
The native pair uses {records['native']['runtime']}, a shared system allocator,
optimization level 3, fat LTO, one codegen unit, and a generic CPU target.
The Node pair uses Node {records['node']['runtime']}, Ferromark's local addon built
with the release-node profile without PGO, and both public JavaScript string APIs.
It includes wrapper/N-API conversion costs and JavaScript GC. The Node result
describes application calls, not isolated language overhead.

CommonMark disables optional syntax. The extension lane enables only tables,
strikethrough and task lists, with raw HTML passthrough. It is not full GFM;
MDX, math, frontmatter, bare URL autolinking, footnotes, heading IDs and other
renderer extras are off. The competitors expose no retained parser/output API:
their reuse column retains only configuration and makes fresh HTML calls.

Every input is checked in both lifecycles before timing and again around timed
windows. The score uses only exact or serialization-equivalent HTML pairs.
Micromark differs on one Vite document's code-block line breaks; that document
remains a measured diagnostic and is excluded from the score. Code whitespace
is not normalized away. Three process rounds with six rotating engine-order
windows per round use equal-document geometric means. These are measurements
on a shared macOS arm64 workstation, not a universal ranking or a significance
claim. MDX and AST capabilities belong in the separate feature inventory.

[Raw results, source hashes, and output differences](https://github.com/sebastian-software/ferromark/blob/main/docs/reports/2026-09-30-markdown-ecosystem-main/README.md),
[reproduction commands](https://github.com/sebastian-software/ferromark/blob/main/benchmarks/markdown-ecosystem/README.md).
'''
    return text


def node_content():
    text = "\n## Node.js ecosystem coverage\n\n"
    text += "Five further public Markdown-to-HTML APIs use the same frozen 57 inputs.\n"
    text += "Each is independently paired with Ferromark's local Node addon without PGO.\n"
    text += "The core and addon source hashes match the earlier micromark run; the adapter\n"
    text += "commit is recorded separately. Ratios use only each pair's agreeing documents,\n"
    text += "so these rows do not establish a shared-set ranking.\n\n"
    text += "| Project | Version | Agreeing documents | Fresh | Reuse |\n"
    text += "| --- | --- | ---: | ---: | ---: |\n"
    for engine, label in NODE_PAIRS:
        folder = REPORT / ('node-' + engine)
        result = json.loads((folder / 'summary.json').read_text())
        lock = json.loads((folder / 'package-lock.json').read_text())
        version = lock['packages']['node_modules/' + engine]['version']
        speed = result['v2_relative_throughput']
        text += f"| {label} | {version} | {result['agreeing_documents']}/{result['documents']} | {speed['fresh']:.2f}× | {speed['reuse']:.2f}× |\n"
    text += """
commonmark.js runs **CommonMark only in both engines on all inputs**, because
its public parser has no GFM extension lane. All other pairs use the frozen
per-document CommonMark or tables/strikethrough/task-list profile. marked uses
its public URL-tokenizer override to disable literal autolinking. markdown-it
keeps its native `<s>` spelling and task-list plugin classes; remark includes
remark-rehype and rehype-stringify, with only the three matched syntax extensions.
Showdown keeps its own Markdown dialect, extra fenced-code classes, and task-list
styles. Its smaller agreement set also includes differences in custom HTML and
Markdown parsing; the retained outputs show each excluded document. Heading IDs,
metadata, ellipsis conversion, and optional renderer extras are disabled.

All pairs retain configured public parser/processor objects outside timing;
each call parses and returns a new JavaScript HTML string. Both Ferromark
lifecycles are verified before timing and around every process's windows.
Three process rounds, six alternating windows, 40 ms timing windows, and 60 ms
warmup use the same median/geometric-mean aggregation as the micromark pair.
The strict output classifier is unchanged; differing text, tags, task classes,
styles, code whitespace, and other attributes are not discarded to improve
agreement. Disagreements remain timed diagnostics and are excluded from ratios.

These macOS arm64 workstation results describe these inputs and build profiles;
Linux cells stay unmeasured. Configuration, agreement counts, raw windows,
locks, source hashes, and adapter snapshots are retained in the report below.
[Reproduction and adapter contracts](https://github.com/sebastian-software/ferromark/blob/main/benchmarks/markdown-ecosystem/README.md),
[project coverage and framework integrations](https://ferromark.dev/guide/feature-comparison#nodejs-projects).
"""
    return text


def content():
    return original_content() + node_content()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--check', action='store_true')
    args = parser.parse_args()
    section = content()
    report = '# Markdown ecosystem comparison — 2026-09-30\n' + section + '''
## Evidence

- Native: [run metadata](native/run.json), [aggregate](native/summary.json),
  [outputs](native/verification.json.gz), [windows](native/samples.json.gz),
  [option guards](native/behavior.json.gz), [corpus](native/corpus.json.gz),
  [registry lock](native/Cargo.lock).
- Node: [run metadata](node/run.json), [aggregate](node/summary.json),
  [outputs](node/verification.json.gz), [windows](node/samples.json.gz),
  [option guards](node/behavior.json.gz), [corpus](node/corpus.json.gz),
  [npm lock](node/package-lock.json).

The retained corpus includes every original input, byte length and SHA-256.
The run metadata records the binary/addon hashes, the measured core commit,
local Git status and source hashes. The local native and Node builds are separate from published PGO release artifacts.
Input attribution remains in the [broad corpus](../../../benchmarks/broad-comparison/README.md).

Regenerate this report and website section with
`python3 benchmarks/markdown-ecosystem/publish.py`; use `--check` to verify them.
The older native report and homepage headline numbers remain historical evidence.
'''
    for engine, label in NODE_PAIRS:
        folder = 'node-' + engine
        revision = json.loads((REPORT / folder / 'run.json').read_text())['git_head']
        report += f"\n- {label}: [metadata]({folder}/run.json), [aggregate]({folder}/summary.json), "
        report += f"[outputs]({folder}/verification.json.gz), [windows]({folder}/samples.json.gz), "
        report += f"[option guards]({folder}/behavior.json.gz), [corpus]({folder}/corpus.json.gz), "
        report += f"[npm lock]({folder}/package-lock.json), [adapter sources](node-adapters/{revision}/).\n"
    results = {track: json.loads((REPORT / track / 'summary.json').read_text()) for track in ('native', 'node')}
    records = {track: json.loads((REPORT / track / 'run.json').read_text()) for track in results}
    figures = {
        'measured': '2026-09-30',
        'report': str(REPORT.relative_to(REPO)),
        'figures': [{
            'id': competitor, 'label': competitor, 'runtime': runtime,
            'profileScope': 'CommonMark or tables/strikethrough/tasks',
            'documents': results[track]['agreeing_documents'],
            'corpusDocuments': results[track]['documents'],
            'fresh': results[track]['v2_relative_throughput']['fresh'],
            'reuse': results[track]['v2_relative_throughput']['reuse'],
            'revision': records[track]['git_head'][:8],
        } for track, competitor, runtime in (
            ('native', 'markdown-rs', 'Native'), ('node', 'micromark', 'Node.js'))],
    }
    for engine, label in NODE_PAIRS:
        folder = REPORT / ('node-' + engine)
        result = json.loads((folder / 'summary.json').read_text())
        record = json.loads((folder / 'run.json').read_text())
        figures['figures'].append({
            'id': engine, 'label': label, 'runtime': 'Node.js',
            'profileScope': 'CommonMark only' if engine == 'commonmark' else 'CommonMark or tables/strikethrough/tasks',
            'documents': result['agreeing_documents'], 'corpusDocuments': result['documents'],
            'fresh': result['v2_relative_throughput']['fresh'],
            'reuse': result['v2_relative_throughput']['reuse'], 'revision': record['git_head'][:8],
        })
    import publish_values
    completed = publish_values.content() if publish_values.FIGURES.exists() else ''
    original = GUIDE.read_text()
    base = original.split(MARKER)[0].rstrip() + '\n'
    for path, expected in ((REPORT / 'README.md', report), (GUIDE, base + section + completed + publish_values.current_content()), (FIGURES, json.dumps(figures, indent=2) + '\n')):
        if args.check:
            if path.read_text() != expected:
                raise SystemExit(f'stale generated content: {path}')
        else:
            path.write_text(expected)


if __name__ == '__main__':
    main()
