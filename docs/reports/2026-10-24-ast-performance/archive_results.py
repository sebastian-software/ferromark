#!/usr/bin/env python3
"""Archive the measured stages and their exact source deltas for this report."""
import argparse
import gzip
import hashlib
import json
from pathlib import Path
import shutil
import subprocess

p = argparse.ArgumentParser(description=__doc__)
p.add_argument("experiment", type=Path)
a = p.parse_args()
report = Path(__file__).resolve().parent
runs = ["head-v-initial", "initial-v-mdx", "mdx-v-inline", "inline-v-checks",
        "head-v-final", "head-v-final-confirm", "control", "head-v-promoted", "promoted-confirm"]
for name in runs:
    source = a.experiment / name
    destination = report / "raw" / name
    destination.mkdir(parents=True, exist_ok=True)
    for filename in ["build.json", "run.json", "summary.json", "summary.csv"]:
        shutil.copyfile(source / filename, destination / filename)
    for filename in ["samples.json", "verification.json", "corpus.json", "arena-capacities.json"]:
        (destination / (filename + ".gz")).write_bytes(gzip.compress((source / filename).read_bytes(), mtime=0))
    shutil.copyfile(a.experiment / (name + ".log"), destination / "runner.log")

patches = report / "patches"
patches.mkdir(exist_ok=True)
for before, after, filename in [
    ("head", "initial", "01-initial-api.patch"),
    ("initial", "mdx-optimized", "02-mdx-stack.patch"),
    ("mdx-optimized", "inline-optimized", "03-completion-inline-experiment.patch"),
    ("inline-optimized", "checks-optimized", "04-config-inline-rejected.patch"),
    ("mdx-optimized", "promoted", "05-preserve-default-path.patch"),
]:
    result = subprocess.run(["git", "diff", "--no-index", "--binary", "--", before, after],
                            cwd=a.experiment, capture_output=True, check=False)
    if result.returncode not in (0, 1):
        raise RuntimeError(result.stderr.decode())
    patch = result.stdout.decode()
    for prefix in (before, after):
        patch = patch.replace("a/" + prefix + "/", "a/").replace("b/" + prefix + "/", "b/")
    (patches / filename).write_text(patch)

manifest = {str(path.relative_to(report)): hashlib.sha256(path.read_bytes()).hexdigest()
            for path in sorted(report.rglob("*")) if path.is_file() and path.name not in ("README.md", "SHA256SUMS.json")}
(report / "SHA256SUMS.json").write_text(json.dumps(manifest, indent=2) + "\n")
print(f"Archived {len(manifest)} files")
