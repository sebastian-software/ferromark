"""Reject altered source provenance and invalid timing evidence before archival."""
import hashlib
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import run

spec = importlib.util.spec_from_file_location('archive_node', Path(__file__).with_name('archive-node.py'))
archive = importlib.util.module_from_spec(spec)
spec.loader.exec_module(archive)


class ArchiveTests(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory()
        self.addCleanup(self.directory.cleanup)
        self.source = Path(self.directory.name)
        self.cases = [{'name': f'doc-{i}', 'input': 'doc', 'byte_count': 3,
                       'sha256': hashlib.sha256(b'doc').hexdigest()} for i in range(57)]
        self.baseline = {'local_source_sha256': {'src/lib.rs': 'original'},
                         'native_addon_sha256': {}, 'corpus_sha256': 'frozen'}
        self.config = dict(self.baseline, engines=['v2', 'marked'], rounds=3, samples=6,
                           window_ms=40, git_head='commit')
        self.write('package-lock.json', {})
        self.config['lock_sha256'] = run.native.sha(self.source / 'package-lock.json')
        for _, key in archive.SOURCES.values():
            self.config[key] = hashlib.sha256(b'adapter').hexdigest()
        output = '<p>doc</p>'
        self.verification = {case['name']: {'outputs': {'v2': output, 'marked': output},
                                           'agreement': 'exact'} for case in self.cases}
        self.rows = [dict(round=r, sample=s, case=case['name'], mode=mode,
                          v2=dict(iterations=1, elapsed_ns=40_000_000, checksum=len(output)),
                          marked=dict(iterations=1, elapsed_ns=80_000_000, checksum=len(output)))
                     for r in range(3) for s in range(6) for case in self.cases for mode in run.native.MODES]
        self.summary = {'agreeing_documents': 57, 'v2_relative_throughput': {'fresh': 2, 'reuse': 2}}

    def write(self, name, value):
        (self.source / name).write_text(json.dumps(value))

    def validate(self):
        for name, value in [('run.json', self.config), ('summary.json', self.summary),
                            ('samples.json', self.rows), ('verification.json', self.verification),
                            ('corpus.json', {'cases': self.cases})]:
            self.write(name, value)
        with patch.object(archive.subprocess, 'check_output', return_value=b'adapter'):
            return archive.validate(self.source, self.baseline, 'marked')

    def test_valid_complete_evidence(self):
        _, sources, count = self.validate()
        self.assertEqual(count, 2052)
        self.assertEqual(set(sources), set(archive.SOURCES))

    def test_changed_core_is_rejected(self):
        self.config['local_source_sha256'] = {'src/lib.rs': 'changed'}
        with self.assertRaises(AssertionError):
            self.validate()

    def test_duplicate_windows_are_rejected(self):
        self.rows[-1] = self.rows[0]
        with self.assertRaises(AssertionError):
            self.validate()

    def test_false_output_agreement_is_rejected(self):
        self.verification['doc-0']['outputs']['marked'] = '<p>different content</p>'
        with self.assertRaises(AssertionError):
            self.validate()

    def test_invalid_consumption_is_rejected(self):
        self.rows[0]['marked']['checksum'] += 1
        with self.assertRaises(AssertionError):
            self.validate()


if __name__ == '__main__':
    unittest.main()
