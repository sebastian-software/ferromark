#!/usr/bin/env python3
"""Build only the frozen optimization-round baseline worker.

This imports the repository helper instead of editing it, overrides its
baseline revision to the requested audit commit, and records the same build
metadata shape for later pairing with a candidate build.
"""

import hashlib
import importlib.util
import json
import os
from pathlib import Path
import subprocess

ROOT = Path("/Users/sebastian/Workspace/ferromark-v2")
HELPER_PATH = ROOT / "benchmarks/optimization-rounds/prepare.py"
BASELINE_REVISION = "4342b310d8a6612b5df67d697b9d2be733c4ca70"
WORKER_PATH = ROOT / "benchmarks/optimization-rounds/worker.rs"

spec = importlib.util.spec_from_file_location("optimization_prepare", HELPER_PATH)
if spec is None or spec.loader is None:
    raise RuntimeError(f"cannot load {HELPER_PATH}")
helper = importlib.util.module_from_spec(spec)
spec.loader.exec_module(helper)
helper.BASELINE_REVISION = BASELINE_REVISION
helper.WORKER = WORKER_PATH


def main() -> None:
    import argparse

    parser = argparse.ArgumentParser()
    parser.add_argument("baseline_source", type=Path)
    parser.add_argument("build_dir", type=Path)
    parser.add_argument("--lto", choices=("fat", "thin", "off"), default="fat")
    args = parser.parse_args()
    source = args.baseline_source.resolve()
    build = args.build_dir.resolve()
    if build.exists():
        raise SystemExit(f"output already exists: {build}")
    build.mkdir(parents=True)

    verification = helper.verify_baseline(source)
    worker = WORKER_PATH.read_bytes()
    worker_sha = hashlib.sha256(worker).hexdigest()
    env = os.environ.copy()
    env.pop("CARGO_ENCODED_RUSTFLAGS", None)
    env["RUSTFLAGS"] = "-C target-cpu=generic"
    env.pop("CARGO_PROFILE_RELEASE_LTO", None)
    env.pop("CARGO_PROFILE_RELEASE_OPT_LEVEL", None)
    rustc = subprocess.check_output(["rustc", "+1.95", "-vV"], text=True)

    (build / "worker.rs").write_bytes(worker)
    (build / "build.json").write_text(json.dumps({
        "schema": 2,
        "rustc": rustc,
        "baseline_revision_expected": BASELINE_REVISION,
        "baseline_verification": verification,
        "worker_sha256": worker_sha,
        "worker_source": str(WORKER_PATH),
        "lto": args.lto,
        "frozen_lock_sha256": helper.sha256(source / "Cargo.lock"),
        "engines": {},
    }, indent=2) + "\n")

    info = helper.build_engine(
        "baseline", source, build, worker, worker_sha, args.lto, cached=None
    )
    record = json.loads((build / "build.json").read_text())
    record["engines"]["baseline"] = info
    (build / "build.json").write_text(json.dumps(record, indent=2) + "\n")
    print(build / "build.json")


if __name__ == "__main__":
    main()
