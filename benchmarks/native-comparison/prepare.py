#!/usr/bin/env python3
"""Prepare the pinned six-engine native comparison workspace.

The script only reads the supplied source checkouts.  It materializes detached
git archives below the disposable build directory, copies the existing local
Bun native dependency caches, and creates a temporary Bun workspace member for
the worker supplied with ``--worker``.  It intentionally does not fetch or
modify any source checkout.

``--pgo`` adds a profile-guided optimization pass on top of the unchanged
release recipe.  It leaves the default build byte-identical: every PGO step is
skipped unless the flag is given.
"""

from __future__ import annotations

import argparse
import gzip
import hashlib
import io
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import tarfile
import tomllib


BUN_REVISION = "76e9dcc6ad272a4fb1ee4a4dbbde4809201b71d1"
BUN_TOOLCHAIN = "nightly-2026-07-20"
FERROMARK_V1_REVISION = "4e151415a15c67e9d3735f719b0bed3e25d8cff8"
FERROMARK_V2_REVISION = "e93394ee4c2d5e3022eaf3fd0c3b542bcd5eb97a"
OX_REVISION = "a71a58939ffe7f154117cea026f6d6e71a139393"
MD4C_REVISION = "65c6c9d72cebd9a731aaa5597414ce04d9ea5de3"
OX_ARCHIVE_SHA256 = "7df34e3e2db30678981f6df838eafe3e7950e966453221ae180acfa7938f1cfd"

MI_ARCHIVE_SHA256 = "f36343416ad823dfcca61bd18ad4bb7f0d8814ab77e0fd5ef169f4a9cddeb8b8"
HWY_ARCHIVE_SHA256 = "741d705781e0b3e406beda8f1f994fbae01321237ce8023a1ad90fbaf7940c25"

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[1]

BASE_RUSTFLAGS = "-C target-cpu=generic"
# The worker's own engine names, harness syntax profiles, and lifecycles.  The
# PGO training pass drives this exact matrix so no engine's Rust code is left
# without profile data in the shared executable.
ENGINES = ("v2", "ox-content", "v1", "md4c", "pulldown-cmark", "bun")
HARNESS_PROFILES = ("commonmark", "gfm")
LIFECYCLES = ("fresh", "reuse")
# Rust profile data reaches every Rust crate in the one shared executable, so
# the four pure-Rust engines are PGO-built.  md4c's engine is C and Bun's
# engine is a Rust/C++ mix; clang compiles those parts without PGO, which
# would need a separate -fprofile-generate recipe this harness does not add.
ENGINE_PROFILE_DATA = {
    "v2": "rust-pgo",
    "ox-content": "rust-pgo",
    "v1": "rust-pgo",
    "pulldown-cmark": "rust-pgo",
    "bun": "rust-pgo-partial: Rust crates only; the C++ Highway/support objects are clang -O3 without PGO",
    "md4c": "none: the C engine is clang -O3 without PGO; only its Rust FFI wrapper is in the Rust build",
}


def sha(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def run(command: list[object], *, cwd: Path | None = None, env: dict[str, str] | None = None) -> None:
    subprocess.run([str(part) for part in command], check=True, cwd=cwd, env=env)


def git(source: Path, *args: str) -> str:
    return subprocess.check_output(["git", "-C", str(source), *args], text=True).strip()


def archive_checkout(
    source: Path, revision: str, destination: Path, *, allow_worktree_fallback: bool = False
) -> dict[str, str]:
    """Export exactly ``revision`` without copying the checkout's .git dir."""
    try:
        actual = git(source, "rev-parse", revision)
    except subprocess.CalledProcessError as error:
        raise SystemExit(f"{source} does not contain pinned revision {revision}") from error
    destination.parent.mkdir(parents=True, exist_ok=True)
    if destination.exists():
        raise SystemExit(f"refusing to replace existing archive directory: {destination}")
    partial_clone = allow_worktree_fallback and subprocess.run(
        ["git", "-C", str(source), "config", "--get", "remote.origin.promisor"],
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    ).returncode == 0
    try:
        if partial_clone:
            raise subprocess.CalledProcessError(128, "git archive (partial clone)")
        raw = subprocess.check_output(["git", "-C", str(source), "archive", "--format=tar", revision])
    except subprocess.CalledProcessError:
        if not allow_worktree_fallback:
            raise
        # The pinned Bun checkout is a blob-filtered local clone. Its commit
        # and all parser files are present, but `git archive` may try to fetch
        # an unrelated omitted blob. Use the clean worktree as a local cache
        # fallback, restoring the two root manifests from the pinned commit.
        if git(source, "diff", revision, "--", "src", "scripts/build"):
            raise SystemExit(f"parser/build sources are modified in {source}")
        tracked = subprocess.check_output(
            ["git", "-C", str(source), "ls-tree", "-r", "-z", revision]
        )
        destination.mkdir()
        for entry in tracked.split(b"\0"):
            if not entry:
                continue
            metadata, raw_relative = entry.split(b"\t", 1)
            mode, kind, expected_blob = metadata.split()
            relative = os.fsdecode(raw_relative)
            if relative in {"Cargo.toml", "Cargo.lock"}:
                (destination / relative).write_bytes(subprocess.check_output(
                    ["git", "-C", str(source), "show", f"{revision}:{relative}"]
                ))
                continue
            relevant = relative.startswith(("src/", "scripts/build/")) or relative in {
                "rust-toolchain.toml", "LICENSE", "LICENSE.md", "UPSTREAM.md", ".cargo/config.toml",
            }
            if not relevant:
                continue
            if kind != b"blob":
                raise SystemExit(f"unexpected non-blob source entry: {relative}")
            source_file = source / relative
            data = os.fsencode(source_file.readlink()) if mode == b"120000" else source_file.read_bytes()
            actual_blob = hashlib.sha1(b"blob " + str(len(data)).encode() + b"\0" + data).hexdigest()
            if actual_blob != expected_blob.decode():
                raise SystemExit(f"tracked Bun file differs from pinned blob: {relative}")
            destination_file = destination / relative
            destination_file.parent.mkdir(parents=True, exist_ok=True)
            if mode == b"120000":
                destination_file.symlink_to(source_file.readlink())
            else:
                shutil.copy2(source_file, destination_file)
        return {"revision": revision, "materialized_sha256": sha_tree(destination), "source": str(source)}
    archive_sha = hashlib.sha256(raw).hexdigest()
    destination.mkdir()
    with tarfile.open(fileobj=io.BytesIO(raw), mode="r:") as archive:
        archive.extractall(destination, filter="data")
    return {"revision": revision, "archive_sha256": archive_sha, "source": str(source)}


def sha_tree(root: Path) -> str:
    digest = hashlib.sha256()
    for path in sorted(path for path in root.rglob("*") if path.is_file()):
        relative = path.relative_to(root).as_posix().encode()
        digest.update(len(relative).to_bytes(8, "big"))
        digest.update(relative)
        digest.update(sha(path).encode())
    return digest.hexdigest()


def copy_native_archives(cache: Path, native: Path) -> dict[str, str]:
    native.mkdir(parents=True, exist_ok=True)
    result: dict[str, str] = {}
    for name, expected in (("mimalloc", MI_ARCHIVE_SHA256), ("highway", HWY_ARCHIVE_SHA256)):
        source = cache / f"{name}.tar.gz"
        if not source.is_file():
            raise SystemExit(f"missing pinned local native archive: {source}")
        actual = sha(source)
        if actual != expected:
            raise SystemExit(f"{source} has sha256 {actual}, expected {expected}")
        destination = native / source.name
        shutil.copyfile(source, destination)
        unpacked = native / name
        unpacked.mkdir()
        with tarfile.open(destination) as archive:
            for member in archive.getmembers():
                parts = Path(member.name).parts
                if len(parts) <= 1:
                    continue
                member.name = str(Path(*parts[1:]))
                archive.extract(member, unpacked, filter="data")
        result[name] = actual
    return result


def extract_source_archive(archive_path: Path, revision: str, destination: Path) -> dict[str, str]:
    """Extract a frozen source tarball whose first component is a repo name."""
    if destination.exists():
        raise SystemExit(f"refusing to replace existing archive directory: {destination}")
    actual_sha = sha(archive_path)
    if actual_sha != OX_ARCHIVE_SHA256:
        raise SystemExit(f"{archive_path} has sha256 {actual_sha}, expected {OX_ARCHIVE_SHA256}")
    destination.mkdir(parents=True)
    with tarfile.open(archive_path) as archive:
        roots = {Path(member.name).parts[0] for member in archive.getmembers() if Path(member.name).parts}
        if roots != {"ubugeeei-prod-ox-content-a71a589"}:
            raise SystemExit(f"unexpected OX archive root(s): {sorted(roots)}")
        for member in archive.getmembers():
            parts = Path(member.name).parts
            if len(parts) <= 1:
                continue
            member.name = str(Path(*parts[1:]))
            archive.extract(member, destination, filter="data")
    return {
        "revision": revision,
        "archive_sha256": actual_sha,
        "source_archive": str(archive_path),
    }


def registry_packages(lock: Path) -> dict[tuple[str, str], str | None]:
    data = tomllib.loads(lock.read_text())
    return {
        (item["name"], item["version"]): item.get("checksum")
        for item in data.get("package", [])
        if "source" in item
    }


def validate_registry_union(locks: list[Path]) -> dict[tuple[str, str], str | None]:
    """Build the allowed registry package set without choosing new versions."""
    union: dict[tuple[str, str], str | None] = {}
    by_key: dict[tuple[str, str], set[str | None]] = {}
    for lock in locks:
        for (name, version), checksum in registry_packages(lock).items():
            by_key.setdefault((name, version), set()).add(checksum)
            union[(name, version)] = checksum
    conflicts = {
        key: sorted(checksums, key=str)
        for key, checksums in by_key.items()
        if len(checksums) > 1
    }
    if conflicts:
        rendered = ", ".join(f"{key}={values}" for key, values in sorted(conflicts.items()))
        raise SystemExit(f"input lockfiles disagree on registry packages: {rendered}")
    return union


def finalize_existing(build: Path, worker_source: Path) -> None:
    """Write metadata for a completed build without invoking Cargo or clang."""
    bun = build / "bun"
    binary = build / "target" / "release" / "native-comparison-worker"
    lockfile = bun / "Cargo.lock"
    if not binary.is_file() or not lockfile.is_file():
        raise SystemExit(f"completed worker or lockfile missing below {build}")
    built_worker = bun / "comparison-worker" / "worker.rs"
    if not built_worker.is_file() or built_worker.read_bytes() != worker_source.read_bytes():
        raise SystemExit("worker source differs from the source embedded in the completed build")
    sources = build / "sources"
    source_records = {}
    for name, revision in (
        ("bun", BUN_REVISION),
        ("ferromark_v1", FERROMARK_V1_REVISION),
        ("ferromark_v2", FERROMARK_V2_REVISION),
        ("ox_content", OX_REVISION),
        ("md4c", MD4C_REVISION),
    ):
        source_records[name] = {
            "revision": revision,
            "tree_sha256": sha_tree(sources / name),
        }
        source_lock = sources / name / "Cargo.lock"
        if source_lock.is_file():
            source_records[name]["lock_sha256"] = sha(source_lock)
    native_records = {}
    for path in sorted((build / "native").glob("lib*.a")):
        native_records[path.name] = sha(path)
    resolved_registry = registry_packages(lockfile)
    registry_differences = {}
    for name in ("bun", "ferromark_v1", "ferromark_v2", "ox_content"):
        source_lock = sources / name / "Cargo.lock"
        if not source_lock.is_file():
            continue
        original = registry_packages(source_lock)
        differences = {}
        for key in sorted(set(original) | set(resolved_registry)):
            if original.get(key) != resolved_registry.get(key):
                differences[f"{key[0]}@{key[1]}"] = {
                    "source": original.get(key),
                    "resolved": resolved_registry.get(key),
                }
        registry_differences[name] = differences
    metadata = {
        "toolchain": BUN_TOOLCHAIN,
        "rustc": subprocess.check_output(["rustc", f"+{BUN_TOOLCHAIN}", "-vV"], text=True),
        "rustflags": "-C target-cpu=generic",
        "optimization": {
            "opt_level": 3,
            "lto": "fat",
            "codegen_units": 1,
            "panic": "abort",
            "debug": "line-tables-only",
        },
        "sources": source_records,
        "native_archives": native_records,
        "binary": str(binary),
        "binary_sha256": sha(binary),
        "lockfile": str(lockfile),
        "lock_sha256": sha(lockfile),
        "adapter_sha256": {
            "worker.rs": sha(worker_source),
        },
        "native_adapter_sources_unverified": True,
        "registry_resolved": {
            f"{name}@{version}": checksum for (name, version), checksum in resolved_registry.items()
        },
        "registry_differences_vs_source_locks": registry_differences,
        "finalized_without_rebuild": True,
    }
    (build / "build.json").write_text(json.dumps(metadata, indent=2) + "\n")
    print(build / "build.json")


def rewrite_bun_members(cargo: Path, member: str) -> None:
    text = cargo.read_text()
    data = tomllib.loads(text)
    dependencies = data["workspace"]["dependencies"]
    seen: set[str] = set()
    paths: set[str] = set()

    def visit(name: str) -> None:
        if name in seen:
            return
        seen.add(name)
        entry = dependencies.get(name, {})
        path = entry.get("path") if isinstance(entry, dict) else None
        if path is None:
            return
        paths.add(name)
        manifest = tomllib.loads((cargo.parent / path / "Cargo.toml").read_text())
        groups = [manifest.get("dependencies", {}), manifest.get("build-dependencies", {})]
        groups.extend(target.get("dependencies", {}) for target in manifest.get("target", {}).values())
        for group in groups:
            for dependency, value in group.items():
                if isinstance(value, dict) and value.get("workspace") and dependency in dependencies:
                    visit(dependency)

    visit("bun_md")
    members = sorted({dependencies[name]["path"] for name in paths}) + [member]
    start = text.index("members = [")
    end = text.index("\n]", start) + 2
    cargo.write_text(text[:start] + "members = " + json.dumps(members) + text[end:])


def compile_native(bun: Path, md4c: Path, native: Path, env: dict[str, str]) -> None:
    mi = native / "mimalloc"
    hwy = native / "highway"
    run([
        "clang", "-O3", "-DNDEBUG", "-DMI_STATIC_LIB", "-DMI_SKIP_COLLECT_ON_EXIT=1",
        "-DMI_NO_PROCESS_DETACH=1", "-DMI_BUILD_RELEASE", "-DMI_CMAKE_BUILD_TYPE=release",
        "-fvisibility=hidden", "-ftls-model=initial-exec", f"-I{mi}/include", "-c",
        mi / "src/static.c", "-o", native / "mimalloc.o",
    ], env=env)
    run(["ar", "rcs", native / "libmimalloc.a", native / "mimalloc.o"], env=env)

    md_objects = []
    for name in ("md4c", "md4c-html", "entity"):
        obj = native / f"{name}.o"
        run([
            "clang", "-O3", "-DNDEBUG", "-std=c99", f"-I{mi}/include", "-include",
            HERE / "md4c_alloc.h", "-c", md4c / "src" / f"{name}.c", "-o", obj,
        ], env=env)
        md_objects.append(obj)
    run(["ar", "rcs", native / "libmd4c.a", *md_objects], env=env)

    flags = [
        "clang++", "-std=c++23", "-O3", "-DNDEBUG", "-DHWY_STATIC_DEFINE",
        "-DHWY_DISABLED_TARGETS=HWY_ALL_SVE-HWY_SVE2_128", "-fno-exceptions",
        "-fmath-errno", f"-I{hwy}",
    ]
    objects = []
    for name in ("abort", "aligned_allocator", "nanobenchmark", "per_target", "perf_counters", "print", "profiler", "targets", "timer"):
        obj = native / f"{name}.o"
        run([*flags, "-c", hwy / "hwy" / f"{name}.cc", "-o", obj], env=env)
        objects.append(obj)
    bindings = bun / "src/jsc/bindings"
    run([*flags, "-include", HERE / "native.h", f"-I{bindings}", "-c", bindings / "highway_strings.cpp", "-o", native / "strings.o"], env=env)
    run(["clang", "-O3", "-c", HERE / "stack.c", "-o", native / "stack.o"], env=env)
    run(["ar", "rcs", native / "libbun_bench_native.a", *objects, native / "strings.o", native / "stack.o"], env=env)


def read_corpus(path: Path) -> dict:
    raw = gzip.decompress(path.read_bytes()) if path.suffix == ".gz" else path.read_bytes()
    return json.loads(raw)


def read_filter(path: Path) -> str:
    """Read one case-selection regular expression from an explicit filter file."""
    pattern = path.read_text().strip()
    if not pattern:
        raise SystemExit(f"empty case filter file: {path}")
    try:
        re.compile(pattern)
    except re.error as error:
        raise SystemExit(f"{path} is not a valid case filter: {error}") from error
    return pattern


def select_cases(cases: list[dict], pattern: str) -> list[dict]:
    matcher = re.compile(pattern)
    return [case for case in cases if matcher.search(case["name"])]


def overlapping_cases(training: list[dict], measurement_pattern: str) -> list[str]:
    """Training documents the measurement filter would also select."""
    matcher = re.compile(measurement_pattern)
    return sorted(case["name"] for case in training if matcher.search(case["name"]))


def training_profiles(case: dict) -> tuple[str, ...]:
    """Harness syntax profiles one training document is rendered under.

    The six-engine worker implements ``commonmark`` and ``gfm`` only.  The
    authored diagnostics also carry corpus profiles that select renderer or
    scanner options this harness deliberately disables (``autolink``, ``mdx``,
    ``extensions``, ``opt-*``).  Their input is still ordinary Markdown, so
    such a document trains under both harness profiles rather than silently
    borrowing one of them.
    """
    profile = case["profile"]
    return (profile,) if profile in HARNESS_PROFILES else HARNESS_PROFILES


def pgo_argument_error(args: argparse.Namespace) -> str | None:
    """Reject PGO option combinations that would silently do nothing."""
    inputs = ("pgo_training_corpus", "pgo_training_filter", "pgo_measurement_filter")
    if not args.pgo:
        supplied = sorted(name for name in inputs if getattr(args, name) is not None)
        if supplied:
            return "--pgo is required by: " + ", ".join("--" + n.replace("_", "-") for n in supplied)
        return None
    if not args.compile:
        return "--pgo requires --compile: the profile is collected by running the built worker"
    missing = sorted(name for name in inputs if getattr(args, name) is None)
    if missing:
        return "--pgo requires: " + ", ".join("--" + n.replace("_", "-") for n in missing)
    if args.pgo_train_ms <= 0:
        return "--pgo-train-ms must be positive"
    return None


def profdata_tool(toolchain: str, rustc_version: str) -> Path:
    """The pinned toolchain's own llvm-profdata, so profile formats match."""
    host = next(
        line.split(":", 1)[1].strip()
        for line in rustc_version.splitlines()
        if line.startswith("host:")
    )
    sysroot = Path(subprocess.check_output(
        ["rustc", f"+{toolchain}", "--print", "sysroot"], text=True
    ).strip())
    tool = sysroot / "lib" / "rustlib" / host / "bin" / "llvm-profdata"
    if not tool.is_file():
        raise SystemExit(
            f"{tool} is missing; run: rustup component add llvm-tools --toolchain {toolchain}"
        )
    return tool


def materialize_training_inputs(cases: list[dict], directory: Path) -> dict[str, list[Path]]:
    """Write the training documents and group their paths by harness profile."""
    directory.mkdir(parents=True)
    by_profile: dict[str, list[Path]] = {profile: [] for profile in HARNESS_PROFILES}
    for case in cases:
        value = case["input"].encode()
        if len(value) != case["byte_count"] or hashlib.sha256(value).hexdigest() != case["sha256"]:
            raise SystemExit(f"training corpus case does not match its own checksum: {case['name']}")
        path = directory / (case["name"] + ".md")
        path.write_bytes(value)
        for profile in training_profiles(case):
            by_profile[profile].append(path)
    if not any(by_profile.values()):
        raise SystemExit("the training filter selected no documents")
    return by_profile


def train(binary: Path, by_profile: dict[str, list[Path]], profraw: Path,
          window_ms: int, env: dict[str, str]) -> list[dict]:
    """Drive every engine through both lifecycles over the training documents."""
    runs = []
    for profile in HARNESS_PROFILES:
        paths = by_profile[profile]
        if not paths:
            continue
        for engine in ENGINES:
            for lifecycle in LIFECYCLES:
                label = f"{engine}-{profile}-{lifecycle}"
                before = set(profraw.glob("*.profraw"))
                # Timing is irrelevant here; the loop exists to execute the
                # engine.  `verify` is skipped so training never depends on
                # output agreement -- run.py --verify-only is that gate.
                child = subprocess.run(
                    [str(binary), engine, profile, lifecycle, *map(str, paths)],
                    input=f"bench {window_ms * 1_000_000}\nquit\n", text=True,
                    capture_output=True, check=False,
                    env={**env, "LLVM_PROFILE_FILE": str(profraw / f"{label}-%p.profraw")},
                )
                if child.returncode != 0:
                    raise SystemExit(f"training run {label} failed: {child.stderr}")
                fields = child.stdout.split()
                if len(fields) != 4 or fields[0] != "timing" or int(fields[1]) <= 0:
                    raise SystemExit(f"training run {label} produced no timing record")
                written = sorted(path.name for path in set(profraw.glob("*.profraw")) - before)
                if not written:
                    raise SystemExit(f"training run {label} wrote no profile data")
                runs.append({
                    "engine": engine, "profile": profile, "lifecycle": lifecycle,
                    "documents": len(paths), "iterations": int(fields[1]), "profraw": written,
                })
                print(f"  trained {label}: {len(paths)} documents", flush=True)
    return runs


def run_pgo(args: argparse.Namespace, build: Path, bun: Path, command: list[str],
            env: dict[str, str], rustc_version: str) -> dict:
    """Instrument, train, and merge; the caller then builds with -Cprofile-use.

    Everything except the added ``-Cprofile-generate``/``-Cprofile-use`` flag
    stays identical to the default build: the same pinned nightly toolchain,
    generic CPU baseline, fat LTO, one codegen unit, and panic abort.
    """
    corpus_path = args.pgo_training_corpus.resolve()
    train_pattern = read_filter(args.pgo_training_filter)
    measure_pattern = read_filter(args.pgo_measurement_filter)
    corpus = read_corpus(corpus_path)
    training = select_cases(corpus["cases"], train_pattern)
    if not training:
        raise SystemExit(f"{args.pgo_training_filter} selected no case in {corpus_path}")
    shared = overlapping_cases(training, measure_pattern)
    if shared:
        raise SystemExit("training and measured sets overlap: " + ", ".join(shared))
    tool = profdata_tool(BUN_TOOLCHAIN, rustc_version)

    profraw = build / "profraw"
    profraw.mkdir()
    (build / "profraw-build").mkdir()
    # Instrumentation keeps Bun's unreachable support code alive, so the
    # training binary alone needs the WebKit/simdutf symbols fat LTO otherwise
    # removes.  The measured -Cprofile-use worker links without these.
    stubs = build / "native" / "pgo_stubs.o"
    run(["clang", "-O3", "-c", HERE / "pgo_stubs.c", "-o", stubs], env=env)
    instrumented_flags = f"{BASE_RUSTFLAGS} -Cprofile-generate={profraw} -Clink-arg={stubs}"
    generate_env = {
        **env,
        "RUSTFLAGS": instrumented_flags,
        "CARGO_TARGET_DIR": str(build / "target-profile-generate"),
        # Build scripts and proc macros are instrumented too; keep the profiles
        # they emit while Cargo runs out of the training directory.
        "LLVM_PROFILE_FILE": str(build / "profraw-build" / "build-%p.profraw"),
    }
    print("PGO: building the instrumented worker", flush=True)
    run(command, cwd=bun, env=generate_env)
    instrumented = Path(generate_env["CARGO_TARGET_DIR"]) / "release" / "native-comparison-worker"

    inputs = build / "pgo-training-inputs"
    by_profile = materialize_training_inputs(training, inputs)
    print(f"PGO: training on {len(training)} documents", flush=True)
    runs = train(instrumented, by_profile, profraw, args.pgo_train_ms, env)

    merged = build / "merged.profdata"
    raw_files = sorted(profraw.glob("*.profraw"))
    run([tool, "merge", "-o", merged, *raw_files])
    return {
        "applied": True,
        "toolchain": BUN_TOOLCHAIN,
        "instrumented_rustflags": instrumented_flags,
        "instrumented_link_stubs": {
            "object": str(stubs),
            "source_sha256": sha(HERE / "pgo_stubs.c"),
            "note": "Bun support symbols kept alive by instrumentation; every stub aborts "
                    "and none is linked into the measured -Cprofile-use worker.",
        },
        "llvm_profdata": str(tool),
        "llvm_profdata_version": subprocess.check_output([str(tool), "--version"], text=True),
        "instrumented_binary_sha256": sha(instrumented),
        "training": {
            "corpus": str(corpus_path),
            "corpus_sha256": sha(corpus_path),
            "corpus_case_count": len(corpus["cases"]),
            "filter_file": str(args.pgo_training_filter.resolve()),
            "filter_file_sha256": sha(args.pgo_training_filter.resolve()),
            "filter": train_pattern,
            "measurement_filter_file": str(args.pgo_measurement_filter.resolve()),
            "measurement_filter_file_sha256": sha(args.pgo_measurement_filter.resolve()),
            "measurement_filter": measure_pattern,
            "disjoint_from_measured_set": True,
            "case_count": len(training),
            "cases": [case["name"] for case in training],
            "documents_per_harness_profile": {p: len(v) for p, v in by_profile.items()},
            "window_ms": args.pgo_train_ms,
            "engines": list(ENGINES),
            "lifecycles": list(LIFECYCLES),
            "runs": runs,
        },
        "profile": {
            "path": str(merged),
            "sha256": sha(merged),
            "bytes": merged.stat().st_size,
            "profraw_files": len(raw_files),
        },
        "engine_profile_data": dict(ENGINE_PROFILE_DATA),
    }


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("build_dir", type=Path)
    parser.add_argument("--bun-source", type=Path, default=Path("/private/tmp/ferromark-bun-publication-20260911"))
    parser.add_argument("--bun-native-cache", type=Path, default=Path("/private/tmp/ferromark-bun-build-20260911/native"))
    parser.add_argument("--bun-lock", type=Path, default=Path("/private/tmp/ferromark-bun-publication-20260911/Cargo.lock"), help="local lock containing the cached comparison crates")
    parser.add_argument("--md4c-source", type=Path, default=Path("/private/tmp/ferromark-md4c-publication-20260911"))
    parser.add_argument("--ferromark-v1-source", type=Path, default=REPO.parent / "ferromark")
    parser.add_argument("--ferromark-v2-source", type=Path, default=REPO)
    parser.add_argument("--ferromark-v2-revision", default=FERROMARK_V2_REVISION, help="committed v2 revision to measure; other engine pins stay unchanged")
    parser.add_argument("--ox-source", type=Path, default=None, help="git checkout containing OX_REVISION (takes precedence over archive)")
    parser.add_argument("--ox-archive", type=Path, default=None)
    parser.add_argument("--lockfile", type=Path, help="seed Bun Cargo.lock and require --locked replay")
    parser.add_argument("--worker", type=Path, required=True, help="six-engine worker.rs supplied by the parent harness")
    parser.add_argument("--compile", action="store_true", help="compile native archives and the worker")
    parser.add_argument("--finalize-existing", action="store_true", help="record metadata for an already completed build")
    parser.add_argument("--pgo", action="store_true", help="apply profile-guided optimization to every Rust engine with one recipe")
    parser.add_argument("--pgo-training-corpus", type=Path, help="corpus holding the PGO training documents")
    parser.add_argument("--pgo-training-filter", type=Path, help="file with the regex selecting training documents")
    parser.add_argument("--pgo-measurement-filter", type=Path, help="file with run.py's measured-set regex; training must not select any of it")
    parser.add_argument("--pgo-train-ms", type=int, default=250, help="training window per engine, profile, and lifecycle")
    args = parser.parse_args()
    if (message := pgo_argument_error(args)) is not None:
        parser.error(message)
    build = args.build_dir.resolve()
    if args.finalize_existing:
        finalize_existing(build, args.worker.resolve())
        return
    if build.exists():
        raise SystemExit(f"build directory already exists: {build}")
    if not args.worker.is_file():
        raise SystemExit(f"worker does not exist: {args.worker}")
    build.mkdir(parents=True)
    sources = build / "sources"
    bun = build / "bun"
    records = {
        "toolchain": BUN_TOOLCHAIN,
        "rustc": subprocess.check_output(["rustc", f"+{BUN_TOOLCHAIN}", "-vV"], text=True),
        "rustflags": "-C target-cpu=generic",
        "optimization": {
            "opt_level": 3,
            "lto": "fat",
            "codegen_units": 1,
            "panic": "abort",
            "debug": "line-tables-only",
        },
        "engines": {},
    }
    for name, source, revision in (
        ("bun", args.bun_source.resolve(), BUN_REVISION),
        ("ferromark_v1", args.ferromark_v1_source.resolve(), FERROMARK_V1_REVISION),
        ("ferromark_v2", args.ferromark_v2_source.resolve(), args.ferromark_v2_revision),
        ("md4c", args.md4c_source.resolve(), MD4C_REVISION),
    ):
        destination = sources / name
        records["engines"][name] = archive_checkout(
            source, revision, destination, allow_worktree_fallback=(name == "bun")
        )
    if args.ox_source:
        records["engines"]["ox_content"] = archive_checkout(
            args.ox_source.resolve(), OX_REVISION, sources / "ox_content"
        )
    elif args.ox_archive or Path("/private/tmp/ferromark-v2-upstream.tar.gz").is_file():
        records["engines"]["ox_content"] = extract_source_archive(
            (args.ox_archive or Path("/private/tmp/ferromark-v2-upstream.tar.gz")).resolve(),
            OX_REVISION,
            sources / "ox_content",
        )
    else:
        raise SystemExit("one of --ox-archive or --ox-source is required")

    # Cargo needs the Bun source as a workspace root because bun_md's local
    # dependencies are declared with workspace=true.  This copy is the only
    # tree mutated by preparation; the checked-out source remains untouched.
    shutil.copytree(sources / "bun", bun, symlinks=True)
    rewrite_bun_members(bun / "Cargo.toml", "comparison-worker")
    # Start from the already-resolved local publication lock so pulldown-cmark
    # and the comparison support crates are available offline. Cargo still
    # records any path-package additions in this disposable copy.
    seed_lock = (args.lockfile or args.bun_lock).resolve()
    shutil.copyfile(seed_lock, bun / "Cargo.lock")
    native = build / "native"
    records["native_archives"] = copy_native_archives(args.bun_native_cache.resolve(), native)
    records["sources"] = {}
    for name, details in records["engines"].items():
        records["sources"][name] = {
            **details,
            "tree_sha256": sha_tree(sources / name),
        }
    shutil.copyfile(args.worker.resolve(), bun / "comparison-worker.rs")
    worker = bun / "comparison-worker"
    worker.mkdir()
    shutil.move(bun / "comparison-worker.rs", worker / "worker.rs")

    # The parent worker owns the exact crate aliases and engine options.  The
    # generated package keeps the source paths explicit so no registry or JS
    # adapter is silently substituted.
    (worker / "Cargo.toml").write_text("""[package]
name = "native-comparison-worker"
version = "0.0.0"
edition = "2024"
publish = false

[[bin]]
name = "native-comparison-worker"
path = "worker.rs"

[dependencies]
bun_md.workspace = true
bun_alloc.workspace = true
bun_core.workspace = true
ferromark_v1 = { package = "ferromark", path = "../sources/ferromark_v1" }
ferromark_v2 = { package = "ferromark", path = "../sources/ferromark_v2/crates/ferromark" }
ox_content_allocator = { package = "ox_content_allocator", path = "../sources/ox_content/crates/ox_content_allocator" }
ox_content_parser = { package = "ox_content_parser", path = "../sources/ox_content/crates/ox_content_parser" }
ox_content_renderer = { package = "ox_content_renderer", path = "../sources/ox_content/crates/ox_content_renderer" }
pulldown-cmark = "=0.13.4"
html-escape = "=0.2.14"
serde_json = "1"
""")
    # These source paths are relative to the Bun checkout after it is copied.
    # Keep all source trees under build/sources so the generated workspace is
    # relocatable as one unit.
    cargo = bun / "comparison-worker" / "Cargo.toml"
    cargo_text = cargo.read_text().replace('path = "../sources/', 'path = "../../sources/')
    cargo.write_text(cargo_text)

    env = os.environ.copy()
    env.pop("CARGO_ENCODED_RUSTFLAGS", None)
    for key in list(env):
        if key.startswith("CARGO_PROFILE_"):
            env.pop(key)
    env["RUSTFLAGS"] = "-C target-cpu=generic"
    env["BUN_CODEGEN_DIR"] = str(build / "codegen")
    (build / "codegen").mkdir()
    (build / "codegen" / "build_options.rs").write_text(
        f'pub const SHA: &str = "{BUN_REVISION}";\n'
        'pub const REPORTED_NODEJS_VERSION: &str = "24.0.0";\n'
        'pub const RELEASE_SAFE: bool = false;\n'
        'pub const IS_CANARY: bool = true;\n'
        'pub const CANARY_REVISION: &str = "benchmark";\n'
        'pub const ENABLE_FUZZILLI: bool = false;\n'
        'pub const FALLBACK_HTML_VERSION: &str = "0000000000000000";\n'
        'pub const VERSION: crate::Version = crate::Version { major: 1, minor: 4, patch: 3 };\n'
        f'pub const BASE_PATH: &[u8] = {json.dumps(str(bun))}.as_bytes();\n'
        f'pub const CODEGEN_PATH: &[u8] = {json.dumps(str(build / "codegen"))}.as_bytes();\n'
        'pub const ENABLE_LOGS: bool = cfg!(bun_debug);\n'
        'pub const ENABLE_ASAN: bool = cfg!(bun_asan);\n'
        'pub const ENABLE_TINYCC: bool = true;\n'
    )
    (worker / "build.rs").write_text(f'''fn main() {{
    println!("cargo:rustc-link-search=native={{}}", {json.dumps(str(native))});
    println!("cargo:rustc-link-lib=static=bun_bench_native");
    println!("cargo:rustc-link-lib=static=md4c");
    println!("cargo:rustc-link-lib=static=mimalloc");
    println!("cargo:rustc-link-lib=c++");
}}
''')

    if args.compile:
        if os.uname().sysname != "Darwin":
            raise SystemExit("the Bun standalone stack shim is macOS-only")
        compile_native(bun, sources / "md4c", native, env)
        # Compilation is deliberately opt-in: parent timing workers should
        # build once, then run without competing preparation processes.
        union = validate_registry_union([
            sources / "ferromark_v1" / "Cargo.lock",
            sources / "ferromark_v2" / "Cargo.lock",
            sources / "ox_content" / "Cargo.lock",
            bun / "Cargo.lock",
        ])
        env["CARGO_NET_OFFLINE"] = "true"
        command = ["cargo", f"+{BUN_TOOLCHAIN}", "build", "--release", "--offline", "-p", "native-comparison-worker"]
        if args.lockfile:
            command.append("--locked")
        if args.pgo:
            records["pgo"] = run_pgo(args, build, bun, command, env, records["rustc"])
            env["RUSTFLAGS"] = (
                f"{BASE_RUSTFLAGS} -Cprofile-use={records['pgo']['profile']['path']}"
                " -Cllvm-args=-pgo-warn-missing-function"
            )
            records["rustflags"] = env["RUSTFLAGS"]
            print("PGO: building the optimized worker", flush=True)
        env["CARGO_TARGET_DIR"] = str(build / "target")
        run(command, cwd=bun, env=env)
        resolved = registry_packages(bun / "Cargo.lock")
        if any(union.get(key) != checksum for key, checksum in resolved.items() if key in union):
            raise SystemExit("Cargo selected a registry checksum/version outside the frozen lock union")
        if any(key not in union for key in resolved):
            raise SystemExit("Cargo resolved a registry package absent from the supplied lock union")
        records["binary"] = str(build / "target" / "release" / "native-comparison-worker")
        records["binary_sha256"] = sha(Path(records["binary"]))
        records["lockfile"] = str(bun / "Cargo.lock")
        records["lock_sha256"] = sha(bun / "Cargo.lock")
        records["adapter_sha256"] = {
            name: sha(HERE / name)
            for name in ("worker.rs", "prepare.py", "native.h", "stack.c", "md4c_alloc.h")
        }
        records["native_libraries"] = {
            path.name: sha(path) for path in sorted(native.glob("lib*.a"))
        }
        records["native_adapter_sources_unverified"] = False
        records["registry_allowed"] = {f"{name}@{version}": checksum for (name, version), checksum in union.items()}
        records["registry_resolved"] = {f"{name}@{version}": checksum for (name, version), checksum in resolved.items()}
    (build / "build.json").write_text(json.dumps(records, indent=2) + "\n")
    print(build / "build.json")


if __name__ == "__main__":
    main()
