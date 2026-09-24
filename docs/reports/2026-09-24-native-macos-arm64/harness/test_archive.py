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


def provenance_facts(host_kind, system, seed_lock):
    """The fields write_provenance reads, for one platform and kind of host."""
    platform = dict(system=system, host_triple='triple', clang='clang version 1\nTarget: x', linker='ld')
    engines = {name: dict(revision=name + '-rev') for name in ('ferromark_v2', 'ferromark_v1', 'ox_content', 'md4c', 'bun')}
    build = dict(platform=platform, engines=engines, rustc='rustc 1.0\nLLVM version: 20.1', toolchain='nightly',
                 rustflags='-C target-cpu=generic', binary_sha256='b', lock_sha256='l',
                 optimization=dict(opt_level=3, lto='fat', codegen_units=1, panic='abort'))
    return dict(build=build, run=dict(observations=[]), host_kind=host_kind, measurement_note='Load was 1.0.',
                platform_label='macOS arm64' if system == 'Darwin' else 'Linux x86-64', source_audit={},
                source=dict(origin='Measured somewhere.', host_summary='a host', harness_revision='abc'),
                registry=dict(seed_lock=seed_lock, identical_to_seed=True, registry_packages=1))


class ArchiveTests(unittest.TestCase):
    def test_paths_are_recorded_relative_to_their_repository(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            (root / 'checkout/.git').mkdir(parents=True)
            report = root / 'checkout/docs/reports/2026-01-01-native-x'
            report.mkdir(parents=True)
            self.assertEqual(archive.portable_path(report / 'Cargo.lock'),
                             'docs/reports/2026-01-01-native-x/Cargo.lock')
            # A worktree's .git is a file, not a directory.
            (root / 'worktree/docs').mkdir(parents=True)
            (root / 'worktree/.git').write_text('gitdir: elsewhere\n')
            self.assertEqual(archive.portable_path(root / 'worktree/docs/Cargo.lock'), 'docs/Cargo.lock')
            # Outside any checkout, only the directory and file name remain.
            (root / 'loose').mkdir()
            self.assertEqual(archive.portable_path(root / 'loose/Cargo.lock'), 'loose/Cargo.lock')

    def test_generated_text_may_not_name_an_absolute_input_path(self):
        with tempfile.TemporaryDirectory() as directory:
            results = Path(directory).resolve() / 'results'
            self.assertEqual(archive.leaked_paths(f'read from {results}/run.json', [results]), [str(results)])
            self.assertEqual(archive.leaked_paths('read from results/run.json', [results]), [])
            self.assertEqual(archive.leaked_paths(f'seeded from {Path.home()}/x.lock', []), [str(Path.home())])

    def test_a_ci_runner_keeps_its_shared_machine_caveat(self):
        claims = '\n'.join(archive.host_claims('github-hosted', 0.0))
        self.assertIn('A GitHub-hosted runner is a virtual machine on shared hardware', claims)
        self.assertIn('**Hypervisor steal** was 0.00%', claims)
        self.assertIn('not recorded', '\n'.join(archive.host_claims('github-hosted', None)))

    def test_a_local_run_claims_a_local_machine_and_points_at_its_load(self):
        claims = '\n'.join(archive.host_claims('local', None))
        self.assertIn('A single local machine', claims)
        self.assertIn('desktop workloads can share', claims)
        self.assertIn('(PROVENANCE.md#measurement-conditions)', claims)
        for word in ('GitHub', 'runner', 'virtual machine', 'Hypervisor'):
            self.assertNotIn(word, claims)
        with self.assertRaises(ValueError):
            archive.host_claims('cloud', None)

    def test_provenance_follows_the_host_kind(self):
        with tempfile.TemporaryDirectory() as directory:
            out = Path(directory)
            archive.write_provenance(out, provenance_facts('local', 'Darwin', 'docs/reports/r/Cargo.lock'))
            local = (out / 'PROVENANCE.md').read_text()
            archive.write_provenance(out, provenance_facts('github-hosted', 'Linux', 'docs/reports/r/Cargo.lock'))
            hosted = (out / 'PROVENANCE.md').read_text()
        self.assertIn('The harness at `abc` exported the measured revisions', local)
        self.assertIn('on a macOS arm64 host with clang and `--ferromark-v2-revision ferromark_v2-rev`', local)
        for text in ('workflow', 'runner', 'virtual machine', 'transparent huge page mode', 'steal'):
            self.assertNotIn(text, local)
        self.assertIn('The workflow checked out `abc` for the harness', hosted)
        self.assertIn('gh workflow run native-comparison.yml -f revision=ferromark_v2-rev', hosted)
        self.assertIn("The runner's transparent huge page mode", hosted)
        for text in (local, hosted):
            self.assertIn('seeded Cargo with `docs/reports/r/Cargo.lock`', text)

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
            self.assertEqual(result['seed_lock'], Path(directory).resolve().name + '/seed.lock')
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
