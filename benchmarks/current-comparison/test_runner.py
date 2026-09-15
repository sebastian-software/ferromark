#!/usr/bin/env python3
"""Contract tests for the current-comparison runner's normalization rules."""

from __future__ import annotations

import importlib.util
from pathlib import Path
import unittest


RUNNER_PATH = Path(__file__).with_name("run.py")
SPEC = importlib.util.spec_from_file_location("current_comparison_run", RUNNER_PATH)
if SPEC is None or SPEC.loader is None:
    raise RuntimeError(f"cannot load runner module: {RUNNER_PATH}")
RUNNER = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(RUNNER)


class CanonicalHtmlTests(unittest.TestCase):
    def assertEquivalent(self, left: str, right: str) -> None:
        self.assertEqual(RUNNER.canonical(left), RUNNER.canonical(right))

    def assertDifferent(self, left: str, right: str) -> None:
        self.assertNotEqual(RUNNER.canonical(left), RUNNER.canonical(right))

    def test_checkbox_boolean_and_self_closing_attribute_serialization(self) -> None:
        self.assertEquivalent(
            '<input type="checkbox" checked>',
            '<input checked="checked" type="checkbox"/>',
        )
        self.assertEquivalent(
            '<input disabled>',
            '<input disabled="disabled" />',
        )

    def test_void_element_slash_is_ignored(self) -> None:
        self.assertEquivalent("<br>", "<br />")
        self.assertEquivalent('<img src="image.png">', '<img src="image.png" />')

    def test_semantic_attributes_remain_significant(self) -> None:
        pairs = (
            ('<a href="/a">link</a>', '<a href="/b">link</a>'),
            ('<a target="_blank">link</a>', '<a target="_self">link</a>'),
            ('<h1 id="one">Title</h1>', '<h1 id="two">Title</h1>'),
            ('<h1 class="one">Title</h1>', '<h1 class="two">Title</h1>'),
        )
        for left, right in pairs:
            with self.subTest(left=left, right=right):
                self.assertDifferent(left, right)

    def test_literal_code_and_pre_whitespace_remains_significant(self) -> None:
        self.assertDifferent("<code>x\n</code>", "<code>x \n</code>")
        self.assertDifferent("<pre>x\n</pre>", "<pre>x \n</pre>")

    def test_inline_separating_space_remains_significant(self) -> None:
        self.assertDifferent("<span>a</span><span>b</span>", "<span>a</span> <span>b</span>")

    def test_entity_serializations_are_equivalent(self) -> None:
        self.assertEquivalent("<p>&amp; &#38; &#x26;</p>", "<p>&#x26; &amp; &#38;</p>")

    def test_whitespace_between_block_elements_is_normalized(self) -> None:
        self.assertEquivalent(
            "<div>one</div>\n  \n<p>two</p>",
            "<div>one</div><p>two</p>",
        )


class SummaryTests(unittest.TestCase):
    def test_paired_ratio_and_document_divisor(self) -> None:
        groups = [{"name": "pair", "profile": "commonmark", "cases": ["one", "two"]}]
        verification = {
            "one": {"status": "exact"},
            "two": {"status": "serialization-equivalent"},
        }
        rows = [
            {
                "group": "pair",
                "mode": "fresh",
                "round": 0,
                "main": {"elapsed_ns": 300, "iterations": 3},
                "v2": {"elapsed_ns": 150, "iterations": 3},
            },
            {
                "group": "pair",
                "mode": "fresh",
                "round": 0,
                "main": {"elapsed_ns": 240, "iterations": 4},
                "v2": {"elapsed_ns": 120, "iterations": 4},
            },
        ]

        summary = RUNNER.summarize(rows, groups, verification)

        self.assertEqual(len(summary), 1)
        result = summary[0]
        self.assertEqual(result["documents_per_iteration"], 2)
        self.assertEqual(result["main_ns_per_document"], 40)
        self.assertEqual(result["v2_ns_per_document"], 20)
        self.assertEqual(result["main_over_v2_median"], 2)
        self.assertEqual(result["main_over_v2_min"], 2)
        self.assertEqual(result["main_over_v2_max"], 2)
        self.assertEqual(result["pairs"], 2)
        self.assertEqual(result["round_medians"], [2])
        self.assertTrue(result["comparable_output"])


if __name__ == "__main__":
    unittest.main()
