#!/usr/bin/env python3
"""Tests for admission boundaries and fixed cohort membership."""

import importlib.util
from pathlib import Path
import unittest

SPEC = importlib.util.spec_from_file_location("broad_runner", Path(__file__).with_name("run.py"))
RUN = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(RUN)
PUBLISH_SPEC = importlib.util.spec_from_file_location("broad_publish", Path(__file__).with_name("publish.py"))
PUBLISH = importlib.util.module_from_spec(PUBLISH_SPEC)
PUBLISH_SPEC.loader.exec_module(PUBLISH)


class AdmissionTests(unittest.TestCase):
    def test_unicode_url_spelling_is_equivalent_but_reserved_escapes_are_not(self):
        self.assertEqual(RUN.classify('<a href="https://example.org/荼?q=ä#é">A</a>', '<a href="https://example.org/%E8%8D%BC?q=%C3%A4#%C3%A9">A</a>'), "serialization-equivalent")
        for left, right in (("/a%2Fb", "/a/b"), ("/a?x=%26", "/a?x=&"), ("/one", "/two"), ("/a#é", "/a#e"), ("https://é.org/", "https://%C3%A9.org/")):
            self.assertEqual(RUN.classify(f'<a href="{left}">A</a>', f'<a href="{right}">A</a>'), "different")

    def test_nonvoid_slash_is_not_a_closing_tag(self):
        self.assertEqual(RUN.classify('<div/>x', '<div></div>x'), "different")
        self.assertEqual(RUN.classify('<br/>', '<br>'), "serialization-equivalent")

    def test_timing_checksum_must_match_output_length(self):
        class FakeWorker:
            def bench(self, budget):
                return {"iterations": 32, "elapsed_ns": budget, "checksum": 32 * 10}
        RUN.checked_bench(FakeWorker(), 75_000_000, 10)
        with self.assertRaises(AssertionError):
            RUN.checked_bench(FakeWorker(), 75_000_000, 20)

    def test_heading_ids_are_diagnostic_and_link_targets_stay_significant(self):
        left = '<h2 id="nodejs">Node.js</h2><p><a href="#nodejs">See</a></p>'
        right = '<h2 id="node-js">Node.js</h2><p><a href="#nodejs">See</a></p>'
        self.assertEqual(RUN.classify(left, right), "heading-id-only")
        self.assertEqual(RUN.classify(left, right.replace('href="#nodejs"', 'href="#node-js"')), "different")
        summary = RUN.BASE.summarize(
            [{"group": "x", "mode": "fresh", "round": 0,
              "main": {"elapsed_ns": 100, "iterations": 1},
              "v2": {"elapsed_ns": 50, "iterations": 1}}],
            [{"name": "x", "profile": "commonmark", "cases": ["x"]}],
            {"x": {"status": "heading-id-only"}})
        self.assertFalse(summary[0]["comparable_output"])

    def test_text_and_non_heading_ids_cannot_be_hidden(self):
        for left, right in (
            ('<h1 id="x">A</h1>', '<h1 id="y">B</h1>'),
            ('<p id="x">A</p>', '<p id="y">A</p>'),
            ('<pre> x </pre>', '<pre>x</pre>'),
            ('<h1 class="x">A</h1>', '<h1 class="y">A</h1>'),
        ):
            self.assertEqual(RUN.classify(left, right), "different")

    def test_cohorts_use_every_member_and_do_not_mix_profiles(self):
        cases = [{"name": name, "profile": profile, "collection": "docs"}
                 for name, profile in (("a", "gfm"), ("b", "gfm"), ("c", "commonmark"))]
        groups = RUN.groups_for(cases)
        rotating = {g["name"]: g["cases"] for g in groups if len(g["cases"]) > 1 or g["name"].startswith("rotating-")}
        self.assertEqual(rotating, {"rotating-docs-commonmark": ["c"], "rotating-docs-gfm": ["a", "b"]})


class ReportTests(unittest.TestCase):
    def test_geomean_uses_only_admitted_cases_and_reports_coverage(self):
        cases = [{"name": name, "category": "docs"} for name in ("fast", "slow", "wrong")]
        data = {(name, mode): {"main_over_v2_median": ratio, "main_over_v2_min": ratio, "main_over_v2_max": ratio}
                for name, ratio in (("fast", 9), ("slow", 1 / 9), ("wrong", 100))
                for mode in RUN.MODES}
        check = {"fast": {"status": "exact"}, "slow": {"status": "serialization-equivalent"}, "wrong": {"status": "heading-id-only"}}
        row = PUBLISH.aggregate(cases, data, check, "category")[0]
        self.assertEqual((row["admitted"], row["total"]), (2, 3))
        self.assertAlmostEqual(row["modes"]["reuse"]["geomean_main_over_v2"], 1)
        self.assertEqual(row["modes"]["reuse"]["leaders"], {"v2": 1, "main": 1})

    def test_variable_ranges_are_not_called_clear_wins(self):
        self.assertEqual(PUBLISH.leader({"main_over_v2_min": .99, "main_over_v2_max": 1.4}), "close/variable")


if __name__ == "__main__":
    unittest.main()
