#!/usr/bin/env python3
"""Freeze and build the current candidate beside the prepared baseline."""

import hashlib
import importlib.util
import json
import os
from pathlib import Path
import subprocess

ROOT = Path("/Users/sebastian/Workspace/ferromark-v2")
HELPER_PATH = ROOT / "benchmarks/optimization-rounds/prepare.py"
BUILD = Path("/private/tmp/ferromark-v2-correctness-fixes/performance/build-01")

spec = importlib.util.spec_from_file_location("optimization_prepare", HELPER_PATH)
if spec is None or spec.loader is None:
    raise RuntimeError(f"cannot load {HELPER_PATH}")
helper = importlib.util.module_from_spec(spec)
spec.loader.exec_module(helper)


def main() -> None:
    metadata_path = BUILD / "build.json"
    metadata = json.loads(metadata_path.read_text())
    if "baseline" not in metadata["engines"] or "candidate" in metadata["engines"]:
        raise SystemExit("build-01 metadata is not baseline-only")
    worker_path = BUILD / "worker.rs"
    worker = worker_path.read_bytes()
    worker_sha = hashlib.sha256(worker).hexdigest()
    if worker_sha != metadata["worker_sha256"]:
        raise SystemExit("worker bytes changed after baseline preparation")
    source = ROOT.resolve()
    env = os.environ.copy()
    env.pop("CARGO_ENCODED_RUSTFLAGS", None)
    env.pop("CARGO_PROFILE_RELEASE_LTO", None)
    env.pop("CARGO_PROFILE_RELEASE_OPT_LEVEL", None)
    env["RUSTFLAGS"] = "-C target-cpu=generic"
    rustc = subprocess.check_output(["rustc", "+1.95", "-vV"], text=True)
    if rustc != metadata["rustc"]:
        raise SystemExit("rustc identity changed after baseline preparation")
    info = helper.build_engine(
        "candidate", source, BUILD, worker, worker_sha, metadata["lto"], cached=None
    )
    metadata["engines"]["candidate"] = info
    metadata["candidate_source_tree_sha256_at_freeze"] = helper.tree_sha(source)
    metadata["candidate_core_revision_note"] = (
        "working tree frozen after root reported correctness gates green"
    )
    metadata_path.write_text(json.dumps(metadata, indent=2) + "\n")
    print(metadata_path)


if __name__ == "__main__":
    main()
