"""Publication must reject screening data, semantic differences, and damaged evidence."""
import copy
import json
import hashlib
from pathlib import Path
import tempfile
import unittest

from prepare import BUN_REV, MD4C_REV
from publish import CASES, ORDER, load_publication


class PublicationTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.folder = Path(self.tmp.name)
        self.metadata = {
            "protocol": {"mode": "publication", "samples": 80, "headline_runs": 3,
                         "window_ms": 63, "warmup_ms": 3000},
            "bun_revision": BUN_REV, "md4c_revision": MD4C_REV,
        }
        self.verification = {"cases": [
            {"case": case, "bytes": 2048, "mismatches": [],
             "output_sha256": {parser: hashlib.sha256(b"<p>ok</p>").hexdigest() for parser in ORDER}}
            for case, *_ in CASES
        ]}
        self.samples = [
            [{"case": case, "parser": parser, "bytes": 2048,
              "output_bytes": 9, "ns_per_render": [1000 + run * 100] * 80}
             for case, *_ in CASES for parser in ORDER]
            for run in range(3)
        ]
        self.summary = [
            {"case": case, "parser": parser, "bytes": 2048, "output_bytes": 9,
             "median_ns": 1100, "run_medians_ns": [1000, 1100, 1200]}
            for case, *_ in CASES for parser in ORDER
        ]

    def write(self):
        for filename, data in [("metadata.json", self.metadata),
                               ("verification.json", self.verification),
                               ("summary.json", self.summary)]:
            (self.folder / filename).write_text(json.dumps(data))
        (self.folder / "verify.jsonl").write_text("\n".join(json.dumps({
            "case": case, "outputs": {parser: "<p>ok</p>" for parser in ORDER}
        }) for case, *_ in CASES))
        for run, rows in enumerate(self.samples):
            (self.folder / f"samples-{run}.jsonl").write_text(
                "\n".join(json.dumps(row) for row in rows))

    def test_complete_verified_evidence_can_be_published(self):
        self.write()
        _, tables, _ = load_publication(self.folder)
        self.assertEqual(len(tables), 8)
        self.assertEqual(tables[0]["rows"][0]["latency"], "1.100 µs")

    def test_short_or_incomplete_protocols_are_rejected(self):
        baseline = copy.deepcopy(self.metadata)
        for key, value in [("mode", "screening"), ("samples", 20),
                           ("headline_runs", 1), ("window_ms", 5), ("warmup_ms", 100)]:
            with self.subTest(key=key):
                self.metadata = copy.deepcopy(baseline)
                self.metadata["protocol"][key] = value
                self.write()
                with self.assertRaises(ValueError):
                    load_publication(self.folder)

    def test_a_parser_output_difference_prevents_publication(self):
        self.verification["cases"][0]["mismatches"] = ["md4c"]
        self.write()
        with self.assertRaisesRegex(ValueError, "not output-equivalent"):
            load_publication(self.folder)

    def test_missing_parser_verification_prevents_publication(self):
        del self.verification["cases"][0]["output_sha256"]["bun_md"]
        self.write()
        with self.assertRaisesRegex(ValueError, "not output-equivalent"):
            load_publication(self.folder)

    def test_edited_summary_is_rejected(self):
        self.summary[0]["median_ns"] = 500
        self.write()
        with self.assertRaisesRegex(ValueError, "disagrees with raw samples"):
            load_publication(self.folder)

    def test_lost_sample_is_rejected(self):
        self.samples[2][0]["ns_per_render"].pop()
        self.write()
        with self.assertRaisesRegex(ValueError, "incomplete sampling"):
            load_publication(self.folder)


if __name__ == "__main__":
    unittest.main()
