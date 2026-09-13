import copy
import unittest

from common import ENGINES, admission, output_units, without_generated_heading_ids


class FieldTests(unittest.TestCase):
    def setUp(self):
        self.corpus = {"documentation": [{"id": "page", "input": "# Hello"}]}
        self.outputs = {}
        for engine in ENGINES:
            self.outputs[engine] = {life: {"engine": "ferromark" if engine == "ferromark-bun" else engine,
                "group": "documentation", "retain": life == "retain", "options": "configured",
                "outputs": [{"id": "page", "html": "<h1>Hello</h1>\n", "metadata": None}]}
                for life in ("stream", "retain")}

    def test_whole_field_is_required_even_for_ineligible_engines(self):
        self.assertTrue(all(r["admitted"] for r in admission(self.outputs, self.corpus).values()))
        del self.outputs["cmark"]
        with self.assertRaisesRegex(ValueError, "Missing engine"):
            admission(self.outputs, self.corpus)

    def test_lifetime_cannot_change_content(self):
        self.outputs["goldmark"]["retain"]["outputs"][0]["html"] = ""
        with self.assertRaisesRegex(ValueError, "lifetime changed"):
            admission(self.outputs, self.corpus)

    def test_additional_heading_navigation_does_not_erase_other_differences(self):
        for life in ("stream", "retain"):
            self.outputs["ox-content"][life]["outputs"][0]["html"] = '<h1 id="hello">Hello</h1>\n'
        self.assertTrue(admission(self.outputs, self.corpus)["ox-content"]["admitted"])
        for changed in ('<h2 id="hello">Hello</h2>', '<h1 id="hello" class="unknown">Hello</h1>', '<h1 id="hello">Lost</h1>'):
            value = copy.deepcopy(self.outputs)
            for life in ("stream", "retain"):
                value["ox-content"][life]["outputs"][0]["html"] = changed
            self.assertFalse(admission(value, self.corpus)["ox-content"]["admitted"])

    def test_duplicate_or_empty_heading_ids_are_not_accepted(self):
        for html in ('<h1 id="">Hello</h1>', '<h1 id="same">A</h1><h2 id="same">B</h2>'):
            with self.assertRaisesRegex(ValueError, "heading ID"):
                without_generated_heading_ids(html)

    def test_native_utf16_units_are_not_reported_as_utf8_bytes(self):
        rows = [{"html": "é✅"}]
        self.assertEqual(output_units(rows, "utf8"), 5)
        self.assertEqual(output_units(rows, "utf16"), 2)


if __name__ == "__main__":
    unittest.main()
