"""Retain parse_paragraph disassembly from the two measured macOS binaries."""
from pathlib import Path
import re
import subprocess

HERE = Path(__file__).resolve().parent
BUILDS = {
    'current': '/private/tmp/ferromark-v2-ox-stages-current',
    'paragraph': '/private/tmp/ferromark-v2-ox-paragraph-map-bypass',
}


def main():
    lines = []
    for label, build in BUILDS.items():
        binary = Path(build) / 'target/release/native-comparison-worker'
        text = subprocess.check_output(['otool', '-tvV', str(binary)], text=True)
        # Generic Rust symbols start with __RI; ordinary ones often with __RN.
        # Stop at every symbol boundary, not just non-generic functions.
        chunks = re.split(r'\n(?=_[^\n]*:\n)', text)
        found = []
        for chunk in chunks:
            body = chunk.splitlines()
            if '16ferromark_parser' in body[0] and body[0].endswith('15parse_paragraph:'):
                found.append(body)
        assert len(found) == 1, label
        body = found[0]
        assert all(re.match(r'^[0-9a-f]{16}\s', line) for line in body[1:])
        (HERE / (label + '-parse-paragraph.txt')).write_text('\n'.join(body) + '\n')
        lines += [label + ' ' + str(binary), *body[:22], f'instruction lines: {len(body)-1}', '']
    (HERE / 'prologues.txt').write_text('\n'.join(lines))


if __name__ == '__main__':
    main()
