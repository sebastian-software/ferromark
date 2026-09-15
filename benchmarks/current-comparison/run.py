#!/usr/bin/env python3
"""Verify output first, then collect paired, alternating timing windows."""

import argparse
import hashlib
from html.parser import HTMLParser
import json
import os
from pathlib import Path
import platform
import random
import shutil
import statistics
import subprocess
import time

MODES = ("fresh", "owned", "reuse")
ENGINES = ("main", "v2")
BLOCKS = {"p", "div", "section", "ul", "ol", "li", "blockquote", "table", "thead", "tbody", "tr", "th", "td", "pre", "hr", "h1", "h2", "h3", "h4", "h5", "h6"}
VOID = {"area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param", "source", "track", "wbr"}


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


class CanonicalHTML(HTMLParser):
    """Only normalize HTML serialization, never URLs, IDs, classes or content."""

    def __init__(self):
        super().__init__(convert_charrefs=True)
        self.tokens = []
        self.literal = 0

    def handle_starttag(self, tag, attrs):
        if tag in {"pre", "code", "script", "style", "textarea"}:
            self.literal += 1
        attrs = tuple(sorted((key, True if key in {"checked", "disabled"} else value) for key, value in attrs))
        self.tokens.append(("start", tag, attrs))

    def handle_startendtag(self, tag, attrs):
        self.handle_starttag(tag, attrs)
        if tag not in VOID:
            self.handle_endtag(tag)

    def handle_endtag(self, tag):
        self.tokens.append(("end", tag))
        if tag in {"pre", "code", "script", "style", "textarea"}:
            self.literal -= 1

    def handle_data(self, data):
        self.tokens.append(("data", data, self.literal > 0))

    def handle_comment(self, data):
        self.tokens.append(("comment", data))

    def handle_decl(self, decl):
        self.tokens.append(("declaration", decl))


def canonical(text):
    parser = CanonicalHTML()
    parser.feed(text)
    result = []
    for index, token in enumerate(parser.tokens):
        if token[0] == "data" and not token[2] and not token[1].strip() and "\n" in token[1]:
            neighbors = parser.tokens[max(0, index - 1):index] + parser.tokens[index + 1:index + 2]
            if neighbors and all(n[0] in {"start", "end"} and n[1] in BLOCKS for n in neighbors):
                continue
        result.append(token)
    return result


class Worker:
    def __init__(self, binary, profile, mode, paths):
        self.process = subprocess.Popen(
            [binary, profile, mode, *map(str, paths)], text=True,
            stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
        )

    def command(self, text):
        self.process.stdin.write(text + "\n")
        self.process.stdin.flush()

    def line(self):
        line = self.process.stdout.readline().strip()
        if not line:
            raise RuntimeError(self.process.stderr.read())
        return line

    def verify(self):
        self.command("verify")
        outputs = []
        while (line := self.line()) != "done":
            _, index, value = line.split(" ", 2)
            assert int(index) == len(outputs)
            outputs.append(bytes.fromhex(value).decode("utf-8"))
        return outputs

    def bench(self, budget_ns):
        self.command(f"bench {budget_ns}")
        kind, iterations, elapsed, checksum = self.line().split()
        assert kind == "timing"
        return {"iterations": int(iterations), "elapsed_ns": int(elapsed), "checksum": int(checksum)}

    def close(self):
        self.command("quit")
        self.process.communicate(timeout=10)
        assert self.process.returncode == 0


def host():
    def probe(args):
        try:
            return subprocess.check_output(args, text=True, stderr=subprocess.STDOUT, timeout=5).strip()
        except (OSError, subprocess.SubprocessError) as error:
            return str(error)
    return {
        "time_utc": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "platform": platform.platform(), "load_average": os.getloadavg(),
        "cpu": os.environ.get("BENCH_CPU") or probe(["sysctl", "-n", "machdep.cpu.brand_string"]),
        "power": probe(["pmset", "-g", "batt"]),
        "thermal": probe(["pmset", "-g", "therm"]),
    }


def summarize(rows, groups, verification):
    summary = []
    for group in groups:
        for mode in MODES:
            selected = [r for r in rows if r["group"] == group["name"] and r["mode"] == mode]
            if not selected:
                continue
            samples = {engine: [r[engine]["elapsed_ns"] / r[engine]["iterations"] / len(group["cases"]) for r in selected] for engine in ENGINES}
            ratios = [r["main"]["elapsed_ns"] / r["main"]["iterations"] / (r["v2"]["elapsed_ns"] / r["v2"]["iterations"]) for r in selected]
            comparable = all(verification[c]["status"] in {"exact", "serialization-equivalent"} for c in group["cases"])
            summary.append({
                "group": group["name"], "mode": mode, "profile": group["profile"],
                "documents_per_iteration": len(group["cases"]),
                "comparable_output": comparable,
                "main_ns_per_document": statistics.median(samples["main"]),
                "v2_ns_per_document": statistics.median(samples["v2"]),
                "main_over_v2_median": statistics.median(ratios),
                "main_over_v2_min": min(ratios), "main_over_v2_max": max(ratios),
                "pairs": len(ratios),
                "round_medians": [statistics.median([ratios[i] for i, r in enumerate(selected) if r["round"] == n]) for n in sorted({r["round"] for r in selected})],
            })
    return summary


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("build", type=Path)
    parser.add_argument("corpus", type=Path)
    parser.add_argument("output", type=Path)
    parser.add_argument("--verify-only", action="store_true")
    parser.add_argument("--rounds", type=int, default=3)
    parser.add_argument("--pairs", type=int, default=3)
    parser.add_argument("--window-ms", type=int, default=75)
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=False)
    build = json.loads((args.build / "build.json").read_text())
    cases = json.loads(args.corpus.read_text())["cases"]
    for engine in ENGINES:
        assert digest(Path(build["engines"][engine]["binary"])) == build["engines"][engine]["binary_sha256"]
        shutil.copyfile(args.build / engine / "Cargo.lock", args.output / f"{engine}.Cargo.lock")
    shutil.copyfile(args.corpus, args.output / "corpus.json")
    shutil.copyfile(args.build / "build.json", args.output / "build.json")
    inputs = args.output / "inputs"
    inputs.mkdir()
    by_name = {c["name"]: c for c in cases}
    for case in cases:
        raw = case["input"].encode()
        assert len(raw) == case["byte_count"]
        assert hashlib.sha256(raw).hexdigest() == case["sha256"]
        (inputs / f'{case["name"]}.md').write_bytes(raw)
    verification = {}
    for case in cases:
        outputs = {}
        for mode in MODES:
            outputs[mode] = {}
            for engine in ENGINES:
                worker = Worker(build["engines"][engine]["binary"], case["profile"], mode, [inputs / f'{case["name"]}.md'])
                outputs[mode][engine] = worker.verify()[0]
                worker.close()
        for engine in ENGINES:
            assert len({outputs[m][engine] for m in MODES}) == 1, (case["name"], engine, "lifecycle changed output")
        first = outputs["fresh"]
        status = "exact" if first["main"] == first["v2"] else "serialization-equivalent" if canonical(first["main"]) == canonical(first["v2"]) else "different"
        verification[case["name"]] = {"status": status, "outputs": first, "all_lifecycles_equal": True}
    (args.output / "verification.json").write_text(json.dumps(verification, indent=2, ensure_ascii=False) + "\n")
    print("Output verification:", {s: sum(v["status"] == s for v in verification.values()) for s in ("exact", "serialization-equivalent", "different")}, flush=True)
    if args.verify_only:
        return
    groups = [{"name": c["name"], "profile": c["profile"], "cases": [c["name"]]} for c in cases]
    for profile in ("commonmark", "gfm"):
        groups.append({"name": f"rotating-{profile}", "profile": profile, "cases": [c["name"] for c in cases if c["profile"] == profile and c["origin"]["kind"] != "repository-document"]})
    config = {"rounds": args.rounds, "pairs_per_round": args.pairs, "window_ms": args.window_ms, "warmup_ms": 75, "seed": 20260913, "groups": groups, "host_before": host()}
    rows = []
    observations = []
    rng = random.Random(config["seed"])
    for round_index in range(args.rounds):
        jobs = [(group, mode) for group in groups for mode in MODES]
        rng.shuffle(jobs)
        observations.append({"round": round_index, "before": host()})
        print(f"Round {round_index + 1}/{args.rounds}: {len(jobs)} groups/lifecycles", flush=True)
        for group, mode in jobs:
            paths = [inputs / f"{name}.md" for name in group["cases"]]
            workers = {e: Worker(build["engines"][e]["binary"], group["profile"], mode, paths) for e in ENGINES}
            for worker in workers.values():
                worker.bench(config["warmup_ms"] * 1_000_000)
            for pair in range(args.pairs):
                order = ENGINES if (pair + round_index) % 2 == 0 else tuple(reversed(ENGINES))
                row = {"round": round_index, "pair": pair, "group": group["name"], "mode": mode, "order": order}
                for engine in order:
                    row[engine] = workers[engine].bench(args.window_ms * 1_000_000)
                rows.append(row)
            for worker in workers.values():
                worker.close()
        observations[-1]["after"] = host()
        (args.output / "samples.json").write_text(json.dumps(rows, indent=2) + "\n")
    config["host_after"] = host()
    config["observations"] = observations
    (args.output / "run.json").write_text(json.dumps(config, indent=2) + "\n")
    (args.output / "summary.json").write_text(json.dumps(summarize(rows, groups, verification), indent=2) + "\n")
    print(args.output / "summary.json", flush=True)


if __name__ == "__main__":
    main()
