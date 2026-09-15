#!/usr/bin/env python3
"""Bind a release to a successful CI run of the exact selected main commit."""
import json
import os
import subprocess
import urllib.error
import urllib.request
from pathlib import Path

def validate_ci(run, commit):
    assert run['head_sha'] == commit and run['head_branch'] == 'main', 'CI must match the selected main commit'
    assert run['path'] == '.github/workflows/ci.yml' and run['event'] == 'push', 'Use the main push CI workflow'
    assert run['status'] == 'completed' and run['conclusion'] == 'success', 'All CI gates must pass first'


def main():
    version = os.environ['RELEASE_VERSION']
    commit = os.environ['GITHUB_SHA']
    run_id = os.environ['CI_RUN_ID']
    assert run_id.isdecimal(), 'CI run ID must be numeric'
    assert os.environ['GITHUB_REF'] == 'refs/heads/main', 'Publish only from main'
    assert Path('version.txt').read_text().strip() == version, 'Selected version differs from checkout'
    run = json.loads(subprocess.check_output(['gh', 'api', f'repos/{os.environ["GITHUB_REPOSITORY"]}/actions/runs/{run_id}']))
    validate_ci(run, commit)
    missing = []
    for name in ['ferromark_allocator', 'ferromark_ast', 'ferromark_parser', 'ferromark_renderer', 'ferromark']:
        try:
            request = urllib.request.Request(f'https://crates.io/api/v1/crates/{name}', headers={'User-Agent': 'Ferromark release preflight'})
            with urllib.request.urlopen(request, timeout=30):
                pass
        except urllib.error.HTTPError as error:
            if error.code != 404:
                raise
            missing.append(name)
    assert not missing or os.environ.get('CRATES_IO_BOOTSTRAP_TOKEN'), 'First publication requires CRATES_IO_BOOTSTRAP_TOKEN for: ' + ', '.join(missing)
    print(f'CI {run_id} passed for {commit}; ready to publish {version}')
    with open(os.environ['GITHUB_OUTPUT'], 'a') as output:
        output.write('bootstrap=' + ('true' if os.environ.get('CRATES_IO_BOOTSTRAP_TOKEN') else 'false') + '\n')


if __name__ == '__main__':
    main()
