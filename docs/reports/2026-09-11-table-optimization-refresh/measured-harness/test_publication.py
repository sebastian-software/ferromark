"""Publication must reject screening data, semantic differences, and damaged evidence."""
import copy
import json
import hashlib
from pathlib import Path
import tempfile
import unittest

from prepare import BUN_REV, MD4C_REV
from publish import CASES, ORDER, load_publication
from run import mismatches


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
        flags = {"commonmark": 0, "tables": 0x100, "strikethrough": 0x200, "gfm_overlap": 0xB00}
        (self.folder / "options.jsonl").write_text("\n".join(json.dumps({
            "lane":lane, "md4c_parser_flags":flag, "md4c_renderer_flags":0
        }) for lane, flag in flags.items()))
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

    def test_active_table_cases_do_not_reintroduce_archived_strikethrough_workloads(self):
        from run import HEADLINE_CASES
        tables = {case for case in HEADLINE_CASES if case.startswith("tables/")}
        self.assertEqual(tables, {
            "tables/tables-plain", "tables/tables-commonmark-inline", "tables/tables-links",
        })
        self.assertNotIn("gfm_overlap/gfm-tables", HEADLINE_CASES)
        self.assertTrue(any(case == "tables/gfm-tables" for case, *_ in CASES))

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
        with self.assertRaisesRegex(ValueError, "verification disagrees"):
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

    def test_wrong_effective_options_are_rejected_even_when_html_matches(self):
        self.write()
        path = self.folder / "options.jsonl"
        path.write_text(path.read_text().replace('"md4c_parser_flags": 2816', '"md4c_parser_flags": 8960'))
        with self.assertRaisesRegex(ValueError, "options do not match"):
            load_publication(self.folder)

    def test_publication_accepts_reviewed_renderer_differences_but_not_missing_tasks(self):
        case = "gfm_overlap/gfm-tables"
        first = '<ul><li><p><input type="checkbox" checked disabled> A</p></li></ul>'
        rendered = '<ul><li class="task-list-item"><input class="task-list-item-checkbox" type="checkbox" checked disabled><p>A</p></li></ul>'
        for other, comparable in [(rendered, True), ('<ul><li><p>[x] A</p></li></ul>', False)]:
            with self.subTest(comparable=comparable):
                self.write()
                outputs = {parser: first for parser in ORDER}
                outputs['bun_md'] = other
                path = self.folder / 'verify.jsonl'
                records = [json.loads(line) for line in path.read_text().splitlines()]
                next(r for r in records if r['case'] == case)['outputs'] = outputs
                path.write_text('\n'.join(json.dumps(r) for r in records))
                path = self.folder / 'verification.json'
                verification = json.loads(path.read_text())
                record = next(r for r in verification['cases'] if r['case'] == case)
                record['mismatches'] = mismatches(outputs)
                record['output_sha256'] = {p: hashlib.sha256(h.encode()).hexdigest() for p, h in outputs.items()}
                path.write_text(json.dumps(verification))
                for filename in ['summary.json', *[f'samples-{i}.jsonl' for i in range(3)]]:
                    path = self.folder / filename
                    jsonl = filename.endswith('.jsonl')
                    rows = [json.loads(line) for line in path.read_text().splitlines()] if jsonl else json.loads(path.read_text())
                    for row in rows:
                        if row['case'] == case:
                            row['output_bytes'] = len(outputs[row['parser']].encode())
                    path.write_text('\n'.join(json.dumps(r) for r in rows) if jsonl else json.dumps(rows))
                if comparable:
                    _, tables, _ = load_publication(self.folder)
                    review = next(t for t in tables if t['case'] == case)['outputReview']
                    self.assertEqual(review['accepted_differences'], ['task-presentation'])
                else:
                    with self.assertRaisesRegex(ValueError, 'not comparable Markdown work'):
                        load_publication(self.folder)

    def test_lost_sample_is_rejected(self):
        self.samples[2][0]["ns_per_render"].pop()
        self.write()
        with self.assertRaisesRegex(ValueError, "incomplete sampling"):
            load_publication(self.folder)


if __name__ == "__main__":
    unittest.main()
