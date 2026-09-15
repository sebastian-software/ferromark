#!/usr/bin/env python3
"""Publish ferromark; verify the source commit on safe retries."""
import argparse
import io
import json
import subprocess
import tarfile
import urllib.error
import urllib.request


def fetch(url):
    request = urllib.request.Request(url, headers={'User-Agent': 'Ferromark release (github.com/sebastian-software/ferromark)'})
    return urllib.request.urlopen(request, timeout=30)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('version')
    parser.add_argument('commit')
    args = parser.parse_args()
    try:
        with fetch(f'https://crates.io/api/v1/crates/ferromark/{args.version}') as response:
            metadata = json.load(response)['version']
    except urllib.error.HTTPError as error:
        if error.code != 404:
            raise
        subprocess.run(['cargo', 'publish', '--locked', '-p', 'ferromark'], check=True)
        return
    assert metadata['num'] == args.version
    with fetch(f'https://static.crates.io/crates/ferromark/ferromark-{args.version}.crate') as response:
        archive = response.read()
    with tarfile.open(fileobj=io.BytesIO(archive), mode='r:gz') as tar:
        source = json.load(tar.extractfile(f'ferromark-{args.version}/.cargo_vcs_info.json'))
    assert source['git']['sha1'] == args.commit and not source['git'].get('dirty'), 'Existing release has different source'
    print(f'Verified existing ferromark@{args.version}', flush=True)


if __name__ == '__main__':
    main()
