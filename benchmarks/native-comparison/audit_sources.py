#!/usr/bin/env python3
"""Independently verify built parser sources against pinned Git blobs / OX archive."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import tarfile

import prepare


def audit_git(repo, revision, root, paths):
    entries = subprocess.check_output(['git', '-C', str(repo), 'ls-tree', '-r', '-z', revision, '--', *paths])
    records = {}
    materialized_links = []
    for entry in entries.split(b'\0'):
        if not entry:
            continue
        meta, raw_name = entry.split(b'\t', 1)
        mode, kind, expected = meta.split()
        assert kind == b'blob', (raw_name, kind)
        name = raw_name.decode()
        path = root / name
        if mode == b'120000':
            data = subprocess.check_output(['git', '-C', str(repo), 'show', f'{revision}:{name}'])
            if path.is_symlink():
                assert str(path.readlink()).encode() == data
            else:
                # The first Bun copy materialized three upstream symlinks.
                # Verify the copied content against its target, whose tracked
                # files are independently checked in this same source audit.
                target = path.parent / data.decode()
                if path.is_dir():
                    assert prepare.sha_tree(path) == prepare.sha_tree(target)
                else:
                    assert path.read_bytes() == target.read_bytes()
                materialized_links.append({'path': name, 'target': data.decode(), 'content_matches': True})
        else:
            data = path.read_bytes()
        actual = hashlib.sha1(b'blob ' + str(len(data)).encode() + b'\0' + data).hexdigest()
        assert actual == expected.decode(), ('source differs from pinned Git blob', root, name)
        records[name] = hashlib.sha256(data).hexdigest()
    assert records
    return {'revision': revision, 'checked_files': len(records), 'mismatches': [], 'materialized_symlinks': materialized_links, 'sha256': records}


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('build', type=Path)
    p.add_argument('output', type=Path)
    p.add_argument('--v1', type=Path, default=Path('/Users/sebastian/Workspace/ferromark'))
    p.add_argument('--v2', type=Path, default=Path(__file__).resolve().parents[2])
    p.add_argument('--bun', type=Path, default=Path('/private/tmp/ferromark-bun-publication-20260911'))
    p.add_argument('--md4c', type=Path, default=Path('/private/tmp/ferromark-md4c-publication-20260911'))
    p.add_argument('--ox-archive', type=Path, default=Path('/private/tmp/ferromark-v2-upstream.tar.gz'))
    args = p.parse_args()
    b = args.build
    result = {}
    for name, repo, rev, root, paths in (
        ('v1', args.v1, prepare.FERROMARK_V1_REVISION, b / 'sources/ferromark_v1', ['src', 'Cargo.toml', 'Cargo.lock']),
        ('v2', args.v2, prepare.FERROMARK_V2_REVISION, b / 'sources/ferromark_v2', ['crates', 'Cargo.toml', 'Cargo.lock']),
        ('bun', args.bun, prepare.BUN_REVISION, b / 'bun', ['src', 'scripts/build']),
        ('md4c', args.md4c, prepare.MD4C_REVISION, b / 'sources/md4c', ['src']),
    ):
        result[name] = audit_git(repo, rev, root, paths)
    assert prepare.sha(args.ox_archive) == prepare.OX_ARCHIVE_SHA256
    ox = {}
    with tarfile.open(args.ox_archive) as archive:
        for member in archive.getmembers():
            if not member.isfile():
                continue
            name = '/'.join(Path(member.name).parts[1:])
            if name not in {'Cargo.toml', 'Cargo.lock'} and not name.startswith(tuple('crates/ox_content_' + c + '/' for c in ('allocator', 'ast', 'parser', 'renderer', 'profiler'))):
                continue
            data = archive.extractfile(member).read()
            assert data == (b / 'sources/ox_content' / name).read_bytes(), ('OX source differs', name)
            ox[name] = hashlib.sha256(data).hexdigest()
    result['ox-content'] = dict(revision=prepare.OX_REVISION, archive_sha256=prepare.OX_ARCHIVE_SHA256,
        checked_files=len(ox), mismatches=[], sha256=ox)
    native = {}
    for name, expected in (('mimalloc', prepare.MI_ARCHIVE_SHA256), ('highway', prepare.HWY_ARCHIVE_SHA256)):
        archive_path = b / 'native' / (name + '.tar.gz')
        assert prepare.sha(archive_path) == expected
        count = 0
        with tarfile.open(archive_path) as archive:
            for member in archive.getmembers():
                if member.isfile():
                    relative = Path(*Path(member.name).parts[1:])
                    assert archive.extractfile(member).read() == (b / 'native' / name / relative).read_bytes()
                    count += 1
        native[name] = dict(archive_sha256=expected, checked_files=count, mismatches=[])
    result['native_dependencies'] = native
    args.output.write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps({n: {'checked_files': v.get('checked_files'), 'mismatches': v.get('mismatches')} for n,v in result.items() if n != 'native_dependencies'}))


if __name__ == '__main__':
    main()
