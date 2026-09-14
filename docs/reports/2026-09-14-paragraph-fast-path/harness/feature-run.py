#!/usr/bin/env python3
"""Replay a feature-scan corpus with frozen, output-adjacent JSON profiles.

The optimization-rounds runner predates runtime-configured profiles and
expects each case's ``profile`` to be an existing path.  This wrapper treats
``runtime_options`` as the source of truth, rewrites every profile into a
persistent sibling replay bundle, then delegates all verification and timing
to the existing runner.
"""

import argparse
import copy
import gzip
import hashlib
import json
from pathlib import Path
import subprocess
import sys


HERE = Path(__file__).resolve().parent
OPTIMIZATION_RUNNER = HERE.parent / "optimization-rounds" / "run.py"


def load_corpus(path: Path):
    """Load either the plain or gzipped corpus representation."""
    raw = path.read_bytes()
    if path.suffix == ".gz":
        raw = gzip.decompress(raw)
    return json.loads(raw)


def canonical_json(value):
    return json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=False)


def config_digest(config):
    return hashlib.sha256(canonical_json(config).encode("utf-8")).hexdigest()


def validate_options(case):
    options = case.get("runtime_options")
    if not isinstance(options, dict) or set(options) != {"parser", "renderer"}:
        raise ValueError(f"{case.get('name', '<unnamed>')}: invalid runtime_options shape")
    if any(not isinstance(options[side], dict) for side in ("parser", "renderer")):
        raise ValueError(f"{case.get('name', '<unnamed>')}: parser/renderer must be objects")
    # Ensure values can be represented by the same JSON file that the worker
    # reads.  Unknown option names are intentionally left to the worker so the
    # wrapper remains aligned with its runtime option contract.
    try:
        json.dumps(options, ensure_ascii=False)
    except (TypeError, ValueError) as error:
        raise ValueError(f"{case.get('name', '<unnamed>')}: invalid runtime option value") from error
    return options


def validate_case(case, names):
    if not isinstance(case, dict):
        raise ValueError("corpus case must be an object")
    name = case.get("name")
    if not isinstance(name, str) or not name:
        raise ValueError("corpus case has no name")
    if name in names:
        raise ValueError(f"duplicate corpus case: {name}")
    names.add(name)
    source = case.get("input")
    if not isinstance(source, str):
        raise ValueError(f"{name}: input must be a UTF-8 string")
    raw = source.encode("utf-8")
    if case.get("byte_count") != len(raw):
        raise ValueError(f"{name}: byte_count does not match input")
    if case.get("sha256") != hashlib.sha256(raw).hexdigest():
        raise ValueError(f"{name}: sha256 does not match input")
    validate_options(case)


def replay_paths(output: Path):
    """Return the persistent sibling directory used by one replay."""
    return output.with_name(output.name + ".feature-scan-replay")


def materialize_replay(corpus_path: Path, output: Path):
    """Validate a corpus and write a path-independent replay bundle.

    The bundle is kept beside ``output`` so the result directory and its
    sibling can be archived together and replayed without the original
    corpus' temporary profile paths.
    """
    corpus = load_corpus(corpus_path)
    if not isinstance(corpus, dict) or not isinstance(corpus.get("cases"), list):
        raise ValueError("corpus must contain a cases array")
    normalized = copy.deepcopy(corpus)
    normalized["schema"] = max(1, int(normalized.get("schema", 1)))
    names = set()
    for case in normalized["cases"]:
        validate_case(case, names)
    if output.exists():
        raise FileExistsError(f"output already exists: {output}")
    stage = replay_paths(output)
    if stage.exists():
        raise FileExistsError(f"replay bundle already exists: {stage}")
    stage.mkdir(parents=True)
    config_dir = stage / "configs"
    config_dir.mkdir()

    configs = {}
    for case in normalized["cases"]:
        options = validate_options(case)
        digest = config_digest(options)
        if digest not in configs:
            config_path = config_dir / f"runtime-{digest}.json"
            config_path.write_text(json.dumps(options, indent=2, sort_keys=True,
                                              ensure_ascii=False) + "\n")
            configs[digest] = config_path.resolve()
        case["profile"] = str(configs[digest])
        # Keep the metadata explicit after normalization, including when an
        # older archive omitted a redundant profile field.
        case["runtime_options"] = options

    normalized["replay"] = {
        "source_corpus": str(corpus_path.resolve()),
        "config_dir": str(config_dir.resolve()),
        "config_count": len(configs),
    }
    normalized_path = stage / "corpus.json"
    normalized_path.write_text(json.dumps(normalized, indent=2, ensure_ascii=False) + "\n")
    return stage, normalized_path


def command(build: Path, corpus: Path, output: Path, runner_args):
    return [sys.executable, str(OPTIMIZATION_RUNNER), str(build), str(corpus), str(output), *runner_args]


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("build", type=Path)
    parser.add_argument("corpus", type=Path, help="JSON or JSON.GZ corpus")
    parser.add_argument("output", type=Path)
    parser.add_argument("runner_args", nargs=argparse.REMAINDER,
                        help="additional arguments passed to optimization-rounds/run.py")
    args = parser.parse_args(argv)
    try:
        stage, normalized = materialize_replay(args.corpus.resolve(), args.output.resolve())
        result = subprocess.run(
            command(args.build.resolve(), normalized, args.output.resolve(), args.runner_args),
            check=False,
        )
    except (OSError, ValueError, json.JSONDecodeError, gzip.BadGzipFile) as error:
        raise SystemExit(str(error)) from error
    if result.returncode:
        raise SystemExit(result.returncode)
    print(f"Replay bundle: {stage}")


if __name__ == "__main__":
    main()
