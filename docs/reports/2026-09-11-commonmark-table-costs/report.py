#!/usr/bin/env python3
"""Generate the diagnosis's numeric tables directly from measured evidence."""
from pathlib import Path
import json

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
PREFIX = HERE.name

def read(path):
    return json.loads((HERE / path).read_text())

def values(path):
    return {(x['case'], x['lane']): x['median_us'] for x in read(path)}

primary = values('confirmation/summary.json')
control = values('raw/summary.json')
counts = {(x['case'], x['lane']): x for x in read('raw/counts.json')}
profiles = {x['file']: x for x in read('raw/profile-summary.json')}

def row(data, case, label):
    f, p = data[case, 'ferro-reuse'], data[case, 'pull-reuse']
    retained = data[case, 'ferro-retained']
    return f'| {label} | {f:.2f} | {p:.2f} | {(f/p-1)*100:+.1f}% | {retained:.2f} |\n'

def leaf(case, symbol):
    entries = profiles[f'{case}-ferro-reuse.sample.txt']['leaf_symbols']
    return sum(e['percent'] for e in entries if symbol in e['symbol'])

text = '''# CommonMark table costs — 2026-09-11

This records the baseline before the subsequent
[table optimization experiments](2026-09-11-table-optimization-variants.md).

The active table comparison now covers plain text, a mixture of plain text,
CommonMark emphasis and strong emphasis, and a Markdown link column. All three
use **identical options: CommonMark plus tables**. CommonMark inline parsing was
never disabled. Strikethrough table cases have been removed from the active
Criterion/native comparison; previous results remain archived evidence.

The remaining plain-table gap is associated with repeated work per short cell.
Grouping the same cells into one table does not eliminate it. Retaining scratch
buffers eliminates allocations but does not eliminate the gap either. This is
an investigation and benchmark correction; production parser code is unchanged.

## Practical cases

All three fixtures contain 80 two-column tables with one header and one body row
(320 cells). `tables-plain.md` contains ordinary words. The mixed fixture contains
40 `<em>` and 40 `<strong>` elements alongside plain cells. `tables-links.md`
contains 80 `[Guide](https://example.com/guide)` links in the second column and
plain/emphasized labels in the first (20 `<em>` and 20 `<strong>` elements).
Semantic tests check the rendered structure and formatting for both parsers.

Microseconds per complete document, median of nine 500 ms window means.
Both principal columns create parser state per call and reuse output. Retained
Renderer is an additional Ferromark lifecycle diagnostic, not a matched pulldown
API. Positive difference means more elapsed time for Ferromark.

| Case | Ferromark | pulldown-cmark | Difference | Retained Renderer |
| --- | ---: | ---: | ---: | ---: |
'''
for case, label in [('plain-many', 'Plain text'), ('mixed-many', 'Plain text + emphasis + strong'), ('links-many', 'Markdown link column')]:
    text += row(primary, case, label)
text += '\nOwned-output control (fresh output and parser state for both):\n\n'
text += '| Case | Ferromark | pulldown-cmark |\n| --- | ---: | ---: |\n'
for case in ['plain-many', 'mixed-many', 'links-many']:
    text += f"| {case} | {primary[case, 'ferro-owned']:.2f} | {primary[case, 'pull-owned']:.2f} |\n"
text += '''
These are synthetic local cases, not a weighted model of real-world documents.
The URL case uses trusted HTML rendering settings shared by the comparison;
it does not measure Ferromark's additional secure-policy work.

## Controlled diagnosis

The hypotheses were ranked before testing: repeated cell work, repeated table
setup, then allocation overhead. Controls use nine 250 ms windows per lane.

The `many`/`one` pairs preserve every cell's source content and order and the
320-cell count. The single-table version has one header and 159 body rows instead
of 80 headers and 80 body rows. Consequently delimiter rows, header/body tags,
table framing, and output bytes change; this is a topology control, not identical
HTML and not a way to assign an additive setup cost.

| Case | Ferromark | pulldown-cmark | Difference | Retained Renderer |
| --- | ---: | ---: | ---: | ---: |
'''
for case, label in [('plain-many','Plain, 80 tables'), ('plain-one','Same plain cells, 1 table'), ('mixed-many','Mixed, 80 tables'), ('mixed-one','Same mixed cells, 1 table'), ('short-cells','320 cells × 8 text bytes'), ('long-cells','32 cells × 80 text bytes')]:
    text += row(control, case, label)
text += f'''
**Table count does not explain the comparative gap.** Combining the plain tables
reduces absolute time for both parsers, while the Ferromark/pulldown difference
remains and increases proportionally. Ferromark's isolated block-only diagnostic
falls from {control['plain-many','ferro-block']:.2f} to
{control['plain-one','ferro-block']:.2f} µs. Block-only timing omits inline parsing
and HTML rendering; it is not comparable to pulldown event iteration or a phase
that can be subtracted exactly from the complete render.

**Short-cell work is the strongest explanation.** The final pair has exactly
2,560 content bytes in both cases, one table and two columns. Reducing cell count
from 320 to 32 changes the ranking. There are fewer row/cell tags to produce, so
this establishes structural cell overhead rather than isolating any single
function. The counters and profiles below locate the associated repeated work.

**Allocations are not the main explanation for these two-column tables.** Every
retained-renderer case performs zero allocations, reallocations, and deallocations
inside the measured counter window. The plain-table gap remains in that lifecycle.
This does not negate the separately measured allocation spill above eight columns
in the [earlier report](2026-09-11-table-profiling.md).

## Counters and CPU profiles

Separate instrumented renders, after warmup; instrumentation is absent from
all timing and CPU-sampling binaries:

| Case | Block events | Inline calls | Plain fast paths | Inline events |
| --- | ---: | ---: | ---: | ---: |
'''
for case in ['plain-many','plain-one','mixed-many','links-many','short-cells','long-cells']:
    c = counts[case,'ferro-retained']['pipeline']
    text += f"| {case} | {c['block_events']} | {c['inline_parses']} | {c['inline_fast_paths']} | {c['inline_events']} |\n"
text += '''
Five-second macOS `sample` captures, 1 ms interval. Percentages are leaf/top-of-stack
attribution and include compiler inlining; they are not exact phase timings or
promised optimization gains. `render_block_event` includes actual rendering work,
not merely event dispatch.

| Ferromark symbol | Plain, 80 tables | Plain, 1 table | Mixed | Links |
| --- | ---: | ---: | ---: | ---: |
'''
for symbol, label in [('render_block_event','`render_block_event`'),('split_table_cells','`split_table_cells`'),('render_inline_content','`render_inline_content`'),('parse_with_options_in_document','`InlineParser::parse_with_options_in_document`'),('is_delimiter_row','`is_delimiter_row`')]:
    text += '| '+label+' | '+' | '.join(f'{leaf(c,symbol):.1f}%' for c in ['plain-many','plain-one','mixed-many','links-many'])+' |\n'
text += '''
Source inspection explains the pattern:

1. [`split_table_cells`](../../src/block/parser.rs) scans table boundaries and
   trims cell ranges. Each ordinary cell then produces start/text/end block events.
2. [`CellState::add_text` and `TableCellEnd`](../../src/lib.rs) collect the range,
   scan for escapes, finish the cell, and invoke the generic inline renderer.
3. `render_inline_content` prepares an event vector and enters the inline parser.
   Even every plain cell takes this route; its fast path emits a single text event,
   which is then consumed by the HTML renderer. The plain case's 320/320 fast-path
   count rules out accidental expensive emphasis resolution as the explanation.
4. The pinned pulldown-cmark 0.13.4 source's
   `firstpass.rs::parse_table_row_inner` calls `parse_line` in active table mode;
   row scanning already collects inline markers. Its decomposition differs from
   Ferromark's separate cell scan and per-cell inline entry. Its one-table profile
   is retained for comparison. Cross-parser symbol percentages cannot establish
   an exact causal speedup.

The next optimization target is reducing repeated cell transitions/scanning while
preserving full CommonMark semantics. A bounded candidate should be measured on
all three practical fixtures plus escapes/code spans, long cells, and wide tables.
The evidence does not justify disabling inline parsing or replacing the event
architecture. No speculative parser optimization is included here.

## Reproduction and evidence

Apple M1 Pro, ARM64, macOS 26.6.2; Rust 1.97.1; pulldown-cmark 0.13.4; System
allocator. Release opt-level 3, fat LTO, one codegen unit, repository apple-m1/NEON
flags. Trusted CommonMark plus tables, no GFM strikethrough/task-list extension,
no literal autolinks, heading IDs, or table-width extras. Each sample warms up
32 renders and checks elapsed time every 16 renders. Case order rotates and
parser/lifecycle order rotates and reverses. No competing builds or benchmark
jobs were started by this task while measuring or sampling.

The first link-only 250 ms run had substantial outliers (retained unchanged in
`links-raw`). Therefore all three practical cases were repeated together with
500 ms windows in `confirmation`; those results supply the practical table above.
The confirmation also contains large early-window outliers: interpret its medians
as approximate local estimates, not percentage-point precision. All windows,
including outliers, are retained. The controls use the original
250 ms run. These are different runs and workloads, not an optimization before/after.

'''
text += f'''```bash
python3 docs/reports/{PREFIX}/run.py time --out /tmp/commonmark-table-all
python3 docs/reports/{PREFIX}/run.py time --case plain-many --case mixed-many --case links-many --window-ms 500 --out /tmp/commonmark-table-confirmation
python3 docs/reports/{PREFIX}/run.py counts --out /tmp/commonmark-table-all
python3 docs/reports/{PREFIX}/run.py sample --out /tmp/commonmark-table-all
python3 docs/reports/2026-09-11-table-profiling/summarize_profiles.py /tmp/commonmark-table-all
```

- [Practical measurement windows]({PREFIX}/confirmation/timings.jsonl),
  [summary]({PREFIX}/confirmation/summary.json),
  [source/binary metadata]({PREFIX}/confirmation/metadata.json).
- [Topology windows]({PREFIX}/raw/timings.jsonl),
  [summary]({PREFIX}/raw/summary.json),
  [metadata]({PREFIX}/raw/metadata.json),
  [counts]({PREFIX}/raw/counts.json).
- [CPU summary]({PREFIX}/raw/profile-summary.json) and
  [capture inventory]({PREFIX}/raw/profiles.json); raw stacks live beside them.
- Every output directory retains input, raw HTML from both parsers, input hashes,
  normalized-output verification, and Cargo.lock. Exact HTML whitespace differs;
  normalized tokens agree. The generated runner uses the
  [original probe source](2026-09-11-table-profiling/probe.rs).
- [Native five-parser output verification]({PREFIX}/native-verification/verification.json)
  and [validation log]({PREFIX}/validation.log) cover the updated benchmark wiring.
  Native verification is not a new five-parser timing publication.

The previous [intermediate experiment](2026-09-11-table-benchmark-isolation.md)
remains archived with its former strikethrough fixture. The active dedicated table corpus and
Criterion/native cases contain only the three requested table content patterns.
Public five-parser numbers retain their measured source provenance; no diagnostic
number has been substituted into the historical publication.
'''
(HERE.parent / f'{PREFIX}.md').write_text(text)
