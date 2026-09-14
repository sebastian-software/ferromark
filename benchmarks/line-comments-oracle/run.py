#!/usr/bin/env python3
"""Build and run the pinned v1/v2 line-comment differential oracle."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import tempfile


EXPECTED_CASE_MISMATCHES = {"list-definition-title"}
EXPECTED_BASELINE_MISMATCHES = {
    "list-definition-title-comments-off",
    "list-definition-title-comment-removed",
}


def cargo_string(path: Path) -> str:
    return json.dumps(str(path), ensure_ascii=False)


def decode_hex(value: str) -> str:
    return bytes.fromhex(value).decode("utf-8")


def parse_worker_output(output: str) -> tuple[list[dict], list[dict], dict]:
    cases: list[dict] = []
    baselines: list[dict] = []
    summary: dict | None = None
    for line in output.splitlines():
        fields = line.split("\t")
        if fields[0] == "PROFILE":
            continue
        if fields[0] == "CASE":
            if fields[2] == "OK":
                cases.append({"name": fields[1], "status": "OK"})
            elif fields[2] == "DIFF":
                cases.append({
                    "name": fields[1],
                    "status": "DIFF",
                    "v1_html": decode_hex(fields[3]),
                    "v2_html": decode_hex(fields[4]),
                })
            else:
                raise RuntimeError(f"unknown case status: {line}")
            continue
        if fields[0] == "BASELINE":
            baselines.append({
                "name": fields[1],
                "status": fields[2],
                "v1_html": decode_hex(fields[3]),
                "v2_html": decode_hex(fields[4]),
            })
            continue
        if fields[0] == "SUMMARY":
            summary = {
                "case_count": int(fields[1]),
                "matching_count": int(fields[2]),
                "mismatch_count": int(fields[3]),
            }
            continue
        raise RuntimeError(f"unexpected worker output: {line}")
    if summary is None:
        raise RuntimeError("worker emitted no summary")
    return cases, baselines, summary


def main() -> int:
    repo_root = Path(__file__).resolve().parents[2]
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--v1-source", type=Path, required=True, help="Pinned v1 source directory")
    parser.add_argument(
        "--output",
        type=Path,
        default=Path(__file__).with_name("results.json"),
        help="Results JSON path",
    )
    parser.add_argument("--no-offline", action="store_true", help="Allow Cargo registry access")
    args = parser.parse_args()
    v1_source = args.v1_source.resolve()
    v2_source = (repo_root / "crates" / "ferromark").resolve()
    if not (v1_source / "Cargo.toml").is_file():
        raise SystemExit(f"v1 source has no Cargo.toml: {v1_source}")
    if not (v2_source / "Cargo.toml").is_file():
        raise SystemExit(f"v2 facade has no Cargo.toml: {v2_source}")

    worker_source = Path(__file__).with_name("worker.rs").read_text()
    manifest_text = f'''[package]
name = "line-comments-oracle"
version = "0.1.0"
edition = "2024"

[dependencies]
ferromark_v1 = {{ package = "ferromark", path = {cargo_string(v1_source)} }}
ferromark_v2 = {{ package = "ferromark", path = {cargo_string(v2_source)} }}
'''
    with tempfile.TemporaryDirectory(prefix="ferromark-line-comments-oracle-") as temporary:
        temporary_root = Path(temporary)
        (temporary_root / "src").mkdir()
        (temporary_root / "Cargo.toml").write_text(manifest_text)
        (temporary_root / "src" / "main.rs").write_text(worker_source)
        command = ["cargo", "run", "--quiet", "--manifest-path", str(temporary_root / "Cargo.toml")]
        if not args.no_offline:
            command.append("--offline")
        completed = subprocess.run(command, cwd=repo_root, text=True, capture_output=True)
    if completed.returncode != 0:
        raise SystemExit(completed.stderr or completed.stdout)

    cases, baselines, summary = parse_worker_output(completed.stdout)
    case_mismatches = {case["name"] for case in cases if case["status"] == "DIFF"}
    baseline_mismatches = {case["name"] for case in baselines if case["status"] == "DIFF"}
    if case_mismatches != EXPECTED_CASE_MISMATCHES:
        raise SystemExit(f"unexpected case mismatches: {sorted(case_mismatches)}")
    if baseline_mismatches != EXPECTED_BASELINE_MISMATCHES:
        raise SystemExit(f"unexpected baseline mismatches: {sorted(baseline_mismatches)}")
    if summary != {"case_count": 40, "matching_count": 39, "mismatch_count": 1}:
        raise SystemExit(f"unexpected summary: {summary}")

    result = {
        "v1_source": str(v1_source),
        "v2_source": str(v2_source),
        "v1_expected_revision": "143ec2ce151d87d2a3d804a048014afc97733ae0",
        "worker_sha256": hashlib.sha256(worker_source.encode()).hexdigest(),
        "profiles": {
            "v1": "Options::commonmark + trusted HTML + tables/tasks/strikethrough/autolinks + line_comments",
            "v2": "ParserOptions::gfm_spec + line_comments; HtmlRendererOptions::gfm with heading IDs and URL scanner off",
        },
        "summary": summary,
        "cases": cases,
        "known_existing_limitation": {
            "case": "list-definition-title",
            "description": "v2 does not collect link reference definitions inside list-item sub-sources",
        },
        "baselines": baselines,
    }
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(result, indent=2, ensure_ascii=False) + "\n")
    print(f"matched {summary['matching_count']}/{summary['case_count']}; known mismatch: list-definition-title")
    print(f"saved {args.output}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
