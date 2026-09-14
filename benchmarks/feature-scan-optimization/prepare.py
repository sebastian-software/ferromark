#!/usr/bin/env python3
"""Build frozen baseline and candidate workers for feature-scan experiments.

The two workers are byte-for-byte identical apart from the frozen core source
tree they link against.  The worker accepts an absolute JSON configuration
path as its profile argument; this keeps the compared runtime options outside
the binaries and makes every corpus row self-describing.
"""

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


BASELINE_REVISION = "1728355"
WORKER = Path(__file__).resolve().parents[1] / "runtime-profiles" / "worker.rs"
ROOT = Path(__file__).resolve().parents[2]
CORE_FILES = ("Cargo.toml", "Cargo.lock", "rust-toolchain.toml", "crates")


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def tree_sha(root: Path) -> str:
    """Hash a source snapshot without filesystem metadata or build products."""
    digest = hashlib.sha256()
    for path in sorted(root.rglob("*")):
        if not path.is_file() or ".git" in path.parts or "target" in path.parts:
            continue
        relative = path.relative_to(root).as_posix().encode()
        digest.update(len(relative).to_bytes(8, "big"))
        digest.update(relative)
        digest.update(sha256(path).encode())
    return digest.hexdigest()


def registry_packages(path: Path):
    lock = tomllib.loads(path.read_text())
    return {
        (package["name"], package["version"]): package.get("checksum")
        for package in lock.get("package", []) if "source" in package
    }


def git_revision(source: Path):
    try:
        return subprocess.check_output(
            ["git", "-C", str(source), "rev-parse", "HEAD"],
            text=True,
            stderr=subprocess.STDOUT,
        ).strip()
    except (OSError, subprocess.CalledProcessError):
        return None


def archive_revision(revision: str) -> bytes:
    return subprocess.check_output(
        ["git", "-C", str(ROOT), "archive", revision, *CORE_FILES]
    )


def extract_archive(archive: bytes, destination: Path):
    destination.mkdir(parents=True)
    with tarfile.open(fileobj=io.BytesIO(archive)) as stream:
        stream.extractall(destination, filter="data")


def copy_core(source: Path, destination: Path):
    missing = [name for name in CORE_FILES if not (source / name).exists()]
    if missing:
        raise SystemExit(f"candidate core is missing: {', '.join(missing)}")
    destination.mkdir(parents=True)
    for name in CORE_FILES[:-1]:
        shutil.copyfile(source / name, destination / name)
    shutil.copytree(source / "crates", destination / "crates")


def manifest(dependency: Path) -> str:
    return (
        '[package]\nname = "feature-scan-worker"\nversion = "0.0.0"\n'
        'edition = "2024"\npublish = false\n\n[workspace]\n\n[dependencies]\n'
        f'ferromark = {{ path = {json.dumps(str(dependency))} }}\n'
        'serde_json = "1.0"\n\n[profile.release]\nopt-level = 3\n'
        'lto = "fat"\ncodegen-units = 1\npanic = "abort"\nstrip = true\n'
    )


def cache_matches(cached, source: Path, worker_sha: str):
    binary = Path(cached.get("binary", ""))
    if not binary.is_file():
        return False
    expected = {
        "source_tree_sha256": tree_sha(source),
        "source_lock_sha256": sha256(source / "Cargo.lock"),
        "worker_sha256": worker_sha,
        "lto": "fat",
        "rustflags": "-C target-cpu=generic",
    }
    return all(cached.get(key) == value for key, value in expected.items()) and (
        sha256(binary) == cached.get("binary_sha256")
    )


def build_engine(name: str, source: Path, build: Path, worker: bytes,
                 worker_sha: str, cached=None):
    package = build / name
    package.mkdir(parents=True, exist_ok=True)
    binary = package / "target" / "release" / "feature-scan-worker"
    if cached and cache_matches(cached, source, worker_sha):
        return cached | {"cache": "reused"}

    frozen = build / "sources" / name
    if frozen.exists():
        shutil.rmtree(frozen)
    copy_core(source, frozen)

    src = package / "src"
    src.mkdir(parents=True, exist_ok=True)
    (src / "main.rs").write_bytes(worker)
    shutil.copyfile(frozen / "Cargo.lock", package / "Cargo.lock")
    (package / "Cargo.toml").write_text(manifest(frozen / "crates" / "ferromark"))

    env = os.environ.copy()
    env.pop("CARGO_ENCODED_RUSTFLAGS", None)
    env["RUSTFLAGS"] = "-C target-cpu=generic"
    env["CARGO_TARGET_DIR"] = str(package / "target")
    command = ["cargo", "+1.95", "build", "--release", "--offline"]
    with (package / "build.log").open("w") as log:
        result = subprocess.run(command, cwd=package, env=env, stdout=log, stderr=log)
    if result.returncode:
        raise SystemExit((package / "build.log").read_text())

    original = registry_packages(source / "Cargo.lock")
    retained = registry_packages(package / "Cargo.lock")
    if any(original.get(key) != value for key, value in retained.items()):
        raise SystemExit(f"{name}: dependency versions/checksums changed")

    return {
        "frozen_source": str(frozen),
        "frozen_source_sha256": tree_sha(frozen),
        "source": str(source),
        "source_revision": git_revision(source),
        "source_tree_sha256": tree_sha(source),
        "source_manifest_sha256": sha256(source / "Cargo.toml"),
        "source_lock_sha256": sha256(source / "Cargo.lock"),
        "worker_sha256": worker_sha,
        "lto": "fat",
        "rustflags": env["RUSTFLAGS"],
        "binary": str(binary),
        "binary_sha256": sha256(binary),
        "lock_sha256": sha256(package / "Cargo.lock"),
        "retained_registry_packages": len(retained),
        "registry_versions_unchanged": True,
        "command": command,
        "cache": "built",
    }


def runtime_views(build: Path, record: dict, worker: bytes):
    """Expose the same binaries to the existing off/on runtime-profile runner."""
    for name, engine in record["engines"].items():
        view = build / "runtime-views" / name
        src = view / "worker" / "src"
        src.mkdir(parents=True, exist_ok=True)
        (src / "main.rs").write_bytes(worker)
        metadata = engine | {
            "rustc": record["rustc"],
            "feature_scan_build": str(build / "build.json"),
            "engine": name,
        }
        (view / "build.json").write_text(json.dumps(metadata, indent=2) + "\n")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--baseline-revision", default=BASELINE_REVISION)
    parser.add_argument("--candidate-path", type=Path, required=True)
    parser.add_argument("--out", type=Path, required=True)
    parser.add_argument("--reuse-baseline-build", type=Path)
    args = parser.parse_args()

    candidate = args.candidate_path.resolve()
    build = args.out.resolve()
    build.mkdir(parents=True, exist_ok=True)
    worker = WORKER.read_bytes()
    worker_sha = hashlib.sha256(worker).hexdigest()
    (build / "worker.rs").write_bytes(worker)

    archive = archive_revision(args.baseline_revision)
    # Keep the archived/copied inputs separate from the package/source
    # snapshots made by build_engine.  In particular, build_engine replaces
    # its frozen source directory on every non-cached build.
    baseline = build / "core" / "baseline"
    if baseline.exists():
        shutil.rmtree(baseline)
    extract_archive(archive, baseline)
    candidate_source = build / "core" / "candidate"
    if candidate_source.exists():
        shutil.rmtree(candidate_source)
    copy_core(candidate, candidate_source)

    if sha256(baseline / "Cargo.lock") != sha256(candidate_source / "Cargo.lock"):
        raise SystemExit("baseline and candidate Cargo.lock files differ")
    if registry_packages(baseline / "Cargo.lock") != registry_packages(candidate_source / "Cargo.lock"):
        raise SystemExit("baseline and candidate registry dependencies differ")

    previous = {}
    if args.reuse_baseline_build:
        cached_build = args.reuse_baseline_build.resolve()
        cached_data = json.loads((cached_build / "build.json").read_text())
        if (cached_data.get("worker_sha256") == worker_sha and
                cached_data.get("baseline_revision_expected") == args.baseline_revision and
                cached_data.get("rustc") == subprocess.check_output(["rustc", "+1.95", "-vV"], text=True)):
            previous["baseline"] = cached_data.get("engines", {}).get("baseline")

    engines = {
        "baseline": build_engine("baseline", baseline, build, worker, worker_sha,
                                  previous.get("baseline")),
        "candidate": build_engine("candidate", candidate_source, build, worker, worker_sha),
    }
    record = {
        "schema": 2,
        "rustc": subprocess.check_output(["rustc", "+1.95", "-vV"], text=True),
        "baseline_revision_expected": args.baseline_revision,
        "baseline_archive_sha256": hashlib.sha256(archive).hexdigest(),
        "worker_sha256": worker_sha,
        "worker_source": str(WORKER),
        "lto": "fat",
        "rustflags": "-C target-cpu=generic",
        "frozen_lock_sha256": sha256(baseline / "Cargo.lock"),
        "engines": engines,
    }
    (build / "build.json").write_text(json.dumps(record, indent=2) + "\n")
    runtime_views(build, record, worker)
    print(build / "build.json")


if __name__ == "__main__":
    main()
