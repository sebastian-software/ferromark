#!/usr/bin/env python3
"""Create the release only after both registry installations have passed."""
import json
import os
from pathlib import Path
import subprocess

version = os.environ['RELEASE_VERSION']
tag = 'v' + version
commit = os.environ['GITHUB_SHA']
notes = Path('docs/releases') / (version + '.md')
assert notes.is_file(), 'Release notes must be reviewed in Git'
assets = sorted(str(p) for p in Path('release-artifacts').rglob('*') if p.suffix in ('.tgz', '.crate'))
assert len(assets) == 14, 'Expected nine npm and five Rust archives'
existing = subprocess.run(['gh', 'release', 'view', tag, '--json', 'tagName'], capture_output=True, text=True)
if existing.returncode == 0:
    published = json.loads(subprocess.check_output(['gh', 'api', f'repos/{os.environ["GITHUB_REPOSITORY"]}/commits/{tag}']))
    assert published['sha'] == commit, 'Existing release tag points to another commit'
    subprocess.run(['gh', 'release', 'upload', tag, *assets, '--clobber'], check=True)
else:
    command = ['gh', 'release', 'create', tag, '--target', commit, '--title', 'Ferromark ' + version,
               '--notes-file', str(notes), '--latest=' + ('false' if '-rc.' in version else 'true')]
    if '-rc.' in version:
        command.append('--prerelease')
    subprocess.run([*command, *assets], check=True)
