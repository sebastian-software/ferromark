"""Tests for the bounded official-cmark differential corpus."""

from __future__ import annotations

import importlib.util
import json
from pathlib import Path
import tempfile
import subprocess
import unittest
from unittest.mock import patch


HERE = Path(__file__).resolve().parent


def load(name: str, path: Path):
    spec = importlib.util.spec_from_file_location(name, path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load {path}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


FIXTURES = load("cmark_fixtures", HERE / "cmark_fixtures.py")
ORACLE = load("cmark_oracle", HERE / "cmark_oracle.py")


class CmarkCorpusTests(unittest.TestCase):
    def test_corpus_is_bounded_and_contains_complex_profiles(self):
        cases = FIXTURES.make_cases()
        self.assertGreaterEqual(len(cases), 100)
        self.assertLessEqual(len(cases), 300)
        self.assertEqual(len({case["id"] for case in cases}), len(cases))
        self.assertEqual({case["profile"] for case in cases}, {"commonmark", "gfm"})
        self.assertGreaterEqual(sum(case["category"] == "tilde-binding" for case in cases), 10)
        self.assertGreaterEqual(sum(case["category"] == "unicode-and-nul" for case in cases), 4)

    def test_requested_tilde_reproductions_are_persistent(self):
        fixture = json.loads((HERE / "cmark-fixtures.json").read_text(encoding="utf-8"))
        names = {case["id"] for case in fixture["cases"]}
        self.assertTrue({"tilde-01", "tilde-02", "tilde-03", "tilde-04", "tilde-05", "tilde-06"} <= names)
        markdown = {case["id"]: case["markdown"] for case in fixture["cases"]}
        self.assertIn("~foo ~ bar~\n", markdown.values())
        self.assertIn("~a `~` b~\n", markdown.values())

    def test_checked_in_fixture_is_exact_generator_output(self):
        expected = json.dumps(
            {"generator": "cmark_fixtures.py", "cases": FIXTURES.make_cases()},
            indent=2,
            ensure_ascii=False,
        ) + "\n"
        actual = (HERE / "cmark-fixtures.json").read_text(encoding="utf-8")
        self.assertEqual(actual, expected)

    def test_fixture_loader_rejects_duplicate_ids_and_wrong_size(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "fixtures.json"
            path.write_text(json.dumps({"cases": [{"id": "same"}] * 100}), encoding="utf-8")
            with self.assertRaises(ValueError):
                ORACLE.load_fixtures(path)

    def test_comparison_keeps_heading_only_separate_from_admitted_statuses(self):
        left = '<h1 id="left">Title</h1>'
        right = '<h1 id="right">Title</h1>'
        self.assertEqual(ORACLE.VERIFY.classify(left, right), "heading-id-only")
        self.assertFalse(ORACLE.VERIFY.admitted(ORACLE.VERIFY.classify(left, right)))

    def test_gfm_reference_profile_declares_extensions_and_tagfilter(self):
        class Completed:
            returncode = 0
            stdout = b""
            stderr = b""

        with patch.object(ORACLE.subprocess, "run", return_value=Completed()) as run:
            ORACLE.run_reference(Path("/tmp/cmark-gfm"), "gfm", "~x~\n")
        command = run.call_args.args[0]
        self.assertEqual(
            command[1:],
            ["-e", "table", "-e", "strikethrough", "-e", "autolink", "-e", "tasklist", "-e", "tagfilter"],
        )
        self.assertIn("tagfilter", command)
        self.assertNotIn("footnotes", command)

    def test_worker_crashes_timeouts_and_partial_json_are_errors(self):
        worker = ORACLE.Worker(Path('/tmp/test-worker'))
        for result in [
            subprocess.CompletedProcess([], 1, b'{"html":"","error":null}', b'failed'),
            subprocess.CompletedProcess([], 0, b'{"html":', b''),
            subprocess.CompletedProcess([], 0, b'{}', b''),
        ]:
            with self.subTest(result=result), patch.object(ORACLE.subprocess, 'run', return_value=result):
                self.assertIsNotNone(worker.render('x', 'commonmark')[1])
        with patch.object(ORACLE.subprocess, 'run', side_effect=subprocess.TimeoutExpired([], 10)):
            self.assertIsNotNone(worker.render('x', 'commonmark')[1])
        with patch.object(ORACLE.subprocess, 'run', return_value=
            subprocess.CompletedProcess([], 0, b'{"html":"<p>x</p>","error":null}', b'')):
            self.assertEqual(worker.render('x', 'commonmark'), ('<p>x</p>', None))


if __name__ == "__main__":
    unittest.main()
