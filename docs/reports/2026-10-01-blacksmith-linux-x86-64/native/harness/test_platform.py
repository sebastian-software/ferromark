#!/usr/bin/env python3
"""Guard the macOS/Linux build differences and the Linux host probe."""
from pathlib import Path
import unittest

import prepare
import run

HERE = Path(__file__).resolve().parent


class BuildPlatformTests(unittest.TestCase):
    def test_macos_keeps_the_original_runtime_and_linker(self):
        darwin = prepare.build_platform('Darwin')
        self.assertEqual(darwin['cxx_runtime'], 'c++')
        self.assertIsNone(darwin['rust_linker'])
        self.assertEqual(prepare.linker_environment(darwin, 'aarch64-apple-darwin'), {})

    def test_linux_uses_libstdcxx_and_clang_as_the_link_driver(self):
        linux = prepare.build_platform('Linux')
        self.assertEqual(linux['cxx_runtime'], 'stdc++')
        self.assertEqual(prepare.linker_environment(linux, 'x86_64-unknown-linux-gnu'),
                         {'CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER': 'clang'})

    def test_other_hosts_are_refused(self):
        with self.assertRaises(SystemExit):
            prepare.build_platform('Windows')

    def test_both_platforms_describe_the_same_aspects(self):
        self.assertEqual(set(prepare.PLATFORMS['Darwin']), set(prepare.PLATFORMS['Linux']))

    def test_host_triple_comes_from_rustc(self):
        self.assertEqual(prepare.host_triple('rustc 1.99.0\nhost: x86_64-unknown-linux-gnu\n'),
                         'x86_64-unknown-linux-gnu')

    def test_native_adapters_select_a_branch_per_platform(self):
        for name in ('native.h', 'stack.c'):
            text = (HERE / name).read_text()
            self.assertIn('defined(__APPLE__)', text)
            self.assertIn('defined(__linux__)', text)
            self.assertIn('#error', text)
        self.assertIn('pthread_getattr_np', (HERE / 'stack.c').read_text())


class HostProbeTests(unittest.TestCase):
    def test_proc_stat_reads_the_aggregate_cpu_line(self):
        text = 'cpu  10 1 5 100 2 0 1 7 0 0\ncpu0 5 0 2 50 1 0 0 3 0 0\n'
        counters = run.proc_stat_cpu(text)
        self.assertEqual(counters['user'], 10)
        self.assertEqual(counters['steal'], 7)
        with self.assertRaises(ValueError):
            run.proc_stat_cpu('intr 1 2 3\n')


if __name__ == '__main__':
    unittest.main()
