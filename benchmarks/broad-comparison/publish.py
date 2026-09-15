#!/usr/bin/env python3
"""Validate and archive the broad benchmark; produce stratified readable tables."""

import argparse
from collections import Counter
import csv
import gzip
import hashlib
import importlib.util
import json
from pathlib import Path
import shutil
import statistics

ROOT = Path(__file__).resolve().parent
SPEC = importlib.util.spec_from_file_location("broad_run", ROOT / "run.py")
RUN = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(RUN)
SPEC2 = importlib.util.spec_from_file_location("broad_corpus", ROOT / "make_corpus.py")
CORPUS = importlib.util.module_from_spec(SPEC2)
SPEC2.loader.exec_module(CORPUS)
ADMITTED = {"exact", "serialization-equivalent"}


def leader(row):
    if row["main_over_v2_min"] > 1.05:
        return "v2"
    if row["main_over_v2_max"] < 1 / 1.05:
        return "main"
    return "close/variable"


def aggregate(cases, by_key, verification, key):
    values = [label for _, label in CORPUS.BINS] if key == "size_bin" else sorted({c[key] for c in cases})
    rows = []
    for value in values:
        group = [c for c in cases if c[key] == value]
        if not group:
            continue
        admitted = [c for c in group if verification[c["name"]]["status"] in ADMITTED]
        row = {"stratum": value, "total": len(group), "admitted": len(admitted), "modes": {}}
        for mode in RUN.MODES:
            data = [by_key[c["name"], mode] for c in admitted]
            row["modes"][mode] = {
                "geomean_main_over_v2": statistics.geometric_mean(d["main_over_v2_median"] for d in data) if data else None,
                "leaders": dict(Counter(leader(d) for d in data)),
            }
        rows.append(row)
    return rows


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("results", type=Path)
    parser.add_argument("report", type=Path)
    args = parser.parse_args()
    source, dest = args.results, args.report
    corpus = json.loads((source / "corpus.json").read_text())
    cases = corpus["cases"]
    verification = json.loads((source / "verification.json").read_text())
    samples = json.loads((source / "samples.json").read_text())
    config = json.loads((source / "run.json").read_text())
    summary = json.loads((source / "summary.json").read_text())
    build = json.loads((source / "build.json").read_text())
    assert RUN.BASE.summarize(samples, config["groups"], verification) == summary
    assert RUN.groups_for(cases) == config["groups"]
    assert RUN.BASE.digest(source / "corpus.json") == config["corpus_sha256"]
    assert RUN.BASE.digest(ROOT / "run.py") == config["runner_sha256"]
    assert RUN.BASE.digest(ROOT.parent / "current-comparison/run.py") == config["shared_runner_sha256"]
    assert RUN.BASE.digest(ROOT.parent / "current-comparison/worker.rs") == build["worker_sha256"]
    expected = config["rounds"] * config["pairs_per_round"]
    assert len(samples) == len(config["groups"]) * len(RUN.MODES) * expected
    assert len(summary) == len(config["groups"]) * len(RUN.MODES)
    assert all(r["pairs"] == expected for r in summary)
    for case in cases:
        raw = case["input"].encode()
        assert hashlib.sha256(raw).hexdigest() == case["sha256"] and len(raw) == case["byte_count"]
        result = verification[case["name"]]
        assert RUN.classify(result["outputs"]["main"], result["outputs"]["v2"]) == result["status"]
    for engine in RUN.ENGINES:
        assert RUN.BASE.digest(source / f"{engine}.Cargo.lock") == build["engines"][engine]["lock_sha256"]
    dest.mkdir(parents=True, exist_ok=False)
    for name in ("corpus.json", "verification.json", "samples.json"):
        (dest / f"{name}.gz").write_bytes(gzip.compress((source / name).read_bytes(), mtime=0))
    for name in ("run.json", "build.json", "summary.json", "main.Cargo.lock", "v2.Cargo.lock"):
        shutil.copyfile(source / name, dest / name)
    for name in ("source-validation.json", "preflight.json"):
        if (source / name).exists():
            shutil.copyfile(source / name, dest / name)
    paths = sorted(ROOT.glob("*.py")) + [ROOT / "README.md"] + sorted((ROOT.parent / "current-comparison").glob("*.py")) + [ROOT.parent / "current-comparison/worker.rs"]
    hashes = {str(p.relative_to(ROOT.parent)): RUN.BASE.digest(p) for p in paths}
    (dest / "harness-hashes.json").write_text(json.dumps(hashes, indent=2) + "\n")
    by_key = {(r["group"], r["mode"]): r for r in summary}
    strata = {key: aggregate(cases, by_key, verification, key) for key in ("category", "size_bin", "collection")}
    (dest / "strata.json").write_text(json.dumps(strata, indent=2, ensure_ascii=False) + "\n")
    with (dest / "summary.csv").open("w", newline="") as handle:
        writer = csv.writer(handle)
        writer.writerow(["case", "category", "size_bin", "bytes", "profile", "output", "mode", "main_us", "v2_us", "main_over_v2", "ratio_min", "ratio_max", "leader"])
        for c in cases:
            for mode in RUN.MODES:
                r = by_key[c["name"], mode]
                writer.writerow([c["name"], c["category"], c["size_bin"], c["byte_count"], c["profile"], verification[c["name"]]["status"], mode, r["main_ns_per_document"] / 1000, r["v2_ns_per_document"] / 1000, r["main_over_v2_median"], r["main_over_v2_min"], r["main_over_v2_max"], leader(r)])
    statuses = Counter(v["status"] for v in verification.values())
    lines = ["# Broad Markdown: Ferromark main vs v2", "", f"Measured {config['host_before']['time_utc']} on {config['host_before']['cpu']}.", "",
             f"**{len(cases)} frozen inputs, {min(c['byte_count'] for c in cases):,}–{max(c['byte_count'] for c in cases):,} UTF-8 bytes.** Same parser versions and exact binaries as the first comparison; broader inputs are the experimental change.", "",
             "Ratios below are **main time / v2 time: above 1 means v2 is faster**. Category and size tables are equal-document geometric means of admitted cases only. They are overlapping views, not scores to combine. No production-wide winner is inferred from this selection.", "",
             f"Output checks: {statuses['exact']} byte-identical, {statuses['serialization-equivalent']} serialization-equivalent, {statuses['heading-id-only']} heading-ID-only differences, {statuses['different']} other differences. Only the first two statuses enter comparable aggregates. Every input remains measured and visible.", ""]
    for title, key in (("By content family", "category"), ("By input size", "size_bin"), ("By source collection", "collection")):
        lines += [f"## {title}", "", "| Stratum | Comparable / total | Fresh ratio | Owned ratio | Reuse ratio | Reuse leads: main / close / v2 |", "| --- | ---: | ---: | ---: | ---: | --- |"]
        for row in strata[key]:
            ratios = [f"{row['modes'][m]['geomean_main_over_v2']:.3f}×" if row['modes'][m]['geomean_main_over_v2'] is not None else "—" for m in RUN.MODES]
            wins = row["modes"]["reuse"]["leaders"]
            lines.append(f"| {row['stratum']} | {row['admitted']} / {row['total']} | {' | '.join(ratios)} | {wins.get('main', 0)} / {wins.get('close/variable', 0)} / {wins.get('v2', 0)} |")
        lines += [""]
    sizes = [label for _, label in CORPUS.BINS if any(c["size_bin"] == label for c in cases)]
    lines += ["## Content × size coverage", "", "Full-reuse geometric mean, with comparable/total document counts. Empty cells are unmeasured combinations; no interpolation is implied.", "",
              "| Content | " + " | ".join(sizes) + " |", "| --- | " + " | ".join("---" for _ in sizes) + " |"]
    for category in sorted({c["category"] for c in cases}):
        cells = []
        for size in sizes:
            group = [c for c in cases if c["category"] == category and c["size_bin"] == size]
            admitted = [c for c in group if verification[c["name"]]["status"] in ADMITTED]
            ratio = statistics.geometric_mean(by_key[c["name"], "reuse"]["main_over_v2_median"] for c in admitted) if admitted else None
            cells.append(f"{ratio:.2f}× ({len(admitted)}/{len(group)})" if ratio is not None else f"diagnostic (0/{len(group)})" if group else "—")
        lines.append(f"| {category} | " + " | ".join(cells) + " |")
    lines += [""]
    lines += ["A lead requires every paired window to exceed a 5% speed advantage. Close/variable includes small differences and overlapping paired ranges; it is not a formal confidence interval.", "",
              "## Individual inputs", "", "Times are microseconds. Each lifecycle cell is `main / v2 (ratio)`. The final column gives the nine paired ratios' range for full reuse. Diagnostic rows cannot establish equivalent-output speedups.", "",
              "| Input | Bytes | Output | Fresh | Owned | Reuse | Reuse range |", "| --- | ---: | --- | --- | --- | --- | --- |"]
    for c in sorted(cases, key=lambda c: (c["category"], c["byte_count"], c["name"])):
        cells = []
        for mode in RUN.MODES:
            r = by_key[c["name"], mode]
            cells.append(f"{r['main_ns_per_document']/1000:.3f} / {r['v2_ns_per_document']/1000:.3f} ({r['main_over_v2_median']:.2f}×)")
        r = by_key[c["name"], "reuse"]
        lines.append(f"| {c['name']} | {c['byte_count']:,} | {verification[c['name']]['status']} | {' | '.join(cells)} | {r['main_over_v2_min']:.2f}–{r['main_over_v2_max']:.2f}× |")
    lines += ["", "## Rotating collections", "", "Fixed cohorts visit each input once per traversal. Times are total microseconds per complete collection, rather than per-document averages. A cohort with any output mismatch is diagnostic; membership is never silently reduced.", "", "| Collection | Mode | Documents | Comparable | Main µs | v2 µs | Ratio |", "| --- | --- | ---: | --- | ---: | ---: | ---: |"]
    for r in summary:
        if not r["group"].startswith("rotating-"):
            continue
        n = r["documents_per_iteration"]
        lines.append(f"| {r['group']} | {r['mode']} | {n} | {r['comparable_output']} | {r['main_ns_per_document']*n/1000:.3f} | {r['v2_ns_per_document']*n/1000:.3f} | {r['main_over_v2_median']:.3f}× |")
    lines += ["", "## Evidence and limits", "",
              f"- Main: `{build['engines']['main']['revision']}`. v2: `{build['engines']['v2']['revision']}`. No parser changes.",
              f"- {config['rounds']} rounds × {config['pairs_per_round']} alternating pairs; {len(samples):,} pairs total. At least {config['window_ms']} ms/window plus 75 ms warmups. Every raw window and round median is retained.",
              "- CommonMark for encyclopedia prose; GFM for comments/docs, trusted HTML, heading IDs enabled, footnotes disabled. Conversion, I/O, process startup, bindings, highlighting, and MDX execution are not timed.",
              "- Fresh includes each API's actual setup costs. Reuse still parses every document; no AST or rendered-result cache is reused.",
              "- Source families, sizes, and comments are deliberately varied; this is not a sampled production traffic distribution. Multiple excerpts from one article are correlated.",
              "- Strict output admission can leave sparse or empty cells. Mismatching cases remain diagnostics; their ratios do not prove compatibility.",
              "- Local Apple Silicon measurement on an active desktop, with browser/backup activity observed at preflight. Host power/load observations are archived; unavailable thermal telemetry is recorded. Small differences are not portable claims.", "",
              "[Methodology and reproduction](../../../benchmarks/broad-comparison/README.md) · [CSV](summary.csv) · [Stratified data](strata.json) · [Per-case summaries and round medians](summary.json)", "",
              "[Frozen corpus and provenance](corpus.json.gz) · [Full HTML and diffs](verification.json.gz) · [Raw timing pairs](samples.json.gz) · [Run configuration](run.json) · [Build hashes](build.json) · [Harness hashes](harness-hashes.json)", ""]
    if (dest / "source-validation.json").exists():
        lines += ["[Independent source/archive byte verification](source-validation.json)", ""]
    (dest / "README.md").write_text("\n".join(lines))
    print(dest / "README.md")


if __name__ == "__main__":
    main()
