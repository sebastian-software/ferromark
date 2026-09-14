"""Guards for benchmark admission and measurement checksums."""

import importlib.util
import io
from pathlib import Path
import tarfile
import tempfile
import unittest
from unittest.mock import patch

SPEC = importlib.util.spec_from_file_location("simd_runner", Path(__file__).with_name("run.py"))
RUN = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(RUN)
PREPARE_SPEC = importlib.util.spec_from_file_location("simd_prepare", Path(__file__).with_name("prepare.py"))
PREPARE = importlib.util.module_from_spec(PREPARE_SPEC)
PREPARE_SPEC.loader.exec_module(PREPARE)


class RunnerGuards(unittest.TestCase):
    def test_baseline_verification_checks_archived_content_without_git_metadata(self):
        contents = {"Cargo.toml": b"manifest", "Cargo.lock": b"lock", "crates/core.rs": b"core"}
        archive = io.BytesIO()
        with tarfile.open(fileobj=archive, mode="w") as stream:
            for name, data in contents.items():
                info = tarfile.TarInfo(name)
                info.size = len(data)
                stream.addfile(info, io.BytesIO(data))
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "crates").mkdir()
            for name, data in contents.items():
                (root / name).write_bytes(data)
            with patch.object(PREPARE.subprocess, "check_output", return_value=archive.getvalue()):
                self.assertEqual(PREPARE.verify_baseline(root)["verified_core_files"], 3)
                (root / "crates/core.rs").write_bytes(b"different core")
                with self.assertRaisesRegex(ValueError, "baseline core differs"):
                    PREPARE.verify_baseline(root)

    def test_empty_html_survives_protocol(self):
        worker = RUN.Worker.__new__(RUN.Worker)
        worker.command = lambda _: None
        lines = iter(["result 0  446f63756d656e74 0 0", "done"])
        worker.line = lambda: next(lines)
        self.assertEqual(worker.verify()[0]["html"], "")

    def test_live_check_rejects_ast_change_with_same_html(self):
        expected = {"html": "same", "ast_debug": "before", "children": 1,
                    "arena_capacity_bytes": 1024}
        with self.assertRaises(AssertionError):
            RUN.assert_live_result([{**expected, "ast_debug": "after"}], expected, "case")

    def test_live_check_allows_arena_reset_to_keep_only_largest_chunk(self):
        expected = {"html": "same", "ast_debug": "same", "children": 1,
                    "arena_capacity_bytes": 1024}
        RUN.assert_live_result([{**expected, "arena_capacity_bytes": 512}], expected, "case")

    def test_cross_engine_capacity_change_is_rejected(self):
        with self.assertRaises(AssertionError):
            RUN.assert_same_capacity({"baseline": 1024, "candidate": 2048}, "case")

    def test_metric_counts_utf8_bytes_and_parse_children(self):
        verified = {"modes": {
            mode: {"baseline": {"html": "é中", "children": 2}}
            for mode in ("reuse", "parse")
        }}
        self.assertEqual(RUN.metric(verified, "reuse", "baseline"), 5)
        self.assertEqual(RUN.metric(verified, "parse", "baseline"), 2)

    def test_corrupt_timing_checksum_is_rejected(self):
        class BadWorker:
            def bench(self, _):
                return {"iterations": 32, "elapsed_ns": 1000, "checksum": 159}

        with self.assertRaises(AssertionError):
            RUN.checked_bench(BadWorker(), 1000, 5)

    def test_timing_checksum_wraps_at_usize_width(self):
        class WrappingWorker:
            def bench(self, _):
                return {"iterations": 32, "elapsed_ns": 1000, "checksum": 64}

        RUN.checked_bench(WrappingWorker(), 1000, (1 << 63) + 2)


if __name__ == "__main__":
    unittest.main()
