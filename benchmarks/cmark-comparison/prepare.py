#!/usr/bin/env python3
"""Build and test two separately linked native cmark comparison executables."""
import argparse
import os
from pathlib import Path
import shutil
import subprocess
import tomllib

from support import HERE, REPO, SOURCES, capture, local_hashes, sha, source_hashes, write_json


def run(args, **kwargs):
    subprocess.run(list(map(str, args)), check=True, **kwargs)


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("work", type=Path, help="New build directory, outside either upstream checkout")
    p.add_argument("--cmark", type=Path, required=True)
    p.add_argument("--cmark-gfm", type=Path, required=True)
    p.add_argument("--cmake", default="cmake")
    args = p.parse_args()
    root_lock = tomllib.loads((REPO / "Cargo.lock").read_text())
    bench_lock = tomllib.loads((HERE / "Cargo.lock").read_text())
    dependencies = tomllib.loads((REPO / "Cargo.toml").read_text())["dependencies"]
    for name in dependencies:
        def versions(lock):
            return {(entry["version"], entry.get("checksum")) for entry in lock["package"] if entry["name"] == name}
        if versions(root_lock) != versions(bench_lock):
            p.error(f"Benchmark lockfile must match root dependency {name}; update the benchmark lockfile first")
    work = args.work.resolve()
    sources = {"cmark": args.cmark.resolve(), "cmark-gfm": args.cmark_gfm.resolve()}
    hashes = {name: source_hashes(path, SOURCES[name]["revision"]) for name, path in sources.items()}
    for source in sources.values():
        if work.is_relative_to(source):
            p.error("Build outside the upstream source checkouts")
    work.mkdir(parents=True, exist_ok=False)
    env = os.environ.copy()
    # Explicit codegen policy: no host-specific CPU tuning, PGO, or custom allocator.
    env["RUSTFLAGS"] = "-C target-cpu=generic"
    for key in list(env):
        if key.startswith(("CARGO_ENCODED_RUSTFLAGS", "CARGO_PROFILE_", "CARGO_TARGET_")) or key in ("CFLAGS", "CXXFLAGS", "LDFLAGS", "RUSTC_WRAPPER", "RUSTC_WORKSPACE_WRAPPER"):
            del env[key]
    metadata = {"sources": SOURCES, "source_sha256": hashes,
        "rustc": capture(["rustc", "-Vv"]), "cargo": capture(["cargo", "-V"]),
        "cmake": capture([args.cmake, "--version"]), "rustflags": env["RUSTFLAGS"],
        "allocation": "system allocator; fresh parser/AST and owned output, freed inside each timed call",
        "profile": "Rust opt-level=3, fat LTO, 1 CGU, panic=abort; C Release -O3 -DNDEBUG; no PGO",
        "build_commands": [], "binaries": {}}
    for name, source in sources.items():
        build = work / name
        gfm = name == "cmark-gfm"
        configure = [args.cmake, "-S", source, "-B", build, "-DCMAKE_BUILD_TYPE=Release",
            "-DCMAKE_C_FLAGS=", "-DCMAKE_C_FLAGS_RELEASE=-O3 -DNDEBUG", "-DCMAKE_EXPORT_COMPILE_COMMANDS=ON"]
        configure += (["-DCMARK_SHARED=OFF", "-DCMARK_STATIC=ON", "-DCMARK_TESTS=OFF"] if gfm
                      else ["-DBUILD_SHARED_LIBS=OFF", "-DBUILD_TESTING=OFF"])
        targets = ["libcmark-gfm_static", "libcmark-gfm-extensions_static"] if gfm else ["cmark"]
        compile_c = [args.cmake, "--build", build, "--config", "Release", "--parallel", "2", "--target", *targets]
        for command in (configure, compile_c):
            metadata["build_commands"].append(list(map(str, command)))
            run(command, env=env)
        env["CMARK_SOURCE"], env["CMARK_BUILD"] = str(source), str(build)
        options = ["--manifest-path", HERE / "Cargo.toml", "--target-dir", work / f"cargo-{name}", "--release"]
        if gfm:
            options += ["--features", "cmark-gfm"]
        for action in ("test", "build", "clippy"):
            command = ["cargo", action, "--locked", *options]
            if action == "clippy":
                command += ["--all-targets", "--", "-D", "warnings"]
            metadata["build_commands"].append(list(map(str, command)))
            run(command, env=env)
        binary = work / f"compare-{name}"
        shutil.copyfile(work / f"cargo-{name}/release/ferromark-cmark-comparison", binary)
        binary.chmod(0o755)
        metadata["binaries"][name] = sha(binary)
        for filename in ("CMakeCache.txt", "compile_commands.json"):
            shutil.copyfile(build / filename, work / f"{name}-{filename}")
        if source_hashes(source, SOURCES[name]["revision"]) != hashes[name]:
            raise SystemExit("Upstream sources changed during build")
    metadata["local_sha256"] = local_hashes()
    shutil.copyfile(HERE / "Cargo.lock", work / "Cargo.lock")
    write_json(work / "build-info.json", metadata)
    print(work)


if __name__ == "__main__":
    main()
