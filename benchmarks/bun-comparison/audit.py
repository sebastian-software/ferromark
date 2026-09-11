#!/usr/bin/env python3
"""Group archived HTML by agreement without treating any parser as the oracle."""
import argparse
import hashlib
import json
from pathlib import Path

from publish import read_text
from run import CanonicalHTML, PARSERS


def groups(outputs):
    if set(outputs) != PARSERS:
        raise ValueError("Expected all five parser outputs")
    result = {}
    for name, html in sorted(outputs.items()):
        tokens = json.dumps(CanonicalHTML(html).tokens, ensure_ascii=False)
        key = hashlib.sha256(tokens.encode()).hexdigest()
        result.setdefault(key, []).append(name)
    return list(result.values())


def audit(original, corrected):
    before = {row["case"]: row for row in json.loads((original / "verification.json").read_text())["cases"]}
    after = {row["case"]: row for row in map(json.loads, read_text(corrected / "verify.jsonl").splitlines())}
    rows = []
    for line in read_text(original / "verify.jsonl").splitlines():
        row = json.loads(line)
        case = row["case"]
        if not before[case]["mismatches"]:
            continue
        rows.append({"case": case, "original_gate_mismatches": before[case]["mismatches"],
                     "original_outputs_flow_groups": groups(row["outputs"]),
                     "corrected_outputs_flow_groups": groups(after[case]["outputs"])})
    return {"normalization": "HTML flow whitespace; preserve markup, attributes, word boundaries, and literal text",
            "cases": rows}


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("original", type=Path)
    p.add_argument("corrected", type=Path)
    p.add_argument("--check", type=Path, help="Verify a saved grouping audit")
    args = p.parse_args()
    result = audit(args.original, args.corrected)
    if args.check:
        if json.loads(args.check.read_text()) != result:
            raise SystemExit("Output groups differ from the archived evidence")
        print("Output groups match archived evidence")
    else:
        print(json.dumps(result, indent=2) + "\n", end="")


if __name__ == "__main__":
    main()
