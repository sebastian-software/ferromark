"""Guard real runner observations against upgrades and cross-platform drift."""
import copy
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

import runner_context as context


def recorded(profile):
    expected = context.PROFILES[profile]
    return {'schema': 1, 'provider': 'Blacksmith', 'profile': profile, 'runner_label': expected['label'],
            'run_url': 'https://github.com/sebastian-software/ferromark/actions/runs/123',
            'observed': {'system': expected['system'], 'architecture': expected['architecture'],
                         'cpus': expected['cpus'], 'available_cpus': expected['cpus'],
                         'memory_bytes': expected['memory_gib'] * context.GIB,
                         'os_version': expected['os_version'] + ('.1' if expected['system'] == 'Darwin' else ''),
                         'cpu': expected.get('cpu', 'Recorded x64 CPU')}}


class RunnerContextTests(unittest.TestCase):
    def test_both_native_profiles_accept_the_advertised_resources(self):
        for profile in context.PROFILES:
            self.assertEqual(context.validate(recorded(profile)), context.PROFILES[profile])

    def test_upgraded_or_restricted_cpu_allocations_are_rejected(self):
        for profile in context.PROFILES:
            for key in ('cpus', 'available_cpus'):
                for delta in (-1, 1):
                    with self.subTest(profile=profile, key=key, delta=delta):
                        value = recorded(profile)
                        value['observed'][key] += delta
                        with self.assertRaises(ValueError):
                            context.validate(value)

    def test_memory_reservations_are_allowed_but_different_sizes_are_rejected(self):
        value = recorded('linux-x86-64')
        value['observed']['memory_bytes'] = int(15.5 * context.GIB)
        context.validate(value)
        for memory in (8, 12, 24, 32):
            value['observed']['memory_bytes'] = memory * context.GIB
            with self.assertRaises(ValueError):
                context.validate(value)

    def test_architecture_os_cpu_and_runner_label_drift_are_rejected(self):
        base = recorded('macos-arm64')
        for key, bad in (('architecture', 'x86_64'), ('system', 'Linux'), ('cpu', 'Apple M2'), ('os_version', '15.7')):
            value = copy.deepcopy(base)
            value['observed'][key] = bad
            with self.assertRaises(ValueError):
                context.validate(value)
        base['runner_label'] = context.PROFILES['linux-x86-64']['label']
        with self.assertRaises(ValueError):
            context.validate(base)

    def test_another_organization_or_repository_is_rejected(self):
        for url in ('https://github.com/another-org/ferromark/actions/runs/123',
                    'https://github.com/sebastian-software/another-repo/actions/runs/123',
                    'https://github.com/sebastian-software/ferromark/actions/runs/123?x=1'):
            value = recorded('macos-arm64')
            value['run_url'] = url
            with self.assertRaises(ValueError):
                context.validate(value)

    def test_real_cli_rejects_wrong_repository_without_writing(self):
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory) / 'context.json'
            env = {**os.environ, 'GITHUB_REPOSITORY': 'another-org/ferromark'}
            result = subprocess.run([sys.executable, context.__file__, 'macos-arm64', str(output)],
                                    env=env, capture_output=True, text=True)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn('sebastian-software/ferromark', result.stderr)
            self.assertFalse(output.exists())


if __name__ == '__main__':
    unittest.main()
