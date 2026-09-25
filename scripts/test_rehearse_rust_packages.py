#!/usr/bin/env python3
"""Contract for the Rust archive rehearsal the CI `rust-packages` job runs.

A rehearsal only proves what it builds. `cargo package` verifies the library,
and the isolated consumer links it, but neither compiles the `tests/`,
`benches/` and `examples/` targets that ship inside the archive — the place
where an `include_str!` reaching past the `include` allow-list fails. These
tests pin the step that closes that gap and the record it leaves behind.
"""
import importlib.util
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts" / "rehearse-rust-packages.py"


def load_rehearsal():
    # The script's file name is not an importable module name.
    spec = importlib.util.spec_from_file_location("rehearse_rust_packages", SCRIPT)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


rehearsal = load_rehearsal()


class PackagedTargets(unittest.TestCase):
    def setUp(self):
        self.package = Path("/unpacked/ferromark-9.9.9")
        self.command = rehearsal.packaged_targets_command(self.package)

    def test_it_compiles_the_unpacked_archive_rather_than_the_checkout(self):
        self.assertEqual(self.command[:2], ["cargo", "check"])
        self.assertEqual(self.command[-2:], ["--manifest-path", str(self.package / "Cargo.toml")])

    def test_it_reaches_the_targets_the_library_build_leaves_out(self):
        # Without this the shipped `tests/` are never compiled from the archive.
        self.assertIn("--all-targets", self.command)

    def test_it_resolves_from_the_packaged_lockfile_without_the_network(self):
        self.assertIn("--locked", self.command)
        self.assertIn("--offline", self.command)

    def test_transform_targets_patch_the_core_to_the_extracted_archive(self):
        core = Path("/unpacked/ferromark-9.9.9")
        command = rehearsal.packaged_targets_command(self.package, core)
        self.assertIn("--config", command)
        self.assertIn(
            'patch.crates-io.ferromark.path="/unpacked/ferromark-9.9.9"',
            command,
        )

    def test_a_failing_step_fails_the_rehearsal(self):
        with tempfile.TemporaryDirectory() as directory:
            log = Path(directory) / "step.log"
            with self.assertRaises(subprocess.CalledProcessError):
                rehearsal.run([sys.executable, "-c", "raise SystemExit(3)"], log)


class RehearsalRecord(unittest.TestCase):
    def setUp(self):
        self.result = rehearsal.rehearsal_result("9.9.9", [])

    def test_every_verified_step_is_recorded(self):
        self.assertEqual(self.result["packaged_consumer"], "passed")
        self.assertEqual(self.result["packaged_targets"], "passed")

    def test_the_method_names_what_each_archive_itself_compiled(self):
        self.assertIn("each unpacked archive", self.result["method"])
        self.assertIn("test, bench and example targets", self.result["method"])

    def test_the_limits_stay_honest_about_publishing(self):
        self.assertIn("No registry upload", self.result["limits"])


if __name__ == "__main__":
    unittest.main()
