#!/usr/bin/env python3
"""Authored stress inputs for output-preserving parser/renderer refactoring."""
import hashlib
import json
import sys


def corpus():
    extended = dict(mdx=True, math=True, definition_lists=True, highlight=True,
        inline_footnotes=True, footnotes=True, tables=True, table_attributes=True,
        strikethrough=True, superscript=True, subscript=True)
    profiles = {'extended': {'parser': extended, 'renderer': {}}}
    bodies = {
        'nested-spans': '> - Héllo **bold** and [link](/path).\n>   - Nested ==mark==.\n\n',
        'jsx-spans': '<Outer>\n\t# Héading\n\t\n\t<Inner title={value} {...props}>\n\t\n\tText **bold** {value} $x$.\n\t\n\t</Inner>\n</Outer>\n\n',
        'table-spans': '| A | B |\n| --- | --- |\n| `a\\|b` ==x\\|y== | <Badge title={x} /> |\n: Caption *text* {#t .wide}\n\n',
        'normalized-spans': '\ufeff# Héading\n\n> A\x00B **bold**.\n\n- Item\x00text\n\n',
        'root-references': '[target]: /url "title"\n\n[target] and ![image][target].\n\n',
        'container-references': '- [target]: /url "title"\n\n[target] and ![image][target].\n\n',
        'fenced-decoys': '```ts\ninterface Map<T> {\n  [key: string]: T;\n}\n```\n\nParagraph [ordinary](/url).\n\n',
        'links': '[same](doc.md "title") **[web](https://example.com/path?q=1&x=2)** [unsafe](javascript:bad) ![image](image.md)\n\n',
    }
    cases = []
    for name, snippet in bodies.items():
        source = snippet * 64
        value = source.encode()
        cases.append(dict(name=name, profile='extended', input=source,
            byte_count=len(value), sha256=hashlib.sha256(value).hexdigest(),
            origin={'kind': 'authored-example', 'license': 'MIT'}))
    return dict(schema=1, profiles=profiles, cases=cases)


if __name__ == '__main__':
    json.dump(corpus(), sys.stdout, indent=2)
