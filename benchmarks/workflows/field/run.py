#!/usr/bin/env python3
"""Audit the full engine field, then time whole collections and record peak RSS."""
import argparse
from pathlib import Path
import shutil
import sys
import time

from common import ENGINES, PROTOCOL, FieldWorker, admission, read_json, sha, write_json
from support import HERE, ROOT, observation, source_hashes


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("build", type=Path)
    parser.add_argument("result", type=Path)
    parser.add_argument("--verify-only", action="store_true")
    args = parser.parse_args()
    if sys.platform != "darwin":
        raise ValueError("This published protocol defines macOS ru_maxrss in bytes")
    build, result = args.build.resolve(), args.result.resolve()
    metadata = read_json(build / "build.json")
    if metadata["source_sha256"] != source_hashes():
        raise ValueError("Source changed after build")
    for file, digest in metadata["binaries"].items():
        if sha(build / file) != digest:
            raise ValueError("Built binary changed")
    for file, digest in metadata["referenced_sha256"].items():
        if sha(ROOT / file) != digest:
            raise ValueError("Referenced adapter changed after build")
    if set(metadata["workers"]) != set(ENGINES):
        raise ValueError("Incomplete engine field")
    result.mkdir(parents=True, exist_ok=False)
    write_json(result / "build.json.gz", metadata)
    shutil.copyfile(build / "build.log", result / "build.log")
    shutil.copyfile(HERE / "corpus.json", result / "corpus.json")
    corpus = read_json(result / "corpus.json")
    outputs = {}
    for engine in ENGINES:
        outputs[engine] = {}
        for lifetime in ("stream", "retain"):
            worker = FieldWorker(metadata["workers"][engine], result / "corpus.json", lifetime=lifetime)
            try:
                outputs[engine][lifetime] = worker.ask("verify")
            finally:
                worker.close()
    reviewed = admission(outputs, corpus)
    write_json(result / "outputs.json.gz", outputs)
    write_json(result / "admission.json", reviewed)
    for engine, row in reviewed.items():
        print(f"{engine}: {row['complete_documents']}/{row['total_documents']} complete documents; collection admitted={row['admitted']}", flush=True)
    if args.verify_only:
        return
    variants = [(engine, lifetime) for engine in ENGINES if reviewed[engine]["admitted"] for lifetime in ("stream", "retain")]
    windows, warmups, rss = [], [], []
    observations = [{"stage": "before", **observation()}]
    if "AC Power" not in observations[0]["power"]:
        raise ValueError("Publication requires AC power")
    start = time.time()
    for round_id in range(3):
        workers = {(engine, lifetime): FieldWorker(metadata["workers"][engine], result / "corpus.json", lifetime=lifetime)
                   for engine, lifetime in variants}
        try:
            for engine, lifetime in variants[round_id:] + variants[:round_id]:
                row = workers[engine, lifetime].ask("time", milliseconds=3000)
                warmups.append({"round": round_id, "engine": engine, "lifetime": lifetime, **row})
            for window in range(80):
                offset = (round_id + window) % len(variants)
                order = variants[offset:] + variants[:offset]
                if window % 2:
                    order.reverse()
                for position, (engine, lifetime) in enumerate(order):
                    row = workers[engine, lifetime].ask("time", milliseconds=63)
                    windows.append({"round": round_id, "window": window, "position": position,
                                    "engine": engine, "lifetime": lifetime, **row})
                if (window + 1) % 20 == 0:
                    print(f"Field round {round_id + 1}/3: {window + 1}/80 windows", flush=True)
        finally:
            for (engine, lifetime), worker in workers.items():
                rss.append({"round": round_id, "engine": engine, "lifetime": lifetime, **worker.close()})
        observations.append({"stage": f"after-round-{round_id}", **observation()})
        write_json(result / "windows.json.gz", windows)
        write_json(result / "rss.json", rss)
    if metadata["source_sha256"] != source_hashes():
        raise ValueError("Source changed during measurement")
    write_json(result / "warmups.json", warmups)
    write_json(result / "observations.json", observations)
    write_json(result / "run.json", {"mode": "publication", "protocol": PROTOCOL, "started_unix": start,
                                    "finished_unix": time.time(), "variants": variants})
    write_json(result / "checksums.json", {p.name: sha(p) for p in sorted(result.iterdir()) if p.is_file()})
    print(f"Completed full field archive: {result}", flush=True)


if __name__ == "__main__":
    main()
