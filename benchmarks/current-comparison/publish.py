#!/usr/bin/env python3
"""Archive measured evidence compactly and generate a report from raw data."""

import argparse
import csv
import gzip
import hashlib
import json
from pathlib import Path
import shutil

from run import canonical, summarize


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("results", type=Path)
    parser.add_argument("report", type=Path)
    args = parser.parse_args()
    rows = json.loads((args.results / "samples.json").read_text())
    config = json.loads((args.results / "run.json").read_text())
    verification = json.loads((args.results / "verification.json").read_text())
    corpus = json.loads((args.results / "corpus.json").read_text())
    build = json.loads((args.results / "build.json").read_text())
    summary = summarize(rows, config["groups"], verification)
    assert summary == json.loads((args.results / "summary.json").read_text())
    assert len(rows) == len(config["groups"]) * 3 * config["rounds"] * config["pairs_per_round"]
    for case in corpus["cases"]:
        assert hashlib.sha256(case["input"].encode()).hexdigest() == case["sha256"]
    for name, value in verification.items():
        left, right = value["outputs"]["main"], value["outputs"]["v2"]
        status = "exact" if left == right else "serialization-equivalent" if canonical(left) == canonical(right) else "different"
        assert status == value["status"], name
    args.report.mkdir(parents=True, exist_ok=False)
    for name in ("samples.json", "verification.json", "corpus.json"):
        data = (args.results / name).read_bytes()
        (args.report / f"{name}.gz").write_bytes(gzip.compress(data, mtime=0))
    for name in ("run.json", "build.json", "main.Cargo.lock", "v2.Cargo.lock", "summary.json"):
        shutil.copyfile(args.results / name, args.report / name)
    with (args.report / "summary.csv").open("w", newline="") as file:
        writer = csv.DictWriter(file, fieldnames=list(summary[0]))
        writer.writeheader()
        writer.writerows(summary)
    harness = Path(__file__).parent
    hashes = {path.name: hashlib.sha256(path.read_bytes()).hexdigest() for path in harness.iterdir() if path.suffix in {".rs", ".py"}}
    assert hashes["worker.rs"] == build["worker_sha256"]
    (args.report / "harness-hashes.json").write_text(json.dumps(hashes, indent=2) + "\n")
    index = {(r["group"], r["mode"]): r for r in summary}
    status_counts = {status: sum(v["status"] == status for v in verification.values()) for status in ("exact", "serialization-equivalent", "different")}
    lines = [
        "# Ferromark main vs. the initial v2 core", "",
        f"Measured on {config['host_before']['time_utc']} on {config['host_before']['cpu']}.", "",
        "The rotating CommonMark mix is effectively tied. The table-heavy GFM mix favors current main; v2 wins several plain-text, fenced-code, and escaping diagnostics. This is a workload-dependent baseline, not a universal ranking.", "",
        "## Rotating document groups", "",
        "Times are microseconds per document. Ratios are paired main-time/v2-time medians: above 1 means v2 is faster. Each group visits every document once per traversal; inputs have distinct contents and allocations.", "",
        "| Group | Lifecycle | Main µs | v2 µs | v2 speed ratio | Paired range |", "| --- | --- | ---: | ---: | ---: | --- |",
    ]
    for group in ("rotating-commonmark", "rotating-gfm"):
        for mode in ("fresh", "owned", "reuse"):
            r = index[group, mode]
            lines.append(f"| {group} ({r['documents_per_iteration']} docs) | {mode} | {r['main_ns_per_document']/1000:.3f} | {r['v2_ns_per_document']/1000:.3f} | {r['main_over_v2_median']:.3f}× | {r['main_over_v2_min']:.3f}–{r['main_over_v2_max']:.3f}× |")
    lines += ["", "## Individual cases", "", "Each cell is main µs / v2 µs, followed by the paired v2 speed ratio. The README has different heading IDs and is diagnostic only; it is excluded from both rotating groups.", "", "| Case | Input bytes | Output check | Fresh | Owned output | Full reuse |", "| --- | ---: | --- | --- | --- | --- |"]
    for case in corpus["cases"]:
        cells = []
        for mode in ("fresh", "owned", "reuse"):
            r = index[case["name"], mode]
            cells.append(f"{r['main_ns_per_document']/1000:.3f} / {r['v2_ns_per_document']/1000:.3f} ({r['main_over_v2_median']:.2f}×)")
        lines.append(f"| {case['name']} | {case['byte_count']:,} | {verification[case['name']]['status']} | " + " | ".join(cells) + " |")
    lines += ["", "## Validation and boundaries", "",
        f"- Output admission: {status_counts['exact']} byte-identical cases, {status_counts['serialization-equivalent']} equivalent HTML serializations, {status_counts['different']} diagnostic mismatch. All lifecycle outputs agree within each engine.",
        "- The README differs substantively in nine heading IDs (for example, `nodejs` versus `node-js`). These affect anchors and are not normalized away. Other observed differences are harmless entity/whitespace/checkbox serialization.",
        "- Both engines use trusted CommonMark/GFM with matched heading IDs, links, tagfilter, and hard-break options; optional footnotes and MDX are outside this comparison. This does not compare default security policies.",
        "- Fresh mode includes v2's owned renderer-option cloning and both libraries' actual allocation strategies. Reuse mode retains scratch/output; the parser/prepass and AST construction still run on every document.",
        "- Owned strings are freed, and v2 documents/arenas are dropped or reset, inside the timer. I/O, normalization, verification, and process startup are excluded.",
        f"- {config['rounds']} independent process rounds × {config['pairs_per_round']} paired windows, at least {config['window_ms']} ms each; {len(rows)} pairs total. Orders alternate and jobs are deterministically shuffled. Per-round medians and every raw window remain in the data.",
        "- Rust 1.95.0 / LLVM 22.1.2, generic AArch64, system allocator, opt-level 3, fat LTO, one codegen unit, panic abort. Each engine retains its own locked dependency versions/checksums.",
        "- Local Apple Silicon results, with AC attached. Load/power observations are archived; thermal telemetry was unavailable. Small single-digit differences should not be treated as portable wins.",
        "- Inputs include synthetic repeated diagnostics and existing synthetic fixtures. The one real README remains separately visible. This selection is not a production traffic distribution.",
        "", "## Next optimization candidates", "",
        "The measured gaps prioritize nested-list handling, GFM tables, and reference-heavy documents for investigation. Current Ferromark is a concrete donor candidate in those areas. These timings identify workloads, not proven internal bottlenecks; profile before choosing the code to port.",
        "", "## Reproduce and inspect", "",
        f"- Main: `{build['engines']['main']['revision']}` (Ferromark 0.9.0).",
        f"- v2 core: `{build['engines']['v2']['revision']}` (2.0.0-dev.0).",
        "- [Harness and exact methodology](../../../benchmarks/current-comparison/README.md).",
        "- [Summary CSV](summary.csv), [summary JSON](summary.json), and [run configuration](run.json).",
        "- [Frozen inputs/provenance](corpus.json.gz), [original HTML outputs](verification.json.gz), and [raw paired samples](samples.json.gz).",
        "- [Build/binary/lock hashes](build.json) and [harness hashes](harness-hashes.json).",
        "", "The benchmark harness and this report are local additions. Neither measured parser implementation was modified.", "",
    ]
    (args.report / "README.md").write_text("\n".join(lines))
    print(args.report / "README.md")


if __name__ == "__main__":
    main()
