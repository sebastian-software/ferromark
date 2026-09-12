#!/usr/bin/env python3
"""Generate heap tables only from the recorded per-operation samples."""
from pathlib import Path
import gzip,json
D=Path(__file__).parent
raw=[json.loads(line) for line in gzip.decompress((D/'raw.jsonl.gz').read_bytes()).decode().splitlines()]
summary=json.loads((D/'summary.json').read_text())
for row in summary:
 for engine in ['ferro','ox-grow','ox-presize']:
  records=[x for x in raw if x['case']==row['case'] and x['engine']==engine]
  assert len(records)==3 and {x['process'] for x in records}=={0,1,2}
  assert all(len(x['samples'])==10 and all(s==row[engine] for s in x['samples']) for x in records)
  assert all(x['normalized_equal']==row['normalized_equal'] for x in records)
lines=['# Native heap measurements','','Generated from `raw.jsonl.gz`; all 30 measurements per engine/case agree exactly.','','## Large-document focus: growing Ox arena','','MiB uses 1,048,576 bytes. These are additional peak live requested Rust heap bytes, including owned HTML and excluding source storage. Negative change favors Ferromark.','','| Case | Input MiB | Ferromark peak MiB | Ox growing peak MiB | Ferromark change |','|---|---:|---:|---:|---:|']
for row in summary:
 if row['case'].startswith('large/') and row['normalized_equal']:
  a=row['ferro']['peak_live_bytes'];b=row['ox-grow']['peak_live_bytes']
  lines.append(f"| {row['case']} | {row['input_bytes']/1048576:.3f} | {a/1048576:.2f} | {b/1048576:.2f} | {(a/b-1)*100:+.1f}% |")
lines+=['','## All matching cases: growing Ox arena','','Incremental peak of simultaneously live, requested Rust heap bytes during a fresh Markdown→owned HTML operation. Source storage and pre-existing options are excluded; parser, renderer, scratch, arena and owned output are included. This is not process RSS or physical allocator consumption. Negative change means fewer peak bytes for Ferromark.','','| Case | Input bytes | Ferromark peak bytes | Ox growing peak bytes | Ferromark change |','|---|---:|---:|---:|---:|']
for row in summary:
 if not row['normalized_equal']:continue
 a=row['ferro']['peak_live_bytes'];b=row['ox-grow']['peak_live_bytes']
 lines.append(f"| {row['case']} | {row['input_bytes']} | {a} | {b} | {(a/b-1)*100:+.1f}% |")
lines+=['','## Excluded from the comparison','','These retain raw diagnostics but differ in normalized HTML, so their memory values do not establish a fair comparison.','']
lines += ['- '+row['case'] for row in summary if not row['normalized_equal']]
lines+=['','## Supplementary allocator diagnostics','','The presized arena is a diagnostic control, not the memory claim baseline. Requested bytes count all requests, including the full new size on reallocation. Peak bytes count only live logical sizes. Allocation counts include reallocations.','','| Case | Configuration | Peak bytes | Cumulative requested bytes | Calls |','|---|---|---:|---:|---:|']
for row in summary:
 if not row['normalized_equal']:continue
 for engine in ['ferro','ox-grow','ox-presize']:
  x=row[engine];lines.append(f"| {row['case']} | {engine} | {x['peak_live_bytes']} | {x['requested_bytes']} | {x['allocation_calls']} |")
(D/'RESULTS.md').write_text('\n'.join(lines)+'\n')
