#!/usr/bin/env python3
"""Build one pinned engine and an independent native Ferromark baseline."""
import argparse
import importlib.util
import json
import os
from pathlib import Path
import shutil
import subprocess
import tomllib

from common import HERE, REPO, capture, local_hashes, sha, source_hashes, write_json


class Build:
    def __init__(self, work, source, adapter):
        self.work, self.source, self.adapter = work, source, adapter
        self.commands, self.locks, self.details = [], {}, {}
        self.env = os.environ.copy()
        for key in list(self.env):
            if key.startswith(("CARGO_PROFILE_", "DOTNET_", "COMPlus_")) or key in (
                    "CARGO_ENCODED_RUSTFLAGS", "RUSTC_WRAPPER", "RUSTC_WORKSPACE_WRAPPER", "LD_PRELOAD", "DYLD_INSERT_LIBRARIES"):
                self.env.pop(key)
        self.env.update(RUSTFLAGS="-C target-cpu=generic")

    def run(self, command, cwd=REPO):
        self.commands.append({"argv": list(map(str, command)), "cwd": str(cwd)})
        subprocess.run(list(map(str, command)), cwd=cwd, env=self.env, check=True)

    def rust(self, manifest, binary):
        # Exact locks are committed, not regenerated during a measured build.
        args = ["--release", "--locked", "--manifest-path", manifest, "--target-dir", self.work / "target"]
        self.run(["cargo", "fmt", "--manifest-path", manifest, "--check"])
        self.run(["cargo", "test", *args])
        self.run(["cargo", "build", *args])
        self.run(["cargo", "clippy", *args, "--all-targets", "--", "-D", "warnings"])
        self.locks[str(manifest.relative_to(HERE).with_name("Cargo.lock"))] = manifest.with_name("Cargo.lock").read_text()
        self.details[binary + "_dependencies"] = capture(["cargo", "tree", "--locked", "--manifest-path", manifest], env=self.env)
        output = self.work / binary
        shutil.copy2(self.work / "target/release" / binary, output)
        return [str(output)]


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("engine", choices=sorted(d.parent.name for d in (HERE / "engines").glob("*/engine.json")))
    p.add_argument("work", type=Path, help="New build directory")
    p.add_argument("--source", type=Path, required=True, help="Clean checkout of the exact pinned upstream revision")
    args = p.parse_args()
    folder = HERE / "engines" / args.engine
    adapter = json.loads((folder / "engine.json").read_text())
    source = args.source.resolve()
    upstream = source_hashes(source, adapter["revision"])
    work = args.work.resolve()
    work.mkdir(parents=True, exist_ok=False)
    ctx = Build(work, source, adapter)
    baseline = HERE / "engines/ferromark/Cargo.toml"
    root_lock = tomllib.loads((REPO / "Cargo.lock").read_text())["package"]
    native_lock = tomllib.loads(baseline.with_name("Cargo.lock").read_text())["package"]
    for name in tomllib.loads((REPO / "Cargo.toml").read_text())["dependencies"]:
        versions = lambda rows: {(r["version"], r.get("checksum")) for r in rows if r["name"] == name}
        if versions(root_lock) != versions(native_lock):
            raise ValueError(f"Ferromark dependency differs from root lock: {name}")
    workers = {"ferromark": ctx.rust(baseline, "ferromark-driver")}
    spec = importlib.util.spec_from_file_location("engine_build", folder / "build.py")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    workers[args.engine] = module.build(ctx)
    if upstream != source_hashes(source, adapter["revision"]):
        raise ValueError("Upstream source changed during the build")
    binaries = {str(f.relative_to(work)): sha(f) for f in work.rglob("*")
                if f.is_file() and not {"target", "obj", ".zig-cache"}.intersection(f.relative_to(work).parts)}
    info = {"engines": [args.engine], "adapter": adapter, "workers": workers,
        "worker_environment": adapter.get("worker_environment", {}), "binaries": binaries,
        "dependency_locks": ctx.locks, "commands": ctx.commands, "details": ctx.details,
        "rustc": capture(["rustc", "-vV"]), "cargo": capture(["cargo", "-V"]),
        "build_environment": {"RUSTFLAGS": ctx.env["RUSTFLAGS"]},
        "upstream_source_sha256": upstream, "local_sha256": local_hashes()}
    write_json(work / "build-info.json", info)
    print(work)


if __name__ == "__main__":
    main()
