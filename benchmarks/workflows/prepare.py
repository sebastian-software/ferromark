#!/usr/bin/env python3
"""Build separate uninstrumented timing and instrumented memory workers."""
import argparse
import os
from pathlib import Path
import shutil
import subprocess
import tomllib

from support import HERE, ROOT, command, observation, sha, source_hashes, write_json


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("build", type=Path)
    args = parser.parse_args()
    build = args.build.resolve()
    build.mkdir(parents=True, exist_ok=False)
    root_lock = tomllib.loads((ROOT / "Cargo.lock").read_text())
    bench_lock = tomllib.loads((HERE / "Cargo.lock").read_text())
    for name in ("memchr", "smallvec", "html-escape", "rustc-hash", "unicode-ident"):
        versions = lambda lock: sorted((p["version"], p.get("checksum")) for p in lock["package"] if p["name"] == name)
        if versions(root_lock) != versions(bench_lock):
            raise ValueError(f"Ferromark dependency differs from root lock: {name}")
    env = os.environ.copy()
    env.pop("CARGO_ENCODED_RUSTFLAGS", None)
    env["RUSTFLAGS"] = "-C target-cpu=generic"
    before = source_hashes()
    commands = []
    for kind in ("timing", "memory"):
        cmd = ["cargo", "build", "--release", "--locked", "--manifest-path", str(HERE / "Cargo.toml"),
               "--target-dir", str(build / kind)]
        if kind == "memory":
            cmd += ["--features", "heap"]
        commands.append(cmd)
        with (build / f"{kind}-build.log").open("w") as log:
            subprocess.run(cmd, cwd=ROOT, env=env, check=True, stdout=log, stderr=subprocess.STDOUT)
        shutil.copyfile(build / kind / "release/ferromark-workflows", build / kind / "worker")
        (build / kind / "worker").chmod(0o755)
    subprocess.run([str(build / "memory/worker"), "--self-test"], check=True)
    if before != source_hashes():
        raise ValueError("Source changed during build")
    metadata = {"schema": 1, "source_revision": command(["git", "rev-parse", "HEAD"]),
                "source_status": command(["git", "status", "--porcelain"]),
                "source_sha256": before, "rustc": command(["rustc", "-Vv"]),
                "cargo": command(["cargo", "-V"]), "cpu": command(["sysctl", "-n", "machdep.cpu.brand_string"]),
                "ram_bytes": int(command(["sysctl", "-n", "hw.memsize"])),
                "logical_cpus": int(command(["sysctl", "-n", "hw.logicalcpu"])),
                "os": command(["sw_vers"]), "architecture": command(["uname", "-m"]),
                "rustflags": env["RUSTFLAGS"], "allocator": "Rust System (macOS system allocator)",
                "build_commands": commands, "environment": observation(),
                "binary_sha256": {kind: sha(build / kind / "worker") for kind in ("timing", "memory")}}
    write_json(build / "build.json", metadata)
    print(f"Built timing and memory workers in {build}")


if __name__ == "__main__":
    main()
