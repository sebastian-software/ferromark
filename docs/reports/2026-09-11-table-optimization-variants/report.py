#!/usr/bin/env python3
"""Generate experiment results from the retained raw measurements."""
from pathlib import Path
import json
HERE=Path(__file__).resolve().parent
NAME=HERE.name

def rows(folder):return {(x['case'],x['lane']):x for x in json.loads((HERE/folder/'summary.json').read_text())}

def val(data,case,lane='ferro-reuse'):return data[case,lane]

text='''# Table optimization experiments — 2026-09-11

Five independent implementation variants were measured against the same frozen
baseline, followed by a combination experiment. **Only batching the ordinary
cell event sequence is retained.** CommonMark emphasis, strong emphasis, links,
and all configured inline features remain enabled. Public block events and parser
options are unchanged.

## Keep or discard

First-pass screening: five alternating paired 120 ms windows per case, fresh
parser state and reused output. Negative change means less elapsed time. Small
changes near 1–2% are not treated as compelling gains. This is pragmatic local
screening, not a statistical guarantee across machines.

| Variant | Plain | Emphasis/strong | Link column | 9 columns | Decision |
| --- | ---: | ---: | ---: | ---: | --- |
'''
variants=[('inline-split','Force inline cell splitter','Discard: negligible gain'),('short-scan','Scalar search for short tails','Discard: no reliable gain'),('delimiter','Direct delimiter-cell validation','Discard: small gain for extra code'),('batch-cells','Batch cell start/text/end','Keep: broad table improvement'),('cells16','16-cell local buffer','Discard: narrow gain, small ordinary-table cost'),('batch-cells+cells16','Batch + 16-cell buffer','Discard buffer: ordinary-table regression')]
for name,label,decision in variants:
 d=rows('screen-'+name)
 text+='| '+label+' | '+' | '.join(f"{val(d,c)['paired_median_percent']:+.2f}%" for c in ['plain','mixed','links','wide9'])+' | '+decision+' |\n'
text+='''
The final row compares the combined candidate against **batching alone**. All other
rows compare their individual candidate against the unmodified baseline. The
buffer experiment helps the nine-column allocation cliff, but the combination
spends some of the gains on the three practical cases and adds little at sixteen
columns. It is not included in production. Archived patches preserve rejected
experiments without leaving alternative implementations in the parser.

## Confirmation and pulldown-cmark comparison

Final real-repository builds: nine 250 ms windows for each practical case and
lifecycle. Before/after/pulldown triplets rotate and reverse within each case;
all three paths receive a one-second warmup before recorded measurements.
The table shows median microseconds per complete document. Both main Ferromark
columns and pulldown create parser state per call and reuse output. The change
columns use median within-round percentage changes; time columns use separate
medians, so their ratio can differ from the paired percentages. Negative values
in the last column mean Ferromark used less time than pulldown.

| Content | Ferromark before | Ferromark after | Paired change | pulldown-cmark | After vs pulldown |
| --- | ---: | ---: | ---: | ---: | ---: |
'''
d={(x['case'],x['lifecycle']):x for x in json.loads((HERE/'production-confirmation/summary.json').read_text())}
for case,label in [('plain','Plain text'),('mixed','Plain + emphasis + strong'),('links','Markdown link column')]:
 x=d[case,'reuse'];v=x['median_us']
 text+=f"| {label} | {v['before']:.2f} | {v['after']:.2f} | {x['median_change_percent']:+.2f}% | {v['pulldown']:.2f} | {x['median_versus_pulldown_percent']:+.2f}% |\n"
text+='\nOwned-output control, median microseconds and paired change:\n\n| Case | Before | After | Change | pulldown |\n| --- | ---: | ---: | ---: | ---: |\n'
for case in ['plain','mixed','links']:
 x=d[case,'owned'];v=x['median_us']
 text+=f"| {case} | {v['before']:.2f} | {v['after']:.2f} | {x['median_change_percent']:+.2f}% | {v['pulldown']:.2f} |\n"
text+='''
These synthetic workloads do not establish a universal parser ranking. The
measurement uses trusted rendering settings, Apple M1 Pro, the pinned dependency
graph, and ARM64 apple-m1/NEON flags. Differences of only a few percent against
pulldown should be read with the raw-window spread, not as a platform-independent
promise. Published five-parser benchmark figures are not replaced by these
local two-parser diagnostics.

## Guard measurements

Seven paired 250 ms windows per case/lifecycle. This includes the three practical
fixtures, one larger short-cell table, long cells, wide tables, escaped pipes/code,
plain prose, ordinary links outside tables, an existing mixed document, and a
deterministic generated edge-case document. Negative change is faster.

| Input | Fresh parser/reused output | Retained renderer |
| --- | ---: | ---: |
'''
g=rows('guard-confirmation')
for case in dict.fromkeys(k[0] for k in g):
 text+='| '+case+' | '+' | '.join(f"{val(g,case,l)['paired_median_percent']:+.2f}%" for l in ['ferro-reuse','ferro-retained'])+' |\n'
text+='''
The edge document combines 256 generated table-like fragments with valid and
invalid delimiter cells, 1–129 columns, optional outer pipes, empty cells,
Unicode, links, emphasis, entities, code, and escaped pipes. It is a differential
correctness/control workload, not a representative table speed headline.

## Retained implementation

The main HTML rendering loop recognizes the existing contiguous sequence
`TableCellStart`, `Text`, `TableCellEnd`. For an ordinary single-range cell without
backslashes, it writes the cell wrapper and calls the **same full inline renderer**
directly. This avoids three trips through the general block-event dispatcher and
`CellState` setup/collection/finalization. Inline parsing is neither disabled nor
replaced with a plain-text-only assumption.

Escaped cells, empty/padded cells, and other event sequences continue through the
existing path. Alignment, column spans, whitespace trimming, link-reference and
footnote state, HTML escaping, and rendering policy retain their behavior. The
public event representation is unchanged; the block parser and rejected scanner
experiments are unchanged in the final production diff. MDX's separate event-stream
entry and the nested footnote event loop retain their existing implementation.

The previously profiled cell-transition hypothesis explains the useful gain.
The profiler's `render_block_event` attribution was never treated as all removable
dispatch overhead; the measured change establishes the actual effect.

## Correctness and reproduction

Each candidate's raw HTML is compared **exactly to the baseline before timing**,
including fresh output and reusable-Renderer behavior inside the probe. The first
inline-only screen predates the generated edge document; the other screens and
all confirmations include their selected documented cases. No normalization is
used to excuse an optimization changing Ferromark output. Pulldown may use its
usual different HTML whitespace.

A renderer regression test alternates ordinary, emphasized, reference-link,
escaped-code and empty/ragged cells under trusted and untrusted policies, then
checks that delimiters and references do not leak into the next document. It was
run on the baseline before applying the retained source change. Existing full
all-feature tests cover GFM alignment, column-width hints, merged cells, MDX,
footnotes, policies, callbacks, and resource limits.

The isolated practical confirmation used nine paired 350 ms windows, including
owned output and retained Renderer controls. Its raw data remain separate from
the final real-repository run above.

All variants are built sequentially from an isolated baseline copy, using the
same probe path and compiler settings; no build runs concurrently with timings.
The System allocator, opt-level 3, fat LTO, one codegen unit, panic=abort, empty
Cargo feature set and repository CPU flags match between measured executables.
The lockfile preserves the prior measured production dependencies, including
smallvec 1.15.2 and html-escape 0.2.14. Each window warms up 32 renders and checks
the timer every 16 iterations. Case order rotates; baseline/candidate order
alternates. All windows, including outliers, are retained.

'''
text+=f'''```bash
python3 docs/reports/{NAME}/experiment.py build baseline
python3 docs/reports/{NAME}/experiment.py build batch-cells
python3 docs/reports/{NAME}/experiment.py measure batch-cells --out recheck --rounds 7 --ms 250 --lane ferro-reuse --lane ferro-retained
```

- [Screening/confirmation runner]({NAME}/experiment.py),
  [frozen lockfile]({NAME}/Cargo.lock),
  [environment]({NAME}/environment.json),
  [baseline source hashes]({NAME}/baseline-source.json),
  [frozen baseline source]({NAME}/baseline.tar.gz), and
  [retained production diff]({NAME}/retained.patch).
- [Real-repository confirmation]({NAME}/production-confirmation/summary.json)
  and [raw windows]({NAME}/production-confirmation/timings.jsonl),
  driven by [production.py]({NAME}/production.py).
- [Isolated practical confirmation]({NAME}/confirmation/summary.json) and
  [raw windows]({NAME}/confirmation/timings.jsonl).
- [Guard confirmation]({NAME}/guard-confirmation/summary.json) and
  [raw windows]({NAME}/guard-confirmation/timings.jsonl).
- Individual `screen-*` directories retain summaries, protocol, raw windows, input
  hashes, and raw HTML. `*.patch` files show each independent candidate; binary
  hashes live beside them. Raw outputs may be gzip-compressed without alteration.
- [Validation log]({NAME}/validation.log) records the baseline regression test and
  checks after applying the retained implementation.

The earlier [profiling report](2026-09-11-commonmark-table-costs.md) documents the
unchanged baseline and motivates these experiments.
'''
(HERE.parent/(NAME+'.md')).write_text(text)
