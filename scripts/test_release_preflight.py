"""Exercise publication guards without network access or registry mutation."""
import importlib.util
from pathlib import Path
import unittest

spec = importlib.util.spec_from_file_location('preflight', Path(__file__).with_name('release-preflight.py'))
preflight = importlib.util.module_from_spec(spec)
spec.loader.exec_module(preflight)


class ReleasePreflightTests(unittest.TestCase):
    def test_accepts_completed_main_push_for_exact_commit(self):
        preflight.validate_ci(self.run_metadata(), 'selected-commit')

    def test_rejects_wrong_source_or_incomplete_checks(self):
        for field, value in [('head_sha', 'other-commit'), ('head_branch', 'codex/v2'),
                             ('path', '.github/workflows/publish.yml'), ('event', 'pull_request'),
                             ('status', 'in_progress'), ('conclusion', 'failure'), ('conclusion', 'skipped')]:
            with self.subTest(field=field, value=value):
                run = self.run_metadata()
                run[field] = value
                with self.assertRaises(AssertionError):
                    preflight.validate_ci(run, 'selected-commit')

    @staticmethod
    def run_metadata():
        return dict(head_sha='selected-commit', head_branch='main', path='.github/workflows/ci.yml',
                    event='push', status='completed', conclusion='success')


if __name__ == '__main__':
    unittest.main()
