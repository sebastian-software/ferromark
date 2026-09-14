"""Exercise the semantic gate used before diagnostic timing."""
import importlib.util
from pathlib import Path
import unittest

spec = importlib.util.spec_from_file_location('ox_diagnostic_runner', Path(__file__).with_name('run.py'))
runner = importlib.util.module_from_spec(spec)
spec.loader.exec_module(runner)


class SemanticGateTests(unittest.TestCase):
    def setUp(self):
        # HTML alone cannot detect corrupted source spans.
        self.data = {'case': {label: {
            'fresh': {'html': '<p>x</p>\n'},
            'parse': {'html': '<p>x</p>\n', 'ast_debug': 'Text(span=0..1)'},
            'render': {'html': '<p>x</p>\n', 'ast_debug': 'Text(span=0..1)'},
        } for label in ('control', 'candidate')}}

    def test_equal_outputs_and_spans_pass(self):
        runner.verify_stages(self.data, ['control', 'candidate'])

    def test_span_drift_is_rejected_even_when_html_agrees(self):
        for stage in ('parse', 'render'):
            self.data['case']['candidate'][stage]['ast_debug'] = 'Text(span=1..2)'
        with self.assertRaisesRegex(AssertionError, 'candidate AST mismatch'):
            runner.verify_stages(self.data, ['control', 'candidate'])

    def test_html_drift_is_rejected_even_when_ast_agrees(self):
        for stage in self.data['case']['candidate'].values():
            stage['html'] = '<p>y</p>\n'
        with self.assertRaisesRegex(AssertionError, 'candidate HTML mismatch'):
            runner.verify_stages(self.data, ['control', 'candidate'])

    def test_missing_ast_cannot_silently_pass(self):
        self.data['case']['candidate'] = {'fresh': {'html': '<p>x</p>\n'}}
        with self.assertRaisesRegex(AssertionError, 'AST stage required'):
            runner.verify_stages(self.data, ['control', 'candidate'])

    def test_stage_disagreement_is_rejected(self):
        self.data['case']['candidate']['render']['ast_debug'] = 'Text(span=9..10)'
        with self.assertRaisesRegex(AssertionError, 'stage AST mismatch'):
            runner.verify_stages(self.data, [])

    def test_historical_ast_shape_changes_require_no_cross_version_gate(self):
        for stage in ('parse', 'render'):
            self.data['case']['candidate'][stage]['ast_debug'] += ', front_matter=None'
        runner.verify_stages(self.data, [])


if __name__ == '__main__':
    unittest.main()
