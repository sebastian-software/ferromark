#!/usr/bin/env python3
"""Publish missing workspace crates; verify the source commit on safe retries."""
import argparse
import io
import json
import subprocess
import tarfile
import urllib.error
import urllib.request

NAMES = ['ferromark_allocator', 'ferromark_ast', 'ferromark_parser', 'ferromark_renderer', 'ferromark']


def fetch(url):
    request = urllib.request.Request(url, headers={'User-Agent': 'Ferromark release (github.com/sebastian-software/ferromark)'})
    return urllib.request.urlopen(request, timeout=30)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('version')
    parser.add_argument('commit')
    args = parser.parse_args()
    missing = []
    for name in NAMES:
        try:
            with fetch(f'https://crates.io/api/v1/crates/{name}/{args.version}') as response:
                metadata = json.load(response)['version']
        except urllib.error.HTTPError as error:
            if error.code != 404:
                raise
            missing.append(name)
            continue
        assert metadata['num'] == args.version
        with fetch(f'https://static.crates.io/crates/{name}/{name}-{args.version}.crate') as response:
            archive = response.read()
        with tarfile.open(fileobj=io.BytesIO(archive), mode='r:gz') as tar:
            source = json.load(tar.extractfile(f'{name}-{args.version}/.cargo_vcs_info.json'))
        assert source['git']['sha1'] == args.commit and not source['git'].get('dirty'), f'{name}: existing release has different source'
        print(f'Verified existing {name}@{args.version}', flush=True)
    if missing:
        command = ['cargo', 'publish', '--locked']
        for name in missing:
            command += ['-p', name]
        subprocess.run(command, check=True)


if __name__ == '__main__':
    main()
