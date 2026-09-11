#!/usr/bin/env python3
"""Verify rendered output before measuring; retain raw HTML, samples, and source metadata."""
import argparse
import hashlib
import json
from pathlib import Path
import platform
import statistics
import subprocess
import time

from prepare import BUN_REV, MI_REV, HWY_REV, MD4C_REV, TOOLCHAIN, HERE, REPO, git

from workload import PARSERS, CanonicalHTML, mismatches, workload_review
HEADLINE_CASES = [
    "commonmark/publication-2k", "commonmark/publication-5k", "commonmark/publication-10k",
    "strikethrough/strikethrough",
    "commonmark/links", "commonmark/entities",
    "tables/tables-plain", "tables/tables-commonmark-inline", "tables/tables-links",
]


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def capture(args):
    return subprocess.check_output(args, text=True).strip()


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("bun", type=Path)
    p.add_argument("result", type=Path, help="New output directory; existing results are never overwritten")
    p.add_argument("--verify-only", action="store_true")
    p.add_argument("--case", action="append", help="Measure only named cases, each in three publication runs")
    p.add_argument("--screening", action="store_true", help="Short smoke run, never used for publication")
    args = p.parse_args()
    bun, result = args.bun.resolve(), args.result.resolve()
    result.mkdir(parents=True, exist_ok=False)
    binary = bun / "target/release/ferromark-bun-comparison"
    if git(bun, "rev-parse", "HEAD") != BUN_REV or git(bun, "diff", "HEAD", "--", "src", "scripts/build"):
        raise SystemExit("Pinned Bun sources changed; refusing measurement")
    if (bun / "ferromark-comparison/driver.rs").read_bytes() != (HERE / "driver.rs").read_bytes():
        raise SystemExit("Rebuild after changing driver.rs")
    (result / "ferromark.patch").write_text(git(REPO, "diff", "HEAD", "--", "src", "Cargo.toml", "Cargo.lock") + "\n")
    (result / "bun-workspace.patch").write_text(git(bun, "diff", "HEAD", "--", "Cargo.toml") + "\n")
    (result / "Cargo.lock").write_bytes((bun / "Cargo.lock").read_bytes())
    metadata = {
        "bun_revision": BUN_REV, "mimalloc_revision": MI_REV, "highway_revision": HWY_REV, "md4c_revision": MD4C_REV,
        "ferromark_revision": git(REPO, "rev-parse", "HEAD"),
        "ferromark_source_sha256": {str(f.relative_to(REPO)): sha(f) for f in sorted((REPO / "src").rglob("*.rs"))},
        "harness_sha256": {f.name: sha(f) for f in sorted(HERE.iterdir()) if f.suffix in (".rs", ".py", ".c", ".h")},
        "binary_sha256": sha(binary), "platform": platform.platform(),
        "cpu": capture(["sysctl", "-n", "machdep.cpu.brand_string"]),
        "rustc": capture(["rustc", f"+{TOOLCHAIN}", "-Vv"]),
        "clang": capture(["clang++", "--version"]),
        "rustflags": "-C target-cpu=generic", "profile": "Bun release: opt-level=3, fat LTO, 1 CGU, panic=abort",
        "allocation": "fresh parser and owned output; Bun mimalloc for Rust and md4c C allocation calls",
        "started_unix": time.time(),
        "fixture_sha256": {str(f.relative_to(REPO)): sha(f) for f in sorted((REPO / "benches/fixtures").glob("*.md"))},
        "spec_sha256": sha(REPO / "tests/spec.json"),
    }
    stamp = json.loads((bun / "ferromark-comparison/build-info.json").read_text())
    for key in ("binary_sha256", "ferromark_source_sha256"):
        if stamp[key] != metadata[key]:
            raise SystemExit(f"Build inputs changed ({key}); rerun prepare.py")
    adapters = {f.name: sha(f) for f in sorted(HERE.iterdir()) if f.suffix in (".rs", ".c", ".h")}
    if stamp["adapter_sha256"] != adapters or stamp["md4c_revision"] != MD4C_REV:
        raise SystemExit("Native adapter changed; rerun prepare.py")
    metadata["native_archives_sha256"] = stamp["archives"]
    metadata["md4c_source_sha256"] = stamp["md4c_source_sha256"]
    headline_cases = args.case or HEADLINE_CASES
    samples, window_ms, warmup_ms = (5, 20, 50) if args.screening else (80, 63, 3000)
    metadata["protocol"] = {"samples":samples, "window_ms":window_ms, "warmup_ms":warmup_ms,
        "headline_runs":1 if args.screening else 3, "mode":"screening" if args.screening else "publication",
        "headline_cases":headline_cases, "selected_cases":args.case, "statistic":"median of run medians", "timer_check_batch":16}
    (result / "metadata.json").write_text(json.dumps(metadata, indent=2) + "\n")
    subprocess.run([binary, "selftest"], check=True)
    for mode in ("verify", "spec", "catalog", "options"):
        with (result / f"{mode}.jsonl").open("w") as out:
            subprocess.run([binary, mode], stdout=out, check=True)
    cases = []
    allowed = []
    for line in (result / "verify.jsonl").read_text().splitlines():
        row = json.loads(line)
        outputs = row["outputs"]
        reference = outputs["ferromark"]
        differences = mismatches(outputs)
        review = workload_review(row["case"], outputs)
        cases.append({"case": row["case"], "workload": review, "bytes": row["bytes"], "exact": all(h == reference for h in outputs.values()),
                      "mismatches": differences, "output_sha256": {k: hashlib.sha256(v.encode()).hexdigest() for k, v in outputs.items()}})
        if review["comparable"]:
            allowed.append(row["case"])
    spec = {}
    for line in (result / "spec.jsonl").read_text().splitlines():
        row = json.loads(line)
        for name, html in row["outputs"].items():
            counts = spec.setdefault(name, {"total": 0, "exact": 0, "normalized": 0, "mismatched_examples": []})
            counts["total"] += 1
            counts["exact"] += html == row["expected"]
            match = CanonicalHTML(html).tokens == CanonicalHTML(row["expected"]).tokens
            counts["normalized"] += match
            if not match:
                counts["mismatched_examples"].append(row["example"])
    (result / "verification.json").write_text(json.dumps({"cases": cases, "spec": spec}, indent=2) + "\n")
    (result / "allowlist.json").write_text(json.dumps(allowed, indent=2) + "\n")
    print(f"Admitted {len(allowed)}/{len(cases)} benchmark workloads; HTML differences recorded.", flush=True)
    if args.verify_only:
        return
    if not allowed:
        raise SystemExit("No comparable output; no timings collected")
    if not set(headline_cases).issubset(allowed):
        raise SystemExit("A headline case failed workload eligibility; refusing publication")
    if args.case:
        allowed = [case for case in allowed if case in args.case]
        (result / "allowlist.json").write_text(json.dumps(allowed, indent=2) + "\n")
    (result / "headline-allowlist.json").write_text(json.dumps(headline_cases) + "\n")
    collected = {}
    for repetition in range(1 if args.screening else 3):
        allowlist = "allowlist.json" if repetition == 0 else "headline-allowlist.json"
        path = result / f"samples-{repetition}.jsonl"
        with path.open("w") as out:
            subprocess.run([binary, "bench", result / allowlist, str(samples), str(window_ms), str(warmup_ms), str(repetition)], stdout=out, check=True)
        for line in path.read_text().splitlines():
            row = json.loads(line)
            entry = collected.setdefault((row["case"],row["parser"]), {**row, "runs":[]})
            entry["runs"].append(row["ns_per_render"])
        print(f"Completed measurement run {repetition + 1}.", flush=True)
    summary = []
    for row in collected.values():
        medians = [statistics.median(run) for run in row.pop("runs")]
        median = statistics.median(medians)
        row.pop("ns_per_render")
        summary.append({**row, "run_medians_ns":medians, "median_ns": median,
                        "mib_per_second": row["bytes"] / median * 1e9 / 1048576})
    (result / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
    metadata["finished_unix"] = time.time()
    (result / "metadata.json").write_text(json.dumps(metadata, indent=2) + "\n")
    print(result)


if __name__ == "__main__":
    main()
