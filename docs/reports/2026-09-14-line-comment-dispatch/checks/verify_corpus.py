#!/usr/bin/env python3
"""Run the existing differential gate across every case, without timing."""

import argparse
import gzip
import importlib.util
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parents[4]


def module(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    result = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(result)
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("build", type=Path)
    parser.add_argument("corpus", type=Path)
    parser.add_argument("output", type=Path)
    args = parser.parse_args()
    runner = module("runner", ROOT / "benchmarks/optimization-rounds/run.py")
    replay = module("replay", ROOT / "benchmarks/feature-scan-optimization/run.py")
    _, corpus_path = replay.materialize_replay(args.corpus, args.output)
    args.output.mkdir()
    build = json.loads((args.build / "build.json").read_text())
    assert runner.digest(args.build / "worker.rs") == build["worker_sha256"]
    for engine in runner.ENGINES:
        info = build["engines"][engine]
        assert runner.digest(Path(info["binary"])) == info["binary_sha256"]
    corpus = json.loads(corpus_path.read_text())
    modes = ["parse", "render", "fresh", "reuse"]
    outputs = {}
    inputs = args.output / "inputs"
    inputs.mkdir()
    for case in corpus["cases"]:
        path = inputs / (case["name"] + ".md")
        path.write_bytes(case["input"].encode())
        outputs[case["name"]] = runner.verify_case(build, path, case, modes)
    raw = json.dumps(outputs, ensure_ascii=False).encode()
    (args.output / "verification.json.gz").write_bytes(gzip.compress(raw, mtime=0))
    result = {"cases": len(outputs), "modes": modes,
              "comparisons": len(outputs) * len(modes),
              "status": "exact HTML, AST Debug including spans, and child counts",
              "build": build}
    (args.output / "result.json").write_text(json.dumps(result, indent=2) + "\n")
    print(f"All {len(outputs)} cases verified across {len(modes)} lifecycles.")


if __name__ == "__main__":
    main()
