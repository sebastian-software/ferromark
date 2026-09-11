"""Public pairs must retain their own baseline and complete archived measurements."""
import copy
import unittest
from publish import data_for, validate_protocol


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
        self.assertTrue(any(r["candidateNs"] < r["ferromarkNs"] for r in overlap))

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
