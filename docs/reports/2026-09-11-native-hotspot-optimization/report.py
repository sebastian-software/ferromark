#!/usr/bin/env python3
"""Generate the hotspot report from measured evidence, without hand-edited figures."""
from pathlib import Path
import json
HERE=Path(__file__).resolve().parent
production=json.loads((HERE/'production-confirmation/summary.json').read_text())
guards=json.loads((HERE/'singlepass-guards/summary.json').read_text())
protocol=json.loads((HERE/'production-confirmation/protocol.json').read_text())
labels={'commonmark/tiny':'Tiny · CommonMark','commonmark/short-100b':'Short · CommonMark','gfm_overlap/tiny':'Tiny · GFM overlap','gfm_overlap/short-100b':'Short · GFM overlap','tables/tables-plain':'Tables · plain','tables/tables-commonmark-inline':'Tables · emphasis/strong','tables/tables-links':'Tables · link column','commonmark/publication-5k':'CommonMark · 5 KiB'}
text='''# Native hotspot optimization: tables and short documents

**Date:** 2026-09-11. This follow-up starts from the exact source of the
[previous five-parser publication](2026-09-11-table-optimization-refresh.md),
including table-cell event batching. It targets the measured remaining gaps:
plain and formatted tables, and tiny/short documents under CommonMark and the
existing GFM-overlap configuration. The link-column table and larger documents
remain controls. Ordinary CommonMark inline syntax is enabled throughout.
No table strikethrough workload is introduced.

## Retained production results

The final confirmation uses the unchanged five-parser native driver, the real
repository source, and the original pre-optimization executable. Both use the
same pinned nightly compiler, generic CPU target, Bun release profile, and shared
mimalloc/native libraries. This checks that gains in the isolated prototype
survive rebuilding the actual library. Full rendering returns owned HTML and
releases fresh parser/output state on every timed call.

Times are microseconds per document, lower is faster. Bold identifies the lower
Ferromark/pulldown median, not a claim of statistical significance. Differences
near one percent are best treated as ties. The change column is the median of
paired before/after process changes, so it need not equal the ratio of separately
aggregated time columns.

| Input | Ferromark before | Ferromark after | pulldown after | Paired change | After vs pulldown |
| --- | ---: | ---: | ---: | ---: | ---: |
'''
for row in production:
 f=row['parsers']['ferromark'];p=row['parsers']['pulldown-cmark'];before=f['before']['median_ns']/1000;after=f['after']['median_ns']/1000;pull=p['after']['median_ns']/1000
 a=f'{after:.3f}';b=f'{pull:.3f}'
 if after<=pull:a=f'**{a}**'
 if pull<=after:b=f'**{b}**'
 text+=f"| {labels[row['case']]} | {before:.3f} | {a} | {b} | {row['paired_change_percent']:+.2f}% | {row['after_vs_pulldown_percent']:+.2f}% |\n"
text+=f'''
The production confirmation has {protocol['rounds']} process pairs per input,
{protocol['windows_per_process']} alternating-parser windows of at least
{protocol['window_ms']} ms per process, and {protocol['warmup_ms']} ms of warmup per
parser. Before/after order reverses between rounds. All windows and both process
medians are retained. This is focused diagnostic evidence, not the longer
publication protocol. At this stage, README/homepage figures retained the earlier publication snapshot;
no partial rows were replaced with these timings. The subsequent
[complete publication](2026-09-11-native-optimization-publication.md) updates
those figures using the regular publication protocol.

## Profiling and hypotheses

The initial focused run reproduces the prior native gaps. Eight macOS `sample`
profiles cover Ferromark and pulldown on tiny, short, plain-table and formatted-table
inputs. Sampling uses the same native/shared-mimalloc probe environment.

- Tiny/short rendering spends substantial sampled time allocating and freeing
  memory, resolving inline marks, and passing through the general paragraph
  renderer. Emphasis previously reserved every modulo stack even when only one
  stack held an opener.
- Plain tables show cell splitting, block parsing, delimiter-row recognition,
  and inline-to-HTML emission prominently. Short escape searches copy input into
  a padded SIMD buffer on the old NEON path.
- List-tightness postprocessing is visible but smaller. It remains unchanged.

The hypotheses were tested separately: reduce fixed allocations; avoid generic
cell splitting for delimiter rows; remove call/copy overhead for small inline
content; and evaluate block-event processing/buffer growth. Profiles are
attribution evidence, not a claim that every sample in a large function can be
removed. The paired experiments below establish the actual effects.

Repeated profiles of the retained variant show less prominence of allocation,
freeing, and copying in the tiny-document path. Cell splitting and escape-scan
setup also account for fewer samples in the plain-table profile. These support
the measured mechanisms; sampling counts are not calibrated per-call costs.

## Independent experiments

Five rotating paired rounds with 150 ms windows, except where each archived
protocol says otherwise. Each variant's complete HTML and resource-limit report
must match the frozen baseline on the input/control catalog before timing.
Negative changes mean less elapsed time. Small differences remain uncertain.

| Variant | Tiny | Short | Plain table | Formatted table | Link table | Decision |
| --- | ---: | ---: | ---: | ---: | ---: | --- |
'''
experiments=[
 ('lazy-emphasis','Reserve only used emphasis stacks','Retain'),
 ('inline-content','Force inline content renderer','Discard: regressions'),
 ('delimiter','Direct delimiter cells with a separator pre-scan','Replace: long-delimiter regression'),
 ('small-buffers','Smaller initial event buffers','Discard: mixed results'),
 ('lazy-ref-label','Lazy reference-label buffer','Retain in confirmed combination'),
 ('lazy-render-state','Lazy render state including heading-ID registry buffers','Discard broad form; keep content scratch lazy'),
 ('dense-events','Larger table-enabled event allocation','Discard: memory cost for small gain'),
 ('scalar-neon-tail','Scalar lookup for short NEON inputs','Retain'),
 ('output-min64','Minimum owned-output buffer','Retain with zero-input allocation preserved'),
 ('inline-emphasis','Inline storage for emphasis stacks','Discard: little beyond lazy stacks; table costs'),
 ('single-paragraph','Borrow a complete single-paragraph event sequence','Retain'),
]
keys=['commonmark/tiny','commonmark/short-100b','tables/tables-plain','tables/tables-commonmark-inline','tables/tables-links']
for ident,label,decision in experiments:
 rows={r['case']:r for r in json.loads((HERE/('screen-'+ident)/'summary.json').read_text())}
 text+='| '+label+' | '+' | '.join(f"{rows[k]['paired_change_percent']:+.2f}%" for k in keys)+' | '+decision+' |\n'
text+='''
Combination trials remain separate: `combined-guards`, `screen-combined-lean`,
`screen-combined-delimiter`, `final-guards`, `extra-guards`, `linear-guards`, and
`singlepass-guards`. The first delimiter rewrite helped short rows but added a
separator scan on long delimiter cells. A targeted guard exposed that regression.
The retained recognizer validates cells in one pass without the preceding full
row filter. Once the column limit is reached, it still validates the remaining
byte set, preserving the original bounded-parser behavior.

The final combination is `final-singlepass`. No rejected alternative remains in
production. Original event-buffer sizing and the existing SSE2 short-input path
are unchanged.

## Retained implementation

- A document whose block events are exactly one ordinary single-range paragraph
  borrows that range, writes its paragraph wrapper, and invokes the same complete
  inline renderer. Normal block parsing still happens first. Other event shapes
  retain the general renderer. The caller resets document state before this path;
  document footnotes and resource-limit reporting still run afterward.
- Paragraph, heading, cell, and reference-label scratch allocate when needed.
  Used buffers remain reusable. Emphasis opener stacks grow only when an opener
  actually uses them; matching rules and the three inline phases are unchanged.
- Nonempty owned output reserves a small minimum so HTML wrappers need not cause
  immediate growth on tiny inputs. Empty input retains zero initial output
  capacity. Caller-owned output keeps its existing lifecycle.
- NEON byte-set searches below one full vector use existing scalar membership
  lookup, avoiding the temporary padded copy. No unsafe load contract, public
  byte-search API, byte set, escaping policy, or x86 SIMD path changes.
- Table delimiter recognition directly consumes optional colons, dash runs,
  whitespace, and cell separators. Normal cell splitting still handles escapes,
  code spans, merged cells, and body rows. Column limits and width hints retain
  their existing behavior.

## Broad controls

Seven paired rounds per case with 200 ms windows and rotating
baseline/candidate/pulldown order. Every selected path receives an initial warmup,
and each window warms 32 calls before its timer. All timing windows remain in the
raw data. These are workload controls, not an additive price for each feature.

| Control | Before (µs) | After (µs) | Paired change |
| --- | ---: | ---: | ---: |
'''
for row in guards:
 if row['case'].startswith('guard/'):
  text+=f"| {row['case'][6:]} | {row['before_us']:.3f} | {row['after_us']:.3f} | {row['paired_change_percent']:+.2f}% |\n"
text+='''
## Correctness, limits, and reproduction

The new single-paragraph regression checks reference definitions, escapes, HTML
policy, whitespace, and alternating reusable-renderer documents. The delimiter
regression checks every short combination of delimiter bytes against a separate
split-and-validate grammar, plus byte validation after the column cap. These
regressions pass on the baseline before applying the production changes.
Existing full tests cover CommonMark, GFM, merged/width-hinted tables, footnotes,
MDX, callbacks, policies, resource limits, and renderer reuse. Shared byte-search
tests compare arbitrary bytes, NUL, duplicates, and vector boundaries against
scalar reference results. The production native driver additionally requires
exact before/after outputs for its complete workload matrix and spec corpus.

Measurements are local Apple M1 Pro results. The unchanged x86 path has not been
performance-tested here. Focused results do not establish a universal parser
ranking or a speedup for every input/lifecycle. Small differences should be read
alongside the retained per-round values; no outliers were removed.

Evidence:

- [Environment](2026-09-11-native-hotspot-optimization/environment.json),
  [frozen baseline source](2026-09-11-native-hotspot-optimization/baseline.tar.gz),
  [source hashes](2026-09-11-native-hotspot-optimization/baseline-source.json),
  and [locked native dependencies](2026-09-11-native-hotspot-optimization/Cargo.lock).
- [Recorded initial setup](2026-09-11-native-hotspot-optimization/setup-recorded-run.py),
  [native prototype runner](2026-09-11-native-hotspot-optimization/experiment.py),
  [driver](2026-09-11-native-hotspot-optimization/probe.rs),
  [production confirmation runner](2026-09-11-native-hotspot-optimization/production.py),
  and [production results](2026-09-11-native-hotspot-optimization/production-confirmation/summary.json).
- [Retained production patch](2026-09-11-native-hotspot-optimization/production-confirmation/retained.patch),
  [production build hashes](2026-09-11-native-hotspot-optimization/production-confirmation/build-info.json),
  [final control results](2026-09-11-native-hotspot-optimization/singlepass-guards/summary.json),
  and [validation log](2026-09-11-native-hotspot-optimization/validation.log).
- `profiles-baseline` and `profiles-final-singlepass` retain CPU samples;
  `screen-*` and combination directories retain raw windows, verification,
  and protocols. Large raw files may be losslessly gzip-compressed.

The isolated setup uses a dedicated pinned Bun checkout and existing native
libraries from the preceding publication; production.py restores its original
workspace/lockfile before rebuilding the real repository. Source revisions,
compiler flags, libraries, lifecycle, and allocator must match to reproduce these
measurements. No build or other benchmark runs concurrently with measured windows.
'''
(HERE.parent/(HERE.name+'.md')).write_text(text)
print('Generated native hotspot optimization report')
