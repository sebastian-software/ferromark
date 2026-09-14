#!/usr/bin/env python3
"""Isolate the extra renderer URL pass after parser GFM autolinking."""

import argparse
import hashlib
import json
from pathlib import Path

from make_cases import PROSE, SIZES, config, scaled


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output', type=Path)
    args = parser.parse_args()
    cases = []
    for size in SIZES:
        for workload, snippet in [('plain', PROSE),
                                  ('urls', 'Visit https://example.org/path and www.example.net today.\n\n')]:
            source = scaled(snippet, size)
            raw = source.encode()
            cases.append(dict(name=f'double-autolinks-{workload}-{size}', group='interaction',
                              feature='double-autolinks', workload=workload, target_bytes=size,
                              input=source, off=config({'autolinks': True}),
                              on=config({'autolinks': True}, {'autolink_urls': True}),
                              expected_effect='none', byte_count=len(raw),
                              sha256=hashlib.sha256(raw).hexdigest(),
                              origin={'kind': 'repeated-synthetic-probe', 'license': 'MIT'}))
    args.output.write_text(json.dumps({'schema': 1, 'cases': cases}, indent=2) + '\n')


if __name__ == '__main__':
    main()
