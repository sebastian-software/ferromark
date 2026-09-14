#!/usr/bin/env python3
"""Predeclare feature probes and use-case candidates without consulting timings."""

import argparse
import gzip
import hashlib
import importlib.util
import json
import math
from pathlib import Path

HERE = Path(__file__).resolve().parent
SIZES = (300, 4096, 65536)
PROSE = ('A small document can describe an observation and give the reader enough context. '
         'The next paragraph explains the result in ordinary words without special notation.\n\n')
GFM = dict(gfm=True, tables=True, task_lists=True, strikethrough=True, autolinks=True)
TABLE = '| Name | Value |\n| --- | --- |\n| Alpha | **One** |\n\n'
HEADINGS = '# Heading *one*\n\nA short paragraph.\n\n## Heading two\n\nAnother paragraph.\n\n'
FENCE = '```rust [example.rs] {1} :line-numbers\nlet answer = 42;\nprintln!("{answer}");\n```\n\n'


def config(parser=None, renderer=None):
    # Unspecified fields use the explicit CommonMark profiles in worker.rs.
    return {'parser': parser or {}, 'renderer': renderer or {}}


def feature(name, snippet, parser=None, renderer=None, value=True, effect='html'):
    side, field = name.split('.')
    off = config(parser, renderer)
    on = {key: dict(fields) for key, fields in off.items()}
    on[side][field] = value
    return dict(feature=name, snippet=snippet, off=off, on=on, effect=effect)


FEATURES = [
    feature('parser.footnotes', 'A claim[^note].\n\n[^note]: Supporting **detail**.\n\n'),
    feature('parser.task_lists', '- [x] Finished work\n- [ ] Remaining work\n\n'),
    feature('parser.tables', TABLE),
    feature('parser.merged_table_cells', '| A | B | C |\n| --- | --- | --- |\n| Wide || Cell |\n\n', parser={'tables': True}),
    feature('parser.table_attributes', TABLE.rstrip() + '\n: Caption {#sample .wide}\n\n', parser={'tables': True}),
    feature('parser.line_comments', 'First line.\n// Private source note\nSecond line.\n\n'),
    feature('parser.front_matter', '---\ntitle: A document\ntags: [one, two]\n---\n\n'),
    feature('parser.strikethrough', 'We ~~removed this~~ but kept ~that~.\n\n'),
    feature('parser.autolinks', 'Visit https://example.org/path and www.example.net or mail person@example.org.\n\n'),
    feature('parser.superscript', 'The formula x^2^ has a small exponent.\n\n'),
    feature('parser.subscript', 'The formula H~2~O has a small index.\n\n'),
    feature('parser.math', 'Use $x^2 + y^2$ here.\n\n$$\nx + y = z\n$$\n\n'),
    feature('parser.definition_lists', 'Term\n: A **formatted** definition.\n\n'),
    feature('parser.heading_attributes', '# A heading {#custom .wide}\n\nBody.\n\n', renderer={'heading_ids': True}),
    feature('parser.wiki_links', 'Read [[Page|the page]] and [[Another page]].\n\n'),
    feature('parser.cjk_emphasis', 'これは**強調。**です。次は*項目、*です。\n\n'),
    feature('parser.mdx', '<Widget title="Example">\n\n**Content** and {value}.\n\n</Widget>\n\n'),
    feature('renderer.xhtml', 'A hard break.  \nNext line.\n\n---\n\n![Alt](/image.png)\n\n'),
    feature('renderer.sanitize', '<div>Raw <b>markup</b> & content.</div>\n\n[Link](https://example.org)\n\n'),
    feature('renderer.disallow_raw_html', '<script>example()</script>\n\n<div>Allowed HTML</div>\n\n'),
    feature('renderer.convert_md_links', '[Read more](./guide.md#section) and [index](../index.md).\n\n'),
    feature('renderer.code_fence_metadata', '```rust:line-numbers\nlet answer = 42;\n```\n\n'),
    feature('renderer.code_annotations', '```rust annotate="highlight:1"\nlet value = 42;\n```\n\n', renderer={'code_fence_metadata': True}),
    feature('renderer.code_annotation_default_line_numbers', '```rust\nlet answer = 42;\n```\n\n', renderer={'code_fence_metadata': True, 'code_annotations': True, 'code_annotation_syntax': 'vitepress'}),
    feature('renderer.autolink_urls', 'Visit https://example.org/path and http://example.net today.\n\n'),
    feature('renderer.autolink_target_blank', 'Visit https://example.org/path today.\n\n', renderer={'autolink_urls': True}),
    feature('renderer.link_target_blank', '[Read more](https://example.org/page) and [local](/page).\n\n'),
    feature('renderer.semantic_footnotes', 'A claim[^note].\n\n[^note]: Supporting **detail**.\n\n', parser={'footnotes': True}),
    feature('renderer.heading_permalinks', HEADINGS, renderer={'heading_ids': True}),
    feature('renderer.source_spans', HEADINGS + '- One\n- Two\n\n'),
    feature('renderer.heading_ids', HEADINGS),
    feature('renderer.callouts', '> [!NOTE]\n> A useful **note**.\n\n'),
    feature('renderer.inline_toc', '[[toc]]\n\n' + HEADINGS, renderer={'heading_ids': True}),
    feature('renderer.table_colgroup', TABLE, parser={'tables': True}),
    feature('renderer.table_column_names', TABLE, parser={'tables': True}, renderer={'table_colgroup': True}),
    feature('renderer.highlight', FENCE, effect='none'),
    feature('renderer.soft_break', 'First soft line.\nSecond soft line.\n\n', value=' ', effect='none'),
]

# These are measurement recipes, not new library presets. Every profile keeps
# the nesting bound; differences in output are recorded, never called speedups.
PROFILES = {
    'commonmark': config(),
    'gfm-spec': config(GFM, {'disallow_raw_html': True}),
    'comments': config(GFM, {'sanitize': True}),
    'article': config({'footnotes': True, 'front_matter': True, 'heading_attributes': True},
                      {'heading_ids': True, 'semantic_footnotes': True}),
    'docs': config(GFM | {'footnotes': True, 'front_matter': True, 'line_comments': True,
                         'heading_attributes': True},
                   {'disallow_raw_html': True, 'heading_ids': True, 'inline_toc': True,
                    'callouts': True, 'code_fence_metadata': True, 'semantic_footnotes': True}),
}
PROFILES['mdx-docs'] = config(PROFILES['docs']['parser'] | {'mdx': True}, dict(PROFILES['docs']['renderer']))
PROFILES['kitchen-sink'] = config(
    {field: True for field in ('gfm footnotes task_lists tables merged_table_cells table_attributes '
                              'line_comments front_matter strikethrough autolinks superscript subscript '
                              'math definition_lists heading_attributes wiki_links cjk_emphasis mdx').split()},
    {'disallow_raw_html': True, 'heading_ids': True, 'heading_permalinks': True,
     'inline_toc': True, 'callouts': True, 'code_fence_metadata': True, 'code_annotations': True,
     'semantic_footnotes': True, 'table_colgroup': True, 'table_column_names': True,
     'source_spans': True, 'autolink_urls': True})


def scaled(snippet, target):
    return snippet * max(1, math.ceil(target / len(snippet.encode())))


def real_cases():
    spec = importlib.util.spec_from_file_location('broad_cases', HERE.parent / 'broad-comparison/make_corpus.py')
    broad = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(broad)
    cases = [dict(name='comment-' + name, input=broad.COMMENTS[name], category='comments',
                  origin={'kind': 'authored-example', 'license': 'MIT'})
             for name in ('ack', 'review', 'table', 'incident')]
    selected = {'rust-book-ch03-04-comments', 'legacy-docs-markdown-extensions',
                'legacy-node-ferromark-readme', 'vite-docs-api-plugin',
                'typescript-handbook-compiler-options', 'wiki-rainbow-first-paragraph',
                'wiki-tea-lead', 'wiki-tea-plain-prose', 'wiki-chess-article-body'}
    for name in ('local-cases.json.gz', 'wiki-cases.json.gz'):
        data = json.loads(gzip.decompress((HERE.parent / 'broad-comparison' / name).read_bytes()))
        cases.extend(c for c in data['cases'] if c['name'] in selected)
    assert len(cases) == 13
    return cases


def make_cases():
    cases = []
    for f in FEATURES:
        for size in SIZES:
            for workload in ('plain', 'active'):
                source = scaled(PROSE if workload == 'plain' else f['snippet'], size)
                if workload == 'active' and f['feature'] == 'parser.front_matter':
                    source = f['snippet'] + scaled(PROSE, size)
                if workload == 'active' and f['feature'] == 'renderer.inline_toc':
                    source = '[[toc]]\n\n' + scaled(HEADINGS, size)
                cases.append(dict(name=f"{f['feature']}-{workload}-{size}", group='feature',
                                  feature=f['feature'], workload=workload, target_bytes=size,
                                  input=source, off=f['off'], on=f['on'],
                                  expected_effect=f['effect'] if workload == 'active' else None,
                                  origin={'kind': 'repeated-synthetic-probe', 'license': 'MIT'}))
    for size in SIZES:
        # A valid opener without closer must scan then preserve ordinary Markdown.
        f = next(f for f in FEATURES if f['feature'] == 'parser.front_matter')
        cases.append(dict(name=f'front-matter-unclosed-{size}', group='diagnostic',
                          feature=f['feature'], workload='unclosed', target_bytes=size,
                          input='---\n' + scaled(PROSE, size), off=f['off'], on=f['on'],
                          expected_effect='none', origin={'kind': 'synthetic-probe', 'license': 'MIT'}))
    for c in real_cases():
        # MDX is deliberately excluded for arbitrary non-MDX corpus inputs.
        for name in ('commonmark', 'gfm-spec', 'comments', 'article', 'docs'):
            cases.append(dict(name=f"{name}--{c['name']}", group='profile', feature=name,
                              workload=c['category'], document=c['name'], input=c['input'],
                              off=PROFILES['commonmark'], on=PROFILES[name], expected_effect=None,
                              origin=c['origin']))
    # Same-output ablations establish the cost of irrelevant enabled features.
    for use_case, source in [
        ('comments', scaled('Please check **this change** and [the guide](/guide).\n\n- [x] Tested\n\n', 300)),
        ('article', '---\ntitle: Article\n---\n\n' + scaled(HEADINGS + PROSE, 4096)),
        ('docs', '---\ntitle: Docs\n---\n\n[[toc]]\n\n' + scaled(HEADINGS + TABLE + FENCE, 65536)),
        ('mdx-docs', '---\ntitle: Components\n---\n\n' + scaled('<Widget>\n\n**Content** and {value}.\n\n</Widget>\n\n', 4096)),
    ]:
        tailored = PROFILES[use_case]
        broad = config(PROFILES['kitchen-sink']['parser'], dict(tailored['renderer']))
        if use_case == 'docs':
            # Wiki-link parsing would consume [[toc]], changing required output.
            broad['parser'] = broad['parser'] | {'wiki_links': False}
        cases.append(dict(name=f'tailored-{use_case}', group='ablation', feature=use_case,
                          workload='syntax-absent-extras', input=source, off=broad, on=tailored,
                          expected_effect='none', origin={'kind': 'synthetic-use-case', 'license': 'MIT'}))
    for case in cases:
        raw = case['input'].encode()
        case.update(byte_count=len(raw), sha256=hashlib.sha256(raw).hexdigest())
    return {'schema': 1, 'profiles': PROFILES, 'cases': cases}


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output', type=Path)
    args = parser.parse_args()
    data = make_cases()
    args.output.write_text(json.dumps(data, indent=2, ensure_ascii=False) + '\n')
    print(f"{len(data['cases'])} cases, {len(FEATURES)} feature toggles")
