"""Exercise publication calculations with retained evidence and CLI rejection paths."""
import gzip
import importlib.util
import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest

import cli


class ManualTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix='ferromark manual ')
        self.addCleanup(self.temp.cleanup)
        self.folder = Path(self.temp.name) / 'evidence'
        self.folder.mkdir()
        self.suite = {'schema': 2, **cli.platform_details('Darwin', 'arm64'), 'revision': '290801961e2433d29d5a32ddb78c8384a0fcd336',
                      'machine': 'Test host: retained measurements, not a new run',
                      'host_platform': 'macOS-27.0-arm64-arm-64bit',
                      'addon_file': 'ferromark.darwin-arm64.node', 'addon_sha256': 'unused', 'node': 'v24.21.0', 'corpus_sha256': cli.native.sha(cli.CORPUS)}
        cli.write(self.folder / 'suite.json', self.suite)

    def test_publisher_can_load_the_manual_cli_with_its_runner_validator(self):
        result = subprocess.run([sys.executable, '-c',
                                 "import sys; sys.path.insert(0, 'benchmarks/markdown-ecosystem'); import publish_values; publish_values.manual_workflow()"],
                                cwd=cli.REPO, capture_output=True, text=True)
        self.assertEqual(result.returncode, 0, result.stderr)

    def retained_matrix(self):
        # Copy real archived data. These heterogeneous historical runs are used
        # only to test calculations/coverage, never to claim one measured suite.
        report = cli.REPO / 'docs/reports/2026-09-30-ecosystem-platforms'
        for track, engine in cli.PAIRS:
            platform = 'macos-arm64' if engine in ('comrak', 'cmark', 'cmark-gfm') else 'linux-x86-64'
            source = report / platform / (track + '-' + engine)
            shutil.copytree(source, self.folder / (track + '-' + engine))
        shutil.copytree(cli.REPO / 'docs/reports/2026-09-24-native-macos-arm64', self.folder / 'native')

    def test_all_fourteen_values_are_derived_and_native_direction_is_correct(self):
        self.retained_matrix()
        values = cli.figures(self.folder, 'docs/reports/test')
        self.assertEqual(len(values), 14)
        self.assertEqual({v['runtime'] for v in values}, {'Native', 'Node.js'})
        self.assertEqual({v['platform'] for v in values}, {'macos-arm64'})
        self.assertNotIn('v1', {v['id'] for v in values})
        original = json.loads((cli.REPO / 'homepage/app/data/native-benchmarks.json').read_text())['platforms'][0]
        expected = {v['id']: v for v in original['figures']}
        for value in values:
            if value['id'] in cli.LEGACY:
                self.assertEqual(round(value['fresh'], 2), expected[value['id']]['fresh'])
                self.assertEqual(round(value['reuse'], 2), expected[value['id']]['reuse'])
                self.assertEqual(value['documents'], expected[value['id']]['documents'])
        markdown = next(v for v in values if v['id'] == 'markdown-rs')
        result = cli.read(self.folder / 'native-markdown-rs', 'summary')
        self.assertEqual(markdown['fresh'], result['v2_relative_throughput']['fresh'])

    def test_mixed_historical_runs_are_not_a_publishable_suite(self):
        self.retained_matrix()
        cli.checksums(self.folder)
        with self.assertRaises(AssertionError):
            cli.publish(self.folder.parent, '2099-01-01-test', check=True)

    def test_missing_project_cannot_produce_values(self):
        self.retained_matrix()
        shutil.rmtree(self.folder / 'node-marked')
        with self.assertRaises(FileNotFoundError):
            cli.figures(self.folder, 'docs/reports/test')

    def test_new_committed_core_requires_complete_source_proof(self):
        source = cli.REPO / 'docs/reports/2026-09-30-ecosystem-platforms/linux-x86-64/node-marked'
        shutil.copytree(source, self.folder / 'pair')
        pair = self.folder / 'pair'
        config, _ = cli.eco_archive.validate(pair, self.suite['revision'])
        config['local_source_sha256'] = {}
        cli.write(pair / 'run.json', config)
        with self.assertRaises(AssertionError):
            cli.eco_archive.validate(pair, self.suite['revision'])

    def test_false_summary_is_rejected_from_compressed_real_evidence(self):
        source = cli.REPO / 'docs/reports/2026-09-30-ecosystem-platforms/macos-arm64/native-cmark'
        shutil.copytree(source, self.folder / 'pair')
        pair = self.folder / 'pair'
        cli.eco_archive.validate(pair)
        summary = cli.read(pair, 'summary')
        summary['v2_relative_throughput']['fresh'] *= 2
        cli.write(pair / 'summary.json', summary)
        with self.assertRaises(AssertionError):
            cli.eco_archive.validate(pair)

    def test_shortened_windows_are_rejected_even_with_updated_file_hashes(self):
        source = cli.REPO / 'docs/reports/2026-09-30-ecosystem-platforms/macos-arm64/native-cmark'
        shutil.copytree(source, self.folder / 'pair')
        pair = self.folder / 'pair'
        rows = cli.read(pair, 'samples')
        rows[0]['v2']['elapsed_ns'] = 1
        (pair / 'samples.json.gz').write_bytes(gzip.compress(json.dumps(rows).encode(), mtime=0))
        cli.checksums(pair)
        cli.checksums(pair, check=True)
        with self.assertRaises(AssertionError):
            cli.eco_archive.validate(pair)

    def test_checksum_drift_and_path_escape_are_rejected(self):
        cli.checksums(self.folder)
        (self.folder / 'new.txt').write_text('unretained change')
        with self.assertRaises(AssertionError):
            cli.checksums(self.folder, check=True)
        for name in ('../escape', '/tmp/report', '2026-10-01-../../escape', 'undated'):
            with self.assertRaises(AssertionError):
                cli.report_path(name)

    def test_only_measured_platform_values_are_replaced(self):
        self.retained_matrix()
        values = cli.figures(self.folder, 'docs/reports/test')
        publisher = cli.load('test_publisher', cli.ECO / 'publish_values.py')
        # Check the real keyed replacement rule without constructing a false
        # publishable report. Publication validity has its own rejection cases.
        historical = publisher.figures()
        for system, architecture in [('Darwin', 'arm64'), ('Darwin', 'x86_64'), ('Linux', 'x86_64'), ('Linux', 'aarch64')]:
            details = cli.platform_details(system, architecture)
            cli.write(self.folder / 'suite.json', {**self.suite, **details})
            values = cli.figures(self.folder, 'docs/reports/test')
            combined = publisher.merge_figures(historical, values)
            key = details['platform']
            self.assertEqual([v for v in combined if v['platform'] != key],
                             [v for v in historical if v['platform'] != key])
            self.assertEqual(len([v for v in combined if v['platform'] == key]), 14)
            self.assertEqual(len({(v['platform'], v['id']) for v in combined}), len(combined))

    def test_platform_aliases_and_addon_libraries(self):
        for system, slug in [('Darwin', 'macos'), ('Linux', 'linux')]:
            for architecture in ('x64', 'AMD64', 'x86_64', 'x86-64'):
                self.assertEqual(cli.platform_details(system, architecture)['platform'], slug + '-x86-64')
            for architecture in ('arm64', 'aarch64'):
                self.assertEqual(cli.platform_details(system, architecture)['platform'], slug + '-arm64')
        self.assertEqual(cli.addon_library('Darwin'), 'libferromark_node.dylib')
        self.assertEqual(cli.addon_library('Linux'), 'libferromark_node.so')
        with self.assertRaises(ValueError):
            cli.platform_details('Windows', 'x64')
        with self.assertRaises(ValueError):
            cli.platform_details('Linux', 'riscv64')

    def test_recorded_platform_checks_allow_import_on_another_host(self):
        for slug, system, architecture in [('macos-arm64', 'Darwin', 'arm64'), ('linux-x86-64', 'Linux', 'x86_64')]:
            directory = cli.REPO / ('docs/reports/2026-09-24-native-' + slug)
            config, build = cli.read(directory, 'run'), cli.read(directory, 'build')
            suite = {**cli.platform_details(system, architecture), 'host_platform': config['host_before']['platform']}
            self.assertEqual(cli.verify_platform(suite, config, build)['platform'], slug)
            other = {**suite, **cli.platform_details(system, 'x86_64' if architecture == 'arm64' else 'arm64')}
            with self.assertRaises(AssertionError):
                cli.verify_platform(other, config, build)
            with self.assertRaises(AssertionError):
                cli.verify_platform({**suite, 'host_platform': 'another host platform'}, config, build)

    def test_report_selection_preserves_other_platforms(self):
        initial = {'reports': {'macos-arm64': '2026-10-01-arm', 'linux-x86-64': '2026-10-01-linux'}}
        selected = cli.select_report(initial, 'linux-x86-64', '2026-10-02-linux')
        selected = cli.select_report(selected, 'macos-x86-64', '2026-10-03-intel')
        self.assertEqual(selected['reports']['macos-arm64'], initial['reports']['macos-arm64'])
        self.assertEqual(selected['reports']['linux-x86-64'], '2026-10-02-linux')
        self.assertEqual(selected['reports']['macos-x86-64'], '2026-10-03-intel')
        self.assertEqual(initial['reports']['linux-x86-64'], '2026-10-01-linux')

    def test_cli_help_and_invalid_cooldown_have_no_filesystem_effect(self):
        command = [sys.executable, str(Path(cli.__file__))]
        result = subprocess.run([*command, '--help'], capture_output=True, text=True)
        self.assertEqual(result.returncode, 0)
        self.assertIn('publish', result.stdout)
        destination = Path(self.temp.name) / 'must not exist'
        result = subprocess.run([*command, 'run', str(destination), '--cooldown-seconds', '-1'], capture_output=True, text=True)
        self.assertEqual(result.returncode, 2)
        self.assertFalse(destination.exists())


if __name__ == '__main__':
    unittest.main()
