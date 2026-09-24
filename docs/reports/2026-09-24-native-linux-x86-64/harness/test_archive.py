#!/usr/bin/env python3
"""Guard the checks archive.py runs while assembling a report directory."""
from pathlib import Path
import tempfile
import unittest

import archive
import run


def lock(directory, name, packages):
    lines = ['version = 4', '']
    for package, version, checksum in packages:
        lines += ['[[package]]', f'name = "{package}"', f'version = "{version}"',
                  'source = "registry+https://github.com/rust-lang/crates.io-index"',
                  f'checksum = "{checksum}"', '']
    lines += ['[[package]]', 'name = "local"', 'version = "1.0.0"', '']
    path = Path(directory) / name
    path.write_text('\n'.join(lines))
    return path


class ArchiveTests(unittest.TestCase):
    def test_steal_share_uses_the_difference_between_observations(self):
        before = {'proc_stat_cpu': dict(user=0, nice=0, system=0, idle=0, iowait=0, irq=0, softirq=0,
                                        steal=0, guest=0, guest_nice=0)}
        after = {'proc_stat_cpu': dict(user=60, nice=0, system=10, idle=20, iowait=0, irq=0, softirq=0,
                                       steal=10, guest=5, guest_nice=0)}
        self.assertAlmostEqual(archive.cpu_steal(before, after), 0.10)
        self.assertIsNone(archive.cpu_steal({'power': 'AC'}, {'power': 'AC'}))

    def test_platform_names(self):
        self.assertEqual(archive.platform_names({'platform': {'system': 'Linux', 'machine': 'x86_64'}}),
                         ('linux-x86-64', 'Linux x86-64'))
        self.assertEqual(archive.platform_names({'platform': {'system': 'Darwin', 'machine': 'arm64'}}),
                         ('macos-arm64', 'macOS arm64'))

    def test_registry_check_reports_any_drift_from_the_seed(self):
        with tempfile.TemporaryDirectory() as directory:
            seed = lock(directory, 'seed.lock', [('a', '1.0.0', 'x'), ('b', '2.0.0', 'y')])
            same = lock(directory, 'same.lock', [('b', '2.0.0', 'y'), ('a', '1.0.0', 'x')])
            drift = lock(directory, 'drift.lock', [('a', '1.0.0', 'z'), ('c', '3.0.0', 'w')])
            result = archive.registry_check(same, seed)
            self.assertTrue(result['identical_to_seed'])
            self.assertEqual(result['registry_packages'], 2)
            result = archive.registry_check(drift, seed)
            self.assertFalse(result['identical_to_seed'])
            self.assertEqual((result['added'], result['removed'], result['checksum_changed']),
                             (['c@3.0.0'], ['b@2.0.0'], ['a@1.0.0']))

    def test_output_comparison_names_every_changed_engine_output(self):
        def entry(**outputs):
            html = {e: '<p>x</p>' for e in run.ENGINES} | outputs
            return {'outputs': html, 'versus_v2': {e: 'exact' for e in run.ENGINES}}
        result = archive.compare_outputs({'a': entry(), 'b': entry(md4c='<p>y</p>')},
                                         {'a': entry(), 'b': entry(), 'c': entry()})
        self.assertEqual(result['documents_compared'], 2)
        self.assertEqual(result['engines']['md4c'], {'identical': 1, 'different': ['b']})
        self.assertEqual(result['engines']['v2']['identical'], 2)
        self.assertEqual(result['only_in_reference'], ['c'])

    def test_window_audit_rejects_checksum_and_coverage_gaps(self):
        engines = list(run.ENGINES)
        config = dict(engines=engines, modes=['fresh'], rounds=1, samples=1, window_ms=1,
                      jobs=[dict(name='doc', members=['doc'])])
        verification = {'doc': {'outputs': {e: 'abc' for e in engines}}}
        timing = dict(iterations=2, elapsed_ns=1_000_000, checksum=6)
        sample = dict(round=0, sample=0, case='doc', mode='fresh', order=engines,
                      **{e: dict(timing) for e in engines})
        self.assertEqual(archive.audit_windows(config, [sample], verification), len(engines))
        broken = dict(sample, v2=dict(timing, checksum=5))
        with self.assertRaises(AssertionError):
            archive.audit_windows(config, [broken], verification)
        with self.assertRaises(AssertionError):
            archive.audit_windows(dict(config, samples=2), [sample], verification)


if __name__ == '__main__':
    unittest.main()
