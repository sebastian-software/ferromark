#!/usr/bin/env python3
"""Guard the PGO recipe's argument parsing and the training/measurement split."""
import gzip
import json
from pathlib import Path
import tempfile
from types import SimpleNamespace
import unittest

import prepare
import run

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[1]
BROAD_CORPUS = REPO / 'docs/reports/2026-09-14-optimization-rounds/broad-corpus.json.gz'
SPLIT = REPO / 'docs/reports/2026-09-16-arm-round-3/harness'
TRAIN_FILTER = SPLIT / 'filter-broad-train.txt'
TEST_FILTER = SPLIT / 'filter-broad-test.txt'


def namespace(**overrides):
    defaults = dict(pgo=False, compile=False, pgo_training_corpus=None,
                    pgo_training_filter=None, pgo_measurement_filter=None, pgo_train_ms=250)
    return SimpleNamespace(**(defaults | overrides))


def written(directory, name, text):
    path = Path(directory) / name
    path.write_text(text)
    return path


class FilterFileTests(unittest.TestCase):
    def test_read_filter_strips_the_trailing_newline(self):
        with tempfile.TemporaryDirectory() as directory:
            self.assertEqual(prepare.read_filter(written(directory, 'f.txt', '^(a|b)$\n')), '^(a|b)$')

    def test_read_filter_rejects_empty_and_invalid_patterns(self):
        with tempfile.TemporaryDirectory() as directory:
            with self.assertRaises(SystemExit):
                prepare.read_filter(written(directory, 'empty.txt', '  \n'))
            with self.assertRaises(SystemExit):
                prepare.read_filter(written(directory, 'bad.txt', '^(unclosed$'))

    def test_run_filter_file_and_inline_filter_are_mutually_exclusive(self):
        with tempfile.TemporaryDirectory() as directory:
            path = written(directory, 'f.txt', '^keep$\n')
            self.assertEqual(run.case_filter(None, path), '^keep$')
            self.assertEqual(run.case_filter('^keep$', None), '^keep$')
            self.assertIsNone(run.case_filter(None, None))
            with self.assertRaises(SystemExit):
                run.case_filter('^keep$', path)
            with self.assertRaises(SystemExit):
                run.case_filter(None, written(directory, 'empty.txt', '\n'))
            with self.assertRaises(SystemExit):
                run.case_filter('^(unclosed$', None)


class CaseSelectionTests(unittest.TestCase):
    cases = [{'name': 'alpha'}, {'name': 'beta'}, {'name': 'alpha-long'}]

    def test_run_without_a_filter_measures_every_case(self):
        self.assertEqual(run.select_cases(self.cases, None), self.cases)

    def test_run_filter_keeps_corpus_order_and_rejects_an_empty_selection(self):
        self.assertEqual([c['name'] for c in run.select_cases(self.cases, '^alpha')],
                         ['alpha', 'alpha-long'])
        with self.assertRaises(SystemExit):
            run.select_cases(self.cases, '^gamma$')

    def test_prepare_and_run_select_cases_agree(self):
        self.assertEqual(prepare.select_cases(self.cases, '^alpha$'),
                         run.select_cases(self.cases, '^alpha$'))

    def test_overlapping_cases_names_every_training_document_that_is_measured(self):
        self.assertEqual(prepare.overlapping_cases(self.cases, '^(alpha|beta)$'), ['alpha', 'beta'])
        self.assertEqual(prepare.overlapping_cases(self.cases, '^gamma$'), [])


class TrainingProfileTests(unittest.TestCase):
    def test_harness_profiles_train_under_themselves(self):
        self.assertEqual(prepare.training_profiles({'profile': 'commonmark'}), ('commonmark',))
        self.assertEqual(prepare.training_profiles({'profile': 'gfm'}), ('gfm',))

    def test_profiles_the_worker_does_not_implement_train_under_both(self):
        for profile in ('autolink', 'mdx', 'extensions', 'opt-3'):
            self.assertEqual(prepare.training_profiles({'profile': profile}),
                             prepare.HARNESS_PROFILES)

    def test_every_engine_and_lifecycle_is_named_for_profile_data(self):
        self.assertEqual(set(prepare.ENGINE_PROFILE_DATA), set(prepare.ENGINES))
        self.assertEqual(set(prepare.ENGINES), set(run.ENGINES))
        self.assertEqual(prepare.LIFECYCLES, run.MODES)
        rust = [e for e, v in prepare.ENGINE_PROFILE_DATA.items() if v == 'rust-pgo']
        self.assertEqual(sorted(rust), ['ox-content', 'pulldown-cmark', 'v1', 'v2'])
        self.assertTrue(prepare.ENGINE_PROFILE_DATA['md4c'].startswith('none'))
        self.assertTrue(prepare.ENGINE_PROFILE_DATA['bun'].startswith('rust-pgo-partial'))


class PgoArgumentTests(unittest.TestCase):
    def test_default_build_needs_no_pgo_input(self):
        self.assertIsNone(prepare.pgo_argument_error(namespace(compile=True)))

    def test_pgo_inputs_without_the_flag_are_rejected(self):
        message = prepare.pgo_argument_error(namespace(pgo_training_corpus=Path('c.json')))
        self.assertIn('--pgo is required', message)
        self.assertIn('--pgo-training-corpus', message)

    def test_pgo_requires_compile_because_the_profile_comes_from_running(self):
        message = prepare.pgo_argument_error(namespace(
            pgo=True, pgo_training_corpus=Path('c'), pgo_training_filter=Path('t'),
            pgo_measurement_filter=Path('m')))
        self.assertIn('--compile', message)

    def test_pgo_lists_every_missing_input(self):
        message = prepare.pgo_argument_error(namespace(pgo=True, compile=True))
        for option in ('--pgo-training-corpus', '--pgo-training-filter', '--pgo-measurement-filter'):
            self.assertIn(option, message)

    def test_pgo_rejects_a_non_positive_training_window(self):
        self.assertIn('--pgo-train-ms', prepare.pgo_argument_error(namespace(
            pgo=True, compile=True, pgo_training_corpus=Path('c'), pgo_training_filter=Path('t'),
            pgo_measurement_filter=Path('m'), pgo_train_ms=0)))

    def test_a_complete_pgo_invocation_is_accepted(self):
        self.assertIsNone(prepare.pgo_argument_error(namespace(
            pgo=True, compile=True, pgo_training_corpus=Path('c'), pgo_training_filter=Path('t'),
            pgo_measurement_filter=Path('m'))))


class BroadCorpusSplitTests(unittest.TestCase):
    """The frozen split must keep training and measurement disjoint."""

    def setUp(self):
        with gzip.open(BROAD_CORPUS, 'rt') as stream:
            self.cases = json.load(stream)['cases']
        self.train = prepare.read_filter(TRAIN_FILTER)
        self.measure = prepare.read_filter(TEST_FILTER)

    def test_the_split_partitions_all_57_broad_documents(self):
        trained = {c['name'] for c in prepare.select_cases(self.cases, self.train)}
        measured = {c['name'] for c in run.select_cases(self.cases, self.measure)}
        self.assertEqual(len(self.cases), 57)
        self.assertEqual(len(trained), 29)
        self.assertEqual(len(measured), 28)
        self.assertEqual(trained & measured, set())
        self.assertEqual(trained | measured, {c['name'] for c in self.cases})

    def test_no_trained_document_can_be_measured(self):
        trained = prepare.select_cases(self.cases, self.train)
        self.assertEqual(prepare.overlapping_cases(trained, self.measure), [])

    def test_both_harness_profiles_are_measured(self):
        measured = run.select_cases(self.cases, self.measure)
        self.assertEqual({c['profile'] for c in measured}, set(prepare.HARNESS_PROFILES))


if __name__ == '__main__':
    unittest.main()
