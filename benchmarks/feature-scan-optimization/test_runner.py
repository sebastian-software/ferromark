"""Focused guards for replay profile materialization and corpus integrity."""

import gzip
import hashlib
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest


HERE = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location("feature_scan_run", HERE / "run.py")
RUN = importlib.util.module_from_spec(spec)
spec.loader.exec_module(RUN)


def corpus_case(source="Term\n: definition\n"):
    raw = source.encode("utf-8")
    return {
        "name": "sample",
        "profile": "/old/tmp/profile.json",
        "input": source,
        "byte_count": len(raw),
        "sha256": hashlib.sha256(raw).hexdigest(),
        "runtime_options": {"parser": {"definition_lists": True}, "renderer": {}},
    }


class ReplayGuards(unittest.TestCase):
    def test_json_and_gzip_replay_rewrite_stale_absolute_profile(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            corpus = {"schema": 1, "cases": [corpus_case()]}
            plain = root / "corpus.json"
            plain.write_text(json.dumps(corpus))
            gzipped = root / "corpus.json.gz"
            gzipped.write_bytes(gzip.compress(plain.read_bytes()))
            self.assertEqual(RUN.load_corpus(gzipped), corpus)

            output = root / "results"
            stage, normalized_path = RUN.materialize_replay(gzipped, output)
            normalized = json.loads(normalized_path.read_text())
            case = normalized["cases"][0]
            self.assertTrue(Path(case["profile"]).is_absolute())
            self.assertTrue(Path(case["profile"]).is_relative_to((stage / "configs").resolve()))
            self.assertEqual(json.loads(Path(case["profile"]).read_text()),
                             case["runtime_options"])
            self.assertNotIn("/old/tmp/", case["profile"])

    def test_replay_rejects_input_or_runtime_option_metadata_drift(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            bad_hash = corpus_case()
            bad_hash["sha256"] = "0" * 64
            bad_options = corpus_case()
            bad_options["runtime_options"] = {"parser": [], "renderer": {}}
            for case in (bad_hash, bad_options):
                corpus = root / f"{case['name']}-{id(case)}.json"
                corpus.write_text(json.dumps({"cases": [case]}))
                with self.assertRaises(ValueError):
                    RUN.materialize_replay(corpus, root / f"result-{id(case)}")

    def test_config_digest_is_order_independent(self):
        left = {"parser": {"line_comments": True}, "renderer": {"xhtml": True}}
        right = {"renderer": {"xhtml": True}, "parser": {"line_comments": True}}
        self.assertEqual(RUN.config_digest(left), RUN.config_digest(right))


if __name__ == "__main__":
    unittest.main()
