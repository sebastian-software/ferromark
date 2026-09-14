"""Contract tests for the compatibility-audit extractor and comparator.

These tests intentionally import the existing helpers instead of changing them.
They cover the HTML-page extraction seams and preserve counterexamples that the
conservative comparator must continue to reject.
"""

from __future__ import annotations

import importlib.util
from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]


def load_module(name: str, path: Path):
    spec = importlib.util.spec_from_file_location(name, path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load {path}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


AUDIT = load_module("compatibility_audit_run", ROOT / "benchmarks/compatibility-audit/run.py")
VERIFY = load_module("native_verify", ROOT / "benchmarks/native-comparison/verify.py")


class SpecTextExtractionTests(unittest.TestCase):
    def test_parse_txt_ignores_non_example_fences_and_keeps_tabs(self):
        fence = "`" * 32
        fixture = f"""# First section

{fence} code
ignored markdown
.
ignored html
{fence}

{fence} example
→foo
.
<p>\tfoo</p>
{fence}

## Second section

{fence} example table
bar
.
<p>bar</p>
{fence}
"""

        examples = AUDIT.parse_txt(fixture)

        self.assertEqual([example["example"] for example in examples], [1, 2])
        self.assertEqual([example["section"] for example in examples], ["First section", "Second section"])
        self.assertEqual(examples[0]["markdown"], "\tfoo\n")
        self.assertEqual(examples[0]["html"], "<p>\tfoo</p>\n")

    def test_gfm_html_extraction_unwraps_space_spans_and_tabs(self):
        html = """
<h1>Section A</h1>
<div class="example" id="example-1">
  <pre><code class="language-markdown"><span class="space"> </span><span class="space"> </span>→foo
</code></pre>
  <pre><code class="language-html">&lt;p&gt;foo\tbar&lt;/p&gt;
</code></pre>
</div>
"""

        examples = AUDIT.GFMExamples(html).examples

        self.assertEqual(len(examples), 1)
        self.assertEqual(examples[0]["example"], 1)
        self.assertEqual(examples[0]["section"], "Section A")
        self.assertEqual(examples[0]["markdown"], "  \tfoo\n")
        self.assertEqual(examples[0]["html"], "<p>foo\tbar</p>\n")

    def test_gfm_html_ignores_code_blocks_outside_examples_and_tracks_sections(self):
        html = """
<h1>Outside</h1>
<pre><code class="language-markdown">must not be captured
</code></pre>
<h2>First</h2>
<div class="example" id="example-1">
  <pre><code class="language-markdown">one
</code></pre>
  <pre><code class="language-html">&lt;p&gt;one&lt;/p&gt;
</code></pre>
</div>
<h2>Second</h2>
<div class="example" id="example-2">
  <div class="decorative-wrapper">
    <pre><code class="language-markdown">two
</code></pre>
    <pre><code class="language-html">&lt;p&gt;two&lt;/p&gt;
</code></pre>
  </div>
</div>
"""

        examples = AUDIT.GFMExamples(html).examples

        self.assertEqual([example["example"] for example in examples], [1, 2])
        self.assertEqual([example["section"] for example in examples], ["First", "Second"])
        self.assertEqual([example["markdown"] for example in examples], ["one\n", "two\n"])
        self.assertEqual([example["html"] for example in examples], ["<p>one</p>\n", "<p>two</p>\n"])

    def test_gfm_html_example_numbers_are_contiguous_and_headings_inside_examples_do_not_change_section(self):
        html = """
<h1>Before</h1>
<div class="example" id="example-1">
  <h2>Not a section</h2>
  <pre><code class="language-markdown">one
</code></pre>
  <pre><code class="language-html">&lt;p&gt;one&lt;/p&gt;
</code></pre>
</div>
<h1>After</h1>
<div class="example" id="example-2">
  <pre><code class="language-markdown">two
</code></pre>
  <pre><code class="language-html">&lt;p&gt;two&lt;/p&gt;
</code></pre>
</div>
"""

        examples = AUDIT.GFMExamples(html).examples

        self.assertEqual([example["example"] for example in examples], [1, 2])
        self.assertEqual([example["section"] for example in examples], ["Before", "After"])


class ConservativeComparatorTests(unittest.TestCase):
    def test_inline_code_spaces_remain_semantically_significant(self):
        expected = "<p><code>  </code></p>"
        actual = "<p><code></code></p>"

        self.assertEqual(VERIFY.classify(expected, actual), "other")
        self.assertFalse(VERIFY.admitted(VERIFY.classify(expected, actual)))

    def test_reserved_url_escape_remains_significant(self):
        escaped = '<p><a href="/a%23b">x</a></p>'
        fragment = '<p><a href="/a#b">x</a></p>'

        self.assertEqual(VERIFY.classify(escaped, fragment), "other")
        self.assertNotEqual(VERIFY.canonical(escaped), VERIFY.canonical(fragment))

    def test_raw_script_and_style_whitespace_remains_significant(self):
        for tag in ("script", "style"):
            expected = f"<{tag}>  x  </{tag}>"
            actual = f"<{tag}>x</{tag}>"
            with self.subTest(tag=tag):
                self.assertEqual(VERIFY.classify(expected, actual), "other")

    def test_heading_ids_are_diagnostic_but_not_strictly_admitted(self):
        left = '<h1 id="one">Title</h1>'
        right = '<h1 id="two">Title</h1>'

        self.assertEqual(VERIFY.classify(left, right), "heading-id-only")
        self.assertFalse(VERIFY.admitted("heading-id-only"))
        self.assertEqual(VERIFY.classify_group({"left": left, "right": right}), "heading-id-only")

    def test_comparator_does_not_hide_task_markup_or_code_class_changes(self):
        task_plain = '<ul><li><input checked disabled> item</li></ul>'
        task_classed = '<ul><li class="task-list-item"><input checked class="task-list-item-checkbox" disabled>item</li></ul>'
        self.assertEqual(VERIFY.classify(task_plain, task_classed), "other")

        code_plain = '<pre><code class="language-js">x</code></pre>'
        code_meta = '<pre><code class="language-js{2}">x</code></pre>'
        self.assertEqual(VERIFY.classify(code_plain, code_meta), "other")


class AuditGateTests(unittest.TestCase):
    def test_gate_rejects_masked_errors_but_keeps_policy_observations(self):
        good = dict(error=None, status="exact", spec_equal=True)
        results = {
            "commonmark-configured": [good.copy()],
            "gfm-extensions-configured": [good.copy()],
            "gfm-public-preset": [dict(good, status="other", spec_equal=False)],
        }
        endings = dict(failures=[])
        self.assertFalse(AUDIT.has_gating_failures(results, endings, []))
        for lane, change in [
            ("commonmark-configured", dict(spec_equal=False)),
            ("gfm-extensions-configured", dict(status="other")),
            ("gfm-public-preset", dict(error="parse failed")),
        ]:
            with self.subTest(lane=lane, change=change):
                changed = dict(results, **{lane: [dict(good, **change)]})
                self.assertTrue(AUDIT.has_gating_failures(changed, endings, []))
        self.assertTrue(AUDIT.has_gating_failures(results, dict(failures=[{}]), []))
        for probe in [
            dict(good, kind="normative", status="other"),
            dict(good, kind="normalizer-counterexample"),
            dict(good, kind="policy", error="panic"),
        ]:
            with self.subTest(probe=probe):
                self.assertTrue(AUDIT.has_gating_failures(results, endings, [probe]))
        rejected = dict(good, kind="normalizer-counterexample", status="other", spec_equal=False)
        self.assertFalse(AUDIT.has_gating_failures(results, endings, [rejected]))


if __name__ == "__main__":
    unittest.main()
