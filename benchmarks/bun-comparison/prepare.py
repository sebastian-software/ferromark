#!/usr/bin/env python3
"""Build a pinned, native Bun Markdown comparison without changing parser sources.

Requires Python 3.11+, macOS, clang++, git, and the pinned Rust toolchain.
Only use a dedicated Bun checkout: its workspace manifest/lockfile are regenerated.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tarfile
import tomllib
import urllib.request

BUN_REV = "76e9dcc6ad272a4fb1ee4a4dbbde4809201b71d1"
MI_REV = "6a64e1ba7f5b2130d4efccb67ec87fd0003f0f6a"
HWY_REV = "2607d3b5b0113992fe84d3848859eae13b3b52c1"
MD4C_REV = "65c6c9d72cebd9a731aaa5597414ce04d9ea5de3"
TOOLCHAIN = "nightly-2026-07-20"
HERE = Path(__file__).resolve().parent
REPO = HERE.parent.parent


def run(args, **kwargs):
    return subprocess.run(list(map(str, args)), check=True, **kwargs)


def git(bun, *args):
    return subprocess.check_output(["git", "-C", str(bun), *args], text=True).strip()


def source_archive(repo, rev, dest):
    archive = dest.with_suffix(".tar.gz")
    if not archive.exists():
        urllib.request.urlretrieve(f"https://codeload.github.com/{repo}/tar.gz/{rev}", archive)
    # Always re-extract into a fresh folder: no stale or locally edited native sources.
    if dest.exists():
        shutil.rmtree(dest)
    dest.mkdir()
    with tarfile.open(archive) as tf:
        for member in tf.getmembers():
            parts = Path(member.name).parts[1:]
            if not parts:
                continue
            member.name = str(Path(*parts))
            tf.extract(member, dest, filter="data")
    return hashlib.sha256(archive.read_bytes()).hexdigest()


def prepare(bun, work, md4c, lockfile=None):
    if sys.platform != "darwin":
        raise SystemExit("The standalone stack adapter currently supports macOS only.")
    if git(bun, "rev-parse", "HEAD") != BUN_REV:
        raise SystemExit(f"Bun checkout must be at {BUN_REV}")
    if git(bun, "diff", "HEAD", "--", "src", "scripts/build"):
        raise SystemExit("Bun parser/dependency/build sources must be unchanged")
    if git(md4c, "rev-parse", "HEAD") != MD4C_REV or git(md4c, "status", "--porcelain", "--", "src"):
        raise SystemExit("md4c must be a clean checkout at the pinned revision")
    work.mkdir(parents=True, exist_ok=True)
    native = work / "native"
    native.mkdir(exist_ok=True)
    archives = {}
    for name, repo, rev in [("mimalloc", "oven-sh/mimalloc", MI_REV), ("highway", "google/highway", HWY_REV)]:
        archives[name] = source_archive(repo, rev, native / name)
    mi, hwy = native / "mimalloc", native / "highway"
    run(["clang", "-O3", "-DNDEBUG", "-DMI_STATIC_LIB", "-DMI_SKIP_COLLECT_ON_EXIT=1",
         "-DMI_NO_PROCESS_DETACH=1", "-DMI_BUILD_RELEASE", "-DMI_CMAKE_BUILD_TYPE=release",
         "-fvisibility=hidden", "-ftls-model=initial-exec", f"-I{mi}/include", "-c", mi / "src/static.c", "-o", native / "mimalloc.o"])
    run(["ar", "rcs", native / "libmimalloc.a", native / "mimalloc.o"])
    md_objects = []
    for name in ("md4c", "md4c-html", "entity"):
        obj = native / f"{name}.o"
        run(["clang", "-O3", "-DNDEBUG", "-std=c99", f"-I{mi}/include",
             "-include", HERE / "md4c_alloc.h", "-c", md4c / f"src/{name}.c", "-o", obj])
        md_objects.append(obj)
    run(["ar", "rcs", native / "libmd4c.a", *md_objects])
    flags = ["clang++", "-std=c++23", "-O3", "-DNDEBUG", "-DHWY_STATIC_DEFINE",
             "-DHWY_DISABLED_TARGETS=HWY_ALL_SVE-HWY_SVE2_128", "-fno-exceptions", "-fmath-errno", f"-I{hwy}"]
    objs = []
    for name in ["abort", "aligned_allocator", "nanobenchmark", "per_target", "perf_counters", "print", "profiler", "targets", "timer"]:
        obj = native / f"{name}.o"
        run([*flags, "-c", hwy / f"hwy/{name}.cc", "-o", obj])
        objs.append(obj)
    bindings = bun / "src/jsc/bindings"
    run([*flags, "-include", HERE / "native.h", f"-I{bindings}", "-c", bindings / "highway_strings.cpp", "-o", native / "strings.o"])
    run(["clang", "-O3", "-c", HERE / "stack.c", "-o", native / "stack.o"])
    run(["ar", "rcs", native / "libbun_bench_native.a", *objs, native / "strings.o", native / "stack.o"])

    original = git(bun, "show", "HEAD:Cargo.toml") + "\n"
    deps = tomllib.loads(original)["workspace"]["dependencies"]
    seen = set()

    def visit(name):
        if name in seen:
            return
        seen.add(name)
        data = tomllib.loads((bun / deps[name]["path"] / "Cargo.toml").read_text())
        groups = [data.get("dependencies", {}), data.get("build-dependencies", {})]
        groups += [t.get("dependencies", {}) for t in data.get("target", {}).values()]
        for group in groups:
            for key, value in group.items():
                if isinstance(value, dict) and value.get("workspace") and "path" in deps.get(key, {}):
                    visit(key)

    visit("bun_md")
    members = sorted(deps[k]["path"] for k in seen) + ["ferromark-comparison"]
    start = original.index("members = [")
    end = original.index("\n]", start) + 2
    (bun / "Cargo.toml").write_text(original[:start] + "members = " + json.dumps(members) + original[end:])
    driver = bun / "ferromark-comparison"
    driver.mkdir(exist_ok=True)
    (driver / "Cargo.toml").write_text(f'''[package]
name = "ferromark-bun-comparison"
version = "0.0.0"
edition = "2024"
publish = false
[[bin]]
name = "ferromark-bun-comparison"
path = "driver.rs"
[dependencies]
bun_md.workspace = true
bun_alloc.workspace = true
bun_core.workspace = true
ferromark = {{ path = {json.dumps(str(REPO))} }}
pulldown-cmark = "=0.13.4"
comrak = {{ version = "=0.54.0", default-features = false }}
serde_json = "1"
# Match Ferromark's checked-in decoder version. A newly resolved 0.2.15
# changes numeric NUL entity behavior and is not the version under comparison.
html-escape = "=0.2.14"
''')
    shutil.copyfile(HERE / "driver.rs", driver / "driver.rs")
    (driver / "build.rs").write_text(f'''fn main() {{
    println!("cargo:rustc-link-search=native={{}}", {json.dumps(str(native))});
    println!("cargo:rustc-link-lib=static=bun_bench_native");
    println!("cargo:rustc-link-lib=static=md4c");
    println!("cargo:rustc-link-lib=static=mimalloc");
    println!("cargo:rustc-link-lib=c++");
    println!("cargo:rustc-env=FERROMARK_SOURCE={{}}", {json.dumps(str(REPO))});
}}
''')
    codegen = work / "codegen"
    codegen.mkdir(exist_ok=True)
    # Configure-only metadata, matching the fields of Bun's buildOptionsRs.ts.
    values = {"SHA": f'&str = "{BUN_REV}"', "REPORTED_NODEJS_VERSION": '&str = "24.0.0"',
              "RELEASE_SAFE": "bool = false", "IS_CANARY": "bool = true",
              "CANARY_REVISION": '&str = "benchmark"', "ENABLE_FUZZILLI": "bool = false",
              "FALLBACK_HTML_VERSION": '&str = "0000000000000000"',
              "VERSION": "crate::Version = crate::Version { major: 1, minor: 4, patch: 3 }",
              "BASE_PATH": f'&[u8] = {json.dumps(str(bun))}.as_bytes()',
              "CODEGEN_PATH": f'&[u8] = {json.dumps(str(codegen))}.as_bytes()',
              "ENABLE_LOGS": "bool = cfg!(bun_debug)", "ENABLE_ASAN": "bool = cfg!(bun_asan)",
              "ENABLE_TINYCC": 'bool = !cfg!(any(target_os = "android", target_os = "freebsd"))'}
    (codegen / "build_options.rs").write_text("\n".join(f"pub const {k}: {v};" for k, v in values.items()) + "\n")
    env = os.environ.copy()
    env["BUN_CODEGEN_DIR"] = str(codegen)
    env["RUSTFLAGS"] = "-C target-cpu=generic"
    (work / "archives.json").write_text(json.dumps(archives, indent=2) + "\n")
    if lockfile:
        shutil.copyfile(lockfile, bun / "Cargo.lock")
    run(["cargo", f"+{TOOLCHAIN}", "build", "--release", "-p", "ferromark-bun-comparison",
         *(["--locked"] if lockfile else [])], cwd=bun, env=env)
    binary = bun / "target/release/ferromark-bun-comparison"
    stamp = {
        "binary_sha256": hashlib.sha256(binary.read_bytes()).hexdigest(),
        "ferromark_source_sha256": {str(f.relative_to(REPO)): hashlib.sha256(f.read_bytes()).hexdigest()
                                   for f in sorted((REPO / "src").rglob("*.rs"))},
        "archives": archives,
        "md4c_revision": MD4C_REV,
        "md4c_source_sha256": {f.name: hashlib.sha256(f.read_bytes()).hexdigest()
                               for f in sorted((md4c / "src").iterdir()) if f.suffix in (".c", ".h")},
        "adapter_sha256": {f.name: hashlib.sha256(f.read_bytes()).hexdigest()
                           for f in sorted(HERE.iterdir()) if f.suffix in (".rs", ".c", ".h")},
    }
    (driver / "build-info.json").write_text(json.dumps(stamp, indent=2) + "\n")
    return binary


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("bun", type=Path)
    parser.add_argument("work", type=Path)
    parser.add_argument("--md4c", type=Path, required=True, help="Clean checkout at MD4C_REV")
    parser.add_argument("--lockfile", type=Path, help="Replay a previously recorded Cargo.lock with --locked")
    args = parser.parse_args()
    print(prepare(args.bun.resolve(), args.work.resolve(), args.md4c.resolve(), args.lockfile))
