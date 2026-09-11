#!/usr/bin/env python3
"""Compare Bencher output measured for base and head on the same CI runner."""

import argparse
import os
from pathlib import Path
import re

LINE = re.compile(r"^test (.+?) \.\.\. bench:\s*([\d,]+) ns/iter \(\+/- [\d,]+\)$")


def parse(text):
    values = {}
    for line in text.splitlines():
        if not line.startswith("test ") or "bench:" not in line:
            continue
        match = LINE.fullmatch(line.strip())
        if not match:
            raise ValueError(f"Malformed benchmark result: {line}")
        name, value = match.groups()
        value = int(value.replace(",", ""))
        if name in values or value <= 0:
            raise ValueError(f"Duplicate or nonpositive benchmark result: {name}")
        values[name] = value
    if not values:
        raise ValueError("No benchmark results found")
    return values


def compare(base, head, threshold=1.2):
    missing = base.keys() - head.keys()
    if missing:
        raise ValueError(
            f"Head omitted baseline benchmarks: {', '.join(sorted(missing))}"
        )
    regressions = []
    lines = ["| Benchmark | Base ns | Head ns | Ratio |", "| --- | ---: | ---: | ---: |"]
    for name, current in head.items():
        previous = base.get(name)
        if previous is None:
            lines.append(f"| {name} | new | {current} | — |")
            continue
        ratio = current / previous
        lines.append(f"| {name} | {previous} | {current} | {ratio:.3f} |")
        if ratio > threshold:
            regressions.append(name)
    return regressions, "\n".join(lines)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("base", type=Path)
    parser.add_argument("head", type=Path)
    parser.add_argument("--base-ref", required=True)
    parser.add_argument("--head-ref", required=True)
    args = parser.parse_args()
    regressions, table = compare(
        parse(args.base.read_text()), parse(args.head.read_text())
    )
    report = (
        f"## Performance comparison on the same runner\n\nBase: `{args.base_ref}`; "
        f"head: `{args.head_ref}`. Failure threshold: >20% slower.\n\n{table}\n"
    )
    print(report)
    if summary := os.environ.get("GITHUB_STEP_SUMMARY"):
        with Path(summary).open("a") as stream:
            stream.write(report)
    if regressions:
        raise SystemExit(f"Performance regressions: {', '.join(regressions)}")


if __name__ == "__main__":
    main()
