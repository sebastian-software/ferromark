#!/usr/bin/env python3
"""Generate reproducible controlled Markdown workloads; no performance claims."""
import json
import pathlib
import sys

units = {'allow_html': '<div>Raw <em>HTML</em> content.</div>\n\n',
 'allow_link_refs': 'Read [guide {i}][r{i}].\n\n[r{i}]: /guide/{i} "Guide"\n\n',
 'tables': '| Name | Value |\n| --- | --- |\n| Alpha | Beta |\n\n',
 'merged_table_cells': '| Name | Value |\n| --- | --- |\n| Combined ||\n\n',
 'table_column_widths': '| Name | Value |\n| -- | ------ |\n| Alpha | Beta |\n\n',
 'strikethrough': 'Ordinary words and ~~removed words~~ at the end.\n\n',
 'highlight': 'Ordinary words and ==marked words== at the end.\n\n',
 'superscript': 'Ordinary words and x^2^ in a sentence.\n\n',
 'subscript': 'Ordinary words and H~2~O in a sentence.\n\n',
 'task_lists': '- [x] First task\n- [ ] Second task\n\n',
 'autolink_literals': 'See https://example.org/guide or www.example.org or '
                      'person@example.org.\n'
                      '\n',
 'disallowed_raw_html': '<script>example</script>\n\nOrdinary words.\n\n',
 'footnotes': 'A short note[^n{i}].\n\n[^n{i}]: A note with *emphasis*.\n\n',
 'inline_footnotes': 'A short note.^[A note with *emphasis*.]\n\n',
 'front_matter': '---\ntitle: Example\n---\n\nOrdinary words after metadata.\n\n',
 'heading_ids': '# Heading number {i}\n\nOrdinary words below.\n\n',
 'math': 'Ordinary words and $x^2 + y^2$ in a sentence.\n\n',
 'callouts': '> [!NOTE]\n> Ordinary words in a note.\n\n',
 'definition_lists': 'Term {i}\n: Definition with ordinary words.\n\n',
 'line_comments': '// Source only comment\nOrdinary visible words.\n\n',
 'indented_code_blocks': '    let value = 42;\n    value + 1\n\n',
 'link_base_path': 'Read [the guide](/guide) and [another page](/other).\n\n'}

plain = 'Ordinary words without extension syntax form a short paragraph.\n\n'

light = ('# Notes\n'
 '\n'
 'A short paragraph with **one important word**.\n'
 '\n'
 '- First item\n'
 '- Second item\n'
 '\n')

core = {'plain': 'Ordinary words without extension syntax form a short paragraph.\n\n',
 'headings': '# Heading\n\nOrdinary words.\n\n',
 'emphasis': 'Words with *emphasis* and **strong emphasis**.\n\n',
 'inline_code': 'Words with `let value = 42` as inline code.\n\n',
 'fenced_code': '```rust\nlet value = 42;\nvalue + 1\n```\n\n',
 'links': 'Read [the guide](/guide "Title") and [another page](/other).\n\n',
 'images': 'An ![image description](/image.png "Title") in a paragraph.\n\n',
 'entities': 'Words &amp; more &#65; words &lt; text &gt; then &quot;quotes&quot;.\n\n',
 'unordered_lists': '- First item\n- Second item\n- Third item\n\n',
 'nested_lists': '- First item\n  - Nested item\n    - Deep item\n\n',
 'blockquotes': '> Ordinary quoted words.\n> A second quoted line.\n\n',
 'soft_breaks': 'First line of words\nSecond line of words\nThird line of words\n\n'}

cases = []


def variant(label, text, options):
    return {"label": label, "input": text, "options": options}


def add(group, feature, size, variants):
    cases.append({
        "id": f"{group}/{feature}/{size}", "group": group,
        "feature": feature, "size": size, "variants": variants,
    })


def configs(feature):
    before = {"preset": "commonmark"}
    after = {"preset": "commonmark"}
    if feature in ["merged_table_cells", "table_column_widths"]:
        before["tables"] = after["tables"] = True
    if feature == "disallowed_raw_html":
        before["trusted"] = after["trusted"] = True
    before[feature] = None if feature == "link_base_path" else False
    after[feature] = "/docs" if feature == "link_base_path" else True
    return before, after


for feature, unit in units.items():
    before, after = configs(feature)
    for size, text in [
        ("tiny", "A short sentence."), ("1k", plain * 16), ("16k", plain * 256),
    ]:
        add("activation", feature, size, [
            variant("off", text, before), variant("on", text, after),
        ])
    for size, repeats in [("small", 1), ("medium", 16)]:
        text = "".join(unit.replace("{i}", str(i)) for i in range(repeats))
        if feature == "front_matter":
            text = unit + plain * (repeats - 1)
        add("syntax", feature, size, [
            variant("off", text, before), variant("on", text, after),
        ])

for feature, unit in core.items():
    for size, repeats in [("small", 1), ("medium", 32)]:
        text = unit * repeats
        add("core", feature, size, [
            variant("plain-control", "a" * len(text.encode()), {"preset": "commonmark"}),
            variant("syntax", text, {"preset": "commonmark"}),
        ])

for size, text in [
    ("empty", ""), ("tiny", "A short sentence."), ("plain-1k", plain * 16),
    ("light-1k", light * 12), ("plain-16k", plain * 256),
    ("readme", pathlib.Path("README.md").read_text()),
]:
    add("lifecycle", "presets", size, [
        variant(preset, text, {"preset": preset})
        for preset in ["commonmark", "minimal", "gfm", "default"]
    ])

path = pathlib.Path(sys.argv[1])
path.parent.mkdir(parents=True, exist_ok=True)
path.write_text(json.dumps(cases, indent=2) + "\n")
pairs = sum(len(case["variants"]) for case in cases)
print(f"{len(cases)} cases, {pairs} input/configuration pairs")
