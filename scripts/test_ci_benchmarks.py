import importlib.util
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

spec = importlib.util.spec_from_file_location(
    "compare_ci", Path(__file__).with_name("compare-ci-benchmarks.py")
)
compare_ci = importlib.util.module_from_spec(spec)
spec.loader.exec_module(compare_ci)


class ComparisonTests(unittest.TestCase):
    def test_parser_accepts_criterion_bencher_output(self):
        self.assertEqual(
            compare_ci.parse(
                "noise\ntest parsing/tiny ... bench:       1,234 ns/iter (+/- 12)\n"
            ),
            {"parsing/tiny": 1234},
        )

    def test_parser_rejects_missing_duplicate_and_malformed_results(self):
        for output in [
            "",
            "test a ... bench: nope",
            "test a ... bench: 0 ns/iter (+/- 0)",
            "test a ... bench: 1 ns/iter (+/- 0)\ntest a ... bench: 2 ns/iter (+/- 0)",
        ]:
            with self.subTest(output=output), self.assertRaises(ValueError):
                compare_ci.parse(output)

    def test_twenty_percent_boundary_is_preserved(self):
        self.assertEqual(compare_ci.compare({"a": 100}, {"a": 120})[0], [])
        self.assertEqual(compare_ci.compare({"a": 100}, {"a": 121})[0], ["a"])

    def test_uniform_slowdown_is_not_normalized_away(self):
        self.assertEqual(
            compare_ci.compare({"a": 100, "b": 1000}, {"a": 130, "b": 1300})[0],
            ["a", "b"],
        )

    def test_missing_baseline_case_fails_but_new_case_is_reported(self):
        with self.assertRaises(ValueError):
            compare_ci.compare({"a": 100}, {"b": 90})
        failures, report = compare_ci.compare({"a": 100}, {"a": 90, "b": 50})
        self.assertFalse(failures)
        self.assertIn("| b | new | 50 |", report)

    def test_cli_returns_failure_for_regression_and_success_at_boundary(self):
        with tempfile.TemporaryDirectory() as directory:
            base = Path(directory) / "base.txt"
            head = Path(directory) / "head.txt"
            summary = Path(directory) / "summary.md"
            base.write_text("test a ... bench: 100 ns/iter (+/- 1)\n")
            for value, expected in [(120, 0), (121, 1)]:
                with self.subTest(value=value):
                    head.write_text(f"test a ... bench: {value} ns/iter (+/- 1)\n")
                    result = subprocess.run(
                        [
                            sys.executable, spec.origin, str(base), str(head),
                            "--base-ref", "base", "--head-ref", "head",
                        ],
                        capture_output=True,
                        text=True,
                        env={**os.environ, "GITHUB_STEP_SUMMARY": str(summary)},
                    )
                    self.assertEqual(result.returncode, expected, result.stderr)
                    self.assertIn("| a | 100 |", result.stdout)
                    self.assertIn(result.stdout.strip(), summary.read_text())


if __name__ == "__main__":
    unittest.main()
