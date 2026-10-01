"""Protect equal-document aggregation and Node's string consumption boundary."""
import unittest
import json
import subprocess
import sys
import publish
import publish_values
import run


class MeasurementTests(unittest.TestCase):
    def test_utf16_units_differ_from_native_bytes(self):
        self.assertEqual(run.output_units('🪐é', 'node'), 3)
        self.assertEqual(run.output_units('🪐é', 'native'), 6)

    def test_documents_and_process_rounds_have_equal_weight(self):
        rows = []
        for name, v2, competitor in [('tiny', 10, 5), ('large', 100_000, 200_000)]:
            for round_index, factors in enumerate(([1, 1, 1], [2], [3])):
                for factor in factors:
                    rows.append(dict(case=name, mode='fresh', round=round_index,
                                     v2=dict(elapsed_ns=v2 * factor, iterations=1),
                                     micromark=dict(elapsed_ns=competitor * factor, iterations=1)))
        self.assertAlmostEqual(run.aggregate(rows, {'tiny', 'large'}, 'fresh', 'micromark'), 1)
        self.assertIsNone(run.aggregate(rows, set(), 'fresh', 'micromark'))

    def test_report_and_website_match_archived_measurements(self):
        subprocess.run([sys.executable, str(publish.REPO / "benchmarks/markdown-ecosystem/publish.py"), "--check"], check=True)
        section = publish.content()
        if publish_values.FIGURES.exists():
            section += publish_values.content()
        guide = publish.GUIDE.read_text()
        self.assertEqual(guide.split(publish.MARKER, 1)[1], section.split(publish.MARKER, 1)[1])
        # The complete campaigns now precede the historical sections. The publisher
        # check above validates the raw evidence; compare the selected table content
        # independently without validating the same archives a second time.
        selection = json.loads((publish.REPO / 'benchmarks/manual-comparison/current.json').read_text())['reports']
        if selection:
            figures = json.loads(publish_values.FIGURES.read_text())['figures']
            current = [figure for figure in figures if figure['platform'] in selection]
            self.assertIn(publish_values.manual_workflow().content(current), guide)
            self.assertEqual(guide.count(publish.CURRENT_START), 1)
            self.assertLess(guide.index(publish.CURRENT_START), guide.index('## Native engine comparison'))
        for track in ('native', 'node', *('node-' + engine for engine, _ in publish.NODE_PAIRS)):
            summary = json.loads((publish.REPORT / track / 'summary.json').read_text())
            self.assertEqual(summary['documents'], 57)
            self.assertLessEqual(summary['agreeing_documents'], summary['documents'])


if __name__ == '__main__':
    unittest.main()
