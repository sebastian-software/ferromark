#!/usr/bin/env python3
"""Archive one run of the `x86-64 paired benchmark` workflow into results/x86/<name>/host-N/.

usage: collect.py <download-dir> <name>

<download-dir> is what `gh run download <run-id> --dir <download-dir>` leaves behind: one
`paired-x86-host-N/` per host with `host.txt`, `paired/results/{summary,grouped,run}.json` and
`paired/build/build.json`. `host.txt` and `run.json` are copied as they are; `summary.json`,
`grouped.json` and `build.json` are gzipped (mtime 0, so the archive hashes are reproducible).
"""
import gzip
import json
import shutil
import sys
from pathlib import Path

REPORT = Path(__file__).resolve().parent.parent.parent
source, name = Path(sys.argv[1]), sys.argv[2]
hosts = sorted(source.glob("paired-x86-host-*"))
if not hosts:
    sys.exit(f"no paired-x86-host-* directories in {source}")


def gzip_copy(src: Path, dst: Path) -> None:
    with open(dst, "wb") as raw:
        with gzip.GzipFile(filename="", mode="wb", fileobj=raw, compresslevel=9, mtime=0) as out:
            out.write(src.read_bytes())


for host in hosts:
    target = REPORT / "results" / "x86" / name / f"host-{host.name.rsplit('-', 1)[1]}"
    target.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(host / "host.txt", target / "host.txt")
    shutil.copyfile(host / "paired/results/run.json", target / "run.json")
    gzip_copy(host / "paired/results/summary.json", target / "summary.json.gz")
    gzip_copy(host / "paired/results/grouped.json", target / "grouped.json.gz")
    gzip_copy(host / "paired/build/build.json", target / "build.json.gz")
    corpus = json.loads((host / "paired/results/run.json").read_text())["corpus_sha256"]
    print(f"{name}/{target.name}: corpus {corpus[:8]}")
