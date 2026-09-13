"""A faster incomplete job must never become an admitted comparison."""
import copy
import unittest

from support import ROOT, PROTOCOL, VARIANTS, read_json, review


class AdmissionTests(unittest.TestCase):
    def setUp(self):
        folder = ROOT / "docs/reports/2026-09-13-practical-workflows"
        self.corpus = read_json(folder / "corpus.json")
        self.outputs = read_json(folder / "outputs.json.gz")
        for variant in VARIANTS:
            if variant not in self.outputs:
                reference = "preview-fresh" if variant.startswith("preview-") else "guide-metadata"
                self.outputs[variant] = copy.deepcopy(self.outputs[reference])
                self.outputs[variant]["variant"] = variant

    def test_every_preview_guide_and_document_must_be_admitted(self):
        result = review(self.outputs, self.corpus)
        self.assertEqual(len(result), 27)
        self.assertTrue(all(row["comparable"] for row in result))

    def test_legacy_evidence_remains_reproducible(self):
        outputs = read_json(ROOT / "docs/reports/2026-09-13-practical-workflows/outputs.json.gz")
        self.assertEqual(len(review(outputs, self.corpus, {**PROTOCOL, "schema": 1})), 12)

    def test_missing_engine_cannot_be_published(self):
        del self.outputs["guide-comrak"]
        with self.assertRaisesRegex(ValueError, "Missing workflow"):
            review(self.outputs, self.corpus)

    def test_a_difficult_input_cannot_be_silently_removed(self):
        self.outputs["preview-pulldown"]["outputs"].pop()
        with self.assertRaisesRegex(ValueError, "complete corpus"):
            review(self.outputs, self.corpus)

    def test_html_only_guide_is_not_equivalent_work(self):
        self.outputs["guide-comrak"]["outputs"][0]["metadata"] = None
        with self.assertRaisesRegex(ValueError, "Complete metadata differs"):
            review(self.outputs, self.corpus)

    def test_changed_navigation_ids_are_not_hidden(self):
        self.outputs["guide-pulldown"]["outputs"][0]["metadata"]["headings"][0]["id"] = "wrong"
        with self.assertRaisesRegex(ValueError, "Complete metadata differs"):
            review(self.outputs, self.corpus)

    def test_missing_rendered_heading_ids_are_not_hidden(self):
        row = self.outputs["guide-pulldown"]["outputs"][0]
        row["html"] = row["html"].replace(' id="quick-start"', '')
        self.assertFalse(all(r["comparable"] for r in review(self.outputs, self.corpus)))

    def test_unsafe_or_omitted_preview_content_is_rejected(self):
        original = copy.deepcopy(self.outputs)
        for replacement in ('<div onclick="bad()">click me</div>', '<!-- omitted -->'):
            self.outputs = copy.deepcopy(original)
            row = next(r for r in self.outputs["preview-comrak"]["outputs"] if r["id"] == "raw-html")
            row["html"] = replacement
            self.assertFalse(all(r["comparable"] for r in review(self.outputs, self.corpus)))

    def test_literal_callout_is_not_a_completed_callout(self):
        row = next(r for r in self.outputs["preview-pulldown"]["outputs"] if r["id"] == "draft")
        row["html"] = row["html"].replace('class="markdown-alert markdown-alert-note"', '')
        self.assertFalse(all(r["comparable"] for r in review(self.outputs, self.corpus)))


if __name__ == "__main__":
    unittest.main()
