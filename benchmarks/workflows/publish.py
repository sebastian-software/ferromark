#!/usr/bin/env python3
"""Derive workflow tables from complete, checked runtime and heap evidence."""
import argparse
import hashlib
import json
import math
import re
from pathlib import Path
import statistics
import tomllib

from support import HERE, ROOT, PROTOCOL, VARIANTS, group, read_json, review, sha

DEFAULT = ROOT / "docs/reports/2026-09-13-practical-workflows"
START = "<!-- workflow-benchmarks:start -->"
END = "<!-- workflow-benchmarks:end -->"
LABELS = {
    "preview-fresh": "Fresh `to_html` calls", "preview-reuse": "Retained `Renderer`",
    "guide-metadata": "`parse`: HTML + front matter + headings",
    "ferromark-stream": "Ferromark · release each page", "ferromark-retain": "Ferromark · keep all pages",
    "pulldown-stream": "pulldown-cmark · release each page", "pulldown-retain": "pulldown-cmark · keep all pages",
    "comrak-stream": "Comrak · release each page", "comrak-retain": "Comrak · keep all pages",
}


def require(condition, message):
    if not condition:
        raise ValueError(message)


def summarize(corpus, outputs, windows, memory, warmups, protocol):
    require(protocol == PROTOCOL, "Publication protocol changed")
    require(set(outputs) == set(VARIANTS), "Missing variant outputs")
    require(len(windows) == len(VARIANTS) * 3 * 80, "Incomplete timing windows")
    require(len(memory) == len(VARIANTS) * 3 * 10, "Incomplete memory observations")
    require(len(warmups) == len(VARIANTS) * 3, "Incomplete warmup observations")
    require({(r["round"], r["window"], r["variant"]) for r in windows} ==
            {(r, w, v) for r in range(3) for w in range(80) for v in VARIANTS}, "Duplicate or missing window")
    require({(r["round"], r["observation"], r["variant"]) for r in memory} ==
            {(r, i, v) for r in range(3) for i in range(10) for v in VARIANTS}, "Duplicate or missing memory observation")
    require({(r["round"], r["variant"]) for r in warmups} ==
            {(r, v) for r in range(3) for v in VARIANTS}, "Duplicate or missing warmup")
    rows = []
    for variant in VARIANTS:
        documents = corpus[group(variant)]
        output_bytes = sum(len(d["html"].encode()) for d in outputs[variant]["outputs"])
        samples = [r for r in windows if r["variant"] == variant]
        warming = [r for r in warmups if r["variant"] == variant]
        for sample in samples + warming:
            floor = (63 if "window" in sample else 3000) * 1_000_000
            require(sample["iterations"] > 0 and sample["elapsed_ns"] >= floor, "Short or empty timing window")
            require(sample["output_bytes"] == sample["iterations"] * output_bytes, "Timed output bytes differ from verification")
            calculated = sample["elapsed_ns"] / sample["iterations"]
            require(math.isfinite(calculated) and calculated == sample["ns_per_workload"], "Timing disagrees with duration/count")
        medians = [statistics.median(r["ns_per_workload"] for r in samples if r["round"] == i) for i in range(3)]
        ns = statistics.median(medians)
        heaps = [r for r in memory if r["variant"] == variant]
        for sample in heaps:
            require(sample["output_bytes"] == output_bytes and sample["live_after_drop"] == 0, "Memory output mismatch or unbalanced allocations")
            for phase in ("cold", "warm"):
                stats = sample[phase]
                require(all(isinstance(value, int) and value >= 0 for value in stats.values()), "Invalid heap counter")
                require(stats["peak_live_bytes"] >= stats["live_after_render"], "Heap peak below retained bytes")
                minimum_output = output_bytes if variant.endswith("-retain") else max(len(d["html"].encode()) for d in outputs[variant]["outputs"])
                require(stats["peak_live_bytes"] >= minimum_output, "Heap scope did not include live HTML output")
        # Counter observations are deterministic, not RSS samples. Do not hide variation in a median.
        for phase in ("cold", "warm"):
            require(all(row[phase] == heaps[0][phase] for row in heaps), "Heap allocation behavior varied; investigate before publication")
        rows.append({"variant": variant, "label": LABELS[variant], "group": group(variant),
                     "documents": len(documents), "input_bytes": sum(len(d["input"].encode()) for d in documents),
                     "output_bytes": output_bytes, "median_ns": ns, "run_medians_ns": medians,
                     "spread_percent": (max(medians) - min(medians)) / ns * 100,
                     "mean_document_ns": ns / len(documents), "cold_heap": heaps[0]["cold"], "heap": heaps[0]["warm"]})
    for round_id in range(3):
        for window in range(80):
            subset = [r for r in windows if r["round"] == round_id and r["window"] == window]
            offset = (window + round_id) % len(VARIANTS)
            expected = VARIANTS[offset:] + VARIANTS[:offset]
            if window % 2:
                expected.reverse()
            require([r["variant"] for r in sorted(subset, key=lambda r: r["position"])] == expected,
                    "Timing order does not follow the rotating protocol")
    return rows


def load(folder):
    run = read_json(folder / "run.json")
    require(run["mode"] == "publication" and run["finished_unix"] > run["started_unix"], "Incomplete or screening run")
    checksums = read_json(folder / "checksums.json")
    required = {"corpus.json", "Cargo.toml", "Cargo.lock", "build.json", "outputs.json.gz", "admission.json",
                "windows.json.gz", "memory.json.gz", "warmups.json", "observations.json", "run.json",
                "timing-build.log", "memory-build.log"}
    require(set(checksums) == required, "Incomplete archive checksums")
    for name, digest in checksums.items():
        require(sha(folder / name) == digest, f"Archive checksum mismatch: {name}")
    corpus, outputs, build = [read_json(folder / p) for p in ("corpus.json", "outputs.json.gz", "build.json")]
    require(build["source_sha256"]["benchmarks/workflows/corpus.json"] == sha(folder / "corpus.json"), "Built corpus differs from archived corpus")
    require(build["source_sha256"]["benchmarks/workflows/Cargo.lock"] == sha(folder / "Cargo.lock"), "Built lock differs from archive")
    require(not build["source_status"], "Publication build requires a committed, clean source tree")
    require(build["rustflags"] == "-C target-cpu=generic", "Unexpected CPU flags")
    pinned = {p["name"]: p["version"] for p in tomllib.loads((folder / "Cargo.lock").read_text())["package"]}
    require(pinned["pulldown-cmark"] == "0.13.4" and pinned["comrak"] == "0.54.0", "Engine pins changed")
    inputs = {d["id"]: d["input"] for name in ("guides", "documentation") for d in corpus[name]}
    for provenance in corpus["provenance"]:
        text = inputs[provenance["id"]].encode()
        require(len(text) == provenance["bytes"] and hashlib.sha256(text).hexdigest() == provenance["sha256"], "Corpus provenance mismatch")
    admission = review(outputs, corpus)
    require(all(r["comparable"] for r in admission) and admission == read_json(folder / "admission.json"), "Output work is not comparable")
    observations = read_json(folder / "observations.json")
    require(len(observations) == 4 and all("AC Power" in r["power"] for r in observations), "Power observations missing or not on AC")
    rows = summarize(corpus, outputs, read_json(folder / "windows.json.gz"), read_json(folder / "memory.json.gz"),
                     read_json(folder / "warmups.json"), run["protocol"])
    return {"schema": 1, "report": str((folder / "REPORT.md").relative_to(ROOT)),
            "run": run, "build": build, "rows": rows, "admission": admission,
            "corpus_summary": {name: {"documents": len(corpus[name]),
                "input_bytes": sum(len(d["input"].encode()) for d in corpus[name]),
                "min_bytes": min(len(d["input"].encode()) for d in corpus[name]),
                "max_bytes": max(len(d["input"].encode()) for d in corpus[name])}
                for name in ("previews", "guides", "documentation")}}


def table(rows):
    lines = ["| API / output lifetime | Time / complete workload | Peak live heap¹ | Allocated per workload² |",
             "| --- | ---: | ---: | ---: |"]
    for row in rows:
        lines.append(f"| {row['label']} | {row['median_ns'] / 1000:.1f} µs | {row['heap']['peak_live_bytes'] / 1024:.1f} KiB | {row['heap']['requested_bytes'] / 1024:.1f} KiB |")
    return "\n".join(lines)


def decisions(data):
    rows = {row["variant"]: row for row in data["rows"]}
    fresh, reuse = rows["preview-fresh"], rows["preview-reuse"]
    lines = []
    if reuse["median_ns"] < fresh["median_ns"] and reuse["heap"]["requested_bytes"] < fresh["heap"]["requested_bytes"]:
        lines.append("Reusing `Renderer` reduced time and allocation traffic for these previews.")
    if reuse["heap"]["peak_live_bytes"] > fresh["heap"]["peak_live_bytes"]:
        lines.append("Its peak heap was higher: retained scratch remains part of the worker's memory budget.")
    lines += ["", "For the documentation collection, compare each engine's two output lifetimes",
              "before using a memory figure to size your pipeline. Keeping completed HTML",
              "in memory is application work included in the retained-output rows.", "",
              "The metadata row measures a complete HTML-and-metadata operation; it does",
              "not isolate the incremental cost of collecting headings or represent an",
              "end-to-end site build."]
    return "\n".join(lines)


def overview(data):
    report = data["report"]
    rows = data["rows"]
    lines = ["## Workflow benchmarks", "",
             "What does the Markdown step cost in an application? These workloads measure",
             "complete sets of documents, including output allocation and release. Time and",
             "memory come from separate runs so allocation tracking does not affect timings.", ""]
    for name, title, explanation in [
        ("previews", "Preview user-authored comments", "Twelve authored examples, with Ferromark's secure defaults. Both APIs return owned HTML; a retained renderer reuses parser scratch."),
        ("guides", "Render documentation with metadata", "Three actual guide pages, with the default untrusted policy. Includes HTML, raw front matter, and headings; each result is released."),
        ("documentation", "Render a documentation collection", "Twelve actual documentation files. Trusted CommonMark plus tables, strikethrough, and task lists; equal reviewed HTML work across all three engines. Compare releasing each page with keeping every HTML result until the workload ends."),
    ]:
        subset = [r for r in rows if r["group"] == name]
        example = subset[0]
        lines += [f"### {title}", "", explanation, "",
                  f"{example['documents']} documents · {example['input_bytes']:,} input bytes · {example['output_bytes']:,} HTML bytes (Ferromark).", "", table(subset), ""]
    lines += [decisions(data), ""]
    os_version = re.search(r"ProductVersion:\s*(\S+)", data["build"]["os"])[1]
    rust_version = data["build"]["rustc"].splitlines()[0].split()[1]
    lines += ["¹ Peak simultaneously live **requested heap**, including retained parser scratch",
              "and the chosen HTML output lifetime. Loaded inputs, stack, allocator overhead,",
              "and cached pages are excluded; this is **not process RAM/RSS**. ² Cumulative",
              "allocation traffic, not simultaneously required memory.", "",
              f"{data['build']['cpu']}, macOS {os_version}, Rust {rust_version}, generic CPU target, system allocator.",
              "Three process rounds per variant; median of round medians. Native, warmed,",
              "single-threaded API work only: no Node.js bindings, I/O, templates, or syntax",
              "highlighting. These project snapshots and authored comments are examples, not",
              "a production-traffic distribution or a complete site build. No x86-64 run is included.", "",
              f"[Full report, run variation, and source provenance]({report}) ·",
              "[Reproduce the workloads](benchmarks/workflows/README.md).", ""]
    return "\n".join(lines)


def report(data):
    rows = data["rows"]
    os_version = re.search(r"ProductVersion:\s*(\S+)", data["build"]["os"])[1]
    rust_version = data["build"]["rustc"].splitlines()[0].split()[1]
    lines = ["# Practical Markdown workflows: time and memory", "",
             "Generated from the archived raw observations by `benchmarks/workflows/publish.py`.", "",
             f"Measured on {data['build']['cpu']} with {data['build']['ram_bytes'] / 2**30:g} GiB of installed RAM, macOS {os_version},",
             f"Rust {rust_version}, `-C target-cpu=generic`, and the system allocator. Installed",
             "RAM describes the host; the tables measure requested heap, not process RAM.", "",
             "## Questions and scope", "",
             "The corpus was frozen before timing. Preview comments are authored examples;",
             "guide pages and documentation are verbatim snapshots of this project's real",
             "files. They cover concrete integration choices without claiming to represent",
             "the distribution of all Markdown content or customer traffic.", "",
             "The secure preview lane compares fresh owned output with a retained renderer.",
             "The metadata lane includes front matter and heading collection. Only the",
             "trusted documentation HTML lane compares engines, with identical syntax flags",
             "and reviewed complete output. It also measures immediate release versus",
             "retaining every output until the end of the workload.", "",
             "## Results", "", table(rows), "",
             "Peak heap includes the session and all retained scratch above a baseline that",
             "excludes loaded input and benchmark bookkeeping. Each memory observation",
             "creates and warms a session, resets the peak while retaining its bytes in the",
             "count, measures one complete workload, then drops the session and verifies a",
             "return to baseline. Thirty observations per variant agree exactly. These",
             "are requested allocation sizes, not allocator physical consumption or RSS.", "",
             "The cumulative allocated column counts allocation/reallocation requests",
             "during the workload; it is allocation traffic, not a memory budget.", "",
             "## Variation and cold-session memory", "",
             "The average-per-document column divides the workload time by its file count.",
             "It is not a measured per-request percentile or a tail-latency prediction.", "",
             "| Variant | Round medians (µs/workload) | Round spread | Average / document | Cold peak heap | Retained session heap | Allocation calls / workload |",
             "| --- | --- | ---: | ---: | ---: | ---: | ---: |"]
    for row in rows:
        medians = ", ".join(f"{n / 1000:.1f}" for n in row["run_medians_ns"])
        lines.append(f"| {row['label']} | {medians} | {row['spread_percent']:.1f}% | {row['mean_document_ns'] / 1000:.1f} µs | {row['cold_heap']['peak_live_bytes'] / 1024:.1f} KiB | {row['heap']['live_after_render'] / 1024:.1f} KiB | {row['heap']['allocation_calls']} |")
    lines += ["", "Cold peak covers session creation and its first complete workload after",
              "process-wide initialization. Warm peak still includes retained session",
              "allocations; it does not pretend reusable parser scratch is free.", "",
              "## Reproduce and inspect", "",
              "See [the harness](../../../benchmarks/workflows/README.md) and",
              "[the measurement contract](../../arch/ARCH-COMP-003-practical-workflows.md).", "",
              "- `corpus.json`: all exact input text, source paths/revisions, licenses, byte counts, and hashes.",
              "- `build.json`, `Cargo.lock`, and build logs: compiler, CPU flags, dependencies, source/binary hashes, and host.",
              "- `outputs.json.gz` and `admission.json`: original HTML, guide metadata, effective options, and output review.",
              "- `windows.json.gz`: all 2,160 timed windows with elapsed nanoseconds, completed-workload counts, and output sizes.",
              "- `warmups.json`: all 27 warmups; `memory.json.gz`: all 270 memory observations.",
              "- `observations.json`: power, thermal, load, and process-CPU observations; `run.json`: protocol and completion times.",
              "- `checksums.json`: hashes of every evidence input; the publisher verifies them before generating tables.", "",
              f"Measured source commit: `{data['build']['source_revision']}`.", "",
              "Timing uses an uninstrumented binary; heap accounting uses a separate binary",
              "with the same source, flags, and dependency lock. The allocator counts",
              "successful allocations, zeroed allocations, reallocations, and frees.",
              "A realloc replaces its old logical size; hidden simultaneous backing storage,",
              "allocator metadata/rounding/caches, stack, and loaded corpus storage are not",
              "counted. Self-tests exercise growth, shrinkage, retained bytes, and balance.", "",
              "Each of three process rounds warms every variant for three seconds, then",
              "rotates its order through 80 windows of at least 63 ms. A time check occurs",
              "after four complete workloads. Output construction and destruction are timed;",
              "input loading, configuration, verification, IPC, and statistics are outside",
              "the timer. This is warmed single-threaded native API work, not CLI startup,",
              "Node.js latency, full-site generation, or a concurrent service benchmark.", "",
              "## Limits on interpretation", "",
              "Use the lifetime comparison to budget your own pipeline: sequentially",
              "releasing pages and retaining every rendered page have different memory",
              "requirements. Do not extrapolate a per-page peak by multiplication, treat",
              "allocation volume as peak memory, or compare rows that deliver different",
             "outputs or trust policies as a speed ranking.", "",
             "The documents come from one project; short comments are hand-authored.",
              "This was an interactive workstation: desktop, indexing, and backup activity",
              "are visible in the host observations. Round variation is reported above;",
              "the process was not CPU-pinned and the host was not an isolated benchmark machine.",
              "All measurements use one Apple Silicon host on AC power, system allocation,",
              "and no PGO. They do not establish x86-64 performance or production p95/p99.",
              "Historical shared-mimalloc comparisons remain separate evidence.", ""]
    corpus_lines = ["## Corpus and integration decisions", "",
                   "| Workload | Documents | Total input | Smallest–largest file |",
                   "| --- | ---: | ---: | ---: |"]
    for name, item in data["corpus_summary"].items():
        corpus_lines.append(f"| {name} | {item['documents']} | {item['input_bytes']:,} bytes | {item['min_bytes']:,}–{item['max_bytes']:,} bytes |")
    corpus_lines += ["", decisions(data), ""]
    return "\n".join(lines).replace("## Results\n", "\n".join(corpus_lines) + "\n## Results\n", 1)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("folder", type=Path, nargs="?", default=DEFAULT)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    folder = args.folder.resolve()
    data = load(folder)
    source = ROOT / "README.md.src"
    original = source.read_text()
    require(original.count(START) == 1 and original.count(END) == 1, "README workflow markers missing or duplicated")
    begin, end = original.index(START) + len(START), original.index(END)
    generated = original[:begin] + "\n\n" + overview(data) + "\n" + original[end:]
    outputs = {source: generated, folder / "REPORT.md": report(data),
               folder / "summary.json": json.dumps(data, ensure_ascii=False, indent=2) + "\n"}
    guide = ROOT / "homepage/app/routes/guide/benchmarks.mdx"
    guide_text = guide.read_text()
    guide_start, guide_end = "{/* workflow-benchmarks:start */}", "{/* workflow-benchmarks:end */}"
    require(guide_text.count(guide_start) == 1 and guide_text.count(guide_end) == 1, "Benchmark guide workflow markers missing or duplicated")
    guide_overview = overview(data).replace(f"]({data['report']})", f"](https://github.com/sebastian-software/ferromark/blob/main/{data['report']})")
    guide_overview = guide_overview.replace("](benchmarks/workflows/README.md)", "](https://github.com/sebastian-software/ferromark/blob/main/benchmarks/workflows/README.md)")
    begin, end = guide_text.index(guide_start) + len(guide_start), guide_text.index(guide_end)
    outputs[guide] = guide_text[:begin] + "\n\n" + guide_overview + "\n" + guide_text[end:]
    if args.check:
        rendered = (ROOT / "README.md").read_text()
        require(rendered.split(START, 1)[1].split(END, 1)[0] == generated.split(START, 1)[1].split(END, 1)[0], "Rendered workflow overview differs")
    for path, text in outputs.items():
        if args.check:
            require(path.read_text() == text, f"Generated result drift: {path}")
        else:
            path.write_text(text)
    print("Workflow publication matches raw time and heap evidence" if args.check else "Published verified workflow results")


if __name__ == "__main__":
    main()
