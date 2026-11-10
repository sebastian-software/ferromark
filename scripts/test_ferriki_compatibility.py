"""Prove the Rust snapshot and public HTML/error peer gate detect regressions."""
import copy
import importlib.util
import json
from pathlib import Path
import unittest
import tempfile
import subprocess
import sys

HERE = Path(__file__).resolve().parent
SPEC = importlib.util.spec_from_file_location('ferriki_contract', HERE / 'ferriki-compatibility/run.py')
CONTRACT = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(CONTRACT)


class FerrikiPeerContract(unittest.TestCase):
    def setUp(self):
        self.snapshots = json.loads((HERE / 'ferriki-compatibility/snapshots.json').read_text(encoding='utf-8'))
        self.rust = self.snapshots['rust']
        self.node = self.snapshots['node']

    def test_current_selection_changes_only_release_identity_not_authored_cases(self):
        original = CONTRACT.CASES.read_bytes()
        with tempfile.TemporaryDirectory() as directory:
            selected, baseline = CONTRACT.release_cases(Path(directory), '1.3.0')
            corpus = json.loads(selected.read_text(encoding='utf-8'))
            self.assertEqual(baseline, '1.1.2')
            self.assertEqual(corpus.pop('ferrikiVersion'), '1.3.0')
            reference = json.loads(original)
            reference.pop('ferrikiVersion')
            self.assertEqual(corpus, reference)
            CONTRACT.compare_peers(self.rust, self.node, selected)
        self.assertEqual(CONTRACT.CASES.read_bytes(), original)

    def test_prepared_sidecars_keep_immutable_paths_and_reject_invalid_locations(self):
        digest = 'a' * 64
        for layout in ['language-hash-v1', 'language-hash-v2']:
            self.assertEqual(CONTRACT.payload_path('languages/rust.fkbc', digest, layout),
                             CONTRACT.payload_path('languages/rust.fkgram', digest, layout))
        for key in ['themes/nord.fkbc', 'languages/rust.fktheme', '../rust.fkbc']:
            with self.assertRaises(ValueError):
                CONTRACT.payload_path(key, digest, 'language-hash-v2')

    def test_current_release_cannot_rewrite_historical_snapshots(self):
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory) / 'evidence'
            result = subprocess.run([sys.executable, str(HERE / 'ferriki-compatibility/run.py'),
                                     str(output), '--node', '--release', str(HERE / 'ferriki-current'),
                                     '--update-node-snapshot'], capture_output=True, text=True)
            self.assertEqual(result.returncode, 2)
            self.assertIn('cannot update the historical snapshots', result.stderr)
            self.assertFalse(output.exists())

    def test_both_peers_cover_the_same_authored_html_corpus(self):
        corpus = json.loads(CONTRACT.CASES.read_text(encoding='utf-8'))
        self.assertEqual(
            [case['id'] for case in corpus['tokens']],
            [case['id'] for case in self.rust['tokens']],
        )
        self.assertEqual(
            [case['id'] for case in corpus['tokens']],
            [case['id'] for case in self.node['highlighting']],
        )
        for category in ['markdown']:
            ids = [case['id'] for case in corpus[category]]
            self.assertEqual([case['id'] for case in self.rust[category]], ids)
            self.assertEqual([case['id'] for case in self.node[category]], ids)
        CONTRACT.compare_peers(self.rust, self.node)

    def test_frozen_rust_snapshot_detects_utf8_offset_regressions(self):
        changed = copy.deepcopy(self.rust)
        changed['tokens'][5]['tokens']['tokens'][0][0]['offset'] += 1
        with self.assertRaises(AssertionError):
            CONTRACT.check_snapshot('rust', changed, self.rust)

    def test_frozen_rust_snapshot_detects_real_scope_regressions(self):
        changed = copy.deepcopy(self.rust)
        changed['tokens'][0]['tokens']['tokens'][0][0]['scopeNames'] = ['source.wrong']
        with self.assertRaises(AssertionError):
            CONTRACT.check_snapshot('rust', changed, self.rust)

    def test_unknown_language_must_not_become_an_internal_error(self):
        changed = copy.deepcopy(self.node)
        next(case for case in changed['highlighting'] if case['id'] == 'unknown-language')['error'] = 'ERR_INTERNAL'
        with self.assertRaises(AssertionError):
            CONTRACT.compare_peers(self.rust, changed)

    def test_unsafe_or_lost_html_text_fails_peer_comparison(self):
        changed = copy.deepcopy(self.node)
        rust_case = next(case for case in changed['highlighting'] if case['id'] == 'rust')
        rust_case['html'] = rust_case['html'].replace('main', '', 1)
        with self.assertRaises(AssertionError):
            CONTRACT.compare_peers(self.rust, changed)

    def test_missing_peer_cases_cannot_silently_drop_out(self):
        with self.assertRaises(ValueError):
            CONTRACT.compare_peers(self.rust, {**self.node, 'highlighting': self.node['highlighting'][:-1]})


if __name__ == '__main__':
    unittest.main()
