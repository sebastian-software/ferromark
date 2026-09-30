"""Exercise retained restoration scripts independently of the repository."""
import importlib.util
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest

HERE = Path(__file__).resolve().parent


class RestoreTests(unittest.TestCase):
    def test_cache_and_archived_scripts_load_their_own_pins(self):
        # A retained report must work after current pins change. Test both cache
        # and archive layouts with independent pins and no repository imports.
        for archived in (False, True):
            with self.subTest(archived=archived), tempfile.TemporaryDirectory() as tmp:
                root = Path(tmp)
                shutil.copyfile(HERE / 'restore.py', root / 'restore.py')
                directory = root / 'harness' if archived else root
                directory.mkdir(exist_ok=True)
                (directory / 'prepare.py').write_text('BUN_REVISION = "archived-pin"\n')
                result = subprocess.run([sys.executable, '-c',
                    'import restore; print(restore.pins.BUN_REVISION)'], cwd=root,
                    capture_output=True, text=True, check=True)
                self.assertEqual(result.stdout.strip(), 'archived-pin')

    def test_existing_cache_is_rejected_before_network_or_file_changes(self):
        spec = importlib.util.spec_from_file_location('restore_test', HERE / 'restore.py')
        restore = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(restore)
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            (root / 'bun').mkdir()
            (root / 'bun/local-work').write_text('keep')
            with self.assertRaisesRegex(SystemExit, 'refusing to replace existing cache input'):
                restore.restore(root)
            self.assertEqual((root / 'bun/local-work').read_text(), 'keep')
            self.assertEqual(sorted(p.name for p in root.iterdir()), ['bun'])


if __name__ == '__main__':
    unittest.main()
