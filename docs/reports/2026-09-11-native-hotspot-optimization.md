# Native hotspot optimization: tables and short documents

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
| Tiny · CommonMark | 0.398 | 0.290 | **0.286** | -27.22% | +1.38% |
| Short · CommonMark | 0.637 | **0.526** | 0.532 | -17.43% | -1.13% |
| Tiny · GFM overlap | 0.397 | 0.287 | **0.286** | -27.61% | +0.36% |
| Short · GFM overlap | 0.632 | **0.523** | 0.532 | -17.33% | -1.65% |
| Tables · plain | 30.396 | **26.704** | 29.713 | -12.14% | -10.13% |
| Tables · emphasis/strong | 38.238 | **33.536** | 36.797 | -12.29% | -8.86% |
| Tables · link column | 43.048 | **38.763** | 44.865 | -9.95% | -13.60% |
| CommonMark · 5 KiB | 24.756 | **23.655** | 27.644 | -4.45% | -14.43% |

The production confirmation has 2 process pairs per input,
20 alternating-parser windows of at least
40 ms per process, and 250 ms of warmup per
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
| Reserve only used emphasis stacks | -11.00% | -5.41% | +0.12% | +0.08% | +0.35% | Retain |
| Force inline content renderer | +1.23% | +4.51% | -1.78% | +2.07% | +2.96% | Discard: regressions |
| Direct delimiter cells with a separator pre-scan | -0.20% | -0.73% | -3.43% | -1.49% | -0.67% | Replace: long-delimiter regression |
| Smaller initial event buffers | -2.45% | +2.10% | -0.37% | +0.36% | +0.02% | Discard: mixed results |
| Lazy reference-label buffer | -1.73% | -0.97% | +1.59% | -0.18% | +0.49% | Retain in confirmed combination |
| Lazy render state including heading-ID registry buffers | -2.65% | -1.05% | +1.67% | +1.74% | +0.99% | Discard broad form; keep content scratch lazy |
| Larger table-enabled event allocation | +0.16% | -0.66% | -1.61% | -1.92% | -1.52% | Discard: memory cost for small gain |
| Scalar lookup for short NEON inputs | -2.84% | -3.42% | -6.24% | -4.93% | -3.91% | Retain |
| Minimum owned-output buffer | -4.15% | +0.01% | +1.21% | -0.61% | +0.38% | Retain with zero-input allocation preserved |
| Inline storage for emphasis stacks | -12.19% | -7.37% | +1.97% | +1.33% | +1.53% | Discard: little beyond lazy stacks; table costs |
| Borrow a complete single-paragraph event sequence | -6.66% | -3.79% | -0.91% | -0.11% | -0.38% | Retain |

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
| plain | 29.412 | 25.904 | -12.90% |
| mixed | 36.272 | 31.895 | -11.51% |
| links | 40.549 | 36.574 | -9.80% |
| mixed-document | 17.330 | 16.828 | -3.85% |
| one-table | 21.085 | 17.689 | -16.01% |
| long-cells | 28.018 | 28.284 | +0.90% |
| wide9 | 42.642 | 36.642 | -14.13% |
| wide16 | 71.433 | 61.251 | -15.39% |
| escapes-code | 68.466 | 63.139 | -7.54% |
| prose | 10.129 | 10.138 | +0.40% |
| links-prose | 35.803 | 33.288 | -6.53% |
| edge-cases | 1454.308 | 1392.048 | -5.70% |
| references | 10.406 | 9.716 | -6.76% |
| empty | 0.120 | 0.108 | -9.95% |
| gfm-document | 23.389 | 22.619 | -3.22% |
| long-delimiters | 24.235 | 13.312 | -45.11% |
| deep-emphasis | 51.614 | 43.847 | -14.81% |

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
