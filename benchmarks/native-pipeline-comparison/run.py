#!/usr/bin/env python3
"""Verify all native outputs, then interleave persistent workers' timed windows."""
import argparse
from contextlib import ExitStack
import json
import math
from pathlib import Path
import platform
import statistics
import time

from common import (HERE, REPO, ENGINES, Worker, CanonicalHTML, capture, catalog, local_hashes,
                    mdx_cases, probes, sha, workload_review, write_json)


def validate_build(work):
    info = json.loads((work / "build-info.json").read_text())
    if info["local_sha256"] != local_hashes():
        raise ValueError("Build inputs changed; prepare a fresh build")
    for name, digest in info["binaries"].items():
        if sha(work / name) != digest:
            raise ValueError("Worker binary changed")
    return info


def verify_row(row, case, engine):
    if (row["case"], row["engine"], row["flags"], row["bytes"]) != (
            case["case"], engine, case["flags"], len(case["input"].encode())):
        raise ValueError("Wrong verification case/configuration")
    if not isinstance(row["html"], str) or not isinstance(row["options"], str):
        raise ValueError("Missing original output/options")


def validate_window(row, case, engine, ms):
    if row["case"] != case or row["engine"] != engine:
        raise ValueError("Wrong timing case/engine")
    count, elapsed, ns = row["count"], row["elapsed_ns"], row["ns_per_render"]
    if not isinstance(count, int) or count <= 0 or count % 16:
        raise ValueError("Invalid render count")
    if not isinstance(elapsed, int) or elapsed < ms * 1_000_000:
        raise ValueError("Incomplete timing window")
    if not math.isfinite(ns) or not math.isclose(ns, elapsed / count, rel_tol=1e-12):
        raise ValueError("Timing ratio does not match the raw clock/count")


def summarize(raw_runs, selected, engine, protocol, outputs):
    expected = {(c, p, sample) for c in selected for p in ("ferromark", engine)
                for sample in range(protocol["samples"])}
    medians = {(c, p): [] for c in selected for p in ("ferromark", engine)}
    if len(raw_runs) != protocol["runs"]:
        raise ValueError("Missing process run")
    for repeat, rows in enumerate(raw_runs):
        if len(rows) != len(expected) or {(r["case"], r["engine"], r["sample"]) for r in rows} != expected:
            raise ValueError("Missing or duplicate timing sample")
        grouped = {key: [] for key in medians}
        for row in rows:
            if row["run"] != repeat:
                raise ValueError("Wrong process run")
            validate_window(row, row["case"], row["engine"], protocol["window_ms"])
            grouped[row["case"], row["engine"]].append(row["ns_per_render"])
        for key, values in grouped.items():
            medians[key].append(statistics.median(values))
    output = {(row["case"], row["engine"]): row for row in outputs}
    return [{"case": case, "engine": parser, "run_medians_ns": values,
        "median_ns": statistics.median(values), "bytes": output[case, parser]["bytes"],
        "output_bytes": len(output[case, parser]["html"].encode())}
        for (case, parser), values in medians.items()]


def check_switches(rows):
    for row in rows[:8]:
        flags = row["flags"]
        tokens = CanonicalHTML(row["html"]).tokens
        starts = [t for t in tokens if t[0] == "start"]
        boxes = [dict(t[2]) for t in starts if t[1] == "input"]
        if (any(t[1] == "table" for t in starts) != bool(flags & 1)
                or any(t[1] == "del" for t in starts) != bool(flags & 2)
                or len(boxes) != (2 if flags & 4 else 0)
                or sum("checked" in a for a in boxes) != (1 if flags & 4 else 0)
                or any(t[1] == "a" for t in starts)
                or "<script>x</script>" not in row["html"]):
            raise ValueError(f"Feature/policy probe failed: {row['engine']} {row['case']}")
    if 'href="javascript:alert"' not in rows[-1]["html"]:
        raise ValueError("Trusted URL policy does not match")


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("work", type=Path)
    p.add_argument("result", type=Path, help="New result directory")
    p.add_argument("--verify-only", action="store_true")
    p.add_argument("--screening", action="store_true")
    p.add_argument("--case", action="append")
    args = p.parse_args()
    work, result = args.work.resolve(), args.result.resolve()
    build = validate_build(work)
    corpus = catalog()
    if args.case and not set(args.case) <= {r["case"] for r in corpus}:
        p.error("Unknown requested case")
    result.mkdir(parents=True, exist_ok=False)
    protocol = {"mode": "verify" if args.verify_only else "screening" if args.screening else "measurement",
        "samples": 5 if args.screening else 80, "window_ms": 20 if args.screening else 63,
        "warmup_ms": 50 if args.screening else 3000, "runs": 1 if args.screening else 3,
        "timer_check_batch": 16, "statistic": "median of run medians",
        "selected_cases": args.case, "GOGC": "100", "GOMAXPROCS": "1", "GOMEMLIMIT": "unset"}
    metadata = {"schema": "native-pipeline-pairs-v1", "build": build, "protocol": protocol,
        "ferromark_revision": capture(["git", "rev-parse", "HEAD"], cwd=REPO),
        "platform": platform.platform(), "machine": platform.machine(),
        "cpu": capture(["sysctl", "-n", "machdep.cpu.brand_string"]) if platform.system() == "Darwin" else platform.processor(),
        "started_unix": time.time(), "pairs": {}}
    write_json(result / "metadata.json", metadata)
    write_json(result / "catalog.json", corpus)
    probe_cases, mdx = probes(), mdx_cases()
    write_json(result / "probes.json", probe_cases)
    write_json(result / "mdx-input.json", mdx)
    spec = json.loads((REPO / "tests/spec.json").read_text())
    spec_cases = [{"case": str(r["example"]), "flags": 0, "input": r["markdown"]} for r in spec]
    write_json(result / "spec-input.json", spec_cases)
    for name in ("Cargo.lock", "goldmark/go.mod", "goldmark/go.sum"):
        (result / Path(name).name).write_bytes((HERE / name).read_bytes())
    (result / "ferromark.patch").write_text(capture(["git", "diff", "HEAD", "--", "src", "crates", "Cargo.toml", "Cargo.lock"], cwd=REPO) + "\n")
    verified = {}
    # Verify every engine and diagnostic before starting any timed process.
    for engine in ("ferromark", *ENGINES):
        rows = []
        for filename, inputs, operation in (("catalog", corpus, "verify"), ("probes", probe_cases, "verify"),
                                             ("spec-input", spec_cases, "verify")):
            responses = []
            with Worker(work, engine, result / f"{filename}.json") as worker:
                for index, case in enumerate(inputs):
                    row = worker.request(operation, index)
                    verify_row(row, case, engine)
                    if filename == "spec-input":
                        row["expected"] = spec[index]["html"]
                        row["mismatch"] = CanonicalHTML(row["html"]).tokens != CanonicalHTML(row["expected"]).tokens
                    responses.append(row)
            write_json(result / f"{engine}-{filename}-outputs.json", responses)
            if filename == "catalog":
                rows = responses
            elif filename == "probes":
                check_switches(responses)
        verified[engine] = rows
        print(f"{engine}: workload, feature and spec outputs retained", flush=True)
    for engine in ("ferromark", "satteri"):
        with Worker(work, engine, result / "mdx-input.json") as worker:
            rows = [worker.request("mdx", index) for index in range(len(mdx))]
        write_json(result / f"{engine}-mdx-outputs.json", rows)
    for engine in ENGINES:
        folder = result / engine
        folder.mkdir()
        reviews = []
        for index, case in enumerate(corpus):
            outputs = {parser: verified[parser][index]["html"] for parser in ("ferromark", engine)}
            reviews.append({"case": case["case"], **workload_review(case["case"], outputs, parsers=set(outputs))})
        write_json(folder / "verification.json", reviews)
        selected = [r["case"] for r in reviews if r["comparable"] and (not args.case or r["case"] in args.case)]
        excluded = [r["case"] for r in reviews if not r["comparable"]]
        if args.case and set(args.case).intersection(excluded) and not args.verify_only:
            raise ValueError(f"Requested cases are ineligible for {engine}: {excluded}")
        metadata["pairs"][engine] = {"selected": selected, "eligible": sum(r["comparable"] for r in reviews),
                                    "total": len(corpus), "excluded": excluded}
        print(f"{engine}: admitted {metadata['pairs'][engine]['eligible']}/{len(corpus)}", flush=True)
    write_json(result / "metadata.json", metadata)
    if not args.verify_only:
        for engine in ENGINES:
            selected = metadata["pairs"][engine]["selected"]
            indices = [i for i, c in enumerate(corpus) if c["case"] in selected]
            runs = []
            for repeat in range(protocol["runs"]):
                samples = []
                warmups = []
                with ExitStack() as stack:
                    workers = {name: stack.enter_context(Worker(work, name, result / "catalog.json"))
                               for name in ("ferromark", engine)}
                    for index in indices:
                        for name, worker in workers.items():
                            row = worker.request("window", index, ms=protocol["warmup_ms"])
                            validate_window(row, corpus[index]["case"], name, protocol["warmup_ms"])
                            warmups.append(row)
                        for sample in range(protocol["samples"]):
                            order = ("ferromark", engine) if (sample + repeat) % 2 == 0 else (engine, "ferromark")
                            for name in order:
                                row = workers[name].request("window", index, ms=protocol["window_ms"])
                                validate_window(row, corpus[index]["case"], name, protocol["window_ms"])
                                samples.append({**row, "run": repeat, "sample": sample})
                folder = result / engine
                write_json(folder / f"warmups-{repeat}.json", warmups)
                write_json(folder / f"samples-{repeat}.json", samples)
                runs.append(samples)
                print(f"{engine}: completed run {repeat + 1}/{protocol['runs']}", flush=True)
            write_json(result / engine / "summary.json", summarize(runs, selected, engine, protocol,
                verified["ferromark"] + verified[engine]))
    validate_build(work)
    metadata["finished_unix"] = time.time()
    write_json(result / "metadata.json", metadata)
    print(result)


if __name__ == "__main__":
    main()
