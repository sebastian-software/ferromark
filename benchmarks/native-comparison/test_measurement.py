#!/usr/bin/env python3
"""Guard aggregation, output framing, and timed output consumption contracts."""
import io
from types import SimpleNamespace
import unittest

import report
import run


class MeasurementTests(unittest.TestCase):
    def test_empty_output_is_a_valid_protocol_record(self):
        worker = object.__new__(run.Worker)
        worker.process = SimpleNamespace(stdin=io.StringIO(), stdout=io.StringIO('html 0 \nhtml 1 78\ndone\n'))
        self.assertEqual(worker.verify(), ['', 'x'])

    def test_checksum_rejects_missing_output(self):
        worker = object.__new__(run.Worker)
        worker.process = SimpleNamespace(stdin=io.StringIO(), stdout=io.StringIO('timing 32 40000000 63\n'))
        with self.assertRaises(AssertionError):
            worker.bench(40_000_000, 2)

    def test_process_rounds_have_equal_weight(self):
        rows = []
        for round_index, values in enumerate(([10, 11, 12], [100], [1000])):
            for value in values:
                row = dict(case='x', mode='fresh', round=round_index)
                row.update({e: dict(iterations=1, elapsed_ns=value) for e in run.ENGINES})
                rows.append(row)
        result = run.summaries(rows, [dict(name='x', members=['x'])])
        self.assertEqual(result[0]['engines']['v2']['ns'], 100)

    def test_ratios_weight_documents_not_bytes_or_elapsed_time(self):
        rows = []
        for name, v2, competitor in [('tiny', 10, 5), ('large', 100_000, 200_000)]:
            row = dict(case=name, mode='fresh', engines={e: {'ns': competitor} for e in report.ENGINES})
            row['engines']['v2']['ns'] = v2
            rows.append(row)
        self.assertAlmostEqual(report.aggregate(rows, {'tiny', 'large'}, 'fresh')['md4c'], 1.0)
        with self.assertRaises(AssertionError):
            report.aggregate(rows, {'tiny', 'missing'}, 'fresh')


if __name__ == '__main__':
    unittest.main()
