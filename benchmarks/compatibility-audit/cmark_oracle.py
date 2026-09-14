#!/usr/bin/env python3
"""Run a bounded offline differential corpus against official cmark CLIs."""

from __future__ import annotations

import argparse
import hashlib
import importlib.util
import json
from pathlib import Path
import subprocess
import sys
from typing import Any


HERE = Path(__file__).resolve().parent
DEFAULT_FIXTURES = HERE / "cmark-fixtures.json"
CMARK_REVISION = "bb3678d7a73cb02d35c8876ecd097072636200a8"
CMARK_GFM_REVISION = "587a12bb54d95ac37241377e6ddc93ea0e45439b"
VERIFY_SPEC = importlib.util.spec_from_file_location("native_verify", HERE.parent / "native-comparison/verify.py")
if VERIFY_SPEC is None or VERIFY_SPEC.loader is None:
    raise RuntimeError("cannot load conservative comparator")
VERIFY = importlib.util.module_from_spec(VERIFY_SPEC)
VERIFY_SPEC.loader.exec_module(VERIFY)


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def version(binary: Path) -> str:
    result = subprocess.run([str(binary), "--version"], capture_output=True, check=False)
    return result.stdout.decode("utf-8", errors="replace").strip()


def run_reference(binary: Path, profile: str, markdown: str) -> tuple[str, str | None]:
    if profile == "commonmark":
        command = [str(binary), "--unsafe"]
    elif profile == "gfm":
        command = [
            str(binary), "-e", "table", "-e", "strikethrough", "-e", "autolink",
            "-e", "tasklist", "-e", "tagfilter",
        ]
    else:
        raise ValueError(f"unknown profile: {profile}")
    try:
        result = subprocess.run(
            command,
            input=markdown.encode("utf-8"),
            capture_output=True,
            timeout=10,
            check=False,
        )
    except (OSError, subprocess.TimeoutExpired) as exc:
        return "", f"{type(exc).__name__}: {exc}"
    output = result.stdout.decode("utf-8", errors="replace")
    if result.returncode:
        detail = result.stderr.decode("utf-8", errors="replace").strip()
        return output, f"exit {result.returncode}: {detail}".rstrip()
    return output, None


class Worker:
    def __init__(self, binary: Path):
        self.binary = binary.resolve()

    def render(self, markdown: str, profile: str) -> tuple[str, str | None]:
        request = {"markdown": markdown, "profile": profile, "html": ""}
        # This is a correctness check, so process startup is not timed. A fresh
        # process per input bounds the entire response (including partial JSON)
        # and lets later cases run even after one input crashes or times out.
        try:
            result = subprocess.run([str(self.binary)],
                input=(json.dumps(request, ensure_ascii=False) + "\n").encode(),
                capture_output=True, timeout=10, check=False)
        except (OSError, subprocess.TimeoutExpired) as exc:
            return "", f"{type(exc).__name__}: {exc}"
        if result.returncode:
            return "", f"worker exit {result.returncode}: {result.stderr.decode(errors='replace').strip()}"
        try:
            response = json.loads(result.stdout)
        except (json.JSONDecodeError, UnicodeDecodeError) as exc:
            return "", f"worker emitted invalid JSON: {exc}"
        if not isinstance(response, dict) or not isinstance(response.get("html"), str) or "error" not in response:
            return "", "worker emitted an invalid response shape"
        return response["html"], response["error"]

    def close(self) -> None:
        pass


def load_fixtures(path: Path) -> list[dict[str, str]]:
    payload = json.loads(path.read_text(encoding="utf-8"))
    cases = payload.get("cases")
    if not isinstance(cases, list) or not 100 <= len(cases) <= 300:
        raise ValueError("fixture corpus must contain 100-300 cases")
    if any(not isinstance(case, dict) for case in cases):
        raise ValueError("fixture cases must be objects")
    ids = [case.get("id") for case in cases]
    if any(not isinstance(case_id, str) for case_id in ids) or len(set(ids)) != len(ids):
        raise ValueError("fixture IDs must be unique objects")
    return cases


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--worker", type=Path, required=True, help="compatibility-audit-worker binary")
    parser.add_argument("--worker-metadata", type=Path, required=True, help="metadata produced with this worker")
    parser.add_argument("--cmark", type=Path, required=True, help="official cmark 0.31.x CLI")
    parser.add_argument("--cmark-gfm", type=Path, required=True, help="official cmark-gfm CLI")
    parser.add_argument("--cmark-build-metadata", type=Path, required=True, help="metadata produced by cmark_build.py")
    parser.add_argument("--fixtures", type=Path, default=DEFAULT_FIXTURES)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--fail-on-differences", action="store_true")
    args = parser.parse_args(argv)

    fixtures = load_fixtures(args.fixtures)
    worker_metadata = json.loads(args.worker_metadata.read_text(encoding="utf-8"))
    if worker_metadata.get("worker_sha256") != sha256(args.worker):
        raise SystemExit("worker hash does not match --worker-metadata")
    build_metadata = json.loads(args.cmark_build_metadata.read_text(encoding="utf-8"))
    for name, expected_revision in (("cmark", CMARK_REVISION), ("cmark-gfm", CMARK_GFM_REVISION)):
        source = build_metadata.get("sources", {}).get(name, {})
        if source.get("revision") != expected_revision or not source.get("clean"):
            raise SystemExit(f"{name} source metadata is not the pinned clean revision")
    for name, path in (("cmark", args.cmark), ("cmark-gfm", args.cmark_gfm)):
        recorded = build_metadata.get("binaries", {}).get(name, {})
        if recorded.get("path") != str(path) or recorded.get("sha256") != sha256(path):
            raise SystemExit(f"{name} hash/path does not match --cmark-build-metadata")
    args.output.mkdir(parents=True, exist_ok=False)
    worker = Worker(args.worker)
    results: list[dict[str, Any]] = []
    try:
        for case in fixtures:
            profile = case["profile"]
            reference = args.cmark if profile == "commonmark" else args.cmark_gfm
            actual, worker_error = worker.render(case["markdown"], profile)
            oracle, oracle_error = run_reference(reference, profile, case["markdown"])
            status = "error" if worker_error or oracle_error else VERIFY.classify(actual, oracle)
            results.append({
                **case,
                "v2_actual": actual,
                "v2_error": worker_error,
                "cmark_actual": oracle,
                "cmark_error": oracle_error,
                "status": status,
            })
    finally:
        worker.close()

    from collections import Counter

    summary: dict[str, Any] = {"total": len(results)}
    for profile in ("commonmark", "gfm"):
        rows = [row for row in results if row["profile"] == profile]
        summary[profile] = {
            "total": len(rows),
            "statuses": dict(Counter(row["status"] for row in rows)),
            "categories": dict(Counter(row["category"] for row in rows)),
        }
    metadata = {
        "fixture_sha256": sha256(args.fixtures),
        "worker": {
            "path": str(args.worker),
            "sha256": sha256(args.worker),
            "metadata_path": str(args.worker_metadata),
            "metadata_sha256": sha256(args.worker_metadata),
            "source_revision": worker_metadata.get("source_revision"),
            "core_sha256": worker_metadata.get("core_sha256"),
            "worker_source_sha256": worker_metadata.get("worker_source_sha256"),
        },
        "worker_source_metadata": worker_metadata,
        "cmark_sha256": sha256(args.cmark),
        "cmark_gfm_sha256": sha256(args.cmark_gfm),
        "cmark_version": version(args.cmark),
        "cmark_gfm_version": version(args.cmark_gfm),
        "reference_build_metadata_path": str(args.cmark_build_metadata),
        "reference_build_metadata_sha256": sha256(args.cmark_build_metadata),
        "reference_build_metadata": build_metadata,
        "reference_commands": {
            "commonmark": [str(args.cmark), "--unsafe"],
            "gfm": [str(args.cmark_gfm), "-e", "table", "-e", "strikethrough", "-e", "autolink", "-e", "tasklist", "-e", "tagfilter"],
        },
        "runner": {"path": str(HERE / "cmark_oracle.py"), "sha256": sha256(HERE / "cmark_oracle.py")},
        "comparator": {"path": str(HERE.parent / "native-comparison/verify.py"), "sha256": sha256(HERE.parent / "native-comparison/verify.py")},
        "comparison_policy": "raw outputs retained; exact, serialization-equivalent, heading-id-only, and other are separate",
    }
    (args.output / "metadata.json").write_text(json.dumps(metadata, indent=2) + "\n")
    (args.output / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
    (args.output / "results.json").write_text(json.dumps(results, indent=2, ensure_ascii=False) + "\n")
    print(json.dumps(summary, indent=2))
    if any(row["status"] == "error" for row in results):
        return 2
    if args.fail_on_differences and any(not VERIFY.admitted(row["status"]) for row in results):
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
