"""Guard the study against inactive probes and misleading pair aggregation."""

import importlib.util
from pathlib import Path
import unittest

HERE = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location('runtime_runner', HERE / 'run.py')
runner = importlib.util.module_from_spec(spec)
spec.loader.exec_module(runner)


class MeasurementContracts(unittest.TestCase):
    def test_active_toggle_must_change_output(self):
        value = {'html': '<p>text</p>', 'ast_debug': 'Document'}
        with self.assertRaises(AssertionError):
            runner.check_effect({'name': 'inactive', 'expected_effect': 'html'},
                                {'off': value, 'on': value})

    def test_same_html_is_insufficient_for_same_output_ablation(self):
        with self.assertRaises(AssertionError):
            runner.check_effect({'name': 'metadata-lost', 'expected_effect': 'none'},
                                {'off': {'html': '', 'ast_debug': 'metadata'},
                                 'on': {'html': '', 'ast_debug': 'empty'}})

    def test_ratios_are_paired_not_ratio_of_medians(self):
        case = {'name': 'x'}
        rows = [{'case': 'x', 'mode': 'reuse', 'round': 0,
                 'off': {'elapsed_ns': off, 'iterations': 1},
                 'on': {'elapsed_ns': on, 'iterations': 1}}
                for off, on in [(1, 100), (10, 20), (100, 50)]]
        verification = {'x': {'html_equal': True, 'ast_equal': True,
                              'values': {'off': {'html': 'x'}, 'on': {'html': 'x'}}}}
        result = runner.summarize([case], rows, verification)[0]
        self.assertEqual(result['on_over_off'], 2)
        self.assertEqual(result['ratio_min'], 0.5)
        self.assertEqual(result['ratio_max'], 100)

    def test_post_timing_checks_include_ast_and_node_count(self):
        expected = {'html': '<p>x</p>', 'ast_debug': 'x', 'children': 1}
        for key, value in [('html', 'other'), ('ast_debug', 'other'), ('children', 2)]:
            self.assertFalse(runner.matches(expected | {key: value}, expected))


if __name__ == '__main__':
    unittest.main()
