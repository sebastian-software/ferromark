#!/usr/bin/env python3
"""Build pinned cmark and cmark-gfm CLIs in an isolated temporary tree."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import sys


CMARK_REVISION = "bb3678d7a73cb02d35c8876ecd097072636200a8"
CMARK_GFM_REVISION = "587a12bb54d95ac37241377e6ddc93ea0e45439b"


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def git(source: Path, *args: str) -> str:
    return subprocess.check_output(["git", "-C", str(source), *args], text=True).strip()


def verify_source(source: Path, expected: str) -> dict[str, object]:
    actual = git(source, "rev-parse", "HEAD")
    if actual != expected:
        raise SystemExit(f"{source}: expected {expected}, found {actual}")
    status = git(source, "status", "--porcelain")
    if status:
        raise SystemExit(f"{source}: source tree is dirty")
    return {"path": str(source), "revision": actual, "clean": True}


def run(command: list[str], log: Path, cwd: Path) -> None:
    result = subprocess.run(command, cwd=cwd, text=True, capture_output=True, check=False)
    log.write_text(result.stdout + result.stderr, encoding="utf-8")
    if result.returncode:
        raise SystemExit(f"command failed ({result.returncode}); see {log}")


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--cmark-source", type=Path, required=True)
    parser.add_argument("--cmark-gfm-source", type=Path, required=True)
    parser.add_argument("--build-root", type=Path, required=True)
    parser.add_argument("--metadata", type=Path, required=True)
    parser.add_argument("--cmake", type=Path, help="CMake executable, otherwise resolved from PATH")
    args = parser.parse_args(argv)

    cmake = args.cmake or shutil.which("cmake")
    if not cmake:
        raise SystemExit("cmake not found; supply --cmake or add an existing CMake to PATH")
    cmake_path = Path(cmake).resolve()
    if not cmake_path.exists():
        raise SystemExit(f"cmake does not exist: {cmake_path}")
    for name in ('cmark_source', 'cmark_gfm_source', 'build_root', 'metadata'):
        setattr(args, name, getattr(args, name).resolve())
    args.build_root.mkdir(parents=True, exist_ok=False)
    logs = args.build_root / "logs"
    logs.mkdir()
    cmark_build = args.build_root / "cmark"
    gfm_build = args.build_root / "cmark-gfm"
    cmark = verify_source(args.cmark_source, CMARK_REVISION)
    gfm = verify_source(args.cmark_gfm_source, CMARK_GFM_REVISION)
    configure = [str(cmake_path), "-S"]
    run(configure + [str(args.cmark_source), "-B", str(cmark_build), "-DCMARK_TESTS=OFF", "-DCMARK_SHARED=OFF"], logs / "cmark-configure.log", args.build_root)
    run(configure + [str(args.cmark_gfm_source), "-B", str(gfm_build), "-DCMARK_TESTS=OFF", "-DCMARK_SHARED=OFF"], logs / "cmark-gfm-configure.log", args.build_root)
    run([str(cmake_path), "--build", str(cmark_build), "--target", "cmark_exe", "--parallel", "2"], logs / "cmark-build.log", args.build_root)
    run([str(cmake_path), "--build", str(gfm_build), "--target", "cmark-gfm", "--parallel", "2"], logs / "cmark-gfm-build.log", args.build_root)
    binaries = {
        "cmark": cmark_build / "src/cmark",
        "cmark-gfm": gfm_build / "src/cmark-gfm",
    }
    for path in binaries.values():
        if not path.is_file():
            raise SystemExit(f"expected CLI was not built: {path}")
    compilers = {}
    for name, folder in (("cmark", cmark_build), ("cmark-gfm", gfm_build)):
        cache = (folder / 'CMakeCache.txt').read_text()
        compiler = next(line.split('=', 1)[1] for line in cache.splitlines()
            if line.startswith('CMAKE_C_COMPILER:FILEPATH='))
        compiler_version = subprocess.check_output([compiler, '--version'], text=True).strip()
        compilers[name] = {"path": compiler, "version": compiler_version,
            "cmake_cache_sha256": sha256(folder / 'CMakeCache.txt')}
    metadata = {
        "sources": {"cmark": cmark, "cmark-gfm": gfm},
        "binaries": {name: {"path": str(path), "sha256": sha256(path)} for name, path in binaries.items()},
        "commands": {
            "cmake": str(cmake_path),
            "cmark_configure": configure + [str(args.cmark_source), "-B", str(cmark_build), "-DCMARK_TESTS=OFF", "-DCMARK_SHARED=OFF"],
            "cmark_gfm_configure": configure + [str(args.cmark_gfm_source), "-B", str(gfm_build), "-DCMARK_TESTS=OFF", "-DCMARK_SHARED=OFF"],
            "cmark_build": [str(cmake_path), "--build", str(cmark_build), "--target", "cmark_exe", "--parallel", "2"],
            "cmark_gfm_build": [str(cmake_path), "--build", str(gfm_build), "--target", "cmark-gfm", "--parallel", "2"],
        },
        "compilers": compilers,
        "builder_sha256": sha256(Path(__file__)),
        "logs": {name: str(path) for name, path in {
            "cmark_configure": logs / "cmark-configure.log",
            "cmark_gfm_configure": logs / "cmark-gfm-configure.log",
            "cmark_build": logs / "cmark-build.log",
            "cmark_gfm_build": logs / "cmark-gfm-build.log",
        }.items()},
    }
    args.metadata.parent.mkdir(parents=True, exist_ok=True)
    args.metadata.write_text(json.dumps(metadata, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(metadata, indent=2))
    return 0


if __name__ == "__main__":
    sys.exit(main())
