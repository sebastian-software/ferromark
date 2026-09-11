import copy
import unittest

from run import eligible_cases, summarize
from support import cases


class RunnerTests(unittest.TestCase):
    def test_core_catalog_never_requests_gfm_features(self):
        self.assertTrue(all(row["flags"] == 0 for row in cases("cmark")))
        rows = cases("cmark-gfm")
        self.assertEqual(len(rows), len({row["case"] for row in rows}))
        for name in ("tables-plain", "tables-links", "tables-commonmark-inline"):
            matches = [row for row in rows if row["case"].endswith("/" + name)]
            self.assertEqual({row["flags"] for row in matches}, {1, 7})
            self.assertEqual(matches[0]["input"], matches[1]["input"])

    def test_missing_feature_output_never_enters_timing(self):
        catalog = [{"case": "task_lists/tasks", "flags": 4, "input": "- [x] A\n"}]
        rows = [{"case": "task_lists/tasks", "flags": 4, "bytes": 8, "outputs": {
            "ferromark": '<ul><li><input type="checkbox" checked disabled> A</li></ul>',
            "cmark-gfm": '<ul><li>[x] A</li></ul>'}}]
        reviews, admitted = eligible_cases(catalog, rows, "cmark-gfm")
        self.assertFalse(reviews[0]["comparable"])
        self.assertEqual(admitted, [])
        rows[0]["outputs"]["cmark-gfm"] = '<ul><li class="task-list-item"><input type="checkbox" disabled checked>A</li></ul>'
        reviews, admitted = eligible_cases(catalog, rows, "cmark-gfm")
        self.assertEqual(admitted, catalog)
        self.assertEqual(reviews[0]["accepted_differences"], ["task-presentation"])

    def test_missing_duplicate_or_wrong_config_verification_is_rejected(self):
        catalog = [{"case": "commonmark/a", "flags": 0, "input": "A\n"}]
        row = {"case": "commonmark/a", "flags": 0, "bytes": 2,
               "outputs": {"ferromark": "<p>A</p>", "cmark": "<p>A</p>"}}
        for rows in ([], [row, row], [{**row, "flags": 1}], [{**row, "bytes": 1}],
                     [{**row, "outputs": {"ferromark": "<p>A</p>"}}]):
            with self.subTest(rows=rows), self.assertRaises(ValueError):
                eligible_cases(catalog, rows, "cmark")

    def test_summaries_require_complete_positive_samples_for_both_parsers(self):
        catalog = [{"case": "commonmark/a"}]
        rows = [{"case": "commonmark/a", "parser": name, "bytes": 10,
                 "output_bytes": 17, "ns_per_render": [100, 200, 300]} for name in ("ferromark", "cmark")]
        summary = summarize([rows, rows, rows], catalog, "cmark", 3, 3)
        self.assertEqual([r["median_ns"] for r in summary], [200, 200])
        for broken in ([rows[:1]], [rows + rows], [rows]):
            with self.subTest(broken=broken), self.assertRaises(ValueError):
                summarize(broken, catalog, "cmark", 3, 3)
        for values in ([], [1, -2, 3], [1, float("nan"), 3], [1, float("inf"), 3]):
            broken = copy.deepcopy(rows)
            broken[0]["ns_per_render"] = values
            with self.subTest(values=values), self.assertRaises(ValueError):
                summarize([broken], catalog, "cmark", 3, 1)


if __name__ == "__main__":
    unittest.main()
