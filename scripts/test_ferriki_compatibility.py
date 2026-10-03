"""Prove the Rust snapshot and public HTML/error peer gate detect regressions."""
import copy
import importlib.util
import json
from pathlib import Path
import unittest

HERE = Path(__file__).resolve().parent
SPEC = importlib.util.spec_from_file_location('ferriki_contract', HERE / 'ferriki-compatibility/run.py')
CONTRACT = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(CONTRACT)


class FerrikiPeerContract(unittest.TestCase):
    def setUp(self):
        self.snapshots = json.loads((HERE / 'ferriki-compatibility/snapshots.json').read_text(encoding='utf-8'))
        self.rust = self.snapshots['rust']
        self.node = self.snapshots['node']

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
