#!/usr/bin/env python3
"""Run a frozen broad corpus through the unchanged current-comparison workers."""

import argparse
from collections import Counter
import difflib
import hashlib
import importlib.util
import json
from pathlib import Path
import random
import shutil
import struct
from urllib.parse import quote, urlsplit

PATH = Path(__file__).resolve().parents[1] / "current-comparison/run.py"
SPEC = importlib.util.spec_from_file_location("comparison", PATH)
BASE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(BASE)
MODES, ENGINES = BASE.MODES, BASE.ENGINES
HEADINGS = {f"h{n}" for n in range(1, 7)}


def unicode_url(value):
    """Normalize UTF-8 URL spelling only; never decode reserved characters."""
    try:
        parts = urlsplit(value)
    except ValueError:
        return value
    if parts.scheme not in {"", "http", "https"} or not parts.netloc.isascii():
        return value
    return "".join(quote(char, safe="") if ord(char) > 127 else char for char in value)


class HTML(BASE.CanonicalHTML):
    def handle_starttag(self, tag, attrs):
        attrs = [(key, unicode_url(value) if key in {"href", "src"} and isinstance(value, str) else value) for key, value in attrs]
        super().handle_starttag(tag, attrs)

    def handle_startendtag(self, tag, attrs):
        # In HTML a slash on a non-void element does not close that element.
        self.handle_starttag(tag, attrs)


def canonical(text):
    parser = HTML()
    parser.feed(text)
    result = []
    for index, token in enumerate(parser.tokens):
        if token[0] == "data" and not token[2] and not token[1].strip() and "\n" in token[1]:
            neighbors = parser.tokens[max(0, index - 1):index] + parser.tokens[index + 1:index + 2]
            if neighbors and all(n[0] in {"start", "end"} and n[1] in BASE.BLOCKS for n in neighbors):
                continue
        result.append(token)
    return result


def checked_bench(worker, budget_ns, output_bytes):
    result = worker.bench(budget_ns)
    expected = (result["iterations"] * output_bytes) % (1 << (8 * struct.calcsize("P")))
    assert result["checksum"] == expected, ("timed output length changed", result, expected)
    return result


def without_heading_ids(tokens):
    # Diagnostic classification only. This does not grant comparable admission:
    # different heading IDs break incoming fragment links.
    return [(t[0], t[1], tuple((k, v) for k, v in t[2] if k != "id"))
            if t[0] == "start" and t[1] in HEADINGS else t for t in tokens]


def classify(main, v2):
    if main == v2:
        return "exact"
    left, right = canonical(main), canonical(v2)
    if left == right:
        return "serialization-equivalent"
    if without_heading_ids(left) == without_heading_ids(right):
        return "heading-id-only"
    return "different"


def groups_for(cases):
    groups = [{"name": c["name"], "profile": c["profile"], "cases": [c["name"]]} for c in cases]
    # Fixed membership independent of observed output or performance. Mismatching
    # collections remain diagnostic; no case is silently dropped from a batch.
    for collection in sorted({c["collection"] for c in cases}):
        for profile in sorted({c["profile"] for c in cases if c["collection"] == collection}):
            members = [c["name"] for c in cases if c["collection"] == collection and c["profile"] == profile]
            groups.append({"name": f"rotating-{collection}-{profile}", "profile": profile, "cases": members})
    return groups


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
    assert min(args.rounds, args.pairs, args.window_ms) > 0
    args.output.mkdir(parents=True, exist_ok=False)
    build = json.loads((args.build / "build.json").read_text())
    assert BASE.digest(PATH.with_name("worker.rs")) == build["worker_sha256"]
    cases = json.loads(args.corpus.read_text())["cases"]
    assert len({c["name"] for c in cases}) == len(cases)
    for engine in ENGINES:
        assert BASE.digest(Path(build["engines"][engine]["binary"])) == build["engines"][engine]["binary_sha256"]
        assert BASE.digest(args.build / engine / "Cargo.lock") == build["engines"][engine]["lock_sha256"]
        shutil.copyfile(args.build / engine / "Cargo.lock", args.output / f"{engine}.Cargo.lock")
    for filename, source in (("corpus.json", args.corpus), ("build.json", args.build / "build.json")):
        shutil.copyfile(source, args.output / filename)
    inputs = args.output / "inputs"
    inputs.mkdir()
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
                worker = BASE.Worker(build["engines"][engine]["binary"], case["profile"], mode, [inputs / f'{case["name"]}.md'])
                try:
                    outputs[mode][engine] = worker.verify()[0]
                finally:
                    worker.close()
        for engine in ENGINES:
            assert len({outputs[m][engine] for m in MODES}) == 1, (case["name"], engine, "lifecycle changed output")
        first = outputs["fresh"]
        status = classify(first["main"], first["v2"])
        diff = list(difflib.unified_diff([repr(t) for t in canonical(first["main"])], [repr(t) for t in canonical(first["v2"])], fromfile="main", tofile="v2", n=2)) if status not in {"exact", "serialization-equivalent"} else []
        verification[case["name"]] = {"status": status, "outputs": first, "all_lifecycles_equal": True, "canonical_diff": diff}
    (args.output / "verification.json").write_text(json.dumps(verification, indent=2, ensure_ascii=False) + "\n")
    print("Output verification:", dict(Counter(v["status"] for v in verification.values())), flush=True)
    groups = groups_for(cases)
    # Verify transitions between distinct documents and complete repeated cycles,
    # not just isolated documents, in every lifecycle before any timing.
    for group in groups:
        if not group["name"].startswith("rotating-"):
            continue
        paths = [inputs / f"{name}.md" for name in group["cases"]]
        for mode in MODES:
            for engine in ENGINES:
                worker = BASE.Worker(build["engines"][engine]["binary"], group["profile"], mode, paths)
                expected = [verification[name]["outputs"][engine] for name in group["cases"]]
                try:
                    for _ in range(2):
                        assert worker.verify() == expected, ("rotating output changed", group["name"], mode, engine)
                finally:
                    worker.close()
    print("Repeated rotating output verified for all lifecycles.", flush=True)
    if args.verify_only:
        return
    config = {"rounds": args.rounds, "pairs_per_round": args.pairs, "window_ms": args.window_ms,
              "warmup_ms": 75, "seed": 20260914, "groups": groups, "host_before": BASE.host(),
              "corpus_sha256": BASE.digest(args.corpus), "runner_sha256": BASE.digest(Path(__file__)),
              "shared_runner_sha256": BASE.digest(PATH), "observations": []}
    config["verification"] = "Isolated modes and two rotating cycles byte-checked before timing; each timing worker verified before and after its windows; output-length checksum checked for every timed and warmup window."
    rows = []
    rng = random.Random(config["seed"])
    for round_index in range(args.rounds):
        jobs = [(group, mode) for group in groups for mode in MODES]
        rng.shuffle(jobs)
        observation = {"round": round_index, "before": BASE.host()}
        config["observations"].append(observation)
        print(f"Round {round_index + 1}/{args.rounds}: {len(jobs)} groups/lifecycles", flush=True)
        for index, (group, mode) in enumerate(jobs):
            paths = [inputs / f"{name}.md" for name in group["cases"]]
            workers = {e: BASE.Worker(build["engines"][e]["binary"], group["profile"], mode, paths) for e in ENGINES}
            expected = {e: [verification[name]["outputs"][e] for name in group["cases"]] for e in ENGINES}
            output_bytes = {e: sum(len(text.encode("utf-8")) for text in expected[e]) for e in ENGINES}
            try:
                for engine, worker in workers.items():
                    assert worker.verify() == expected[engine], (group["name"], mode, engine)
                    checked_bench(worker, config["warmup_ms"] * 1_000_000, output_bytes[engine])
                for pair in range(args.pairs):
                    order = ENGINES if (pair + round_index) % 2 == 0 else tuple(reversed(ENGINES))
                    row = {"round": round_index, "pair": pair, "group": group["name"], "mode": mode, "order": order}
                    for engine in order:
                        row[engine] = checked_bench(workers[engine], args.window_ms * 1_000_000, output_bytes[engine])
                    rows.append(row)
                for engine, worker in workers.items():
                    assert worker.verify() == expected[engine], ("post-timing output changed", group["name"], mode, engine)
            finally:
                for worker in workers.values():
                    worker.close()
            if (index + 1) % 30 == 0:
                print(f"  {index + 1}/{len(jobs)} completed", flush=True)
        observation["after"] = BASE.host()
        (args.output / "samples.json").write_text(json.dumps(rows, indent=2) + "\n")
        (args.output / "run.json").write_text(json.dumps(config, indent=2) + "\n")
    config["host_after"] = BASE.host()
    (args.output / "run.json").write_text(json.dumps(config, indent=2) + "\n")
    (args.output / "summary.json").write_text(json.dumps(BASE.summarize(rows, groups, verification), indent=2) + "\n")
    print(args.output / "summary.json", flush=True)


if __name__ == "__main__":
    main()
