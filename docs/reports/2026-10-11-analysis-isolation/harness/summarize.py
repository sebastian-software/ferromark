"""Summarize retained Criterion estimates and alternating Node sample pairs."""
import argparse
import json
from pathlib import Path
import random
import statistics

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("results", type=Path)
root = parser.parse_args().results
summary = {"rust": [], "node": []}
for fixture in sorted((root / "rust").iterdir()):
    for before in sorted(fixture.glob("before_*")):
        operation = before.name.removeprefix("before_")
        estimates = [json.loads((item / "estimates.json").read_text())["slope"]
                     for item in (before, fixture / f"after_{operation}")]
        summary["rust"].append({
            "fixture": fixture.name, "operation": operation,
            "before_nanoseconds": estimates[0]["point_estimate"],
            "after_nanoseconds": estimates[1]["point_estimate"],
            "change_percent": 100 * (estimates[1]["point_estimate"] / estimates[0]["point_estimate"] - 1),
            "before_confidence_interval": estimates[0]["confidence_interval"],
            "after_confidence_interval": estimates[1]["confidence_interval"],
        })
node = json.loads((root / "node.json").read_text())
random.seed(521)
for case in node["results"]:
    pairs = case["pairs"]
    ratios = [pair["after"]["nanoseconds"] / pair["before"]["nanoseconds"] for pair in pairs]
    bootstraps = sorted(
        statistics.median(random.choices(ratios, k=len(ratios))) for _ in range(10000)
    )
    summary["node"].append({
        "operation": case["name"],
        "before_nanoseconds": statistics.median(pair["before"]["nanoseconds"] for pair in pairs),
        "after_nanoseconds": statistics.median(pair["after"]["nanoseconds"] for pair in pairs),
        "paired_change_percent": 100 * (statistics.median(ratios) - 1),
        "paired_bootstrap_95_percent_interval": [100 * (bootstraps[250] - 1), 100 * (bootstraps[9750] - 1)],
    })
print(json.dumps(summary, indent=2))
