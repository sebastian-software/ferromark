#!/usr/bin/env python3
"""Deterministic tilde/inline-boundary combinations for the cmark oracle."""
import argparse
import json
from pathlib import Path


def make_cases():
    inner_cases = [
        'x', 'x y', 'x ~ y', 'x ~~ y',
        '[label](u~v)', '[label](u~~v)', '[x ~ y](u)', '[x ~~ y](u)',
        '[link](u "t~t")', '[link](u "t~~t")', '![image](u~v "t~~t")',
        '<span title="~">x</span>', '<span title="~~">x</span>',
        '<http://example.com/x~y>', '<http://example.com/x~~y>',
        '`x~y`', '`` x~~ ` y ``', r'x\~y', r'x\~~y',
        '*em ~ x*', '**bold ~~ x**', '*x*', '**x**', 'a [b *c*](u)',
        'x\ny', ' x', 'x ', 'a ~b~ c', 'a ~~b~~ c', 'a ~~~b~~~ c',
        '**~x~**', '[x](u) [x]',
    ]
    cases = []
    for marker in ('~', '~~'):
        for index, inner in enumerate(inner_cases):
            span = marker + inner + marker
            for context, source in [
                ('paragraph', span), ('emphasis', '*' + span + '*'),
                ('link', '[' + span + '](target)'),
                ('quote', '> ' + span.replace('\n', '\n> ')),
            ]:
                cases.append(dict(id=f'binding-{len(marker)}-{index:02d}-{context}',
                    category='tilde-combinations', profile='gfm', markdown=source+'\n'))
    return cases


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output', type=Path)
    args = parser.parse_args()
    args.output.write_text(json.dumps(dict(generator=Path(__file__).name, cases=make_cases()),
        indent=2, ensure_ascii=False)+'\n')
