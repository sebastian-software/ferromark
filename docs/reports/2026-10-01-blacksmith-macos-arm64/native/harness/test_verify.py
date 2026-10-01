#!/usr/bin/env python3
"""Contract tests for native-comparison HTML admission."""

from __future__ import annotations

import importlib.util
from pathlib import Path
import unittest


SPEC = importlib.util.spec_from_file_location("native_verify", Path(__file__).with_name("verify.py"))
assert SPEC and SPEC.loader
VERIFY = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(VERIFY)


class VerificationTests(unittest.TestCase):
    def test_normal_flow_entities_and_void_boolean_serialization(self) -> None:
        self.assertEqual(VERIFY.classify("<p>A &amp; B<br /></p>\n", "<p>A &#38; B<br></p>"), "serialization-equivalent")
        self.assertEqual(VERIFY.classify('<input disabled="disabled" checked />', "<input checked disabled>"), "serialization-equivalent")
        self.assertEqual(VERIFY.classify("<p>first word</p>", "<p>first other</p>"), "other")

    def test_urls_preserve_fragments_and_reserved_encoding(self) -> None:
        self.assertEqual(
            VERIFY.classify(
                '<a href="https://example.org/荼?q=ä#é">word</a>',
                '<a href="https://example.org/%E8%8D%BC?q=%C3%A4#%C3%A9">word</a>',
            ),
            "serialization-equivalent",
        )
        for left, right in (("/a%2Fb", "/a/b"), ("/a?x=%26", "/a?x=&"), ("/a#one", "/a#two")):
            self.assertEqual(VERIFY.classify(f'<a href="{left}">x</a>', f'<a href="{right}">x</a>'), "other")

    def test_code_whitespace_and_missing_features_remain_significant(self) -> None:
        self.assertEqual(VERIFY.classify("<p>a \nb</p>", "<p>a b</p>"), "serialization-equivalent")
        self.assertEqual(VERIFY.classify("<pre><code>a  b</code></pre>", "<pre><code>a b</code></pre>"), "other")
        self.assertEqual(VERIFY.classify("<p>~~old~~</p>", "<p><del>old</del></p>"), "other")

    def test_task_state_and_table_alignment_are_not_dropped(self) -> None:
        self.assertEqual(VERIFY.classify('<input type="checkbox" checked>', '<input checked="checked" type="checkbox" />'), "serialization-equivalent")
        self.assertEqual(VERIFY.classify('<input type="checkbox" checked>', '<input type="checkbox">'), "other")
        self.assertEqual(VERIFY.classify('<td align="right">1</td>', '<td style="text-align: right;">1</td>'), "serialization-equivalent")
        self.assertEqual(VERIFY.classify('<td align="right">1</td>', '<td align="center">1</td>'), "other")
        self.assertEqual(VERIFY.classify('<td>1</td>', '<td align="left">1</td>'), "other")
        self.assertEqual(VERIFY.classify('<td align="right" style="color:red">1</td>', '<td style="color:red;text-align:right">1</td>'), "other")

    def test_nonvoid_self_closing_is_not_a_close_tag(self) -> None:
        self.assertEqual(VERIFY.classify("<div/>x", "<div></div>x"), "other")

    def test_heading_only_is_diagnostic_and_not_admitted(self) -> None:
        status = VERIFY.classify('<h2 id="nodejs">Node.js</h2>', '<h2 id="node-js">Node.js</h2>')
        self.assertEqual(status, "heading-id-only")
        self.assertFalse(VERIFY.admitted(status))
        self.assertTrue(VERIFY.admitted("exact"))
        self.assertTrue(VERIFY.admitted("serialization-equivalent"))

    def test_group_classification_has_no_oracle_and_keeps_all_members(self) -> None:
        self.assertEqual(VERIFY.classify_group({"a": "<p>x</p>", "b": "<p>x</p>"}), "exact")
        self.assertEqual(VERIFY.classify_group({"a": '<h1 id="a">x</h1>', "b": '<h1 id="b">x</h1>', "c": '<h1 id="c">x</h1>'}), "heading-id-only")
        self.assertEqual(VERIFY.classify_group({"a": "<p>x</p>", "b": "<p>x</p>", "c": "<p>y</p>"}), "other")
        self.assertEqual(VERIFY.groups({"b": "<p>x</p>", "a": "<p>x</p>", "c": "<p>y</p>"}), [["a", "b"], ["c"]])
        with self.assertRaises(ValueError):
            VERIFY.classify_group({})


if __name__ == "__main__":
    unittest.main()
