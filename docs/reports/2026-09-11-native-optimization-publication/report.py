#!/usr/bin/env python3
"""Generate the fresh publication report from complete measured evidence."""
from pathlib import Path
import json,sys
HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[2]
sys.path.insert(0,str(ROOT/'benchmarks/bun-comparison'))
from publish import data_for,native_report_section,ORDER
folder=HERE/'native'
data=data_for(folder)
metadata=json.loads((folder/'metadata.json').read_text())
summary=json.loads((folder/'summary.json').read_text())
verification=json.loads((folder/'verification.json').read_text())
cases={}
for row in summary:cases.setdefault(row['case'],{})[row['parser']]=row
assert len(cases)==len(verification['cases'])==42
assert all(set(rows)==set(ORDER) for rows in cases.values())
headline=set(metadata['protocol']['headline_cases'])
assert all(len(row['run_medians_ns'])==(3 if case in headline else 1) for case,rows in cases.items() for row in rows.values())
faster=[case for case,rows in cases.items() if rows['ferromark']['median_ns']<rows['pulldown-cmark']['median_ns']]
slower=[case for case,rows in cases.items() if rows['ferromark']['median_ns']>rows['pulldown-cmark']['median_ns']]
text=f'''# Native benchmark publication after table and short-document optimization

**Date:** 2026-09-11. **Measured source:** `{metadata['ferromark_revision']}` plus
the [archived production patch](2026-09-11-native-optimization-publication/native/ferromark.patch).
The full [source hashes and environment](2026-09-11-native-optimization-publication/native/metadata.json)
identify the measured build; the working tree was not described as a clean commit.

The [retained table and short-document optimizations](2026-09-11-native-hotspot-optimization.md) are included, alongside the earlier table-cell batching.
This is a completely fresh five-parser run. None of the earlier publication's
samples or mixed table/strikethrough headline cases are reused.

## Does Ferromark beat pulldown everywhere?

No universal speed claim follows from a synthetic corpus. Within this run,
Ferromark uses less median time than pulldown-cmark in **{len(faster)} of {len(cases)}**
measured input/configuration pairs; pulldown uses less time in **{len(slower)}**.
Of the **{len(headline)} repeated publication cases**, Ferromark is lower in
**{len(headline.intersection(faster))}**. Counts are descriptive and not a weighted
score for real Markdown documents. The remaining diagnostic cases have only one
measurement process each, so small differences there deserve particular caution.

Pulldown has the lower median in:

'''
for case in slower:
 r=cases[case];f=r['ferromark']['median_ns'];p=r['pulldown-cmark']['median_ns']
 text+=f'- `{case}`: Ferromark {f/1000:.3f} µs, pulldown {p/1000:.3f} µs ({(f/p-1)*100:.1f}% more elapsed time).\n'
text+='''
## Repeated publication results

The README and homepage display these nine cases. Pulldown follows Ferromark;
bold values identify the lowest unrounded measured time, including exact ties.

All samples are retained. The tables use the preselected median-of-three
statistic; individual run medians and their spread are shown for review.
Differences near one percent should be treated as practical ties.

<!-- native-results:start -->
'''+native_report_section(data)+'''
<!-- native-results:end -->

## Complete input/configuration matrix

This table includes the full diagnostic matrix. The publication cases use three
run medians; other cases use one. Time per complete document in microseconds,
lower is faster. Bold identifies the lowest median within that row. Table-shaped
input in a CommonMark-only diagnostic is not an enabled-table performance result.

| Case | Runs | ferromark | pulldown-cmark | Bun (native) | comrak | md4c (C) |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
'''
for case,rows in cases.items():
 lowest=min(r['median_ns'] for r in rows.values())
 values=[]
 for name in ORDER:
  r=rows[name];value=f"{r['median_ns']/1000:.3f}"
  values.append(f'**{value}**' if r['median_ns']==lowest else value)
 text+='| '+case+' | '+str(len(rows['ferromark']['run_medians_ns']))+' | '+' | '.join(values)+' |\n'
text+=f'''
## Workload and measurement contract

{data['conditions']}

{data['scope']}

All {len(cases)} cases passed workload review before timing. Normalized HTML agrees
in {data['verification']['htmlEquivalent']} cases; original outputs, hashes, spec
checks, and accepted renderer differences remain in the verification evidence.
HTML normalization is never included in timed rendering.

Each process uses 80 alternating-order windows of at least 63 ms per parser/input
and three seconds of warmup per parser. The first process measures the complete
matrix; two more processes repeat the nine displayed cases with rotated parser
order. Each timed call creates parser state, produces owned HTML, and releases it.
The raw windows and three individual run medians remain available. Spread is not
a confidence interval. All measured parser sources, native dependencies, and
compiler settings remain frozen for the entire run.

The table rows cover plain cells, mixed plain/emphasis/strong cells, and a Markdown
link column. Each enables CommonMark plus tables; ordinary inline parsing is
always active. Table strikethrough is outside the published comparison. Standalone
strikethrough remains a separate feature case. The complete diagnostic matrix also
retains the broader existing GFM and CommonMark controls; it is not presented as
an additive feature-cost measurement.

The earlier stable/System-allocator paired experiment has different compiler,
CPU-target, allocator, and output-reuse settings. Its margins must not be copied
into this native/shared-mimalloc publication. Earlier figures remain in the
[previous publication](2026-09-11-table-optimization-refresh.md) and the
[historical report](2026-09-11-benchmark-refresh.md), with their own source revisions
and the md4c flag-correction evidence.

## Evidence and reproduction

- [Raw summaries](2026-09-11-native-optimization-publication/native/summary.json),
  [metadata](2026-09-11-native-optimization-publication/native/metadata.json),
  [workload and spec verification](2026-09-11-native-optimization-publication/native/verification.json).
- [Locked dependencies](2026-09-11-native-optimization-publication/native/Cargo.lock),
  [production patch](2026-09-11-native-optimization-publication/native/ferromark.patch),
  and [measurement-time harness](2026-09-11-native-optimization-publication/measured-harness).
  The harness snapshot records the measurement and publication scripts used for
  this run. Native adapters, inputs, and parser sources did not change during
  measurement.
- Three `samples-*.jsonl` files retain all windows; `verify`, `spec`, `catalog`, and
  `options` retain original data. Files may be gzip-compressed without alteration.
- [Validation log](2026-09-11-native-optimization-publication/validation.log) records
  publication guards, generated-data contracts, and homepage validation.

Follow the [native harness instructions](../../benchmarks/bun-comparison/README.md)
to prepare the pinned checkouts, then run:

```bash
python3 benchmarks/bun-comparison/run.py "$BUN_BENCH_DIR" /private/tmp/new-complete-benchmark-run
python3 benchmarks/bun-comparison/publish.py
python3 benchmarks/bun-comparison/publish.py --check
```

The generator chooses winners from unrounded medians and places pulldown-cmark
immediately after Ferromark. Historical publications remain reproducible through
their own case catalog; they cannot supply partial rows to the new publication.
'''
(HERE.parent/(HERE.name+'.md')).write_text(text)
