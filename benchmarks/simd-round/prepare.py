#!/usr/bin/env python3
"""Build isolated baseline-v2 and candidate-v2 SIMD measurement workers."""

import argparse
import hashlib
import io
import json
import os
from pathlib import Path
import shutil
import subprocess
import tarfile
import tomllib

BASELINE_REVISION = "ae963d425a4c15841ced5f7359a4b9adc47ca931"
WORKER = Path(__file__).with_name("worker.rs")


def verify_baseline(source: Path):
    """Check archived core bytes, including snapshots with no .git directory."""
    repository = Path(__file__).resolve().parents[2]
    raw = subprocess.check_output([
        "git", "-C", str(repository), "archive", BASELINE_REVISION,
        "Cargo.toml", "Cargo.lock", "crates",
    ])
    expected = {}
    with tarfile.open(fileobj=io.BytesIO(raw)) as archive:
        for member in archive.getmembers():
            if member.isfile():
                expected[member.name] = hashlib.sha256(archive.extractfile(member).read()).hexdigest()
    actual = {str(path.relative_to(source)): file_sha(path)
              for path in (source / "crates").rglob("*") if path.is_file()}
    for name in ("Cargo.toml", "Cargo.lock"):
        actual[name] = file_sha(source / name)
    if actual != expected:
        changed = sorted(name for name in actual.keys() | expected.keys()
                         if actual.get(name) != expected.get(name))
        raise ValueError(f"baseline core differs from {BASELINE_REVISION}: {changed[:10]}")
    return {"revision": BASELINE_REVISION, "verified_core_files": len(expected),
            "core_file_hashes": expected}


def file_sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def tree_sha(root: Path) -> str:
    digest = hashlib.sha256()
    for path in sorted(root.rglob("*")):
        if not path.is_file() or ".git" in path.parts or "target" in path.parts:
            continue
        relative = path.relative_to(root).as_posix().encode()
        digest.update(len(relative).to_bytes(8, "big"))
        digest.update(relative)
        digest.update(file_sha(path).encode())
    return digest.hexdigest()


def registry_packages(path: Path):
    lock = tomllib.loads(path.read_text())
    return {
        (package["name"], package["version"]): package.get("checksum")
        for package in lock.get("package", [])
        if "source" in package
    }


def revision(source: Path):
    try:
        return subprocess.check_output(
            ["git", "-C", str(source), "rev-parse", "HEAD"], text=True, stderr=subprocess.STDOUT
        ).strip()
    except (OSError, subprocess.CalledProcessError):
        return None


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("baseline_source", type=Path)
    parser.add_argument("candidate_source", type=Path)
    parser.add_argument("build_dir", type=Path)
    args = parser.parse_args()
    baseline_verification = verify_baseline(args.baseline_source.resolve())
    if registry_packages(args.baseline_source / "Cargo.lock") != registry_packages(args.candidate_source / "Cargo.lock"):
        raise SystemExit("baseline and candidate registry dependencies differ")
    build = args.build_dir.resolve()
    build.mkdir(parents=True, exist_ok=False)
    env = os.environ.copy()
    env.pop("CARGO_ENCODED_RUSTFLAGS", None)
    env["RUSTFLAGS"] = "-C target-cpu=generic"
    worker_bytes = WORKER.read_bytes()
    worker_sha256 = hashlib.sha256(worker_bytes).hexdigest()
    record = {
        "rustc": subprocess.check_output(["rustc", "+1.95", "-vV"], text=True),
        "rustflags": env["RUSTFLAGS"],
        "worker_sha256": worker_sha256,
        "baseline_revision_expected": BASELINE_REVISION,
        "baseline_verification": baseline_verification,
        "engines": {},
    }
    for name, source in (("baseline", args.baseline_source), ("candidate", args.candidate_source)):
        source = source.resolve()
        root = build / name
        (root / "src").mkdir(parents=True)
        root.joinpath("src/main.rs").write_bytes(worker_bytes)
        shutil.copyfile(source / "Cargo.lock", root / "Cargo.lock")
        dependency = source / "crates/ferromark"
        root.joinpath("Cargo.toml").write_text(
            '[package]\nname = "simd-round-worker"\nversion = "0.0.0"\n'
            'edition = "2024"\npublish = false\n\n[workspace]\n\n[dependencies]\n'
            f'ferromark = {{ path = {json.dumps(str(dependency))} }}\n\n'
            '[profile.release]\nopt-level = 3\nlto = "fat"\n'
            'codegen-units = 1\npanic = "abort"\nstrip = true\n'
        )
        env["CARGO_TARGET_DIR"] = str(root / "target")
        # The copied source lockfile has the library workspace packages but
        # not this temporary worker package. Cargo must add that path package;
        # all registry versions are checked against the source lock below.
        command = ["cargo", "+1.95", "build", "--release", "--offline"]
        with root.joinpath("build.log").open("w") as log:
            result = subprocess.run(command, cwd=root, env=env, stdout=log, stderr=log)
        if result.returncode:
            raise SystemExit(root.joinpath("build.log").read_text())
        original = registry_packages(source / "Cargo.lock")
        retained = registry_packages(root / "Cargo.lock")
        if any(original.get(key) != value for key, value in retained.items()):
            raise SystemExit(f"{name}: dependency versions/checksums changed")
        binary = root / "target/release/simd-round-worker"
        record["engines"][name] = {
            "source": str(source),
            "source_revision": revision(source),
            "source_tree_sha256": tree_sha(source),
            "source_manifest_sha256": file_sha(source / "Cargo.toml"),
            "source_lock_sha256": file_sha(source / "Cargo.lock"),
            "binary": str(binary),
            "binary_sha256": file_sha(binary),
            "lock_sha256": file_sha(root / "Cargo.lock"),
            "retained_registry_packages": len(retained),
            "registry_versions_unchanged": True,
            "command": command,
        }
    build.joinpath("build.json").write_text(json.dumps(record, indent=2) + "\n")
    print(build / "build.json")


if __name__ == "__main__":
    main()
