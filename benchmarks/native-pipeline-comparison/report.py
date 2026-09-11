#!/usr/bin/env python3
"""Render a native pipeline report only after rechecking all archived timing samples."""
import argparse
import gzip
import json
from pathlib import Path

from common import ENGINES
from run import summarize


def load(path):
    if path.exists():
        return json.loads(path.read_text())
    return json.loads(gzip.decompress(path.with_suffix(path.suffix + ".gz").read_bytes()))


def render(folder):
    metadata = load(folder / "metadata.json")
    protocol = metadata["protocol"]
    if protocol["mode"] != "measurement" or metadata["finished_unix"] <= metadata["started_unix"]:
        raise ValueError("Only completed full measurements can produce this report")
    lines = ["# Native Goldmark and Sätteri comparison", "",
        f"Measured Ferromark revision: `{metadata['ferromark_revision']}`.", "",
        f"Environment: {metadata['cpu']}, {metadata['platform']}. Each pair has an independent Ferromark baseline.", "",
        "| Native pair | Input | Bytes | Ferromark µs | Competitor µs | Competitor / Ferromark | Run-median range, Ferromark / competitor µs |",
        "| --- | --- | ---: | ---: | ---: | ---: | --- |"]
    diagnostics = []
    for engine in ENGINES:
        pair = metadata["pairs"][engine]
        runs = [load(folder / engine / f"samples-{i}.json") for i in range(protocol["runs"])]
        outputs = load(folder / "ferromark-catalog-outputs.json") + load(folder / f"{engine}-catalog-outputs.json")
        computed = summarize(runs, pair["selected"], engine, protocol, outputs)
        if computed != load(folder / engine / "summary.json"):
            raise ValueError(f"{engine} summary differs from raw samples")
        rows = {(r["case"], r["engine"]): r for r in computed}
        for case in pair["selected"]:
            a, b = (rows[case, name] for name in ("ferromark", engine))
            ranges = [f"{min(r['run_medians_ns'])/1000:.2f}–{max(r['run_medians_ns'])/1000:.2f}" for r in (a, b)]
            lines.append(f"| {engine} | `{case}` | {a['bytes']:,} | {a['median_ns']/1000:.2f} | {b['median_ns']/1000:.2f} | {b['median_ns']/a['median_ns']:.2f}× | {' / '.join(ranges)} |")
        mismatches = [r["case"] for r in load(folder / f"{engine}-spec-input-outputs.json") if r["mismatch"]]
        reviews = load(folder / engine / "verification.json")
        differences = [r["case"] for r in reviews if r["comparable"] and not r["html_equivalent"]]
        diagnostics.append(f"- **{engine}:** {pair['eligible']}/{pair['total']} admitted workloads; {len(mismatches)} normalized mismatches against the 652 stored CommonMark examples. "
                           f"Excluded: {', '.join(pair['excluded']) or 'none'}. Admitted renderer differences: {', '.join(differences) or 'none'}.")
        if engine.startswith("goldmark-"):
            gc = sum(r["gc_cycles"] for run in runs for r in run if r["engine"] == engine)
            diagnostics.append(f"  Recorded Go GC cycles during sampled windows: {gc:,}.")
    lines += ["", "Smaller times are better. These are selected workload observations, not a general engine ranking. The ranges describe observed run medians, not confidence intervals.",
        "", "## Protocol and native work", "",
        f"{protocol['runs']} process runs per pair; {protocol['warmup_ms']} ms warmup and {protocol['samples']} alternating-order windows of at least {protocol['window_ms']} ms per engine/case. "
        "Each worker times batches of 16 fresh parse/render calls using its native monotonic clock. Reported values are medians of process-run medians. "
        "Startup, fixture loading, IPC, JSON, validation, and memory-statistics reads are outside timing.", "",
        "Rust uses its system allocator and destroys the arena/AST and owned HTML output inside each call. Goldmark uses fresh ASTs and bytes.Buffer outputs, reuses immutable parser/renderer configuration, "
        "and keeps normal automatic GC enabled: GOGC=100, GOMAXPROCS=1, no explicit GOMEMLIMIT or forced collections. Go GC during a window is timed; GC may also progress between windows. "
        "This measures steady-state render-window latency, not total process CPU or a forced full-collection cost per document. Allocation/GC counters are retained in every Go sample.", "",
        "Sätteri runs its public parse-to-MDAST arena stage with position tracking, then mdast_to_html with its normal fused/fallback renderer. Only syntax flags differ from the convenience API's broad defaults; "
        "no parser, AST, position, or rendering stage is removed. Its native MDX capability is compiled in but disabled by the Markdown lane's runtime flags. No Node.js bindings, JavaScript plugins, WASM, cgo, or per-document CLI calls are used.", "",
        "CommonMark disables extensions. Table lanes enable tables alone; gfm_overlap enables tables, strikethrough, and task lists. Trusted HTML and URL schemes are preserved. "
        "Bare autolinks, tag filtering, footnotes, math, and typography remain off. This overlap is not full GFM. Single-tilde dialect differences remain visible in the probes and are not patched away.", "",
        "## Output and capability diagnostics", "", *diagnostics, "",
        "Full original outputs, effective options, all eight feature-switch probes, and specification diagnostics are archived. Workload admission uses the shared ARCH-COMP-002 review; it is distinct from HTML fidelity and specification conformance.", "",
        "## MDX boundary", "",
        "The archive includes native results for Markdown-only MDX, root JSX, inline expressions, container JSX, and invalid JavaScript. Ferromark emits a JSX module after segmentation; Sätteri compiles MDX to JavaScript with OXC. "
        "These are different output stages and validation contracts, so no MDX throughput ratio is reported. The invalid-JavaScript probe and adapter test demonstrate the boundary without executing JavaScript.", "",
        "## Reproduction", "",
        "The adjacent metadata pins upstream revisions, dependency checksums, compiler versions, build flags, binary hashes, and local input hashes. The native harness README gives the build/run commands. "
        "The source patch records any uncommitted production changes at measurement time. These system-allocator measurements are separate from the published Bun/mimalloc figures.", "",
        "Regenerate or verify this report from the repository root:", "", "```sh",
        f"python3 benchmarks/native-pipeline-comparison/report.py docs/reports/{folder.name} --check",
        "```", ""]
    return "\n".join(lines)


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("result", type=Path)
    p.add_argument("--check", action="store_true")
    args = p.parse_args()
    report = render(args.result)
    path = args.result / "REPORT.md"
    if args.check:
        if path.read_text() != report:
            raise SystemExit("Report differs from archived samples")
        print("Report matches all archived samples and summaries")
    else:
        path.write_text(report)


if __name__ == "__main__":
    main()
