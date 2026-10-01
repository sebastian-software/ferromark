"""Protect campaign completeness and rotating-workload publication checks."""
import json
from pathlib import Path
import unittest
import run
import importlib.util

spec = importlib.util.spec_from_file_location('candidate_archive', Path(__file__).with_name('archive-results.py'))
archive = importlib.util.module_from_spec(spec)
spec.loader.exec_module(archive)


class CandidateContractTests(unittest.TestCase):
    def test_every_homepage_entry_has_an_executable_campaign_lane_and_pinned_dependency(self):
        catalog = json.loads((run.REPO / 'homepage/app/data/benchmark-projects.json').read_text())
        projects = run.contracts.PROJECTS
        self.assertEqual(catalog, [{key: project[key] for key in ('id', 'label', 'runtime', 'github', 'backend')} for project in run.contracts.campaign_projects(projects)])
        self.assertEqual(len(projects), 23)
        self.assertEqual(len(run.contracts.campaign_projects(projects)), 20)
        self.assertEqual(len(run.contracts.campaign_pairs()), 16)
        self.assertEqual(len(run.contracts.campaign_pairs('extended')), 19)
        self.assertEqual({project['id'] for project in projects if project.get('optional')},
                         {'markdown-exit', 'markdown-it-ts', 'md4x-wasm'})
        with self.assertRaises(ValueError):
            run.contracts.campaign_projects(projects, 'unknown')
        self.assertEqual(len(run.contracts.PAIRS), 19)
        self.assertEqual(sum(project['runtime'] == 'Native' for project in projects), 9)
        lock = json.loads((run.HERE / 'package-lock.json').read_text())['packages']
        package = json.loads((run.HERE / 'package.json').read_text())['dependencies']
        for project in projects:
            if project['track'] == 'node':
                name = project['package']
                self.assertEqual(package[name], lock['node_modules/' + name]['version'])
        self.assertEqual({project['backend'] for project in projects if project['id'].startswith('md4x-')}, {'Native addon', 'WASM'})

    def rotation(self):
        # Unit fixture only; not retained or published as measurement evidence.
        config = dict(engines=['v2', 'md4x-wasm'], track='node', rounds=3, samples=6, window_ms=40, warmup_ms=60)
        cases = [dict(name='one', profile='commonmark'), dict(name='two', profile='commonmark')]
        outputs = {name: {'outputs': {'v2': '🪐é', 'md4x-wasm': '🪐é'}} for name in ('one', 'two')}
        rows = [dict(round=r, sample=s, profile='commonmark', mode=mode, members=['one', 'two'],
                     order=['v2', 'md4x-wasm'], **{engine: dict(iterations=2, elapsed_ns=40_000_000, checksum=12) for engine in config['engines']})
                for r in range(3) for s in range(6) for mode in run.native.MODES]
        return config, cases, outputs, rows

    def test_rotation_requires_every_document_window_and_utf16_consumption(self):
        config, cases, outputs, rows = self.rotation()
        archive.validate_rotation(config, cases, outputs, rows)
        with self.assertRaises(AssertionError):
            archive.validate_rotation(config, cases, outputs, rows[:-1])
        rows[0]['members'] = ['one']
        with self.assertRaises(AssertionError):
            archive.validate_rotation(config, cases, outputs, rows)
        rows[0]['members'] = ['one', 'two']
        rows[0]['md4x-wasm']['checksum'] = 24  # UTF-8 bytes cannot replace string units.
        with self.assertRaises(AssertionError):
            archive.validate_rotation(config, cases, outputs, rows)

    def test_rotation_rejects_shortened_windows_and_duplicate_samples(self):
        config, cases, outputs, rows = self.rotation()
        rows[0]['v2']['elapsed_ns'] = 1
        with self.assertRaises(AssertionError):
            archive.validate_rotation(config, cases, outputs, rows)
        rows[0]['v2']['elapsed_ns'] = 40_000_000
        rows[-1] = rows[0]
        with self.assertRaises(AssertionError):
            archive.validate_rotation(config, cases, outputs, rows)

    def test_balanced_controls_still_require_complete_rounds_and_full_windows(self):
        config, cases, outputs, rows = self.rotation()
        config.update(run.contracts.TIMING_PROFILES['balanced'])
        rows = [row for row in rows if row['sample'] < 3]
        for row in rows:
            for engine in config['engines']:
                row[engine]['elapsed_ns'] = 10_000_000
        archive.validate_rotation(config, cases, outputs, rows)
        with self.assertRaises(AssertionError):
            archive.validate_rotation(config, cases, outputs, rows[:-1])
        rows[0]['v2']['elapsed_ns'] -= 1
        with self.assertRaises(AssertionError):
            archive.validate_rotation(config, cases, outputs, rows)

    def test_named_profile_cannot_license_arbitrary_shortening_or_relabel_history(self):
        policy = json.loads((run.HERE / 'scoring-policy.json').read_text())
        config = dict(timing_profile='balanced', **run.contracts.TIMING_PROFILES['balanced'])
        self.assertEqual(run.contracts.validate_timing(config, policy), 'balanced')
        for change in ({'samples': 1}, {'rounds': 1}, {'warmup_ms': 1},
                       {'window_ms': 1}, {'timing_profile': 'standard'}, {'timing_profile': 'diagnostic'}):
            with self.assertRaises(AssertionError):
                run.contracts.validate_timing(config | change, policy)
        with self.assertRaises(AssertionError):
            run.contracts.validate_timing({key: value for key, value in config.items() if key != 'timing_profile'}, policy)
        with self.assertRaises(AssertionError):
            run.contracts.validate_timing(config, {'documents': 57})
