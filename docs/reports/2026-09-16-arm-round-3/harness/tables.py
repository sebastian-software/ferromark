#!/usr/bin/env python3
"""Per-case markdown tables for one or more result directories: tables.py <S> <out.md> <title> name=heading ..."""
import json, sys
S, out, title = sys.argv[1], sys.argv[2], sys.argv[3]
lines = [f"# {title}", "", "Ratios are baseline time over candidate time (median of the paired windows); higher is faster.",
         "`rounds` lists the per-round medians. Baseline time is the median nanoseconds per document.", ""]
for spec in sys.argv[4:]:
    name, heading = spec.split("=", 1)
    rows = json.load(open(f"{S}/results/{name}/summary.json"))
    corpus = {c["name"]: c for c in json.load(open(f"{S}/results/{name}/corpus.json"))["cases"]}
    lines += [f"## {heading}", ""]
    for mode in ("fresh", "reuse", "parse", "render"):
        sel = sorted((r for r in rows if r["mode"] == mode), key=lambda r: r["baseline_over_candidate_median"])
        if not sel: continue
        lines += [f"### {mode}", "", "| Case | Category | Bytes | Ratio | Min | Max | Rounds | Baseline ns/doc |", "| --- | --- | ---: | ---: | ---: | ---: | --- | ---: |"]
        for r in sel:
            c = corpus[r["case"]]
            lines.append(f"| `{r['case']}` | {c['category']} | {c['byte_count']} | {r['baseline_over_candidate_median']:.3f} | {r['baseline_over_candidate_min']:.3f} | {r['baseline_over_candidate_max']:.3f} | {' / '.join(f'{x:.3f}' for x in r['round_medians'])} | {r['baseline_ns_per_document']:,.0f} |")
        lines.append("")
open(out, "w").write("\n".join(lines) + "\n")
print("wrote", out, sum(1 for l in lines if l.startswith("| `")), "rows")
