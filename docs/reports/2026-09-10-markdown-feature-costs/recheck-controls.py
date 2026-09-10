"""Longer paired recheck of the two noisy retained-renderer controls."""
import hashlib
import json
from pathlib import Path
import statistics
import subprocess

root = Path("target/feature-costs")
cases = {
    "inline-code": {
        "binaries": {"before": root / "baseline", "after": root / "final"},
        "args": ["measure", str(root / "catalog.json"), "core/inline_code/medium", "renderer", "500", "1"],
        "variant": "syntax",
    },
    "readme-commonmark": {
        "binaries": {"before": root / "gfm-baseline", "after": root / "gfm-final"},
        "args": ["measure", "readme", "commonmark", "renderer", "500", "1"],
    },
}
rows = []
for round_index in range(9):
    order = list(cases) if round_index % 2 == 0 else list(reversed(cases))
    for name in order:
        case = cases[name]
        labels = ["before", "after"] if round_index % 2 == 0 else ["after", "before"]
        pair = {}
        for label in labels:
            result = subprocess.run(
                [str(case["binaries"][label]), *case["args"]],
                check=True, capture_output=True, text=True,
            )
            samples = json.loads(result.stdout)
            if "variant" in case:
                samples = [row for row in samples if row["variant"] == case["variant"]]
            assert len(samples) == 1
            pair[label] = samples[0]["ns"]
        rows.append({"case": name, "round": round_index, **pair})
summary = []
for name in cases:
    samples = [row for row in rows if row["case"] == name]
    changes = [(row["after"] / row["before"] - 1) * 100 for row in samples]
    summary.append({
        "case": name,
        "before_ns": statistics.median(row["before"] for row in samples),
        "after_ns": statistics.median(row["after"] for row in samples),
        "paired_change_percent": statistics.median(changes),
        "paired_min_percent": min(changes), "paired_max_percent": max(changes),
    })
result = {
    "rounds": 9, "window_ms": 500,
    "binary_sha256": {
        name: {label: hashlib.sha256(path.read_bytes()).hexdigest()
               for label, path in case["binaries"].items()}
        for name, case in cases.items()
    },
    "summary": summary, "samples": rows,
}
(root / "control-recheck.json").write_text(json.dumps(result, indent=2) + "\n")
print(json.dumps(summary, indent=2))
