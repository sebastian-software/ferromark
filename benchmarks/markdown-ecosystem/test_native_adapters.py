"""Integration guard against accidentally timing process startup or unsupported profiles."""
import json
from pathlib import Path
import subprocess
import tempfile
import unittest
import os
import run


@unittest.skipUnless(os.environ.get('ECOSYSTEM_BUILD'), 'Set ECOSYSTEM_BUILD to the prepared native build directory')
class NativeAdapterTests(unittest.TestCase):
    def test_public_api_option_guards_and_lifecycle_isolation(self):
        build = Path(os.environ['ECOSYSTEM_BUILD'])
        metadata = json.loads((build / 'build.json').read_text())
        for item in metadata['binaries'].values():
            self.assertEqual(run.native.sha(item['path']), item['sha256'])
        with tempfile.TemporaryDirectory() as directory:
            for engine in ('comrak', 'cmark', 'cmark-gfm'):
                profiles = ('commonmark',) if engine == 'cmark' else ('commonmark', 'gfm-shared')
                result = run.native.behavior_checks(build / 'worker', Path(directory), ('v2', engine), profiles)
                self.assertEqual(set(result), set(profiles))


if __name__ == '__main__':
    unittest.main()
