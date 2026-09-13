#!/usr/bin/env python3
"""Verify complete workflows, then measure time and heap in separate processes."""
import argparse
import hashlib
from pathlib import Path
import shutil
import time

from support import (HERE, PROTOCOL, VARIANTS, Worker, observation, read_json,
                     review, sha, source_hashes, write_json)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("build", type=Path)
    parser.add_argument("result", type=Path)
    parser.add_argument("--verify-only", action="store_true")
    args = parser.parse_args()
    build, result = args.build.resolve(), args.result.resolve()
    result.mkdir(parents=True, exist_ok=False)
    metadata = read_json(build / "build.json")
    if metadata["source_sha256"] != source_hashes():
        raise ValueError("Source changed after build; prepare again")
    for kind in ("timing", "memory"):
        if sha(build / kind / "worker") != metadata["binary_sha256"][kind]:
            raise ValueError("Worker binary changed")
    for file in ("corpus.json", "Cargo.lock", "Cargo.toml"):
        shutil.copyfile(HERE / file, result / file)
    for file in ("timing-build.log", "memory-build.log", "build.json"):
        shutil.copyfile(build / file, result / file)
    corpus_path = result / "corpus.json"
    corpus = read_json(corpus_path)
    verification = {}
    for variant in VARIANTS:
        worker = Worker(build / "timing/worker", corpus_path, variant)
        try:
            verification[variant] = worker.ask("verify")
        finally:
            worker.close()
    write_json(result / "outputs.json.gz", verification)
    admission = review(verification, corpus)
    write_json(result / "admission.json", admission)
    if not all(row["comparable"] for row in admission):
        raise ValueError("Review all differing outputs before measuring; corpus cannot be silently filtered")
    print(f"Admitted all {len(admission)} documentation files; previews and metadata verified", flush=True)
    if args.verify_only:
        return
    started = time.time()
    observations = [{"stage": "before", **observation()}]
    if "AC Power" not in observations[0]["power"]:
        raise ValueError("Publication run requires AC power")
    warmups, windows, memory = [], [], []
    for round_id in range(PROTOCOL["rounds"]):
        workers = {v: Worker(build / "timing/worker", corpus_path, v) for v in VARIANTS}
        try:
            order = VARIANTS[round_id:] + VARIANTS[:round_id]
            for variant in order:
                sample = workers[variant].ask("time", milliseconds=PROTOCOL["warmup_ms"])
                warmups.append({"round": round_id, "variant": variant, **sample})
            for index in range(PROTOCOL["windows"]):
                offset = (index + round_id) % len(VARIANTS)
                order = VARIANTS[offset:] + VARIANTS[:offset]
                if index % 2:
                    order = list(reversed(order))
                for position, variant in enumerate(order):
                    sample = workers[variant].ask("time", milliseconds=PROTOCOL["window_ms"])
                    windows.append({"round": round_id, "window": index, "position": position,
                                    "variant": variant, **sample})
                if (index + 1) % 20 == 0:
                    print(f"Round {round_id + 1}/3: {index + 1}/80 windows per variant", flush=True)
        finally:
            for worker in workers.values():
                worker.close()
        observations.append({"stage": f"after-timing-{round_id}", **observation()})
        for variant in VARIANTS:
            worker = Worker(build / "memory/worker", corpus_path, variant)
            try:
                # Instrumented and normal builds must do exactly the same output work.
                if worker.ask("verify") != verification[variant]:
                    raise ValueError("Instrumented worker changed output or options")
                for index in range(PROTOCOL["memory_observations"]):
                    memory.append({"round": round_id, "observation": index, "variant": variant,
                                   **worker.ask("memory")})
            finally:
                worker.close()
        write_json(result / "windows.json.gz", windows)
        write_json(result / "memory.json.gz", memory)
    if metadata["source_sha256"] != source_hashes():
        raise ValueError("Source changed during measurement")
    write_json(result / "warmups.json", warmups)
    write_json(result / "observations.json", observations)
    write_json(result / "run.json", {"mode": "publication", "protocol": PROTOCOL,
                                    "started_unix": started, "finished_unix": time.time()})
    checksums = {p.name: sha(p) for p in sorted(result.iterdir()) if p.is_file()}
    write_json(result / "checksums.json", checksums)
    print(f"Completed evidence archive: {result}", flush=True)


if __name__ == "__main__":
    main()
