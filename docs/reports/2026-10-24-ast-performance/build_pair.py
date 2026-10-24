#!/usr/bin/env python3
"""Freeze and build this experiment with the repository's existing worker builder."""
import argparse
import hashlib
import importlib.util
import json
from pathlib import Path
import shutil
import subprocess

ROOT = Path(__file__).resolve().parents[3]
spec = importlib.util.spec_from_file_location("prepare", ROOT / "benchmarks/optimization-rounds/prepare.py")
prepare = importlib.util.module_from_spec(spec)
spec.loader.exec_module(prepare)
p = argparse.ArgumentParser(description=__doc__)
p.add_argument("baseline", type=Path)
p.add_argument("candidate", type=Path)
p.add_argument("output", type=Path)
p.add_argument("--verify-baseline", "--verify-head", dest="verify_baseline", action="store_true")
p.add_argument("--baseline-revision", default="b4e4377a541a15aa72559cda1ab26fc8e3f322e8")
p.add_argument("--reuse-baseline", type=Path)
p.add_argument("--baseline-engine", choices=("baseline", "candidate"), default="baseline")
p.add_argument("--reuse-candidate", type=Path)
p.add_argument("--candidate-engine", choices=("baseline", "candidate"), default="candidate")
a = p.parse_args()
sources = {"baseline": a.baseline.resolve(), "candidate": a.candidate.resolve()}
original_manifest = prepare.manifest

def manifest(dependency, lto):
    # The historical builder predates the transforms workspace member.
    shutil.copytree(sources[dependency.name] / "transforms", dependency / "transforms")
    return original_manifest(dependency, lto).replace(
        f'ferromark = {{ path = {json.dumps(str(dependency))} }}',
        f'ferromark = {{ path = {json.dumps(str(dependency))}, features = ["jsx"] }}',
    )

prepare.manifest = manifest
worker = Path(__file__).with_name("worker.rs").read_bytes()
worker_sha = hashlib.sha256(worker).hexdigest()
a.output.mkdir(parents=True, exist_ok=False)
(a.output / "worker.rs").write_bytes(worker)
verification = prepare.verify_baseline(a.baseline, a.baseline_revision) if a.verify_baseline else None
assert prepare.sha256(a.baseline / "Cargo.lock") == prepare.sha256(a.candidate / "Cargo.lock")
cached = {}
for name, directory, engine in [("baseline", a.reuse_baseline, a.baseline_engine), ("candidate", a.reuse_candidate, a.candidate_engine)]:
    if directory:
        previous = json.loads((directory / "build.json").read_text())
        assert previous["features"] == ["jsx"]
        assert previous["rustc"] == subprocess.check_output(["rustc", "+1.95", "-vV"], text=True)
        cached[name] = previous["engines"][engine]
engines = {
    name: prepare.build_engine(name, source, a.output, worker, worker_sha, "fat", cached.get(name))
    for name, source in sources.items()
}
record = {
    "schema": 2,
    "rustc": subprocess.check_output(["rustc", "+1.95", "-vV"], text=True),
    "worker_sha256": worker_sha,
    "lto": "fat",
    "features": ["jsx"],
    "baseline_verification": verification,
    "engines": engines,
}
(a.output / "build.json").write_text(json.dumps(record, indent=2) + "\n")
print(a.output / "build.json")
