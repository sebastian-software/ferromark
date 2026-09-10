#!/usr/bin/env python3
"""Render measured feature-cost tables from saved probe output."""
import json
import pathlib
import statistics as st
import sys
folder=pathlib.Path(sys.argv[1]);out=pathlib.Path(sys.argv[2])
rows=json.loads((folder/'baseline-timings.json').read_text())
counts=json.loads((folder/'baseline-counts.json').read_text())
def group(id,variant,lane): return [r for r in rows if r['id']==id and r['variant']==variant and r['lane']==lane]
def ns(id,v,l): return st.median(r['ns'] for r in group(id,v,l))
def delta(id,l):
 a=sorted(group(id,'off',l),key=lambda r:r['round']);b=sorted(group(id,'on',l),key=lambda r:r['round'])
 return st.median((y['ns']/x['ns']-1)*100 for x,y in zip(a,b,strict=True))
def kib(id,v,l):
 r=group(id,v,l)[0];return ns(id,v,l)/1000*1024/max(1,r['input_bytes'])
text='''# Markdown feature costs and light documents — 2026-09-10

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
features=list(dict.fromkeys(r['feature'] for r in rows if r['group']=='activation'))
for f in features:
 for size in ['tiny','1k','16k']: assert all(r['same_html_as_control'] for r in group(f'activation/{f}/{size}','on','owned'))
 text+=f'| {f} | {delta(f"activation/{f}/tiny","owned"):+.1f}% | {delta(f"activation/{f}/1k","renderer"):+.1f}% | {delta(f"activation/{f}/16k","renderer"):+.1f}% |\n'
text+='''
## Features actually used

Small is one syntax unit; medium repeats it 16 times (except front matter).
Units differ in size and syntax density. The per-KiB column normalizes input
bytes, not output size or semantic work; the off/on percentage is a different
comparison and must not be used as a universal ranking.

| Feature | Small fresh owned (µs) | Medium retained (µs/KiB) | Medium off→on time | Medium retained allocation calls |
| --- | ---: | ---: | ---: | ---: |
'''
for f in features:
 id=f'syntax/{f}/medium'; count=next(r['allocation_calls'] for r in counts if r['id']==id and r['variant']=='on' and r['lane']=='renderer')
 text+=f'| {f} | {ns(f"syntax/{f}/small","on","owned")/1000:.3f} | {kib(id,"on","renderer"):.2f} | {delta(id,"renderer"):+.1f}% | {count} |\n'
text+='''
## CommonMark workload comparison

Medium repeats each syntax unit 32 times. The plain control is a single ASCII
run with exactly the same byte count. Differences include block count and output
expansion. Use these figures to choose profiling targets, not to sum feature costs.

| Construct | Small fresh owned (µs) | Medium retained (µs/KiB) | Equal-byte plain retained (µs/KiB) |
| --- | ---: | ---: | ---: |
'''
for f in dict.fromkeys(r['feature'] for r in rows if r['group']=='core'):
 id=f'core/{f}/medium';text+=f'| {f} | {ns(f"core/{f}/small","syntax","owned")/1000:.3f} | {kib(id,"syntax","renderer"):.2f} | {kib(id,"plain-control","renderer"):.2f} |\n'
text+='''
## Fixed per-call cost

| Input / preset | Fresh owned (µs) | Retained Renderer (µs) | Owned allocation calls |
| --- | ---: | ---: | ---: |
'''
for size in ['empty','tiny','plain-1k','light-1k','plain-16k','readme']:
 for v in ['commonmark','default']:
  id=f'lifecycle/presets/{size}';count=next(r['allocation_calls'] for r in counts if r['id']==id and r['variant']==v and r['lane']=='owned')
  text+=f'| {size} / {v} | {ns(id,v,"owned")/1000:.3f} | {ns(id,v,"renderer")/1000:.3f} | {count} |\n'
text+='''
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
out.write_text(text)
