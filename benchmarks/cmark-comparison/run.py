#!/usr/bin/env python3
"""Verify workloads, then measure native Ferromark/cmark pairs in separate runs."""
import argparse
import json
from pathlib import Path
import platform
import statistics
import subprocess
import sys
import time

from support import HERE, REPO, SOURCES, capture, cases, local_hashes, sha, write_json

sys.path.insert(0, str(REPO / "benchmarks/bun-comparison"))
from workload import CanonicalHTML, workload_review


def validate_build(work):
    metadata = json.loads((work / "build-info.json").read_text())
    if metadata["sources"] != SOURCES or metadata["local_sha256"] != local_hashes():
        raise ValueError("Build inputs changed; rerun prepare.py into a fresh directory")
    for name in SOURCES:
        if metadata["binaries"][name] != sha(work / f"compare-{name}"):
            raise ValueError(f"{name} binary changed; rebuild before measuring")
    return metadata


def read_rows(binary, mode, catalog, *args):
    return [json.loads(line) for line in capture([binary, mode, catalog, *args]).splitlines()]


def eligible_cases(catalog, rows, competitor):
    expected = {row["case"]: row for row in catalog}
    if len(expected) != len(catalog) or len(rows) != len(expected) or {r["case"] for r in rows} != set(expected):
        raise ValueError("Missing or duplicate verification cases")
    reviews, admitted = [], []
    for row in rows:
        case = expected[row["case"]]
        if row["bytes"] != len(case["input"].encode()) or row["flags"] != case["flags"]:
            raise ValueError("Verification input/configuration mismatch")
        review = workload_review(row["case"], row["outputs"], parsers={"ferromark", competitor})
        reviews.append({"case": row["case"], **review})
        if review["comparable"]:
            admitted.append(case)
    return reviews, admitted


def summarize(rows, catalog, competitor, samples, runs):
    expected = {(row["case"], parser) for row in catalog for parser in ("ferromark", competitor)}
    grouped = {}
    for run_rows in rows:
        if len(run_rows) != len(expected) or {(r["case"], r["parser"]) for r in run_rows} != expected:
            raise ValueError("Incomplete or duplicate timing rows")
        for row in run_rows:
            values = row["ns_per_render"]
            if len(values) != samples or any(not (0 < x < float("inf")) for x in values):
                raise ValueError("Invalid timing samples")
            entry = grouped.setdefault((row["case"], row["parser"]),
                {"case": row["case"], "parser": row["parser"], "bytes": row["bytes"],
                 "output_bytes": row["output_bytes"], "run_medians_ns": []})
            if (entry["bytes"], entry["output_bytes"]) != (row["bytes"], row["output_bytes"]):
                raise ValueError("Output/input size changed between runs")
            entry["run_medians_ns"].append(statistics.median(values))
    result = []
    for entry in grouped.values():
        if len(entry["run_medians_ns"]) != runs:
            raise ValueError("Missing measurement runs")
        median = statistics.median(entry["run_medians_ns"])
        result.append({**entry, "median_ns": median, "mib_per_second": entry["bytes"] / median * 1e9 / 1048576})
    return result


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("work", type=Path)
    p.add_argument("result", type=Path, help="New result directory")
    p.add_argument("--verify-only", action="store_true")
    p.add_argument("--screening", action="store_true", help="Short validation; not publication evidence")
    p.add_argument("--case", action="append", help="Select an exact case; unavailable core-cmark extensions are skipped")
    args = p.parse_args()
    work, result = args.work.resolve(), args.result.resolve()
    build = validate_build(work)
    catalogs = {name: cases(name) for name in SOURCES}
    known = {row["case"] for catalog in catalogs.values() for row in catalog}
    if args.case and not set(args.case) <= known:
        p.error("Unknown requested case")
    result.mkdir(parents=True, exist_ok=False)
    samples, window, warmup, runs = (5, 20, 50, 1) if args.screening else (80, 63, 3000, 3)
    metadata = {"schema": "native-cmark-pairs-v1", "build": build, "platform": platform.platform(),
        "machine": platform.machine(), "cpu": platform.processor(), "started_unix": time.time(),
        "ferromark_revision": capture(["git", "-C", REPO, "rev-parse", "HEAD"]),
        "spec_sha256": sha(REPO / "tests/spec.json"),
        "protocol": {"mode": "verify" if args.verify_only else "screening" if args.screening else "measurement",
            "samples": samples, "window_ms": window, "warmup_ms": warmup, "runs": runs,
            "timer_check_batch": 16, "statistic": "median of run medians", "selected_cases": args.case},
        "pairs": {}}
    if sys.platform == "darwin":
        metadata["cpu"] = capture(["sysctl", "-n", "machdep.cpu.brand_string"])
    write_json(result / "metadata.json", metadata)
    (result / "ferromark.patch").write_text(capture(["git", "-C", REPO, "diff", "HEAD", "--", "src", "crates", "Cargo.toml", "Cargo.lock"]) + "\n")
    (result / "Cargo.lock").write_bytes((HERE / "Cargo.lock").read_bytes())
    for path in work.glob("*-CMakeCache.txt"):
        (result / path.name).write_bytes(path.read_bytes())
    for path in work.glob("*-compile_commands.json"):
        (result / path.name).write_bytes(path.read_bytes())
    # Verify both pairs in full before starting any timed process.
    for name, catalog in catalogs.items():
        binary = work / f"compare-{name}"
        folder = result / name
        folder.mkdir()
        write_json(folder / "catalog.json", catalog)
        rows = read_rows(binary, "verify", folder / "catalog.json")
        write_json(folder / "outputs.json", rows)
        reviews, admitted = eligible_cases(catalog, rows, name)
        write_json(folder / "verification.json", reviews)
        spec = json.loads((REPO / "tests/spec.json").read_text())
        spec_cases = [{"case": str(row["example"]), "flags": 0, "input": row["markdown"]} for row in spec]
        write_json(folder / "spec-input.json", spec_cases)
        spec_rows = read_rows(binary, "verify", folder / "spec-input.json")
        expected = {str(row["example"]): row["html"] for row in spec}
        for row in spec_rows:
            row["expected"] = expected[row["case"]]
            row["mismatches"] = [parser for parser, html in row["outputs"].items()
                if CanonicalHTML(html).tokens != CanonicalHTML(row["expected"]).tokens]
        write_json(folder / "spec-output.json", spec_rows)
        selected = admitted if not args.case else [row for row in admitted if row["case"] in args.case]
        failed = {row["case"] for row in catalog} - {row["case"] for row in admitted}
        if args.case and failed.intersection(args.case) and not args.verify_only:
            raise ValueError(f"Requested workload failed verification for {name}")
        write_json(folder / "allowlist.json", selected)
        metadata["pairs"][name] = {"eligible": len(admitted), "total": len(catalog),
            "selected": [row["case"] for row in selected], "excluded": sorted(failed),
            "unavailable": sorted(known - {row["case"] for row in catalog})}
        print(f"{name}: admitted {len(admitted)}/{len(catalog)} workloads; spec diagnostics retained", flush=True)
    write_json(result / "metadata.json", metadata)
    if not args.verify_only:
        for name in SOURCES:
            folder = result / name
            selected = json.loads((folder / "allowlist.json").read_text())
            if not selected:
                continue
            collected = []
            for repeat in range(runs):
                rows = read_rows(work / f"compare-{name}", "bench", folder / "allowlist.json",
                    samples, window, warmup, repeat)
                write_json(folder / f"samples-{repeat}.json", rows)
                collected.append(rows)
                print(f"{name}: completed run {repeat + 1}/{runs}", flush=True)
            write_json(folder / "summary.json", summarize(collected, selected, name, samples, runs))
    validate_build(work)
    metadata["finished_unix"] = time.time()
    write_json(result / "metadata.json", metadata)
    print(result)


if __name__ == "__main__":
    main()
