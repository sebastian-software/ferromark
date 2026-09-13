import copy
import unittest

from publish import summarize
from support import PROTOCOL, VARIANTS, command


class PublicationTests(unittest.TestCase):
    def test_host_observations_use_portable_numeric_locale(self):
        import sys
        self.assertEqual(command([sys.executable, "-c", "import os; print(os.environ['LC_ALL'])"]), "C")

    def setUp(self):
        self.corpus = {g: [{"id": g, "input": "# Hello"}] for g in ("previews", "guides", "documentation")}
        self.outputs = {v: {"outputs": [{"html": "<h1>Hello</h1>\n"}]} for v in VARIANTS}
        stats = {"peak_live_bytes": 128, "live_after_render": 32, "requested_bytes": 256, "allocation_calls": 4}
        self.windows, self.memory, self.warmups = [], [], []
        for round_id in range(3):
            for window in range(80):
                offset = (window + round_id) % len(VARIANTS)
                order = VARIANTS[offset:] + VARIANTS[:offset]
                if window % 2:
                    order.reverse()
                for position, variant in enumerate(order):
                    self.windows.append({"round": round_id, "window": window, "position": position,
                                         "variant": variant, **self.sample(63_000_000)})
            for variant in VARIANTS:
                self.warmups.append({"round": round_id, "variant": variant, **self.sample(3_000_000_000)})
                for observation in range(10):
                    self.memory.append({"round": round_id, "observation": observation, "variant": variant,
                                        "cold": stats.copy(), "warm": stats.copy(),
                                        "output_bytes": 15, "live_after_drop": 0})

    def sample(self, elapsed):
        return {"elapsed_ns": elapsed, "iterations": 4, "output_bytes": 60, "ns_per_workload": elapsed / 4}

    def validate(self):
        return summarize(self.corpus, self.outputs, self.windows, self.memory, self.warmups, PROTOCOL)

    def test_recomputes_complete_measurements(self):
        rows = self.validate()
        self.assertEqual(len(rows), 9)
        self.assertEqual(rows[0]["median_ns"], 15_750_000)
        self.assertEqual(rows[0]["heap"]["peak_live_bytes"], 128)

    def test_rejects_missing_window(self):
        self.windows.pop()
        with self.assertRaisesRegex(ValueError, "Incomplete timing"):
            self.validate()

    def test_rejects_duplicate_window(self):
        self.windows[-1] = self.windows[0].copy()
        with self.assertRaisesRegex(ValueError, "Duplicate or missing window"):
            self.validate()

    def test_rejects_short_warmup(self):
        self.warmups[0].update(self.sample(10_000_000))
        with self.assertRaisesRegex(ValueError, "Short or empty"):
            self.validate()

    def test_rejects_short_window(self):
        self.windows[0].update(self.sample(10_000_000))
        with self.assertRaisesRegex(ValueError, "Short or empty"):
            self.validate()

    def test_rejects_changed_output_work(self):
        self.windows[0]["output_bytes"] = 0
        with self.assertRaisesRegex(ValueError, "Timed output"):
            self.validate()

    def test_rejects_invented_timing_summary(self):
        self.windows[0]["ns_per_workload"] = 1
        with self.assertRaisesRegex(ValueError, "Timing disagrees"):
            self.validate()

    def test_rejects_unbalanced_memory(self):
        self.memory[0]["live_after_drop"] = 1
        with self.assertRaisesRegex(ValueError, "unbalanced"):
            self.validate()

    def test_rejects_hidden_memory_variation(self):
        self.memory[0]["warm"]["peak_live_bytes"] += 1
        with self.assertRaisesRegex(ValueError, "varied"):
            self.validate()

    def test_rejects_unrotated_order(self):
        self.windows[0]["position"], self.windows[1]["position"] = self.windows[1]["position"], self.windows[0]["position"]
        with self.assertRaisesRegex(ValueError, "rotating protocol"):
            self.validate()

    def test_rejects_missing_memory_observation(self):
        self.memory.pop()
        with self.assertRaisesRegex(ValueError, "Incomplete memory"):
            self.validate()


if __name__ == "__main__":
    unittest.main()
