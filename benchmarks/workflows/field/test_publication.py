"""Reject incomplete timings and mislabeled cross-runtime process memory."""
import copy
import os
import sys
import unittest

from common import ROOT, read_json
from publish import DEFAULT, summarize


class PublicationTests(unittest.TestCase):
    def setUp(self):
        self.build = read_json(DEFAULT / "build.json.gz")
        self.outputs = read_json(DEFAULT / "outputs.json.gz")
        self.reviews = read_json(DEFAULT / "admission.json")
        self.run = read_json(DEFAULT / "run.json")
        self.windows = read_json(DEFAULT / "windows.json.gz")
        self.warmups = read_json(DEFAULT / "warmups.json")
        self.rss = read_json(DEFAULT / "rss.json")

    def validate(self):
        return summarize(self.build, self.outputs, self.reviews, self.run, self.windows, self.warmups, self.rss)

    def test_complete_archive_covers_both_lifetimes_for_every_admitted_engine(self):
        rows = self.validate()
        self.assertEqual(len(rows), 26)
        self.assertFalse(any(r["engine"] == "cmark" for r in rows))

    def test_incomplete_engine_cannot_receive_a_collection_time(self):
        self.run["variants"].append(["cmark", "stream"])
        with self.assertRaisesRegex(ValueError, "complete-collection admission"):
            self.validate()

    def test_duplicate_window_cannot_replace_a_missing_measurement(self):
        self.windows[-1] = copy.deepcopy(self.windows[0])
        with self.assertRaisesRegex(ValueError, "Duplicate or missing"):
            self.validate()

    def test_omitted_process_memory_is_not_treated_as_zero(self):
        self.rss.pop()
        with self.assertRaisesRegex(ValueError, "Incomplete field"):
            self.validate()

    def test_a_changed_native_output_length_is_not_an_equivalent_workload(self):
        row = next(r for r in self.windows if r["engine"] == "markdig")
        row["output_units"] += 1
        with self.assertRaisesRegex(ValueError, "output work changed"):
            self.validate()

    def test_zero_rss_and_short_warmup_are_rejected(self):
        self.rss[0]["peak_rss_bytes"] = 0
        with self.assertRaisesRegex(ValueError, "process-memory"):
            self.validate()
        self.rss = read_json(DEFAULT / "rss.json")
        self.warmups[0]["elapsed_ns"] = 1
        with self.assertRaisesRegex(ValueError, "Short field"):
            self.validate()

    @unittest.skipUnless(sys.platform == "darwin", "Published RSS protocol uses macOS bytes")
    def test_macos_wait4_reports_bytes_for_a_touched_allocation(self):
        # Independent unit check for the OS metric, not another benchmark sample.
        size = 64 * 2**20
        pid = os.fork()
        if pid == 0:
            value = bytearray(size)
            for offset in range(0, size, 4096):
                value[offset] = 1
            os._exit(0)
        _, status, usage = os.wait4(pid, 0)
        self.assertEqual(os.waitstatus_to_exitcode(status), 0)
        self.assertGreaterEqual(usage.ru_maxrss, size)


if __name__ == "__main__":
    unittest.main()
