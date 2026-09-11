#!/usr/bin/env python3
"""Publish native pairs from complete, independently verified measurement archives."""
import argparse
import hashlib
import json
from pathlib import Path

from common import HERE, REPO
from report import load, render
from run import summarize

SOURCES = [
    "2026-09-11-native-pipeline-comparison",
    "2026-09-11-native-rushdown",
    "2026-09-11-native-markdig",
    "2026-09-11-native-markdown-rs",
    "2026-09-11-native-ox-content",
]
LABELS = {"goldmark": "Goldmark", "satteri": "Sätteri"}
RUNTIMES = {"goldmark": "Go", "satteri": "Rust", "rushdown": "Rust", "markdig": ".NET (warmed JIT)", "markdown-rs": "Rust", "ox-content": "Rust"}
NOTES = {
    "goldmark": "Fresh AST and HTML buffer; normal automatic Go GC remains enabled.",
    "satteri": "Full native Markdown-to-MDAST-to-HTML pipeline, including source positions. MDX compilation is outside timing.",
    "rushdown": "Fresh arena AST and owned HTML; full default entity support is retained.",
    "markdig": "Direct .NET API with tiered JIT/PGO and normal concurrent GC. Startup and warmup are outside timing; GC within a sampled window is included. The measured package lock targets macOS ARM64.",
    "markdown-rs": "Direct native events-to-HTML API. MDX support is available upstream but is not measured here.",
    "ox-content": "Generated heading IDs cannot be disabled; affected workloads are excluded. Specification differences include more than heading IDs. Native renderer scratch reuse is retained; every call creates a fresh arena, AST and owned HTML.",
}
CASES = {"commonmark/5k": "CommonMark · 5 KiB", "commonmark/short": "CommonMark · short", "tables/tables-commonmark-inline": "Tables + inline CommonMark", "gfm_overlap/features": "Tables + strikethrough + tasks"}
CONDITIONS = "Each candidate is paired with its own measured Ferromark baseline. These system-allocator runs are separate from the shared-mimalloc five-parser experiment; do not combine their times into one ranking."
SCOPE = "Trusted Markdown-to-HTML only, with independently selected syntax and fresh document state and owned output. No Node.js wrappers, WASM, per-document process startup, cached documents or MDX compilation are timed. The three-extension overlap is not full GFM."
RATIO = "Candidate / Ferromark is the elapsed-time ratio within that pair: below 1 means the candidate took less time. Bold identifies the lower unrounded time in each pair."


def validate_protocol(metadata):
    protocol = metadata["protocol"]
    if (protocol["mode"] != "measurement" or metadata["finished_unix"] <= metadata["started_unix"]
            or protocol["runs"] != 3 or protocol["samples"] < 80
            or protocol["samples"] * protocol["window_ms"] < 5000 or protocol["warmup_ms"] < 3000):
        raise ValueError("Public native comparisons require completed full repeated measurements")


def data_for():
    engines = []
    for source in SOURCES:
        folder = REPO / "docs/reports" / source
        metadata = load(folder / "metadata.json")
        validate_protocol(metadata)
        # Recompute reports and every summary from the original timed windows.
        if render(folder) != (folder / "REPORT.md").read_text():
            raise ValueError(f"{source}: report differs from raw evidence")
        for line in (folder / "SHA256SUMS").read_text().splitlines():
            digest, name = line.split("  ", 1)
            if hashlib.sha256((folder / name).read_bytes()).hexdigest() != digest:
                raise ValueError(f"{source}: archive checksum mismatch: {name}")
        for engine, pair in metadata["pairs"].items():
            build = metadata["build"]
            adapter = build.get("adapter", {})
            label = adapter.get("label", LABELS.get(engine, engine))
            version = adapter.get("version")
            if engine == "goldmark":
                version = next(iter(build["go_modules"].values())).removeprefix("v")
            elif engine == "satteri":
                version = build["satteri_revision"][:12]
            outputs = load(folder / "ferromark-catalog-outputs.json") + load(folder / f"{engine}-catalog-outputs.json")
            runs = [load(folder / engine / f"samples-{i}.json") for i in range(metadata["protocol"]["runs"])]
            computed = summarize(runs, pair["selected"], engine, metadata["protocol"], outputs)
            if not computed or computed != load(folder / engine / "summary.json"):
                raise ValueError(f"{engine}: summary differs from raw samples")
            reviews = {r["case"]: r for r in load(folder / engine / "verification.json")}
            values = {(r["case"], r["engine"]): r for r in computed}
            rows = []
            for case in pair["selected"]:
                if not reviews[case]["comparable"]:
                    raise ValueError(f"{engine}/{case}: excluded workload cannot be published")
                baseline, candidate = (values[case, name] for name in ("ferromark", engine))
                rows.append({"case": case, "label": CASES[case], "bytes": baseline["bytes"],
                    "ferromarkNs": baseline["median_ns"], "candidateNs": candidate["median_ns"],
                    "ferromarkTime": f"{baseline['median_ns']/1000:.2f}", "candidateTime": f"{candidate['median_ns']/1000:.2f}",
                    "ratio": f"{candidate['median_ns']/baseline['median_ns']:.2f}×",
                    "ferromarkRunMediansNs": baseline["run_medians_ns"], "candidateRunMediansNs": candidate["run_medians_ns"]})
            spec = load(folder / f"{engine}-spec-input-outputs.json")
            engines.append({"id": engine, "label": label, "version": version, "runtime": RUNTIMES[engine],
                "source": str(folder.relative_to(REPO)), "report": str((folder / "REPORT.md").relative_to(REPO)),
                "ferromarkRevision": metadata["ferromark_revision"], "cpu": metadata["cpu"], "platform": metadata["platform"],
                "rustc": build["rustc"], "protocol": metadata["protocol"],
                "eligible": pair["eligible"], "total": pair["total"], "excluded": pair["excluded"],
                "specMismatches": sum(r["mismatch"] for r in spec), "specTotal": len(spec),
                "notes": NOTES[engine], "rows": rows})
    return {"note": "Generated from verified native archives by benchmarks/native-pipeline-comparison/publish.py. Do not hand-edit figures.",
        "conditions": CONDITIONS, "scope": SCOPE, "ratioExplanation": RATIO, "engines": engines}


def readme_section(data):
    lines = ["### Additional native engine comparisons", "", data["conditions"], "", data["scope"], "",
        "Apple M1 Pro, macOS 26.6.2, September 2026. Three process runs per pair, three seconds of warmup and at least five seconds of sampling per engine/workload in each run. Values are medians of run medians; the linked archives retain exact toolchains, source revisions, options and run variation.", "",
        "| Engine | Runtime | Admitted workloads | Spec mismatches | Evidence |", "| --- | --- | ---: | ---: | --- |"]
    for engine in data["engines"]:
        lines.append(f"| {engine['label']} | {engine['runtime']} | {engine['eligible']}/{engine['total']} | {engine['specMismatches']}/{engine['specTotal']} | [{engine['version']}]({engine['report']}) |")
    lines += ["", "Admission checks comparable Markdown work. Spec mismatches are normalized output diagnostics, not a conformance certification. Only the workloads below received full timing runs.", "", data["ratioExplanation"], "",
        "| Native pair | Input | Bytes | Ferromark µs | Candidate µs | Candidate / Ferromark |", "| --- | --- | ---: | ---: | ---: | ---: |"]
    for engine in data["engines"]:
        for row in engine["rows"]:
            a, b = row["ferromarkTime"], row["candidateTime"]
            if row["ferromarkNs"] <= row["candidateNs"]:
                a = f"**{a}**"
            if row["candidateNs"] <= row["ferromarkNs"]:
                b = f"**{b}**"
            lines.append(f"| {engine['label']} | {row['label']} | {row['bytes']} | {a} | {b} | {row['ratio']} |")
    lines += [""] + [f"- **{e['label']}:** {e['notes']}" for e in data["engines"]]
    lines += ["", "Ox Content's short CommonMark case is a different document from the other engines' 5 KiB case. Its heading-ID exclusions prevent a corresponding 5 KiB result; no missing time is estimated.", "",
        "The [native cmark and cmark-gfm report](docs/reports/2026-09-11-native-cmark-comparison.md) additionally covers the C CommonMark reference parser and GitHub's fork, each with its own Ferromark baseline.", "",
        "Reproduce these pairs with the [native harness](benchmarks/native-pipeline-comparison/README.md) and each engine's adapter README. Regenerate this overview with `python3 benchmarks/native-pipeline-comparison/publish.py`, then `python3 benchmarks/bun-comparison/publish.py` and `mise run readme:write`.", ""]
    return "\n".join(lines)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true")
    parser.add_argument("--readme", action="store_true")
    args = parser.parse_args()
    data = data_for()
    if args.readme:
        print(readme_section(data))
        return
    path = REPO / "homepage/app/data/native-benchmarks.json"
    content = json.dumps(data, indent=2, ensure_ascii=False) + "\n"
    if args.check:
        if path.read_text() != content:
            raise SystemExit("Native homepage data differs from verified measurement archives")
    else:
        path.write_text(content)
    print("Native publication matches verified measurement archives")


if __name__ == "__main__":
    main()
