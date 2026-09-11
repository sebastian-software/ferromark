#!/usr/bin/env python3
"""Generate public benchmark tables from verified, repeated native measurements."""
import argparse
import gzip
import hashlib
import json
from pathlib import Path
import statistics
import tomllib
from datetime import datetime, timezone

from prepare import REPO, BUN_REV, MD4C_REV
from run import HEADLINE_CASES, PARSERS, mismatches, workload_review

LABELS = {
    "ferromark": "ferromark", "bun_md": "Bun (native)",
    "pulldown-cmark": "pulldown-cmark", "comrak": "comrak", "md4c": "md4c (C)",
}
ORDER = list(LABELS)
CASES = [
    ("commonmark/publication-2k", "commonmark-2k", "CommonMark · 2 KiB", "size"),
    ("commonmark/publication-5k", "commonmark-5k", "CommonMark · 5 KiB", "size"),
    ("commonmark/publication-10k", "commonmark-10k", "CommonMark · 10 KiB", "size"),
    ("tables/gfm-tables", "tables", "Tables only", "feature"),
    ("strikethrough/strikethrough", "strikethrough", "Strikethrough only", "feature"),
    ("gfm_overlap/gfm-tables", "gfm-overlap", "GFM subset: tables + strikethrough", "feature"),
    ("commonmark/links", "links", "CommonMark links and images", "feature"),
    ("commonmark/entities", "entities", "CommonMark entities and inline markup", "feature"),
]


def read_text(path):
    if path.exists():
        return path.read_text()
    with gzip.open(str(path) + ".gz", "rt") as stream:
        return stream.read()


def load_publication(folder, cases=CASES):
    metadata = json.loads((folder / "metadata.json").read_text())
    protocol = metadata["protocol"]
    if protocol["mode"] != "publication" or protocol["samples"] < 80 or protocol["headline_runs"] != 3:
        raise ValueError("Public tables require three publication runs with at least 80 samples")
    if protocol["window_ms"] * protocol["samples"] < 5000 or protocol["warmup_ms"] < 3000:
        raise ValueError("Public tables require at least five seconds of sampling and three seconds of warmup")
    if metadata["bun_revision"] != BUN_REV or metadata["md4c_revision"] != MD4C_REV:
        raise ValueError("Measured parser revisions do not match the harness pins")
    verified = {row["case"]: row for row in json.loads((folder / "verification.json").read_text())["cases"]}
    outputs = {row["case"]: row for row in map(json.loads, read_text(folder / "verify.jsonl").splitlines())}
    summaries = {(row["case"], row["parser"]): row for row in json.loads((folder / "summary.json").read_text())}
    raw = [{(r["case"], r["parser"]): r for r in map(json.loads, read_text(folder / f"samples-{i}.jsonl").splitlines())} for i in range(3)]
    options = {row["lane"]: row for row in map(json.loads, read_text(folder / "options.jsonl").splitlines())}
    expected_flags = {"commonmark": 0, "tables": 0x100, "strikethrough": 0x200,
                      "task_lists": 0x800, "gfm_overlap": 0xB00}
    tables = []
    for case, identifier, label, kind in cases:
        lane = case.split("/")[0]
        if options[lane]["md4c_parser_flags"] != expected_flags[lane] or options[lane]["md4c_renderer_flags"] != 0:
            raise ValueError(f"{case}: md4c options do not match the named feature set")
        if case not in HEADLINE_CASES or set(verified[case]["output_sha256"]) != PARSERS:
            raise ValueError(f"{case}: not output-equivalent across all five parsers")
        original = outputs[case]
        hashes = {name: hashlib.sha256(html.encode()).hexdigest() for name, html in original["outputs"].items()}
        if mismatches(original["outputs"]) != verified[case]["mismatches"] or hashes != verified[case]["output_sha256"]:
            raise ValueError(f"{case}: verification disagrees with original parser outputs")
        review = workload_review(case, original["outputs"])
        if not review["comparable"]:
            raise ValueError(f"{case}: not comparable Markdown work")
        if "workload" in verified[case] and verified[case]["workload"] != review:
            raise ValueError(f"{case}: recorded workload review disagrees with output")
        values = {}
        for parser in ORDER:
            rows = [run[(case, parser)] for run in raw]
            if any(len(row["ns_per_render"]) != protocol["samples"] for row in rows):
                raise ValueError(f"{case}/{parser}: incomplete sampling")
            medians = [statistics.median(row["ns_per_render"]) for row in rows]
            ns = statistics.median(medians)
            summary = summaries[(case, parser)]
            if ns != summary["median_ns"] or medians != summary["run_medians_ns"]:
                raise ValueError(f"{case}/{parser}: summary disagrees with raw samples")
            if summary["bytes"] != verified[case]["bytes"] or any(row["bytes"] != verified[case]["bytes"] for row in rows):
                raise ValueError(f"{case}/{parser}: input size changed")
            expected_bytes = len(original["outputs"][parser].encode())
            if summary["output_bytes"] != expected_bytes or any(row["output_bytes"] != expected_bytes for row in rows):
                raise ValueError(f"{case}/{parser}: measured output size changed")
            values[parser] = (ns, medians, summary)
        ferro_ns = values["ferromark"][0]
        rows = []
        for parser in ORDER:
            ns, medians, summary = values[parser]
            rows.append({"parser": LABELS[parser], "latency": f"{ns / 1000:.3f} µs",
                "throughput": f"{summary['bytes'] / ns * 1e9 / 1048576:.1f} MiB/s",
                "ratio": "baseline" if parser == "ferromark" else f"{ferro_ns / ns:.2f}x",
                "medianNs": ns, "runMediansNs": medians,
                "spreadPercent": (max(medians) - min(medians)) / ns * 100,
                "outputBytes": summary["output_bytes"]})
        tables.append({"id":identifier, "case":case, "label":label, "kind":kind,
            "description": f"{verified[case]['bytes']:,} input bytes; fresh parser and owned output",
            "inputBytes":verified[case]["bytes"], "rows":rows, "outputReview":review})
    return metadata, tables, verified


def data_for(folder):
    # Corrections replace a complete five-parser case, never an individual cell.
    manifest = folder.parent / "publication-sources.json"
    overrides = json.loads(manifest.read_text())["overrides"] if manifest.exists() else {}
    if set(overrides) - {case for case, *_ in CASES}:
        raise ValueError("Unknown publication override case")
    metadata, tables, verified = load_publication(folder, [row for row in CASES if row[0] not in overrides])
    for case, directory in overrides.items():
        corrected_folder = folder.parent / directory
        corrected_meta, corrected_tables, corrected_verified = load_publication(
            corrected_folder, [row for row in CASES if row[0] == case])
        for key in ("ferromark_source_sha256", "bun_revision", "md4c_revision", "rustc", "cpu", "allocation",
                    "rustflags", "profile", "mimalloc_revision", "highway_revision", "md4c_source_sha256"):
            if corrected_meta[key] != metadata[key]:
                raise ValueError(f"Correction changed comparison environment: {key}")
        if read_text(corrected_folder / "catalog.jsonl") != read_text(folder / "catalog.jsonl"):
            raise ValueError("Correction changed frozen benchmark inputs")
        if (corrected_folder / "Cargo.lock").read_bytes() != (folder / "Cargo.lock").read_bytes():
            raise ValueError("Correction changed locked dependencies")
        corrected_tables[0]["source"] = str(corrected_folder.relative_to(REPO))
        tables.extend(corrected_tables)
        verified = corrected_verified
    tables.sort(key=lambda table: [row[0] for row in CASES].index(table["case"]))
    source = str(folder.relative_to(REPO))
    rustc = metadata["rustc"].splitlines()[0].split()[1]
    packages = tomllib.loads((folder / "Cargo.lock").read_text())["package"]
    versions = {}
    for name in ("pulldown-cmark", "comrak"):
        matches = [p["version"] for p in packages if p["name"] == name]
        if len(matches) != 1:
            raise ValueError(f"Expected one locked version of {name}")
        versions[name] = matches[0]
    measured_date = datetime.fromtimestamp(metadata["started_unix"], timezone.utc).strftime("%B %Y")
    return {
        "note":"Generated by benchmarks/bun-comparison/publish.py from reviewed raw samples. Do not hand-edit figures.",
        "source":source, "corrections":overrides, "ferromarkRevision":metadata["ferromark_revision"],
        "run":{"date":measured_date, "machine":metadata["cpu"], "os":" ".join(metadata["platform"].split("-")[:2]), "rustc":rustc},
        "competitors":{**versions, "md4c":MD4C_REV[:7], "bun":BUN_REV[:7]},
        "conditions":"All five parsers render trusted input with fresh parser state and owned HTML output. The feature set is named per table; bare autolinks, tag filtering, heading IDs, and other extensions are disabled. All parsers use Bun's pinned mimalloc, including md4c's C allocation calls. Rust uses the same pinned nightly compiler and generic CPU target; Bun retains its native Highway support. No PGO is used.",
        "scope":"These measurements are Apple Silicon results only; this comparison has not been re-measured on x86-64. The shared Bun-native support environment is part of the experiment, and the results do not measure the JavaScript runtime or Ferromark's secure-default rendering.",
        "summary":"Every displayed case passed the five-parser workload check and three alternating-order measurement runs. Speed ratios apply to these inputs, feature sets, and allocation lifecycle.",
        "headline":{"table":"commonmark-5k", "parser":"ferromark"},
        "verification":{"htmlEquivalent":sum(not c["mismatches"] for c in verified.values()), "total":len(verified)},
        "tables":[t for t in tables if t["kind"] == "size"],
        "featureTables":[t for t in tables if t["kind"] == "feature"],
    }


def table_markdown(table):
    lines = [f"### {table['label']}", "", table["description"] + ".", "",
        "| Parser | Time / document | Throughput | vs ferromark |", "| --- | ---: | ---: | ---: |"]
    for row in table["rows"]:
        cells = [row[k] for k in ("parser", "latency", "throughput", "ratio")]
        if row["parser"] == "ferromark":
            cells = [f"**{c}**" for c in cells]
        lines.append("| " + " | ".join(cells) + " |")
    return "\n".join(lines)


def readme_section(data):
    source = data["source"]
    env = data["run"]
    text = f'''## Benchmarks

Five native Markdown-to-HTML implementations, measured together: Ferromark,
Bun, pulldown-cmark, Comrak, and C-md4c. {env['machine']}, {env['os']},
rustc {env['rustc']}, {env['date']}. The [full report](docs/reports/2026-09-11-benchmark-refresh.md)
links raw timings, output differences, options, and source hashes.

{data['conditions']}

{data['scope']}

The main cases use 2, 5, and 10 KiB of synthetic Markdown with headings, emphasis,
links, lists, quotes, and fenced code. These are controlled size examples, not a
claim about typical usage. Tiny inputs diagnose per-call overhead; 50 KiB and
1 MiB remain long-document/stress cases in the detailed results.

'''
    text += "\n\n".join(table_markdown(t) for t in data["tables"])
    text += f'''

{data['summary']} Locked versions: pulldown-cmark {data['competitors']['pulldown-cmark']},
comrak {data['competitors']['comrak']}, md4c @ {data['competitors']['md4c']}, and bun @ {data['competitors']['bun']}.

### Feature-set comparisons

Time per complete document (microseconds, lower is faster). Each row uses the
same input and selected options for all five parsers. Input sizes and syntax
density differ between rows, so these are not additive feature prices.

| Input / feature set | Bytes | ferromark | Bun (native) | pulldown-cmark | comrak | md4c (C) |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
'''
    for table in data["featureTables"]:
        text += "| " + " | ".join([table["label"], str(table["inputBytes"]), *[r["latency"] for r in table["rows"]]]) + " |\n"
    text += f'''
The tables-only and GFM-subset rows use identical table/strikethrough input.
The first leaves strikethrough literal; the second renders it and also enables
unused task-list parsing. Neither enables the full five-extension GFM preset.
The GFM-subset row was remeasured for all five parsers after correcting the md4c
task-list flag; its [replacement evidence](docs/reports/2026-09-11-benchmark-refresh/native-corrected/summary.json)
remains separate from the original run.
Task-list rendering conventions and the recorded Bun alignment bug are accepted
for workload comparisons; the [output audit](docs/reports/2026-09-11-output-parity-audit.md)
identifies agreeing groups, renderer conventions, a Bun table bug, and Ferromark's
former reference-resolution limit. Ordinary HTML flow whitespace is accepted. {data['verification']['htmlEquivalent']} of
{data['verification']['total']} archived input/configuration pairs had matching HTML under the
revised whitespace check. HTML agreement is a diagnostic, not the timing gate.
The [workload contract](docs/arch/ARCH-COMP-002-workload-comparability.md) accepts
reviewed rendering differences while excluding missing features or unfinished work.

### Native Bun Markdown comparison

The [five-parser harness](benchmarks/bun-comparison/README.md) builds Bun's native
parser without JavaScript. Its [provenance](benchmarks/bun-comparison/PROVENANCE.md)
traces md4c through Zig and Rust ports. Bun retains C++ Highway search routines
and C mimalloc; all five parsers share the measured allocator environment.
The limited HTML comparison records text, URLs, code whitespace, and attributes.
Workload admission separately accepts documented table-alignment and task-rendering
differences; output fidelity remains visible rather than blocking timing.

Each displayed result is the median of three run medians. Each run has 80
alternating-order windows totaling at least five seconds per parser/input,
after three seconds of warmup. The [raw evidence]({source}/summary.json)
retains individual run medians and the complete samples. The [September 5 study](docs/reports/2026-09-05-bun-comparison.md)
and former stable/System-allocator comparisons are historical experiments;
their numbers are not mixed into these tables.

Follow the harness README to prepare Bun and the pinned md4c checkout:

```bash
git -C "$MD4C_DIR" checkout --detach {data['competitors']['md4c']}
python3 benchmarks/bun-comparison/prepare.py "$BUN_BENCH_DIR" "$BUN_BENCH_WORK" \\
  --md4c "$MD4C_DIR" --lockfile {source}/Cargo.lock
python3 benchmarks/bun-comparison/run.py "$BUN_BENCH_DIR" /private/tmp/new-benchmark-run
```

Normal library builds and package consumers do not build Bun or md4c.

### Fine-grained feature and document benchmarks

The [feature study](docs/reports/2026-09-11-benchmark-refresh.md#feature-and-lifecycle-costs)
replays the complete 140-scenario, 292-input/configuration catalog, including
core CommonMark constructs, boolean Markdown options, and link-base rewriting:

- **Activation:** options off/on on plain input with identical HTML isolate
  detection and setup without using the feature.
- **Actual syntax:** input uses the feature, so changed output includes real
  parsing/rendering work rather than only avoidable overhead.
- **Lifecycle:** fresh owned output versus a retained `Renderer` separates
  per-call setup from repeated parsing with existing buffers.

Allocation counters run separately from timings; requested bytes are not peak memory.
The [earlier generated feature tables](docs/reports/2026-09-10-markdown-feature-costs-final.md),
[GFM profile](docs/reports/2026-09-10-gfm-profiling.md), and
[optimization decisions](docs/reports/2026-09-10-markdown-feature-optimizations.md)
explain the hypotheses and known tradeoffs behind this rerun. Most unused options
were cheap in that study, with literal-autolink detection a measurable exception.
Feature-heavy documents differ in structure and output volume: choose options
for semantics and measure the workload your application actually uses.

The [two-parser harness](benchmarks/pulldown-comparison/README.md) also covers
extended syntax intersections. Those diagnostics and the stable-toolchain
Criterion suites use different environments/lifecycles and remain separate from
the verified five-parser tables above.
'''
    return text.rstrip() + "\n\n"



def native_report_section(data):
    winners = {}
    for table in [*data["tables"], *data["featureTables"]]:
        winner = min(table["rows"], key=lambda row: row["medianNs"])["parser"]
        winners.setdefault(winner, []).append(table["label"])
    observations = [f"{name} has the lowest measured median for: {', '.join(labels)}."
                    for name, labels in winners.items()]
    lines = [*observations, "",
        "These are results for the named inputs and contracts, not a universal parser ranking.", "",
        "Latencies are medians of three run medians. Spread is the range of those",
        "run medians divided by their median; it is not a confidence interval.",
        "Output byte counts may differ because of accepted serialization or renderer differences.",
        "The main README and homepage tables also report input throughput.", "",
        "| Input / configuration | Parser | Time / document | Run medians (µs) | Spread | Output bytes |",
        "| --- | --- | ---: | --- | ---: | ---: |",
    ]
    for table in [*data["tables"], *data["featureTables"]]:
        for row in table["rows"]:
            medians = ", ".join(f"{ns / 1000:.3f}" for ns in row["runMediansNs"])
            lines.append(f"| {table['label']} | {row['parser']} | {row['latency']} | {medians} | {row['spreadPercent']:.1f}% | {row['outputBytes']} |")
    return "\n".join(lines)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("folder", type=Path, nargs="?", default=REPO / "docs/reports/2026-09-11-benchmark-refresh/native")
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    data = data_for(args.folder.resolve())
    readme = REPO / "README.md"
    original = readme.read_text()
    start = original.index("## Benchmarks\n")
    end = original.index("\n## ", start + 1) + 1
    generated = original[:start] + readme_section(data) + original[end:]
    outputs = {REPO / "homepage/app/data/benchmarks.json":json.dumps(data, indent=2, ensure_ascii=False) + "\n", readme:generated}
    report = REPO / "docs/reports/2026-09-11-benchmark-refresh.md"
    report_text = report.read_text()
    start_marker, end_marker = "<!-- native-results:start -->", "<!-- native-results:end -->"
    start = report_text.index(start_marker) + len(start_marker)
    end = report_text.index(end_marker)
    outputs[report] = report_text[:start] + "\n" + native_report_section(data) + "\n" + report_text[end:]
    for path, content in outputs.items():
        if args.check:
            if path.read_text() != content:
                raise SystemExit(f"{path.relative_to(REPO)} differs from measured publication data")
        else:
            path.write_text(content)
    print("Publication tables match verified raw measurements" if args.check else "Published measured tables to README, homepage data, and report")


if __name__ == "__main__":
    main()
