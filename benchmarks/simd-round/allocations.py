#!/usr/bin/env python3
"""Build isolated allocation workers and compare their aggregated allocation counters."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import tomllib

from prepare import verify_baseline

ENGINES = ("baseline", "candidate")
MODES = ("fresh", "reuse", "parse")
BASELINE_REVISION = "ae963d425a4c15841ced5f7359a4b9adc47ca931"
WORKER = Path(__file__).with_name("allocation_worker.rs")


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


def revision(source: Path):
    try:
        return subprocess.check_output(
            ["git", "-C", str(source), "rev-parse", "HEAD"],
            text=True,
            stderr=subprocess.STDOUT,
        ).strip()
    except (OSError, subprocess.CalledProcessError):
        return None


def registry_packages(path: Path):
    lock = tomllib.loads(path.read_text())
    return {
        (package["name"], package["version"]): package.get("checksum")
        for package in lock.get("package", [])
        if "source" in package
    }


def build_workers(baseline: Path, candidate: Path, build: Path):
    build.mkdir(parents=True, exist_ok=False)
    worker_bytes = WORKER.read_bytes()
    env = os.environ.copy()
    env.pop("CARGO_ENCODED_RUSTFLAGS", None)
    env["RUSTFLAGS"] = "-C target-cpu=generic"
    record = {
        "rustc": subprocess.check_output(["rustc", "+1.95", "-vV"], text=True),
        "rustflags": env["RUSTFLAGS"],
        "worker_sha256": hashlib.sha256(worker_bytes).hexdigest(),
        "baseline_revision_expected": BASELINE_REVISION,
        "engines": {},
    }
    for name, source in (("baseline", baseline), ("candidate", candidate)):
        source = source.resolve()
        root = build / name
        (root / "src").mkdir(parents=True)
        (root / "src/main.rs").write_bytes(worker_bytes)
        shutil.copyfile(source / "Cargo.lock", root / "Cargo.lock")
        dependency = source / "crates/ferromark"
        (root / "Cargo.toml").write_text(
            '[package]\nname = "simd-round-allocation-worker"\nversion = "0.0.0"\n'
            'edition = "2024"\npublish = false\n\n[workspace]\n\n[dependencies]\n'
            f"ferromark = {{ path = {json.dumps(str(dependency))} }}\n"
            + "\n[profile.release]\nopt-level = 3\nlto = \"fat\"\n"
            "codegen-units = 1\npanic = \"abort\"\nstrip = true\n"
        )
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
        binary = root / "target/release/simd-round-allocation-worker"
        record["engines"][name] = {
            "source": str(source),
            "source_revision": revision(source),
            "source_tree_sha256": tree_sha(source),
            "source_manifest_sha256": sha256(source / "Cargo.toml"),
            "source_lock_sha256": sha256(source / "Cargo.lock"),
            "binary": str(binary),
            "binary_sha256": sha256(binary),
            "lock_sha256": sha256(root / "Cargo.lock"),
            "retained_registry_packages": len(retained),
            "registry_versions_unchanged": True,
            "command": command,
        }
    (build / "build.json").write_text(json.dumps(record, indent=2) + "\n")
    return record


class Worker:
    def __init__(self, binary: Path, profile: str, mode: str, input_path: Path):
        self.process = subprocess.Popen(
            [str(binary), profile, mode, str(input_path)],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
        )

    def measure(self):
        self.process.stdin.write("measure\n")
        self.process.stdin.flush()
        line = self.process.stdout.readline().rstrip("\n")
        if not line:
            error = self.process.stderr.read()
            raise RuntimeError(error or "allocation worker exited without output")
        fields = line.split()
        if len(fields) != 15 or fields[0] != "allocation":
            raise RuntimeError(f"invalid allocation line: {line!r}")
        (
            _,
            html_len,
            html_checksum,
            children,
            arena_capacity_bytes,
            alloc_count,
            alloc_bytes,
            zeroed_count,
            zeroed_bytes,
            dealloc_count,
            dealloc_bytes,
            realloc_count,
            realloc_old_bytes,
            realloc_new_bytes,
            requested_bytes,
        ) = fields
        return {
            "html_len": int(html_len),
            "html_checksum": html_checksum,
            "children": int(children),
            "arena_capacity_bytes": int(arena_capacity_bytes),
            "alloc_count": int(alloc_count),
            "alloc_bytes": int(alloc_bytes),
            "zeroed_count": int(zeroed_count),
            "zeroed_bytes": int(zeroed_bytes),
            "dealloc_count": int(dealloc_count),
            "dealloc_bytes": int(dealloc_bytes),
            "realloc_count": int(realloc_count),
            "realloc_old_bytes": int(realloc_old_bytes),
            "realloc_new_bytes": int(realloc_new_bytes),
            "requested_bytes": int(requested_bytes),
        }

    def close(self):
        try:
            self.process.stdin.write("quit\n")
            self.process.stdin.flush()
            self.process.communicate(timeout=10)
        finally:
            if self.process.poll() is None:
                self.process.kill()
        if self.process.returncode:
            raise RuntimeError(f"allocation worker exited with {self.process.returncode}")


def load_corpus(path: Path):
    return json.loads(path.read_text())


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("baseline_source", type=Path)
    parser.add_argument("candidate_source", type=Path)
    parser.add_argument("build_dir", type=Path)
    parser.add_argument("output", type=Path)
    parser.add_argument(
        "corpus_pos",
        type=Path,
        nargs="?",
        help="optional corpus path; accepted after output for compatibility",
    )
    parser.add_argument("--corpus", type=Path)
    parser.add_argument("--modes", nargs="+", choices=MODES, default=list(MODES))
    args = parser.parse_args()
    baseline_verification = verify_baseline(args.baseline_source.resolve())
    if registry_packages(args.baseline_source / "Cargo.lock") != registry_packages(args.candidate_source / "Cargo.lock"):
        raise SystemExit("baseline and candidate registry dependencies differ")
    corpus_path = args.corpus
    output = args.output
    if corpus_path is None and args.corpus_pos is not None:
        # Also accept the conventional ``baseline candidate build corpus output``
        # ordering used by the timing runner.
        if output.suffix == ".json" or output.is_file():
            corpus_path, output = output, args.corpus_pos
        else:
            corpus_path = args.corpus_pos
    if corpus_path is None:
        parser.error("provide --corpus PATH or a positional corpus path")
    corpus = load_corpus(corpus_path)
    cases = corpus["cases"]
    if not cases:
        raise SystemExit("corpus has no cases")
    build = args.build_dir.resolve()
    output = output.resolve()
    build_data = build_workers(args.baseline_source.resolve(), args.candidate_source.resolve(), build)
    build_data["baseline_verification"] = baseline_verification
    (build / "build.json").write_text(json.dumps(build_data, indent=2) + "\n")
    output.mkdir(parents=True, exist_ok=False)
    (output / "build.json").write_text(json.dumps(build_data, indent=2) + "\n")
    (output / "corpus.json").write_text(json.dumps(corpus, indent=2, ensure_ascii=False) + "\n")
    inputs = output / "inputs"
    inputs.mkdir()
    input_paths = {}
    for case in cases:
        raw = case["input"].encode("utf-8")
        if len(raw) != case["byte_count"] or hashlib.sha256(raw).hexdigest() != case["sha256"]:
            raise AssertionError((case["name"], "corpus integrity"))
        path = inputs / f'{case["name"]}.md'
        path.write_bytes(raw)
        input_paths[case["name"]] = path

    rows = []
    for index, case in enumerate(cases, 1):
        for mode in args.modes:
            engines = {}
            for engine in ENGINES:
                worker = Worker(
                    Path(build_data["engines"][engine]["binary"]),
                    case["profile"],
                    mode,
                    input_paths[case["name"]],
                )
                try:
                    engines[engine] = worker.measure()
                finally:
                    worker.close()
            left, right = engines["baseline"], engines["candidate"]
            if left != right:
                raise AssertionError((case["name"], case["profile"], mode, left, right))
            rows.append(
                {
                    "case": case["name"],
                    "profile": case["profile"],
                    "mode": mode,
                    "baseline": left,
                    "candidate": right,
                }
            )
        if index % 20 == 0:
            print(f"  {index}/{len(cases)} allocation cases checked", flush=True)
    result = {
        "schema": 1,
        "worker_sha256": build_data["worker_sha256"],
        "corpus_sha256": sha256(output / "corpus.json"),
        "binary_hashes": {
            engine: build_data["engines"][engine]["binary_sha256"] for engine in ENGINES
        },
        "source_hashes": {
            engine: build_data["engines"][engine]["source_tree_sha256"] for engine in ENGINES
        },
        "modes": args.modes,
        "case_count": len(cases),
        "rows": rows,
    }
    (output / "allocations.json").write_text(json.dumps(result, indent=2) + "\n")
    print(output / "allocations.json")


if __name__ == "__main__":
    main()
