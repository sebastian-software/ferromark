#!/usr/bin/env python3
"""Regenerate the measured table from archived samples; --check verifies it."""
import argparse
import json
from pathlib import Path
import statistics

HERE = Path(__file__).resolve().parent
REPORT = HERE.with_suffix(".md")
START = "<!-- native-cmark-measurements:start -->"
END = "<!-- native-cmark-measurements:end -->"


def load(path):
    return json.loads(path.read_text())


def table():
    metadata = load(HERE / "metadata.json")
    assert metadata["finished_unix"] > metadata["started_unix"]
    protocol = metadata["protocol"]
    assert protocol["mode"] == "measurement"
    lines = [
        "| Native pair | Input | Bytes | Ferromark µs | Competitor µs | Competitor / Ferromark | Run-median range, Ferromark / competitor µs |",
        "| --- | --- | ---: | ---: | ---: | ---: | --- |",
    ]
    for name, pair in metadata["pairs"].items():
        folder = HERE / name
        summary = {(r["case"], r["parser"]): r for r in load(folder / "summary.json")}
        expected = {(case, parser) for case in pair["selected"] for parser in ("ferromark", name)}
        assert set(summary) == expected
        medians = {key: [] for key in expected}
        for repeat in range(protocol["runs"]):
            rows = load(folder / f"samples-{repeat}.json")
            assert len(rows) == len(expected)
            assert {(r["case"], r["parser"]) for r in rows} == expected
            for row in rows:
                key = row["case"], row["parser"]
                samples = row["ns_per_render"]
                assert len(samples) == protocol["samples"]
                assert all(0 < value < float("inf") for value in samples)
                assert row["bytes"] == summary[key]["bytes"]
                assert row["output_bytes"] == summary[key]["output_bytes"]
                medians[key].append(statistics.median(samples))
        for key, values in medians.items():
            assert values == summary[key]["run_medians_ns"]
            assert statistics.median(values) == summary[key]["median_ns"]
        for case in pair["selected"]:
            ferro, other = (summary[case, parser] for parser in ("ferromark", name))
            assert ferro["bytes"] == other["bytes"]
            a, b = ferro["median_ns"], other["median_ns"]
            ranges = [f"{min(r['run_medians_ns']) / 1000:.2f}–{max(r['run_medians_ns']) / 1000:.2f}"
                      for r in (ferro, other)]
            lines.append(f"| {name} | `{case}` | {ferro['bytes']:,} | {a / 1000:.2f} | {b / 1000:.2f} | {b / a:.2f}× | {' / '.join(ranges)} |")
    return "\n".join(lines)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    current = REPORT.read_text()
    before, remainder = current.split(START)
    _, after = remainder.split(END)
    generated = before + START + "\n\n" + table() + "\n\n" + END + after
    if args.check:
        if generated != current:
            raise SystemExit("Measured table differs from archived samples")
        print("Measured table matches all archived samples and run summaries")
    else:
        REPORT.write_text(generated)


if __name__ == "__main__":
    main()
