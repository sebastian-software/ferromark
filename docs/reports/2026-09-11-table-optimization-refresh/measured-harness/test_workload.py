import unittest

from run import PARSERS, workload_review


class WorkloadTests(unittest.TestCase):
    def review(self, lane, first, other):
        outputs = {name: first for name in PARSERS}
        outputs['bun_md'] = other
        return workload_review(lane + '/fixture', outputs)

    def test_table_alignment_bug_is_disclosed_but_does_not_block_timing(self):
        result = self.review('gfm_overlap', '<table><tr><td>A</td></tr></table>',
                             '<table><tr><td align="right">A</td></tr></table>')
        self.assertEqual(result, {'comparable': True, 'html_equivalent': False,
                                  'accepted_differences': ['table-alignment']})

    def test_checkbox_placement_classes_and_spacing_are_renderer_conventions(self):
        result = self.review('task_lists',
            '<ul><li><p><input type="checkbox" checked disabled> A</p></li></ul>',
            '<ul><li class="task-list-item"><input class="task-list-item-checkbox" type="checkbox" checked disabled><p>A</p></li></ul>')
        self.assertTrue(result['comparable'])
        self.assertEqual(result['accepted_differences'], ['task-presentation'])

    def test_missing_tasks_wrong_states_and_changed_content_remain_ineligible(self):
        first = '<ul><li><input type="checkbox" checked disabled> A</li></ul>'
        for other in ('<ul><li>[x] A</li></ul>',
                      '<ul><li><input type="checkbox" disabled> A</li></ul>',
                      '<ul><li><input type="checkbox" checked disabled> B</li></ul>'):
            with self.subTest(other=other):
                self.assertFalse(self.review('task_lists', first, other)['comparable'])

    def test_missing_links_and_tables_are_not_equivalent_work(self):
        self.assertFalse(self.review('commonmark', '<p><a href="/a">A</a></p>', '<p>[A]</p>')['comparable'])
        self.assertFalse(self.review('tables', '<table><tr><td>A</td></tr></table>', '<p>| A |</p>')['comparable'])

    def test_unreviewed_differences_are_not_silently_accepted(self):
        self.assertFalse(self.review('tables', '<td style="color:red">A</td>', '<td style="color:blue">A</td>')['comparable'])
        self.assertFalse(self.review('commonmark', '<p><a href="/a">A</a></p>', '<p><a href="/b">A</a></p>')['comparable'])
