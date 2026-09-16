#!/usr/bin/env python3
"""Build isolated workers while retaining each engine's locked dependencies."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import tomllib

MAIN_REVISION = "a6e9906f7b4a01355d336f209fd12534796419df"
FORK_REVISION = "d1481ca7687e94d30e56f473d3044a175dc6c9a6"


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def crate_source(source: Path) -> Path:
    """Return the `ferromark` package directory inside a checkout.

    The crate is the repository root package since the release-blueprint move;
    checkouts of older revisions keep it under `crates/ferromark`.
    """
    nested = source / "crates" / "ferromark"
    return nested if (nested / "Cargo.toml").is_file() else source


def registry_packages(path):
    return {
        (p["name"], p["version"]): p.get("checksum")
        for p in tomllib.loads(path.read_text())["package"] if "source" in p
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("main_source", type=Path)
    parser.add_argument("fork_source", type=Path)
    parser.add_argument("build_dir", type=Path)
    args = parser.parse_args()
    build = args.build_dir.resolve()
    build.mkdir(parents=True, exist_ok=False)
    env = os.environ.copy()
    env.pop("CARGO_ENCODED_RUSTFLAGS", None)
    env["RUSTFLAGS"] = "-C target-cpu=generic"
    record = {
        "rustc": subprocess.check_output(["rustc", "+1.95", "-vV"], text=True),
        "rustflags": env["RUSTFLAGS"],
        "worker_sha256": sha(Path(__file__).with_name("worker.rs")),
        "engines": {},
    }
    for name, source, revision in [
        ("main", args.main_source.resolve(), MAIN_REVISION),
        ("v2", args.fork_source.resolve(), FORK_REVISION),
    ]:
        root = build / name
        (root / "src").mkdir(parents=True)
        shutil.copyfile(Path(__file__).with_name("worker.rs"), root / "src/main.rs")
        shutil.copyfile(source / "Cargo.lock", root / "Cargo.lock")
        dependency = crate_source(source) if name == "v2" else source
        (root / "Cargo.toml").write_text(
            '[package]\nname = "comparison-worker"\nversion = "0.0.0"\n'
            'edition = "2024"\npublish = false\n\n[workspace]\n\n'
            '[features]\nv2 = []\n\n[dependencies]\n'
            f'ferromark = {{ path = {json.dumps(str(dependency))} }}\n\n'
            '[profile.release]\nopt-level = 3\nlto = "fat"\n'
            'codegen-units = 1\npanic = "abort"\nstrip = true\n'
        )
        env["CARGO_TARGET_DIR"] = str(root / "target")
        command = ["cargo", "+1.95", "build", "--release", "--offline"]
        if name == "v2":
            command += ["--features", "v2"]
        with (root / "build.log").open("w") as log:
            result = subprocess.run(command, cwd=root, env=env, stdout=log, stderr=log)
        if result.returncode:
            raise SystemExit((root / "build.log").read_text())
        original = registry_packages(source / "Cargo.lock")
        retained = registry_packages(root / "Cargo.lock")
        assert all(original.get(key) == value for key, value in retained.items()), (
            "dependency versions changed", name
        )
        binary = root / "target/release/comparison-worker"
        record["engines"][name] = {
            "revision": revision,
            "source": str(source),
            "source_manifest_sha256": sha(source / "Cargo.toml"),
            "source_lock_sha256": sha(source / "Cargo.lock"),
            "binary": str(binary),
            "binary_sha256": sha(binary),
            "lock_sha256": sha(root / "Cargo.lock"),
            "retained_registry_packages": len(retained),
            "registry_versions_unchanged": True,
            "command": command,
        }
    (build / "build.json").write_text(json.dumps(record, indent=2) + "\n")
    print(build / "build.json")


if __name__ == "__main__":
    main()
