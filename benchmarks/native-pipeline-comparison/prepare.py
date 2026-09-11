#!/usr/bin/env python3
"""Build pinned native workers, run adapter checks, and record executable provenance."""
import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
import tomllib

from common import HERE, REPO, GO_VERSION, GO_MODULES, SATTERI, capture, local_hashes, sha, source_hashes, write_json


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("work", type=Path, help="New build directory")
    p.add_argument("--satteri", type=Path, required=True, help="Clean checkout of the pinned upstream release")
    args = p.parse_args()
    work = args.work.resolve()
    upstream = source_hashes(args.satteri.resolve(), SATTERI)
    version = capture(["go", "version"])
    if version.split()[2] != GO_VERSION:
        raise ValueError(f"Expected {GO_VERSION}, got {version}")
    lock = tomllib.loads((HERE / "Cargo.lock").read_text())["package"]
    root = tomllib.loads((REPO / "Cargo.lock").read_text())["package"]
    direct = tomllib.loads((REPO / "Cargo.toml").read_text())["dependencies"]
    for name in direct:
        wanted = [r for r in root if r["name"] == name]
        actual = [r for r in lock if r["name"] == name]
        if {(r["version"], r.get("checksum")) for r in wanted} != {(r["version"], r.get("checksum")) for r in actual}:
            raise ValueError(f"Ferromark dependency differs from root lock: {name}")
    work.mkdir(parents=True, exist_ok=False)
    env = os.environ.copy()
    for key in list(env):
        if key.startswith("CARGO_PROFILE_") or key in ("CARGO_ENCODED_RUSTFLAGS", "RUSTC_WRAPPER", "RUSTC_WORKSPACE_WRAPPER"):
            env.pop(key)
    env.update(RUSTFLAGS="-C target-cpu=generic", GOENV="off", GOTOOLCHAIN="local", GOWORK="off", GOFLAGS="", CGO_ENABLED="0",
               GOEXPERIMENT="", GOARM64="v8.0", GOAMD64="v1")
    commands = []
    def run(command, cwd=REPO):
        commands.append({"argv": list(map(str, command)), "cwd": str(cwd)})
        subprocess.run(list(map(str, command)), cwd=cwd, env=env, check=True)
    rust = ["--release", "--locked", "--manifest-path", HERE / "Cargo.toml", "--target-dir", work / "target"]
    run(["cargo", "test", *rust])
    run(["cargo", "build", *rust])
    run(["cargo", "clippy", *rust, "--all-targets", "--", "-D", "warnings"])
    run(["go", "mod", "verify"], HERE / "goldmark")
    run(["go", "test", "-mod=readonly", "./..."], HERE / "goldmark")
    run(["go", "vet", "-mod=readonly", "./..."], HERE / "goldmark")
    run(["go", "build", "-mod=readonly", "-pgo=off", "-trimpath", "-o", work / "goldmark-driver", "."], HERE / "goldmark")
    shutil.copyfile(work / "target/release/ferromark-native-pipeline-comparison", work / "rust-driver")
    (work / "rust-driver").chmod(0o755)
    modules = capture(["go", "list", "-mod=readonly", "-m", "all"], cwd=HERE / "goldmark", env=env)
    actual = dict(line.split() for line in modules.splitlines()[1:])
    if actual != GO_MODULES:
        raise ValueError(f"Unexpected Go dependencies: {actual}")
    metadata = {"satteri_revision": SATTERI, "satteri_source_sha256": upstream,
        "rustc": capture(["rustc", "-vV"]), "cargo": capture(["cargo", "-V"]),
        "go": version, "go_modules": actual, "go_binary_info": capture(["go", "version", "-m", work / "goldmark-driver"]),
        "go_environment": json.loads(capture(["go", "env", "-json", "GOARCH", "GOOS", "GOARM64", "GOAMD64", "GOEXPERIMENT"], env=env)),
        "rust_dependency_tree": capture(["cargo", "tree", "--locked", "--manifest-path", HERE / "Cargo.toml"]),
        "build_environment": {k: env[k] for k in ("RUSTFLAGS", "GOENV", "GOTOOLCHAIN", "GOWORK", "GOFLAGS", "CGO_ENABLED")},
        "profile": "Rust opt-level=3, fat LTO, 1 CGU, panic=abort, system allocator; Go default optimized build, no PGO profile",
        "commands": commands, "binaries": {n: sha(work / n) for n in ("rust-driver", "goldmark-driver")},
        "local_sha256": local_hashes()}
    write_json(work / "build-info.json", metadata)
    print(work)


if __name__ == "__main__":
    main()
