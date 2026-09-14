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

    def test_agreement_subsets_do_not_hide_heading_ids_or_other_differences(self):
        equal = {e: 'serialization-equivalent' for e in report.ENGINES}
        data = {
            'all': {'versus_v2': equal},
            'ox-id': {'versus_v2': equal | {'ox-content': 'heading-id-only'}},
            'task-markup': {'versus_v2': equal | {'bun': 'other'}},
            'v1-id': {'versus_v2': equal | {'v1': 'heading-id-only'}},
        }
        self.assertEqual(report.agreement_sets(data), ({'all'}, {'all', 'ox-id'}))

    def test_five_engine_aggregate_never_scores_ox(self):
        rows = [dict(case='x', mode='fresh', engines={e: {'ns': 10} for e in report.ENGINES})]
        values = report.aggregate(rows, {'x'}, 'fresh', report.CONFIGURABLE)
        self.assertEqual(set(values), set(report.CONFIGURABLE))
        self.assertNotIn('ox-content', values)


if __name__ == '__main__':
    unittest.main()
