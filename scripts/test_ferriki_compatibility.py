"""Prove the peer gate detects offset, scope and error regressions."""
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

    def test_both_peers_cover_the_same_complete_authored_corpus(self):
        corpus = json.loads(CONTRACT.CASES.read_text(encoding='utf-8'))
        for category in ['tokens', 'markdown']:
            ids = [case['id'] for case in corpus[category]]
            self.assertEqual(len(ids), len(set(ids)), category)
            for peer in [self.rust, self.node]:
                self.assertEqual([case['id'] for case in peer[category]], ids, category)
        CONTRACT.compare_peers(self.rust, self.node)

    def test_an_offset_regression_fails_the_gate(self):
        changed = copy.deepcopy(self.node)
        changed['tokens'][0]['tokens']['tokens'][0][0]['offset'] += 1
        with self.assertRaises(AssertionError):
            CONTRACT.compare_peers(self.rust, changed)

    def test_a_real_scope_regression_cannot_be_hidden_by_root_fallback(self):
        changed = copy.deepcopy(self.node)
        changed['tokens'][0]['tokens']['tokens'][0][0]['scopeNames'] = ['source.wrong']
        with self.assertRaises(AssertionError):
            CONTRACT.compare_peers(self.rust, changed)

    def test_unknown_language_must_not_become_an_internal_error(self):
        changed = copy.deepcopy(self.node)
        next(case for case in changed['tokens'] if case['id'] == 'unknown-language')['error'] = 'ERR_INTERNAL'
        with self.assertRaises(AssertionError):
            CONTRACT.compare_peers(self.rust, changed)

    def test_missing_peer_cases_cannot_silently_drop_out(self):
        with self.assertRaises(ValueError):
            CONTRACT.compare_peers(self.rust, {**self.node, 'tokens': self.node['tokens'][:-1]})


if __name__ == '__main__':
    unittest.main()
