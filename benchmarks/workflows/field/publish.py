#!/usr/bin/env python3
"""Publish only complete, revalidated collection measurements for the full field."""
import argparse
import json
import math
import re
from pathlib import Path
import statistics
import tomllib

from common import ENGINES, LABELS, PROTOCOL, ROOT, admission, output_units, read_json, sha

DEFAULT = ROOT / "docs/reports/2026-09-13-workflow-engine-field"


def require(ok, message):
    if not ok:
        raise ValueError(message)


def summarize(build, outputs, reviews, run, windows, warmups, rss):
    require(run["protocol"] == PROTOCOL and run["mode"] == "publication", "Not the publication protocol")
    require(run["finished_unix"] > run["started_unix"], "Incomplete field run")
    variants = [(e, life) for e in ENGINES if reviews[e]["admitted"] for life in ("stream", "retain")]
    require([list(v) for v in variants] == run["variants"], "Selected field differs from complete-collection admission")
    require(len(windows) == len(variants) * 240 and len(warmups) == len(variants) * 3 and len(rss) == len(variants) * 3, "Incomplete field observations")
    require({(r["round"], r["window"], r["engine"], r["lifetime"]) for r in windows} ==
            {(r, w, e, life) for r in range(3) for w in range(80) for e, life in variants}, "Duplicate or missing field window")
    for values in (warmups, rss):
        require({(r["round"], r["engine"], r["lifetime"]) for r in values} ==
                {(r, e, life) for r in range(3) for e, life in variants}, "Duplicate or missing round observation")
    for round_id in range(3):
        for window in range(80):
            offset = (round_id + window) % len(variants)
            order = variants[offset:] + variants[:offset]
            if window % 2:
                order.reverse()
            actual = sorted((r for r in windows if r["round"] == round_id and r["window"] == window), key=lambda r: r["position"])
            require([(r["engine"], r["lifetime"]) for r in actual] == order, "Field rotation changed")
    rows = []
    for engine, lifetime in variants:
        samples = [r for r in windows if (r["engine"], r["lifetime"]) == (engine, lifetime)]
        warming = [r for r in warmups if (r["engine"], r["lifetime"]) == (engine, lifetime)]
        units = output_units(outputs[engine][lifetime]["outputs"], build["workers"][engine]["units"])
        for r in samples + warming:
            require(r["iterations"] > 0 and r["iterations"] % 4 == 0, "Empty or incomplete native workload batch")
            require(r["elapsed_ns"] >= (63 if "window" in r else 3000) * 1_000_000, "Short field measurement")
            require(r["output_units"] == units * r["iterations"], "Field output work changed")
            require(math.isclose(r["ns_per_workload"], r["elapsed_ns"] / r["iterations"], rel_tol=1e-15), "Field timing disagrees with count/duration")
        medians = [statistics.median(r["ns_per_workload"] for r in samples if r["round"] == i) for i in range(3)]
        peaks = [r["peak_rss_bytes"] for r in rss if (r["engine"], r["lifetime"]) == (engine, lifetime)]
        require(all(isinstance(value, int) and value > 0 for value in peaks), "Missing process-memory observations")
        median = statistics.median(medians)
        rows.append({"engine": engine, "lifetime": lifetime, "median_ns": median, "round_medians_ns": medians,
                     "spread_percent": (max(medians) - min(medians)) / median * 100,
                     "median_peak_rss_bytes": statistics.median(peaks), "peak_rss_observations_bytes": peaks,
                     "html_utf8_bytes": sum(len(d["html"].encode()) for d in outputs[engine][lifetime]["outputs"])})
    return rows


def load(folder):
    hashes = read_json(folder / "checksums.json")
    required = {"build.json.gz", "build.log", "corpus.json", "outputs.json.gz", "admission.json", "windows.json.gz",
                "warmups.json", "rss.json", "observations.json", "run.json"}
    require(set(hashes) == required, "Missing field evidence")
    for name, digest in hashes.items():
        require(sha(folder / name) == digest, "Field evidence checksum differs: " + name)
    build = read_json(folder / "build.json.gz")
    require(not build["source_status"], "Field source must be committed and clean")
    require(build["source_sha256"]["benchmarks/workflows/corpus.json"] == sha(folder / "corpus.json"), "Field corpus changed")
    corpus, outputs = read_json(folder / "corpus.json"), read_json(folder / "outputs.json.gz")
    reviews = admission(outputs, corpus)
    require(reviews == read_json(folder / "admission.json"), "Field admission changed")
    observations = read_json(folder / "observations.json")
    require(len(observations) == 4 and all("AC Power" in r["power"] for r in observations), "Missing AC power observations")
    run = read_json(folder / "run.json")
    rows = summarize(build, outputs, reviews, run, read_json(folder / "windows.json.gz"), read_json(folder / "warmups.json"), read_json(folder / "rss.json"))
    packages = {name: {p["name"]: p["version"] for p in tomllib.loads(lock)["package"]}
                for name, lock in build["locks"].items() if name not in ("go.mod", "go.sum", "packages.lock.json")}
    revisions = {v["revision"] for v in build["upstream"].values() if "revision" in v}
    pins = {"md4c": "65c6c9d72cebd9a731aaa5597414ce04d9ea5de3", "cmark": "bb3678d7a73cb02d35c8876ecd097072636200a8",
            "cmark-gfm": "587a12bb54d95ac37241377e6ddc93ea0e45439b", "bun": "76e9dcc6ad272a4fb1ee4a4dbbde4809201b71d1"}
    require(set(pins.values()).issubset(revisions), "Field native source pins changed")
    versions = {"ferromark": packages["basic"]["ferromark"], "pulldown": packages["basic"]["pulldown-cmark"],
                "comrak": packages["basic"]["comrak"], "md4c": pins["md4c"][:12], "cmark": "0.31.1 · " + pins["cmark"][:12],
                "cmark-gfm": "0.29.0.gfm.13 · " + pins["cmark-gfm"][:12],
                "goldmark": re.search(r"github\.com/yuin/goldmark/v2\s+(\S+)", build["locks"]["go.mod"])[1],
                "satteri": packages["satteri"]["satteri"] + " (native crate); fork " + packages["satteri"]["satteri-pulldown-cmark"],
                "rushdown": packages["rushdown"]["rushdown"], "markdown-rs": packages["markdown-rs"]["markdown"],
                "ox-content": packages["ox-content"]["ox_content_parser"],
                "markdig": json.loads(build["locks"]["packages.lock.json"])["dependencies"]["net10.0"]["Markdig"]["resolved"],
                "bun": pins["bun"][:12], "ferromark-bun": packages["bun-Cargo.lock"]["ferromark"]}
    return {"report": str((folder / "REPORT.md").relative_to(ROOT)), "source_revision": build["source_revision"],
            "cpu": build["cpu"], "os": build["os"], "rows": rows, "admission": reviews, "protocol": PROTOCOL,
            "versions": versions, "toolchains": {"rustc": build["rustc"], **build["toolchains"]},
            "input_bytes": sum(len(d["input"].encode()) for d in corpus["documentation"])}


def table(data, engines):
    rows = {(r["engine"], r["lifetime"]): r for r in data["rows"]}
    minimum = {(life, metric): min(rows[e, life][metric] for e in engines if data["admission"][e]["admitted"])
               for life in ("stream", "retain") for metric in ("median_ns", "median_peak_rss_bytes")}

    def cell(row, metric):
        value = row[metric]
        text = f"{value / 1000:.1f} µs" if metric == "median_ns" else f"{value / 2**20:.1f} MiB"
        return f"**{text}**" if value == minimum[row["lifetime"], metric] else text

    result = ["| Engine | Release each: time | Keep all: time | Release each: peak process RSS | Keep all: peak process RSS |",
              "| --- | ---: | ---: | ---: | ---: |"]
    for engine in engines:
        if not data["admission"][engine]["admitted"]:
            count = data["admission"][engine]["complete_documents"]
            result.append(f"| {LABELS[engine]} | Not comparable ({count}/12 complete documents) | — | — | — |")
            continue
        a, b = rows[engine, "stream"], rows[engine, "retain"]
        result.append(f"| {LABELS[engine]} | {cell(a, 'median_ns')} | {cell(b, 'median_ns')} | {cell(a, 'median_peak_rss_bytes')} | {cell(b, 'median_peak_rss_bytes')} |")
    return "\n".join(result)


def overview(data):
    system_stream = [r for r in data["rows"] if r["lifetime"] == "stream" and r["engine"] not in ("bun", "ferromark-bun")]
    fastest = min(system_stream, key=lambda r: r["median_ns"])
    ferro = next(r for r in system_stream if r["engine"] == "ferromark")
    return "\n".join([
        "### Render a documentation collection across the native engine field", "",
        f"Twelve actual documentation files, {data['input_bytes']:,} input bytes, with trusted",
        "CommonMark plus tables, strikethrough, and tasks. Every timed row completes the entire",
        "collection. Native defaults for allocation, GC, and output representation",
        "remain in place; Markdig returns UTF-16 strings, the other workers UTF-8.", "",
        "**API lifecycle:** Ferromark uses fresh `to_html_with_options` calls.",
        "Ox retains its `HtmlRenderer` scratch, but creates and drops a fresh parser",
        "and growing AST arena for each document; owned HTML is moved out and released.",
        "These are integration choices, not identical scratch lifetimes. The",
        "[lifecycle and cache audit](docs/reports/2026-09-13-ox-workflow-study/REPORT.md)",
        "also measures fresh and reusable renderers for both engines and checks",
        "changed inputs against fresh-process output.", "",
        table(data, ENGINES[:-2]), "",
        f"{LABELS[fastest['engine']]} had the lowest collection time with immediate release in this run: {fastest['median_ns'] / 1000:.1f} µs. Ferromark took {ferro['median_ns'] / 1000:.1f} µs. This result applies to the complete archived workload, not every Markdown application.", "",
        "Bun's native support uses its pinned nightly compiler and shared mimalloc.",
        "This separate environment has its own freshly measured Ferromark baseline:", "",
        table(data, ENGINES[-2:]), "",
        "Bold marks the lowest unrounded observation in each column and environment.", "",
        "**Process RSS is a different memory measurement from the Rust heap table.**",
        "It includes the runtime/JIT, stacks, input, allocator/GC reserves, and worker",
        "infrastructure. Each cell is the median of three whole-process peaks during",
        "startup, warmup, repeated collection work, and shutdown. It is not incremental",
        "parser memory, a single-request peak, or a concurrent-service capacity estimate.", "",
        "Timing still surrounds only completed Markdown work. GC in those windows is",
        "included; OS peak-RSS accounting needs no instrumented allocator. Collection",
        "timings and the earlier Rust heap measurements are separate fresh runs.", "",
        "Ox Content's extra generated heading IDs may be admitted as additional output;",
        "content, heading levels, links, tables, and checkbox states must remain intact.",
        "cmark's core-only dialect cannot complete the GFM collection. No input is",
        "removed to obtain a timing row. Full secure-preview and metadata adapters are",
        "currently measured for Ferromark, pulldown-cmark, and Comrak only; the wider",
        "HTML-only collection comparison does not establish those additional contracts.", "",
        f"[All engine versions, reviewed output differences, variation, and raw evidence]({data['report']}).", "",
    ])


def report(data):
    versions = ["## Engine versions and compiler settings", "", "| Engine | Measured version / revision |", "| --- | --- |"]
    versions += [f"| {LABELS[e]} | {data['versions'][e]} |" for e in ENGINES]
    versions += ["", "Full lockfiles and compiler details are in `build.json.gz`. Rust compiler:",
                 f"`{data['toolchains']['rustc'].splitlines()[0]}`. Bun compiler:",
                 f"`{data['toolchains']['bun_rustc'].splitlines()[0]}`.",
                 f"`{data['toolchains']['go']}`; .NET SDK 10.0.401;",
                 f"`{data['toolchains']['clang'].splitlines()[0]}`.", ""]
    introduction = overview(data).replace("### Render a documentation collection across the native engine field", "## Documentation collection results", 1)
    introduction = introduction.replace(f"]({data['report']})", "](#engine-versions-and-compiler-settings)")
    introduction = introduction.replace("](docs/reports/2026-09-13-ox-workflow-study/REPORT.md)", "](../2026-09-13-ox-workflow-study/REPORT.md)")
    lines = ["# Practical documentation workflows across the native engine field", "", introduction,
             "\n".join(versions),
             "## Provenance and reproduction", "",
             f"Measured source: `{data['source_revision']}`. Host: {data['cpu']}.", "",
             "See [the field harness](../../../benchmarks/workflows/field/README.md).",
             "`build.json.gz` records exact Cargo, Go, and NuGet locks, upstream revisions,",
             "build commands, compiler settings, native-support hashes, and executable hashes.",
             "`corpus.json` preserves the exact earlier inputs and their source provenance.",
             "`outputs.json.gz` retains every engine's original HTML and effective options",
             "for both lifetimes, including engines ineligible for timing. `admission.json`",
             "records every document, with no timing-based selection or corpus trimming.", "",
             "Three fresh process rounds warm each variant for 3 seconds, then rotate",
             "80 windows of at least 63 ms. Four whole collections run between clock",
             "checks. `windows.json.gz` and `warmups.json` retain all observations and",
             "native output lengths; `rss.json` stores the OS-reported peak and process",
             "CPU usage for each worker. `observations.json` records host activity and",
             "power. `checksums.json` protects every evidence input. All tables are",
             "recomputed from these records by `field/publish.py --check`.", "",
             "## Round variation and process-memory range", "",
             "| Engine / lifetime | Round medians (µs) | Round spread | Peak RSS range |",
             "| --- | --- | ---: | ---: |"]
    for row in data["rows"]:
        medians = ", ".join(f"{v / 1000:.1f}" for v in row["round_medians_ns"])
        peaks = row["peak_rss_observations_bytes"]
        lines.append(f"| {LABELS[row['engine']]} / {row['lifetime']} | {medians} | {row['spread_percent']:.1f}% | {min(peaks) / 2**20:.1f}–{max(peaks) / 2**20:.1f} MiB |")
    lines += ["", "This is one Apple Silicon workstation, not a cross-machine ranking. Background",
              "desktop and OS work is recorded; the CPU was not pinned. .NET uses normal",
              "workstation concurrent GC and tiered JIT/PGO. Go uses normal automatic GC",
              "with cgo and static PGO disabled. Rust and C use generic release builds.",
              "The RSS totals include startup and harness overhead and must not be treated",
              "as intrinsic parser allocation sizes. The exact heap accounting in the",
              "[separate Rust workflow report](../2026-09-13-workflow-comparisons/REPORT.md) answers that narrower question.", ""]
    return "\n".join(lines)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("folder", type=Path, nargs="?", default=DEFAULT)
    parser.add_argument("--check", action="store_true")
    parser.add_argument("--overview", action="store_true")
    args = parser.parse_args()
    data = load(args.folder.resolve())
    if args.overview:
        print(overview(data), end="")
        return
    for name, text in (("REPORT.md", report(data)), ("summary.json", json.dumps(data, ensure_ascii=False, indent=2) + "\n")):
        path = args.folder / name
        if args.check:
            require(path.read_text() == text, "Generated field report drift: " + name)
        else:
            path.write_text(text)
    print("Field publication verified" if args.check else "Published measured engine field")


if __name__ == "__main__":
    main()
