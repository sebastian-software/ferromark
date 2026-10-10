import importlib.util
from pathlib import Path
import json
import shutil
import subprocess
import tempfile
import unittest

HERE = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location('conformance', HERE / 'cli.py')
cli = importlib.util.module_from_spec(spec)
spec.loader.exec_module(cli)


class ConformanceTests(unittest.TestCase):
    def test_frozen_example_identity_and_counts(self):
        examples = cli.fixtures()
        self.assertEqual(len(examples['commonmark']), 652)
        self.assertEqual(len(examples['gfm']), 28)
        self.assertEqual(examples['commonmark'][0]['example'], 1)
        self.assertEqual(len({e['example'] for e in examples['gfm']}), 28)

    def test_conservative_equivalence_and_negative_counterexamples(self):
        self.assertEqual(cli.classify('<p>A &amp; B</p>\n', '<p>A &#38; B</p>'), 'serialization-equivalent')
        self.assertEqual(cli.classify('<input disabled="" checked="" />', '<input checked disabled>'), 'serialization-equivalent')
        for left, right in [
            ('<pre><code>a  b\n</code></pre>', '<pre><code>a b\n</code></pre>'),
            ('<p>&lt;b&gt;x&lt;/b&gt;</p>', '<p><b>x</b></p>'),
            ('<a href="/%2F">x</a>', '<a href="//">x</a>'),
            ('<input checked>', '<input>'),
            ('<svg><text>a  b</text></svg>', '<svg><text>a b</text></svg>'),
            ('<p id="a" id="b">x</p>', '<p id="b">x</p>'),
            ('<h1>x</h1>', '<h1 id="x">x</h1>'),
        ]:
            self.assertFalse(cli.verify.admitted(cli.classify(left, right)), (left, right))

    def test_worker_crash_does_not_hide_following_inputs(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            worker = root / 'worker.py'
            worker.write_text('import sys\nfor i,p in enumerate(sys.argv[1:]):\n s=open(p).read()\n if s=="crash": sys.exit(3)\n print("html",i,s.encode().hex(),flush=True)\nprint("done")\n')
            paths = []
            for index, source in enumerate(('first', 'crash', 'last')):
                path = root / str(index)
                path.write_text(source)
                paths.append(path)
            results = cli.native_results([cli.sys.executable, str(worker)], paths)
            self.assertEqual(results[0], {'html': 'first', 'error': None})
            self.assertIn('exit 3', results[1]['error'])
            self.assertEqual(results[2], {'html': 'last', 'error': None})

    def test_incomplete_success_is_rejected(self):
        with self.assertRaises(ValueError):
            cli.parse_native(subprocess.CompletedProcess([], 0, 'html 0 61\n', ''), 2)
        with self.assertRaises(ValueError):
            cli.parse_native(subprocess.CompletedProcess([], 0, 'html 0 61\nhtml 0 62\ndone\n', ''), 2)

    def test_errors_are_measured_failures(self):
        counts = cli.summarize([{'status': 'exact'}, {'status': 'serialization-equivalent'}, {'status': 'error'}, {'status': 'heading-id-only'}])
        self.assertEqual((counts['passed'], counts['failed'], counts['errors'], counts['percent']), (2, 2, 1, 50))

    def test_retained_evidence_and_homepage_values(self):
        # Ordinary CI reclassifies every stored output without downloading or
        # rebuilding external libraries. Hashes prevent stale adapter claims.
        cli.check(cli.REPO / cli.REPORT, cli.DATA)

    def test_corrupt_counts_are_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            report = Path(directory) / 'report'
            shutil.copytree(cli.REPO / cli.REPORT, report)
            summary = cli.read(report / 'summary.json')
            summary['rows'][0]['suites']['commonmark']['passed'] -= 1
            cli.write(report / 'summary.json', summary)
            with self.assertRaisesRegex(ValueError, 'Counts do not match'):
                cli.check(report)


if __name__ == '__main__':
    unittest.main()
