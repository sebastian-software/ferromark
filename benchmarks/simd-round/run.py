#!/usr/bin/env python3
"""Verify and measure baseline-v2 versus candidate-v2 on a frozen corpus."""

import argparse
import csv
import hashlib
import json
import os
from pathlib import Path
import platform
import random
import re
import statistics
import subprocess
import time

ENGINES = ("baseline", "candidate")
DEFAULT_MODES = ("fresh", "reuse", "parse", "render")


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def load_corpus(path):
    if path.suffix == ".gz":
        import gzip
        return json.loads(gzip.decompress(path.read_bytes()))
    return json.loads(path.read_text())


class Worker:
    def __init__(self, binary, profile, mode, paths):
        self.process = subprocess.Popen(
            [binary, profile, mode, *map(str, paths)],
            text=True,
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )

    def command(self, text):
        self.process.stdin.write(text + "\n")
        self.process.stdin.flush()

    def line(self):
        line = self.process.stdout.readline()
        if not line:
            error = self.process.stderr.read()
            raise RuntimeError(error or "worker exited without output")
        return line.rstrip("\n")

    def verify(self):
        self.command("verify")
        results = []
        while True:
            line = self.line()
            if line == "done":
                return results
            fields = line.split(" ", 5)
            if len(fields) != 6 or fields[0] != "result":
                raise RuntimeError(f"invalid verify line: {line!r}")
            _, index, html, ast, arena_capacity, children = fields
            result = {
                "index": int(index),
                "html": bytes.fromhex(html).decode("utf-8"),
                "ast_debug": bytes.fromhex(ast).decode("utf-8"),
                "arena_capacity_bytes": int(arena_capacity),
                "children": int(children),
            }
            if result["index"] != len(results):
                raise RuntimeError("worker returned results out of order")
            results.append(result)

    def bench(self, budget_ns):
        self.command(f"bench {budget_ns}")
        fields = self.line().split()
        if len(fields) != 4 or fields[0] != "timing":
            raise RuntimeError(f"invalid timing line: {fields}")
        _, iterations, elapsed_ns, checksum = fields
        return {
            "iterations": int(iterations),
            "elapsed_ns": int(elapsed_ns),
            "checksum": int(checksum),
        }

    def close(self):
        try:
            self.command("quit")
            self.process.communicate(timeout=10)
        finally:
            if self.process.poll() is None:
                self.process.kill()
        if self.process.returncode != 0:
            raise RuntimeError(f"worker exited with {self.process.returncode}")


def host():
    def probe(args):
        try:
            return subprocess.check_output(args, text=True, stderr=subprocess.STDOUT, timeout=5).strip()
        except (OSError, subprocess.SubprocessError) as error:
            return str(error)

    return {
        "time_utc": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "platform": platform.platform(),
        "load_average": os.getloadavg(),
        "cpu": os.environ.get("BENCH_CPU") or probe(["sysctl", "-n", "machdep.cpu.brand_string"]),
        "power": probe(["pmset", "-g", "batt"]),
        "thermal": probe(["pmset", "-g", "therm"]),
    }


def checked_bench(worker, budget_ns, metric_per_iteration):
    result = worker.bench(budget_ns)
    expected = (result["iterations"] * metric_per_iteration) % (1 << 64)
    if result["checksum"] != expected:
        raise AssertionError((result, expected, metric_per_iteration))
    return result


def verify_case(build, input_path, case, modes):
    outputs = {mode: {} for mode in modes}
    for mode in modes:
        for engine in ENGINES:
            worker = Worker(build["engines"][engine]["binary"], case["profile"], mode, [input_path])
            try:
                result = worker.verify()
            finally:
                worker.close()
            if len(result) != 1:
                raise AssertionError((case["name"], mode, engine, len(result)))
            outputs[mode][engine] = result[0]
    for engine in ENGINES:
        first = outputs[modes[0]][engine]
        for mode in modes[1:]:
            other = outputs[mode][engine]
            if (other["html"], other["ast_debug"]) != (first["html"], first["ast_debug"]):
                raise AssertionError((case["name"], engine, "lifecycle output changed", mode))
    for mode in modes:
        left, right = outputs[mode]["baseline"], outputs[mode]["candidate"]
        if any(left[key] != right[key] for key in ("html", "ast_debug", "children", "arena_capacity_bytes")):
            raise AssertionError((case["name"], mode, "baseline/candidate output changed"))
    first = outputs[modes[0]]["baseline"]
    return {
        "status": "exact",
        "html": first["html"],
        "ast_debug": first["ast_debug"],
        "modes": outputs,
    }


def assert_live_result(actual, expected, label):
    if len(actual) != 1:
        raise AssertionError((label, "unexpected document count", len(actual)))
    for key in ("html", "ast_debug", "children"):
        if actual[0][key] != expected[key]:
            raise AssertionError((label, key, actual[0][key], expected[key]))


def assert_same_capacity(capacities, label):
    if capacities["baseline"] != capacities["candidate"]:
        raise AssertionError((label, "baseline/candidate arena capacity changed", capacities))


def metric(verification, mode, engine):
    values = [verification["modes"][mode][engine]]
    if mode == "parse":
        return sum(value["children"] for value in values)
    return sum(len(value["html"].encode("utf-8")) for value in values)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("build", type=Path)
    parser.add_argument("corpus", type=Path)
    parser.add_argument("output", type=Path)
    parser.add_argument("--rounds", type=int, default=3)
    parser.add_argument("--pairs", type=int, default=3)
    parser.add_argument("--window-ms", type=int, default=50)
    parser.add_argument("--modes", nargs="+", choices=DEFAULT_MODES, default=list(DEFAULT_MODES))
    parser.add_argument("--filter", dest="case_filter")
    args = parser.parse_args()
    if min(args.rounds, args.pairs, args.window_ms) <= 0:
        parser.error("rounds, pairs, and window-ms must be positive")
    args.output.mkdir(parents=True, exist_ok=False)
    build_data = json.loads((args.build / "build.json").read_text())
    if digest(Path(__file__).with_name("worker.rs")) != build_data["worker_sha256"]:
        raise SystemExit("worker.rs changed after prepare.py")
    corpus = load_corpus(args.corpus)
    cases = corpus["cases"]
    if args.case_filter:
        pattern = re.compile(args.case_filter)
        cases = [case for case in cases if pattern.search(case["name"])]
    if not cases:
        raise SystemExit("case filter selected no cases")
    for engine in ENGINES:
        info = build_data["engines"][engine]
        if digest(Path(info["binary"])) != info["binary_sha256"]:
            raise SystemExit(f"{engine} binary changed after prepare.py")
    (args.output / "build.json").write_text(json.dumps(build_data, indent=2) + "\n")
    (args.output / "corpus.json").write_text(json.dumps(corpus, indent=2, ensure_ascii=False) + "\n")
    inputs = args.output / "inputs"
    inputs.mkdir()
    input_paths = {}
    for case in cases:
        raw = case["input"].encode("utf-8")
        if len(raw) != case["byte_count"] or hashlib.sha256(raw).hexdigest() != case["sha256"]:
            raise AssertionError((case["name"], "corpus integrity"))
        path = inputs / f'{case["name"]}.md'
        path.write_bytes(raw)
        input_paths[case["name"]] = path
    verification = {}
    print(f"Verifying {len(cases)} cases across {len(args.modes)} stages...", flush=True)
    for index, case in enumerate(cases, 1):
        verification[case["name"]] = verify_case(build_data, input_paths[case["name"]], case, args.modes)
        if index % 20 == 0:
            print(f"  {index}/{len(cases)} verified", flush=True)
    (args.output / "verification.json").write_text(
        json.dumps(verification, indent=2, ensure_ascii=False) + "\n"
    )
    print("Output verification: exact HTML and AST Debug for every case/stage.", flush=True)

    seed = 20260914
    config = {
        "rounds": args.rounds,
        "pairs_per_round": args.pairs,
        "window_ms": args.window_ms,
        "warmup_ms": max(5, min(50, args.window_ms)),
        "seed": seed,
        "modes": args.modes,
        "case_filter": args.case_filter,
        "cases": [case["name"] for case in cases],
        "corpus_sha256": digest(args.output / "corpus.json"),
        "runner_sha256": digest(Path(__file__)),
        "worker_sha256": build_data["worker_sha256"],
        "host_before": host(),
    }
    rows = []
    capacity_checks = []
    jobs = [(case, mode) for case in cases for mode in args.modes]
    rng = random.Random(seed)
    for round_index in range(args.rounds):
        rng.shuffle(jobs)
        print(f"Round {round_index + 1}/{args.rounds}: {len(jobs)} cases/stages", flush=True)
        for job_index, (case, mode) in enumerate(jobs, 1):
            workers = {
                engine: Worker(
                    build_data["engines"][engine]["binary"],
                    case["profile"],
                    mode,
                    [input_paths[case["name"]]],
                )
                for engine in ENGINES
            }
            expected = {engine: metric(verification[case["name"]], mode, engine) for engine in ENGINES}
            try:
                capacities_before = {}
                for engine in ENGINES:
                    checked = workers[engine].verify()
                    capacities_before[engine] = checked[0]["arena_capacity_bytes"]
                    assert_live_result(
                        checked,
                        verification[case["name"]]["modes"][mode][engine],
                        (case["name"], mode, engine, "pre-timing output changed"),
                    )
                    checked_bench(workers[engine], config["warmup_ms"] * 1_000_000, expected[engine])
                assert_same_capacity(capacities_before, (case["name"], mode, "before"))
                for pair in range(args.pairs):
                    order = ENGINES if (pair + round_index) % 2 == 0 else tuple(reversed(ENGINES))
                    row = {
                        "round": round_index,
                        "pair": pair,
                        "case": case["name"],
                        "profile": case["profile"],
                        "mode": mode,
                        "order": list(order),
                    }
                    for engine in order:
                        row[engine] = checked_bench(
                            workers[engine], args.window_ms * 1_000_000, expected[engine]
                        )
                    rows.append(row)
                capacities_after = {}
                for engine in ENGINES:
                    checked = workers[engine].verify()
                    capacities_after[engine] = checked[0]["arena_capacity_bytes"]
                    assert_live_result(
                        checked,
                        verification[case["name"]]["modes"][mode][engine],
                        (case["name"], mode, engine, "post-timing output changed"),
                    )
                assert_same_capacity(capacities_after, (case["name"], mode, "after"))
                capacity_checks.append({
                    "round": round_index, "case": case["name"], "mode": mode,
                    "before": capacities_before, "after": capacities_after,
                })
            finally:
                for worker in workers.values():
                    worker.close()
            if job_index % 20 == 0:
                print(f"  {job_index}/{len(jobs)} completed", flush=True)
    config["host_after"] = host()
    (args.output / "run.json").write_text(json.dumps(config, indent=2) + "\n")
    (args.output / "samples.json").write_text(json.dumps(rows, indent=2) + "\n")
    (args.output / "arena-capacities.json").write_text(json.dumps(capacity_checks, indent=2) + "\n")
    summary = []
    for case in cases:
        for mode in args.modes:
            selected = [row for row in rows if row["case"] == case["name"] and row["mode"] == mode]
            ratios = [
                row["baseline"]["elapsed_ns"] / row["baseline"]["iterations"]
                / (row["candidate"]["elapsed_ns"] / row["candidate"]["iterations"])
                for row in selected
            ]
            summary.append({
                "case": case["name"],
                "profile": case["profile"],
                "mode": mode,
                "baseline_ns_per_document": statistics.median(
                    row["baseline"]["elapsed_ns"] / row["baseline"]["iterations"] for row in selected
                ),
                "candidate_ns_per_document": statistics.median(
                    row["candidate"]["elapsed_ns"] / row["candidate"]["iterations"] for row in selected
                ),
                "baseline_over_candidate_median": statistics.median(ratios),
                "baseline_over_candidate_min": min(ratios),
                "baseline_over_candidate_max": max(ratios),
                "pairs": len(ratios),
                "round_medians": [
                    statistics.median(
                        row["baseline"]["elapsed_ns"] / row["baseline"]["iterations"]
                        / (row["candidate"]["elapsed_ns"] / row["candidate"]["iterations"])
                        for row in selected
                        if row["round"] == round_index
                    )
                    for round_index in range(args.rounds)
                ],
            })
    (args.output / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
    with (args.output / "summary.csv").open("w", newline="") as stream:
        writer = csv.DictWriter(stream, fieldnames=list(summary[0]))
        writer.writeheader()
        writer.writerows(summary)
    print(args.output / "summary.json", flush=True)


if __name__ == "__main__":
    main()
