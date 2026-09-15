#!/usr/bin/env python3
"""Bind a release to a successful CI run of the exact selected main commit."""
import json
import os
import subprocess
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
    print(f'CI {run_id} passed for {commit}; ready to publish {version}')



if __name__ == '__main__':
    main()
