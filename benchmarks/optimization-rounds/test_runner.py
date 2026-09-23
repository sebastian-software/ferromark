"""Unit guards for exact output admission and generic build metadata."""

import importlib.util
import json
from pathlib import Path
import re
import sys
import tempfile
import unittest
from unittest.mock import patch


ROOT = Path(__file__).parent


def load(name):
    spec = importlib.util.spec_from_file_location(name, ROOT / f"{name}.py")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


RUN = load("run")
PREPARE = load("prepare")
MAKE_CORPUS = load("make_corpus")


class RunnerGuards(unittest.TestCase):
    @staticmethod
    def _build_payload(capacity=(1024, 512), mutation=None):
        html, ast, children = "<p>same</p>\n", "Document { ... }", 1
        if mutation == "html":
            html = "<p>changed</p>\n"
        elif mutation == "ast":
            ast = "Document { changed }"
        elif mutation == "children":
            children = 2
        return {
            "baseline": {"html": html, "ast_debug": ast, "children": children,
                          "arena_capacity_bytes": capacity[0]},
            "candidate": {"html": html, "ast_debug": ast, "children": children,
                           "arena_capacity_bytes": capacity[1]},
        }

    def test_verify_case_accepts_capacity_change_and_records_both_values(self):
        payload = self._build_payload()

        class FakeWorker:
            def __init__(self, binary, profile, mode, paths):
                self.engine = Path(binary).name

            def verify(self):
                return [payload[self.engine]]

            def close(self):
                pass

        build = {"engines": {"baseline": {"binary": "/tmp/baseline"},
                              "candidate": {"binary": "/tmp/candidate"}}}
        case = {"name": "capacity-case", "profile": "commonmark"}
        with patch.object(RUN, "Worker", FakeWorker):
            result = RUN.verify_case(build, Path("input.md"), case, ["fresh"])
        self.assertEqual(result["modes"]["fresh"]["baseline"]["arena_capacity_bytes"], 1024)
        self.assertEqual(result["modes"]["fresh"]["candidate"]["arena_capacity_bytes"], 512)

    def test_verify_case_rejects_html_ast_and_children_changes(self):
        build = {"engines": {"baseline": {"binary": "/tmp/baseline"},
                              "candidate": {"binary": "/tmp/candidate"}}}
        case = {"name": "output-case", "profile": "commonmark"}
        for mutation in ("html", "ast", "children"):
            payload = self._build_payload(mutation=mutation)

            class FakeWorker:
                def __init__(self, binary, profile, mode, paths):
                    self.engine = Path(binary).name

                def verify(self):
                    value = dict(payload[self.engine])
                    if self.engine == "candidate":
                        value["html"] = payload["baseline"]["html"]
                        value["ast_debug"] = payload["baseline"]["ast_debug"]
                        value["children"] = payload["baseline"]["children"]
                        if mutation == "html": value["html"] = "<p>candidate</p>\n"
                        if mutation == "ast": value["ast_debug"] = "Document { candidate }"
                        if mutation == "children": value["children"] = 3
                    return [value]

                def close(self):
                    pass

            with self.subTest(mutation=mutation), patch.object(RUN, "Worker", FakeWorker):
                with self.assertRaises(AssertionError):
                    RUN.verify_case(build, Path("input.md"), case, ["fresh"])

    def test_metric_counts_utf8_html_bytes_and_parse_children(self):
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

    def test_cache_requires_matching_source_worker_lock_and_lto(self):
        with tempfile.TemporaryDirectory() as directory:
            source = Path(directory) / "source"
            source.mkdir()
            (source / "Cargo.lock").write_text("lock")
            binary = Path(directory) / "worker"
            binary.write_bytes(b"binary")
            cached = {
                "binary": str(binary), "binary_sha256": PREPARE.sha256(binary),
                "source_tree_sha256": PREPARE.tree_sha(source),
                "source_lock_sha256": PREPARE.sha256(source / "Cargo.lock"),
                "worker_sha256": "worker", "lto": "fat",
            }
            self.assertTrue(PREPARE.cache_matches(cached, source, "worker", "fat"))
            self.assertFalse(PREPARE.cache_matches(cached, source, "worker", "thin"))

    def test_corpus_case_records_byte_integrity_and_category(self):
        item = RUN.load_corpus(ROOT / "../../docs/reports/2026-09-14-simd-round/corpus.json.gz")["cases"][0]
        raw = item["input"].encode()
        self.assertEqual(len(raw), item["byte_count"])
        self.assertEqual(RUN.hashlib.sha256(raw).hexdigest(), item["sha256"])

    def test_generated_corpus_has_sparse_escapes_long_closers_and_all_scanner_profiles(self):
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory) / "corpus.json"
            with patch.object(sys, "argv", ["make_corpus.py", str(output), "--include-scanner-diagnostics"]):
                MAKE_CORPUS.main()
            payload = json.loads(output.read_text())
        cases = {case["name"]: case for case in payload["cases"]}
        for size in (256, 4096, 16000):
            sparse = cases[f"table-sparse-{size}"]["input"]
            self.assertIn(r"\|", sparse)
        for size in (64, 512, 2048):
            closers = cases[f"autolink-closers-{size}"]["input"]
            run = re.search(r"(?:\)\]\})+", closers)
            self.assertIsNotNone(run)
            self.assertEqual(len(run.group()), (size // 3) * 3)
        for bits in range(8):
            for shape in ("plain", "sparse", "disabled"):
                for size in (4096, 16000):
                    self.assertIn(f"scanner-opt-{bits}-{shape}-{size}", cases)

    def test_run_rejects_corrupt_frozen_worker_and_binary(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            build = root / "build"
            build.mkdir()
            frozen_worker = build / "worker.rs"
            frozen_worker.write_bytes(b"worker")
            binaries = {}
            for engine in RUN.ENGINES:
                binary = build / engine
                binary.write_bytes(engine.encode())
                binaries[engine] = {"binary": str(binary), "binary_sha256": RUN.digest(binary)}
            metadata = {"worker_sha256": RUN.digest(frozen_worker), "engines": binaries}
            (build / "build.json").write_text(json.dumps(metadata))
            corpus = root / "corpus.json"
            text = "plain\n"
            raw = text.encode()
            corpus.write_text(json.dumps({"cases": [{
                "name": "one", "profile": "commonmark", "input": text,
                "byte_count": len(raw), "sha256": RUN.hashlib.sha256(raw).hexdigest(),
            }]}))
            output = root / "output"
            frozen_worker.write_bytes(b"corrupt")
            with patch.object(sys, "argv", ["run.py", str(build), str(corpus), str(output)]):
                with self.assertRaises(SystemExit):
                    RUN.main()
            frozen_worker.write_bytes(b"worker")
            binaries["candidate"]["binary_sha256"] = "wrong"
            (build / "build.json").write_text(json.dumps(metadata))
            with patch.object(sys, "argv", ["run.py", str(build), str(corpus), str(root / "output-2")]):
                with self.assertRaises(SystemExit):
                    RUN.main()


class StepSummary(unittest.TestCase):
    def test_summary_lists_geomeans_rounds_and_cases_below_the_floor(self):
        summary = load("step_summary")
        rows = [
            {"case": "slow", "mode": "parse", "baseline_over_candidate_median": 0.95,
             "round_medians": [0.94, 0.95, 0.96]},
            {"case": "fast", "mode": "parse", "baseline_over_candidate_median": 1.10,
             "round_medians": [1.09, 1.10, 1.11]},
        ]
        grouped = {"all": {"parse": {
            "cases": 2, "geometric_mean": 1.0223, "round_geometric_means": [1.01, 1.02, 1.03],
            "measured_above_1": 1,
        }}}
        with tempfile.TemporaryDirectory() as directory:
            run = Path(directory)
            (run / "summary.json").write_text(json.dumps(rows))
            (run / "grouped.json").write_text(json.dumps(grouped))
            text = summary.render(run, "A/A", host="x86_64 host")
        self.assertIn("### A/A", text)
        self.assertIn("x86_64 host", text)
        self.assertIn("| parse | 2 | 1.0223 | 1.010 / 1.020 / 1.030 | 1 | `slow` 0.950 |", text)
        self.assertNotIn("`fast`", text)


if __name__ == "__main__":
    unittest.main()
