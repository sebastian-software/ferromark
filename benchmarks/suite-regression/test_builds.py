import hashlib
import importlib.util
from pathlib import Path
import tempfile
import unittest

spec = importlib.util.spec_from_file_location('suite_run', Path(__file__).with_name('run.py'))
runner = importlib.util.module_from_spec(spec)
spec.loader.exec_module(runner)


class BuildCompatibilityTests(unittest.TestCase):
    def test_local_crate_consolidation_preserves_comparison(self):
        self.compare('[[package]]\nname="ferromark_ast"\nversion="2.0.0-rc.1"\n', '', '1.0.0')

    def test_external_dependency_change_is_rejected(self):
        with self.assertRaisesRegex(AssertionError, 'external dependency graph changed'):
            self.compare('', '', '1.0.1')

    def compare(self, before_local, after_local, after_version):
        with tempfile.TemporaryDirectory() as tmp:
            directories, builds = [], []
            for index, (local, version) in enumerate([(before_local, '1.0.0'), (after_local, after_version)]):
                directory = Path(tmp) / str(index)
                (directory / 'worker').mkdir(parents=True)
                content = f'[[package]]\nname="external"\nversion="{version}"\nsource="registry+https://example.com"\nchecksum="abc"\n' + local
                (directory / 'worker/Cargo.lock').write_text(content)
                builds.append(dict(worker_sha256='worker', rustc='rustc', rustflags='flags', lto='fat', worker_lock_sha256=hashlib.sha256(content.encode()).hexdigest()))
                directories.append(directory)
            runner.check_build_compatibility(builds, directories)
