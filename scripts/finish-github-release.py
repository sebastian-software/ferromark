#!/usr/bin/env python3
"""Create the release only after both registry installations have passed.

The release body is the CHANGELOG.md section that the Release Please pull
request added for this version, so the notes reviewed in that pull request are
what the GitHub release shows.
"""
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import tempfile


def changelog_section(changelog, version):
    """Return the body of the `## <version>` section, without its heading.

    Release Please writes `## [2.0.0-rc.2](compare-url) (date)`; authored
    sections such as the first candidate's use a plain `## 2.0.0-rc.1`. Both
    forms are accepted. The section ends at the next second-level heading.
    """
    escaped = re.escape(version)
    heading = re.compile(rf'^## (?:\[{escaped}\]\([^)]*\)|{escaped})(?:\s.*)?$', re.MULTILINE)
    match = heading.search(changelog)
    assert match, f'CHANGELOG.md has no section for {version}; merge the release pull request first'
    rest = changelog[match.end():]
    following = re.search(r'^## ', rest, re.MULTILINE)
    body = rest[:following.start()] if following else rest
    body = body.strip()
    assert body, f'The CHANGELOG.md section for {version} is empty'
    return body + '\n'


def main():
    version = os.environ['RELEASE_VERSION']
    notes = changelog_section(Path('CHANGELOG.md').read_text(encoding='utf-8'), version)
    if '--notes-only' in sys.argv[1:]:
        sys.stdout.write(notes)
        return
    tag = 'v' + version
    commit = os.environ['GITHUB_SHA']
    assets = sorted(str(p) for p in Path('release-artifacts').rglob('*') if p.suffix in ('.tgz', '.crate'))
    assert len(assets) == 10, 'Expected nine npm archives and one Rust archive'
    existing = subprocess.run(['gh', 'release', 'view', tag, '--json', 'tagName'], capture_output=True, text=True)
    if existing.returncode == 0:
        published = json.loads(subprocess.check_output(['gh', 'api', f'repos/{os.environ["GITHUB_REPOSITORY"]}/commits/{tag}']))
        assert published['sha'] == commit, 'Existing release tag points to another commit'
        subprocess.run(['gh', 'release', 'upload', tag, *assets, '--clobber'], check=True)
        return
    with tempfile.NamedTemporaryFile('w', suffix='.md', delete=False, encoding='utf-8') as handle:
        handle.write(notes)
        notes_file = handle.name
    command = ['gh', 'release', 'create', tag, '--target', commit, '--title', 'Ferromark ' + version,
               '--notes-file', notes_file, '--latest=' + ('false' if '-rc.' in version else 'true')]
    if '-rc.' in version:
        command.append('--prerelease')
    subprocess.run([*command, *assets], check=True)


if __name__ == '__main__':
    main()
