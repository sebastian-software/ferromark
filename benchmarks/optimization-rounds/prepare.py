#!/usr/bin/env python3
"""Build reproducible baseline/candidate workers for optimization rounds."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import tomllib

BASELINE_REVISION = "4de75d4843747a218771b5ec46df9171d4f54a15"
WORKER = Path(__file__).with_name("worker.rs")


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def tree_sha(root: Path) -> str:
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


def revision(source: Path):
    try:
        return subprocess.check_output(
            ["git", "-C", str(source), "rev-parse", "HEAD"],
            text=True, stderr=subprocess.STDOUT,
        ).strip()
    except (OSError, subprocess.CalledProcessError):
        return None


def archived_hashes(repository: Path):
    raw = subprocess.check_output(
        ["git", "-C", str(repository), "archive", BASELINE_REVISION,
         "Cargo.toml", "Cargo.lock", "crates"],
    )
    import io
    import tarfile
    with tarfile.open(fileobj=io.BytesIO(raw)) as archive:
        return {
            member.name: hashlib.sha256(archive.extractfile(member).read()).hexdigest()
            for member in archive.getmembers() if member.isfile()
        }


def verify_baseline(source: Path):
    """Verify the baseline against the frozen 4de75d4 core when git is available."""
    repository = Path(__file__).resolve().parents[2]
    expected = archived_hashes(repository)
    actual = {}
    for path in [source / "Cargo.toml", source / "Cargo.lock", *sorted((source / "crates").rglob("*"))]:
        if path.is_file():
            actual[str(path.relative_to(source))] = sha256(path)
    if actual != expected:
        changed = sorted(set(actual) | set(expected))
        changed = [name for name in changed if actual.get(name) != expected.get(name)]
        raise ValueError(f"baseline core differs from {BASELINE_REVISION}: {changed[:10]}")
    return {"revision": BASELINE_REVISION, "verified_core_files": len(expected), "core_file_hashes": expected}


def manifest(dependency: Path, lto: str) -> str:
    lto_value = {"fat": '"fat"', "thin": '"thin"', "off": "false"}[lto]
    return (
        '[package]\nname = "optimization-round-worker"\nversion = "0.0.0"\n'
        'edition = "2024"\npublish = false\n\n[workspace]\n\n[dependencies]\n'
        f'ferromark = {{ path = {json.dumps(str(dependency))} }}\n\n'
        '[profile.release]\nopt-level = 3\n'
        f'lto = {lto_value}\ncodegen-units = 1\npanic = "abort"\nstrip = true\n'
    )


def cache_matches(cached, source, worker_sha, lto):
    if not cached or not Path(cached.get("binary", "")).is_file():
        return False
    return all(cached.get(key) == value for key, value in {
        "source_tree_sha256": tree_sha(source),
        "source_lock_sha256": sha256(source / "Cargo.lock"),
        "worker_sha256": worker_sha,
        "lto": lto,
    }.items()) and sha256(Path(cached["binary"])) == cached.get("binary_sha256")


def build_engine(name, source: Path, build: Path, worker: bytes, worker_sha: str, lto: str, cached):
    root = build / name
    root.mkdir(parents=True, exist_ok=True)
    binary = root / "target" / "release" / "optimization-round-worker"
    if cache_matches(cached, source, worker_sha, lto):
        return cached | {"cache": "reused"}
    snapshot = build / "sources" / name
    if snapshot.exists():
        shutil.rmtree(snapshot)
    snapshot.mkdir(parents=True)
    for filename in ("Cargo.toml", "Cargo.lock", "rust-toolchain.toml"):
        if (source / filename).exists():
            shutil.copyfile(source / filename, snapshot / filename)
    shutil.copytree(source / "crates", snapshot / "crates")
    if (source / "node/native").exists():
        shutil.copytree(source / "node/native", snapshot / "node/native")
    (root / "src").mkdir(parents=True, exist_ok=True)
    (root / "src" / "main.rs").write_bytes(worker)
    (root / "Cargo.lock").write_bytes((source / "Cargo.lock").read_bytes())
    (root / "Cargo.toml").write_text(manifest(snapshot / "crates" / "ferromark", lto))
    env = os.environ.copy()
    env.pop("CARGO_ENCODED_RUSTFLAGS", None)
    env["RUSTFLAGS"] = "-C target-cpu=generic"
    env["CARGO_TARGET_DIR"] = str(root / "target")
    command = ["cargo", "+1.95", "build", "--release", "--offline"]
    with (root / "build.log").open("w") as log:
        result = subprocess.run(command, cwd=root, env=env, stdout=log, stderr=log)
    if result.returncode:
        raise SystemExit((root / "build.log").read_text())
    original = registry_packages(source / "Cargo.lock")
    retained = registry_packages(root / "Cargo.lock")
    if any(original.get(key) != value for key, value in retained.items()):
        raise SystemExit(f"{name}: dependency versions/checksums changed")
    return {
        "frozen_source": str(snapshot), "frozen_source_sha256": tree_sha(snapshot),
        "source": str(source), "source_revision": revision(source),
        "source_tree_sha256": tree_sha(source),
        "source_manifest_sha256": sha256(source / "Cargo.toml"),
        "source_lock_sha256": sha256(source / "Cargo.lock"),
        "worker_sha256": worker_sha, "lto": lto, "rustflags": env["RUSTFLAGS"],
        "binary": str(binary), "binary_sha256": sha256(binary),
        "lock_sha256": sha256(root / "Cargo.lock"),
        "retained_registry_packages": len(retained), "registry_versions_unchanged": True,
        "command": command, "cache": "built",
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--baseline-path", type=Path, required=True)
    parser.add_argument("--candidate-path", type=Path, required=True)
    parser.add_argument("--out", type=Path, required=True)
    parser.add_argument("--reuse-baseline-build", type=Path)
    parser.add_argument("--lto", choices=("fat", "thin", "off"), default="fat")
    args = parser.parse_args()
    baseline = args.baseline_path.resolve()
    candidate = args.candidate_path.resolve()
    build = args.out.resolve()
    build.mkdir(parents=True, exist_ok=True)
    worker = WORKER.read_bytes()
    worker_sha = hashlib.sha256(worker).hexdigest()
    (build / "worker.rs").write_bytes(worker)
    previous = {}
    metadata = build / "build.json"
    if metadata.is_file():
        previous = json.loads(metadata.read_text()).get("engines", {})
    if args.reuse_baseline_build:
        cached = json.loads((args.reuse_baseline_build / "build.json").read_text())
        if cached["rustc"] != subprocess.check_output(["rustc", "+1.95", "-vV"], text=True):
            raise SystemExit("cached rustc identity differs")
        previous["baseline"] = cached["engines"]["baseline"]
    baseline_verification = verify_baseline(baseline)
    if sha256(baseline / "Cargo.lock") != sha256(candidate / "Cargo.lock"):
        raise SystemExit("baseline and candidate Cargo.lock files differ")
    if registry_packages(baseline / "Cargo.lock") != registry_packages(candidate / "Cargo.lock"):
        raise SystemExit("baseline and candidate registry dependencies differ")
    engines = {
        name: build_engine(name, source, build, worker, worker_sha, args.lto, previous.get(name))
        for name, source in (("baseline", baseline), ("candidate", candidate))
    }
    record = {
        "schema": 2, "rustc": subprocess.check_output(["rustc", "+1.95", "-vV"], text=True),
        "baseline_revision_expected": BASELINE_REVISION,
        "baseline_verification": baseline_verification,
        "worker_sha256": worker_sha, "worker_source": str(WORKER), "lto": args.lto,
        "frozen_lock_sha256": sha256(baseline / "Cargo.lock"),
        "engines": engines,
    }
    metadata.write_text(json.dumps(record, indent=2) + "\n")
    print(metadata)


if __name__ == "__main__":
    main()
