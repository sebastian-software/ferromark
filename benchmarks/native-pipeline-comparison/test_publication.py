"""Public pairs must retain their own baseline and complete archived measurements."""
import copy
import unittest
from publish import data_for, overview_tables, validate_protocol
import statistics


class PublicationTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.data = data_for()

    def test_all_candidates_keep_independent_baselines_and_ox_exclusions(self):
        engines = {e["id"]: e for e in self.data["engines"]}
        self.assertEqual(set(engines), {"goldmark", "satteri", "rushdown", "markdig", "markdown-rs", "ox-content"})
        self.assertEqual(engines["ox-content"]["eligible"], 10)
        self.assertNotIn("commonmark/5k", [r["case"] for r in engines["ox-content"]["rows"]])
        overlap = [next(r for r in e["rows"] if r["case"] == "gfm_overlap/features") for e in engines.values()]
        self.assertEqual(len({r["ferromarkNs"] for r in overlap}), len(engines))

    def test_overview_uses_one_median_reference_and_keeps_candidate_times(self):
        for table in self.data["tables"]:
            baseline = table["rows"][0]
            self.assertEqual(baseline["id"], "ferromark")
            self.assertEqual(sum(r["id"] == "ferromark" for r in table["rows"]), 1)
            self.assertEqual(baseline["medianNs"], statistics.median(table["referenceMediansNs"]))
            for row in table["rows"][1:]:
                engine = next(e for e in self.data["engines"] if e["id"] == row["id"])
                original = next(r for r in engine["rows"] if r["case"] == table["case"])
                self.assertEqual(row["medianNs"], original["candidateNs"])
                self.assertEqual(row["relativeSpeed"], f"{baseline['medianNs']/original['candidateNs']:.2f}×")
        broken = copy.deepcopy(self.data["engines"])
        broken[0]["rows"][0]["inputSha256"] = "different document of the same size"
        with self.assertRaises(ValueError):
            overview_tables(broken)

    def test_publication_rejects_incomplete_or_shortened_runs(self):
        metadata = {"started_unix": 1, "finished_unix": 2, "protocol": self.data["engines"][0]["protocol"]}
        validate_protocol(metadata)
        for key, value in (("mode", "verify"), ("runs", 1), ("samples", 79), ("window_ms", 1), ("warmup_ms", 100)):
            broken = copy.deepcopy(metadata)
            broken["protocol"][key] = value
            with self.subTest(key=key), self.assertRaisesRegex(ValueError, "full repeated measurements"):
                validate_protocol(broken)
        with self.assertRaises(ValueError):
            validate_protocol({**metadata, "finished_unix": 0})


if __name__ == "__main__":
    unittest.main()
