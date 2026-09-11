#!/usr/bin/env python3
"""Render measured feature-cost tables from saved probe output."""
import argparse
import json
import gzip
import pathlib
import statistics as st

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("folder", type=pathlib.Path)
parser.add_argument("output", type=pathlib.Path)
parser.add_argument("--snapshot", choices=["baseline", "final"], default="baseline")
parser.add_argument("--revision", default="99fbf0107c8a08851ba48d6173da99003c5ca22f")
parser.add_argument("--replay-metadata", type=pathlib.Path,
                    help="Describe a fresh replay without relabeling historical CPU profiles")
args = parser.parse_args()


def load_json(path):
    if path.exists():
        return json.loads(path.read_text())
    with gzip.open(str(path) + ".gz", "rt") as stream:
        return json.load(stream)

rows = load_json(args.folder / f"{args.snapshot}-timings.json")
counts = load_json(args.folder / f"{args.snapshot}-counts.json")


def group(case_id, variant, lane):
    return [row for row in rows
            if (row["id"], row["variant"], row["lane"]) == (case_id, variant, lane)]


def ns(case_id, variant, lane):
    return st.median(row["ns"] for row in group(case_id, variant, lane))


def delta(case_id, lane):
    before = sorted(group(case_id, "off", lane), key=lambda row: row["round"])
    after = sorted(group(case_id, "on", lane), key=lambda row: row["round"])
    return st.median((y["ns"] / x["ns"] - 1) * 100
                     for x, y in zip(before, after, strict=True))


def kib(case_id, variant, lane):
    row = group(case_id, variant, lane)[0]
    return ns(case_id, variant, lane) / 1000 * 1024 / max(1, row["input_bytes"])


text = '''# Markdown feature costs and light documents — 2026-09-10

Baseline production: `99fbf0107c8a08851ba48d6173da99003c5ca22f`, including the
previous GFM optimizations. Apple M1 Pro, macOS 26.6.2, Rust 1.97.1 / LLVM 22.1.6,
release-debug, fat LTO, one codegen unit, apple-m1/NEON, System allocator.

The catalog has 140 scenarios and 292 input/configuration pairs. Five >=60 ms
windows per variant and API lifecycle, 16 warmup renders, alternating off/on
order. Allocation counters run in a separate profiling-feature executable.
A retained Renderer also retains output capacity; the owned lane constructs
fresh parser state and returns/drops a fresh String inside timing. These API
lifecycles answer different questions. CPU samples use the fresh-parser,
reused-output profile harness and are not phase-accurate timing percentages.

## Reading the results

- **Activation:** same plain input and byte-identical HTML, with one option off/on.
  This isolates detection/setup overhead without using the feature.
- **Syntax:** same input, feature off/on, intentionally different output. This is
  the total cost of interpreting that syntax, not avoidable overhead. Enabling
  comments, metadata removal, task lists, or code parsing can remove work.
- **Core constructs:** Markdown versus equally many ASCII `a` bytes. CommonMark
  constructs cannot all be switched off. These are workload comparisons with
  different semantics, structure, and output volume, not additive feature prices.

All cases use Untrusted rendering except the disallowed-HTML filter, which needs
Trusted rendering to exercise its contract; that policy is fixed across its pair.
Merged cells and width hints enable tables in both variants. Front matter is only
recognized at the document start. Footnote/reference labels are unique per unit.
The catalog covers all 21 boolean Markdown options and link-base rewriting;
MDX and custom code-renderer callbacks are outside this Markdown study.

Values below describe this baseline. Follow-up optimizations and measurements
are recorded separately. Small differences around 3% should be treated as noise
unless a targeted longer run confirms them. These synthetic workloads describe
cost on one machine; they do not establish universal feature rankings.

## Option activation without matching syntax

Percent change in elapsed time; negative is faster. All activation pairs have
identical HTML. The inputs contain 17 bytes, about 1 KiB, or about 16 KiB of prose.

| Option | 17 B fresh owned | 1 KiB retained Renderer | 16 KiB retained Renderer |
| --- | ---: | ---: | ---: |
'''
features = list(dict.fromkeys(r['feature'] for r in rows if r['group'] == 'activation'))
for f in features:
    for size in ['tiny', '1k', '16k']:
        assert all(r['same_html_as_control']
                   for r in group(f'activation/{f}/{size}', 'on', 'owned'))
    text += f'| {f} | {delta(f"activation/{f}/tiny","owned"):+.1f}% | {delta(f"activation/{f}/1k","renderer"):+.1f}% | {delta(f"activation/{f}/16k","renderer"):+.1f}% |\n'
text += '''
## Features actually used

Small is one syntax unit; medium repeats it 16 times (except front matter).
Units differ in size and syntax density. The per-KiB column normalizes input
bytes, not output size or semantic work; the off/on percentage is a different
comparison and must not be used as a universal ranking.

| Feature | Small fresh owned (µs) | Medium retained (µs/KiB) | Medium off→on time | Medium retained allocation calls |
| --- | ---: | ---: | ---: | ---: |
'''
for f in features:
    id = f'syntax/{f}/medium'
    count = next(r['allocation_calls'] for r in counts if r['id']==id and r['variant']=='on' and r['lane']=='renderer')
    text += f'| {f} | {ns(f"syntax/{f}/small","on","owned")/1000:.3f} | {kib(id,"on","renderer"):.2f} | {delta(id,"renderer"):+.1f}% | {count} |\n'
text += '''
## CommonMark workload comparison

Medium repeats each syntax unit 32 times. The plain control is a single ASCII
run with exactly the same byte count. Differences include block count and output
expansion. Use these figures to choose profiling targets, not to sum feature costs.

| Construct | Small fresh owned (µs) | Medium retained (µs/KiB) | Equal-byte plain retained (µs/KiB) |
| --- | ---: | ---: | ---: |
'''
for f in dict.fromkeys(r['feature'] for r in rows if r['group']=='core'):
    id = f'core/{f}/medium'
    text += f'| {f} | {ns(f"core/{f}/small","syntax","owned")/1000:.3f} | {kib(id,"syntax","renderer"):.2f} | {kib(id,"plain-control","renderer"):.2f} |\n'
text += '''
## Fixed per-call cost

| Input / preset | Fresh owned (µs) | Retained Renderer (µs) | Owned allocation calls |
| --- | ---: | ---: | ---: |
'''
for size in ['empty','tiny','plain-1k','light-1k','plain-16k','readme']:
    for v in ['commonmark', 'default']:
        id = f'lifecycle/presets/{size}'
        count = next(r['allocation_calls'] for r in counts if r['id']==id and r['variant']==v and r['lane']=='owned')
        text += f'| {size} / {v} | {ns(id,v,"owned")/1000:.3f} | {ns(id,v,"renderer")/1000:.3f} | {count} |\n'
text += '''
## Reproduction and evidence

The adjacent directory preserves the full catalog, all timing windows, allocation
counts, and four CPU profiles. `scripts/markdown-feature-catalog.py` recreates the
catalog; `examples/feature_cost_probe.rs` measures it. For example:

```sh
python3 scripts/markdown-feature-catalog.py target/feature-costs/catalog.json
cargo run --locked --profile release-debug --example feature_cost_probe -- measure target/feature-costs/catalog.json
cargo run --locked --profile release-debug --features profiling --example feature_cost_probe -- counts target/feature-costs/catalog.json
```

Always preserve an uninstrumented binary before building allocation counters.
`compare-feature-probes.py` requires exact HTML for all 292 pairs before timing
before/after binaries. Requested bytes are cumulative, not peak/live memory.
The report tables are generated by `scripts/summarize-markdown-features.py`.
'''
if args.snapshot == 'final':
    text = text.replace('Baseline production: `99fbf0107c8a08851ba48d6173da99003c5ca22f`, including the\nprevious GFM optimizations.',f'Final production: `{args.revision}`, including the\nmeasured lightweight-document and feature optimizations.')
    text = text.replace('Values below describe this baseline. Follow-up optimizations and measurements\nare recorded separately.', 'Values below describe the final optimized implementation. See the\n[optimization log](2026-09-10-markdown-feature-optimizations.md) for paired\nbefore/after comparisons and the original baseline study for earlier costs.')
    text = text.replace('The adjacent directory preserves the full catalog, all timing windows, allocation\ncounts, and four CPU profiles.', 'The `2026-09-10-markdown-feature-costs/` directory preserves the full catalog,\nfinal timing windows and allocation counts, plus four baseline CPU profiles.')
    text = text.replace('# Markdown feature costs and light documents —', '# Final Markdown feature costs and light documents —')
if args.replay_metadata:
    metadata = json.loads(args.replay_metadata.read_text())
    opening_end = text.index("\n## Reading the results")
    text = f'''# Markdown feature costs and light documents — {metadata['date']}

Production revision: `{metadata['revision']}`. {metadata['environment']}.
Frozen catalog replay; no new parser optimization or CPU profile in this run.
The catalog contains 140 scenarios and 292 input/configuration pairs. Five
windows of at least 60 ms per variant and lifecycle, 16 warmup renders, with
alternating off/on order. Allocation counters run in a separate executable.
Fresh owned output includes parser construction and String destruction;
retained Renderer keeps parser buffers and output capacity between calls.
''' + text[opening_end:]
    start = text.index("Values below describe")
    end = text.index("Small differences around", start)
    text = text[:start] + "Values below describe this replay. " + text[end:]
    start = text.index("## Reproduction and evidence")
    text = text[:start] + f'''## Reproduction and evidence

The [{metadata['artifact_directory']}/]({metadata['artifact_directory']}/)
directory retains the frozen catalog, separate timing/counter outputs, exact
commands, source and executable hashes, and compiler details. Timing executables
were preserved before the profiling-feature counter build. Raw JSON may be gzip
compressed; decompress it before passing the catalog to the Rust probe.

```bash
feature-cost-probe measure catalog.json all all 60 5
feature-count-probe counts catalog.json all all
python3 scripts/summarize-markdown-features.py {args.folder.as_posix()} {args.output.as_posix()} \\
  --snapshot final --revision {metadata['revision']} \\
  --replay-metadata {args.replay_metadata.as_posix()}
```

The executable paths and build commands are recorded in the replay metadata.
Requested bytes are cumulative, not peak/live memory. These are short internal
feature diagnostics, not the longer repeated five-parser publication protocol.
Historical profiles and before/after experiments remain in the
[optimization report](2026-09-10-markdown-feature-optimizations.md).
'''
args.output.write_text(text)
