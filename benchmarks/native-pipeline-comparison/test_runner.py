"""Contracts for equivalent-work admission and persistent-worker timing evidence."""
import copy
import unittest

from common import catalog, probes, workload_review
from run import check_switches, summarize, validate_window, verify_row


class ProtocolTests(unittest.TestCase):
    def test_catalog_reuses_core_and_realistic_extension_workloads(self):
        rows = catalog()
        self.assertEqual(len(rows), 18)
        self.assertEqual(len({r["case"] for r in rows}), len(rows))
        self.assertEqual({r["flags"] for r in rows}, {0, 1, 2, 4, 7})
        self.assertTrue(any(r["case"] == "tables/tables-commonmark-inline" for r in rows))

    def test_verification_rejects_wrong_configuration(self):
        case = {"case": "commonmark/x", "flags": 0, "input": "hello"}
        row = {"case": case["case"], "flags": 0, "bytes": 5, "engine": "satteri", "html": "<p>hello</p>", "options": "core"}
        verify_row(row, case, "satteri")
        for key, value in (("case", "other"), ("flags", 1), ("bytes", 6), ("engine", "goldmark")):
            with self.assertRaises(ValueError):
                verify_row({**row, key: value}, case, "satteri")

    def test_missing_feature_is_not_admitted(self):
        review = workload_review("strikethrough/features", {
            "ferromark": "<p><del>old</del></p>", "satteri": "<p>~~old~~</p>"}, parsers={"ferromark", "satteri"})
        self.assertFalse(review["comparable"])

    def test_switch_probe_rejects_missing_tables_even_when_both_engines_would_agree(self):
        rows = [{**c, "engine": "satteri", "html": ""} for c in probes()]
        with self.assertRaisesRegex(ValueError, "probe failed"):
            check_switches(rows)

    def test_windows_require_complete_time_and_consistent_counts(self):
        row = {"case": "x", "engine": "satteri", "count": 16, "elapsed_ns": 20_000_000, "ns_per_render": 1_250_000.0}
        validate_window(row, "x", "satteri", 20)
        for key, value in (("count", 0), ("count", 17), ("elapsed_ns", 100), ("ns_per_render", float("nan")), ("ns_per_render", 1)):
            with self.assertRaises(ValueError):
                validate_window({**row, key: value}, "x", "satteri", 20)

    def test_summaries_require_every_sample_once_in_every_run(self):
        protocol = {"runs": 2, "samples": 2, "window_ms": 20}
        rows = [[{"case": "x", "engine": engine, "count": 16, "elapsed_ns": 20_000_000,
                  "ns_per_render": 1_250_000.0, "sample": sample, "run": repeat}
                 for engine in ("ferromark", "satteri") for sample in range(2)] for repeat in range(2)]
        outputs = [{"case": "x", "engine": engine, "bytes": 3, "html": "<p>x</p>"} for engine in ("ferromark", "satteri")]
        self.assertEqual(summarize(rows, ["x"], "satteri", protocol, outputs)[0]["median_ns"], 1_250_000.0)
        for broken in (rows[:1], [rows[0][:-1], rows[1]], [rows[0] + rows[0][:1], rows[1]]):
            with self.assertRaises(ValueError):
                summarize(broken, ["x"], "satteri", protocol, outputs)
        broken = copy.deepcopy(rows)
        broken[1][0]["run"] = 0
        with self.assertRaises(ValueError):
            summarize(broken, ["x"], "satteri", protocol, outputs)



class FixedDialectTests(unittest.TestCase):
    def test_fixed_dialect_requires_actual_extended_output(self):
        from run import check_fixed_dialect
        good = '<table></table><del>old</del><input type="checkbox"><a href="http://example.com">x</a>'
        rows = [{"html": good} for _ in range(8)]
        check_fixed_dialect(rows)
        for token in ('<table>', '<del>old</del>', 'type="checkbox"', 'href="http'):
            broken = copy.deepcopy(rows)
            broken[0]["html"] = broken[0]["html"].replace(token, '')
            with self.assertRaises(ValueError):
                check_fixed_dialect(broken)

    def test_fixed_dialect_report_rejects_admitted_timing(self):
        from pathlib import Path
        from report import render_engine
        metadata = {"protocol": {"mode": "verify"}, "started_unix": 1, "finished_unix": 2,
            "build": {"adapter": {"name": "md4x", "label": "MD4X", "fixed_dialect": True}},
            "pairs": {"md4x": {"selected": ["commonmark/5k"]}}}
        with self.assertRaisesRegex(ValueError, "must not admit timing"):
            render_engine(Path("unused"), metadata)


class WorkerCommandTests(unittest.TestCase):
    def test_registered_executables_are_independent_of_the_callers_directory(self):
        from common import validate_worker_command
        command = ["/tmp/build/engine-driver", "--native"]
        self.assertEqual(validate_worker_command(command), command)
        for invalid in ([], "driver", ["driver"], ["./driver"], ["/tmp/driver", 1]):
            with self.assertRaisesRegex(ValueError, "absolute executable"):
                validate_worker_command(invalid)


class EngineArchiveTests(unittest.TestCase):
    def test_registered_engine_archives_regenerate_and_match_checksums(self):
        import hashlib
        from common import REPO
        from report import load, render
        for metadata_path in sorted((REPO / "docs/reports").glob("*/metadata.json.gz")):
            folder = metadata_path.parent
            metadata = load(folder / "metadata.json")
            build = metadata.get("build", {})
            if not isinstance(build, dict) or "adapter" not in build:
                continue
            with self.subTest(archive=folder.name):
                self.assertEqual((folder / "REPORT.md").read_text(), render(folder))
                for line in (folder / "SHA256SUMS").read_text().splitlines():
                    digest, name = line.split("  ", 1)
                    self.assertEqual(hashlib.sha256((folder / name).read_bytes()).hexdigest(), digest)


if __name__ == "__main__":
    unittest.main()
