"""Reject incomplete or altered raw evidence from either platform."""
import hashlib
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch
import run

spec = importlib.util.spec_from_file_location('archive_results', Path(__file__).with_name('archive-results.py'))
archive = importlib.util.module_from_spec(spec)
spec.loader.exec_module(archive)


class PlatformArchiveTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.source = self.root / 'source'
        self.source.mkdir()
        digest = hashlib.sha256(b'adapter').hexdigest()
        baseline = self.root / 'docs/reports/2026-09-30-markdown-ecosystem-main/node'
        baseline.mkdir(parents=True)
        (baseline / 'run.json').write_text(json.dumps({'local_source_sha256': {'src/lib.rs': digest}, 'corpus_sha256': 'frozen'}))
        self.config = dict(track='node', engines=['v2', 'marked'], rounds=3, samples=6, window_ms=40,
                           warmup_ms=60, corpus_sha256='frozen', git_head='commit', local_source_sha256={'src/lib.rs': digest},
                           runner_sha256=digest, verifier_sha256=digest, guards_sha256=digest,
                           worker_sha256=digest, node_adapters_sha256=digest)
        (self.source / 'package-lock.json').write_text('{}')
        self.config['lock_sha256'] = run.native.sha(self.source / 'package-lock.json')
        self.cases = [dict(name=f'doc-{i}', input='doc', byte_count=3,
                           sha256=hashlib.sha256(b'doc').hexdigest()) for i in range(57)]
        html = '<p>doc</p>'
        self.outputs = {case['name']: dict(outputs={'v2': html, 'marked': html}, agreement='exact') for case in self.cases}
        self.rows = [dict(round=r, sample=s, case=case['name'], mode=mode, order=['v2', 'marked'],
                          v2=dict(iterations=1, elapsed_ns=40_000_000, checksum=len(html)),
                          marked=dict(iterations=1, elapsed_ns=80_000_000, checksum=len(html)))
                     for r in range(3) for s in range(6) for case in self.cases for mode in run.native.MODES]
        self.summary = dict(documents=57, agreeing_documents=57, agreement_counts={'exact': 57},
                            v2_relative_throughput={'fresh': 2, 'reuse': 2})

    def validate(self):
        for name, value in [('run', self.config), ('summary', self.summary), ('corpus', {'cases': self.cases}),
                            ('verification', self.outputs), ('samples', self.rows)]:
            (self.source / (name + '.json')).write_text(json.dumps(value))
        with patch.object(archive, 'REPO', self.root), patch.object(run.native, 'read_json', return_value={'cases': self.cases}), patch.object(archive.subprocess, 'check_output', side_effect=lambda command, **kwargs: b'{}' if command[-1].endswith('package-lock.json') else b'adapter'):
            return archive.validate(self.source)

    def test_complete_evidence(self):
        config, sources = self.validate()
        self.assertEqual(config['engines'], ['v2', 'marked'])
        self.assertIn('worker.mjs', sources)

    def test_incomplete_round_is_rejected(self):
        self.rows.pop()
        with self.assertRaises(AssertionError): self.validate()

    def test_duplicate_window_is_rejected(self):
        self.rows[-1] = self.rows[0]
        with self.assertRaises(AssertionError): self.validate()

    def test_changed_core_is_rejected(self):
        self.config['local_source_sha256'] = {}
        with self.assertRaises(AssertionError): self.validate()

    def test_false_agreement_is_rejected(self):
        self.outputs['doc-0']['outputs']['marked'] = '<p>different</p>'
        with self.assertRaises(AssertionError): self.validate()

    def test_wrong_checksum_is_rejected(self):
        self.rows[0]['marked']['checksum'] += 1
        with self.assertRaises(AssertionError): self.validate()

    def test_false_aggregate_is_rejected(self):
        self.summary['v2_relative_throughput']['fresh'] = 3
        with self.assertRaises(AssertionError): self.validate()


if __name__ == '__main__': unittest.main()
