# Measurement method and interpretation

## Scope and baseline

The previous benchmark work was already committed as `d173c24` with a clean
working tree before this investigation began. The measured production core is
`33c216b`; this round changes diagnostics and documentation only.

The comparison uses exactly the 14 all-six-agreeing documents from the
[matched native comparison](../2026-09-14-native-matched/README.md): ten comments
and four plain-prose views, 37–80,966 UTF-8 bytes. Inputs and attribution remain
in each run's `corpus.json.gz`. This subset explains the published OX comparison;
it is not representative of every extension or every Markdown workload.

CommonMark and the shared tables/strikethrough/task-list lane retain the previous
explicit flags. Line comments, frontmatter, definition lists, MDX, and other
optional extras are disabled. Historical builds without strict renderer APIs
use documented adapters and must produce exactly the same HTML on all 14 inputs.
No output normalization is used in this investigation.

## Stages

| Mode | Timed work | Retained outside the timed operation |
| --- | --- | --- |
| `fresh` | Original native adapter: setup, parse, render, owned output, destruction | Input text |
| `reuse` | Original native adapter: parse, render, output consumption, arena reset | Renderer and arena |
| `init` | `Parser::with_options`, consume the parser with `black_box`, drop, reset arena | Options and arena |
| `parse` | Parser construction and complete AST parse, consume AST with `black_box`, drop, reset arena | Options and arena |
| `render` | Render an existing AST using `render_borrowed`, consume output | Parsed AST, its arena, and renderer |

Initialization includes both source normalization and the reference-definition
prepass. The `parse` stage includes initialization. Stages are **not additive**:
their optimization boundaries, observable results, allocations, and output
ownership differ. In particular, exposing the entire parser to `black_box` can
affect construction code. Full-pipeline measurements remain the control.

Stage diagnostics also record native `size_of` values, AST Debug, document child
counts, and arena capacity. A larger structure is evidence of layout change,
not a measurement of cache misses or proof that every added byte costs time.

## Controls

The staged worker keeps the existing six-engine native fresh/reuse implementation
and native allocator. It adds stage entry points for OX/v2. Before attributing
anything to stages, its full-pipeline OX gap was checked against the unchanged
native executable and the committed results.

Builds use the same nightly compiler, generic CPU target, frozen native support
libraries and Bun source, and the same pinned Cargo lock. `builds/*/build.json`
records exact revisions, source file hashes, worker hashes, executable hashes,
compiler command, and API adaptations. The original native build metadata is
embedded. Historical core sources come from `git archive`; dependencies are not
updated while moving between commits.

Every run checks exact HTML before timing and before/after each timed job.
The `init`, `parse`, and `render` stages must agree on full AST Debug within an
engine/revision; `fresh` and `reuse` expose HTML only. Current-core
bypass experiments additionally require full AST and HTML equality across
control and candidate **before any timing**. AST Debug contains source spans.
Historical AST shapes differ, so cross-revision AST text is not required equal.

The normalization bypass additionally requires all 14 inputs to contain no NUL
and no leading BOM. Separate NUL/BOM fixtures prove it fails outside that domain;
their outputs are retained in `normalization-guards/`. The paragraph mapping
bypass is valid for the tested disabled-comment profile only. Neither is a
production optimization or a proposed feature flag.

## Sampling and aggregation

The initial reproduction uses the original binary, two rounds of three 40 ms
pairs with 10 ms warmups. It was an ad hoc invocation of the existing native
worker protocol; raw windows and the build record are retained. It did not save
host observations or the invocation as a script. The subsequent reproducible
stage/history/bypass runs independently confirm its finding.

Unless a run's `run.json` says otherwise, diagnostic runs use two rounds of three
30 ms windows per variant/case/mode, with 10 ms warmups. Jobs are shuffled with
seed 20260914. Variant order alternates forward/reverse. Only one worker performs
timed work at a time; compilation and tests are not run during timed samples.
Workers for other variants remain idle. Every checksum is checked against the
stage-specific result metric. JSON, AST inspection, process startup, I/O, and
verification are outside timing.

For the diagnostic tables, each case/variant time is the median of its windows;
the aggregate is the geometric mean of the per-document time ratios. Each
document has equal weight. Per-round medians and all windows are retained.
Reproduction's original summary instead takes the median of paired ratios.
These closely agree at aggregate level but are not identical estimators.

These are local timing observations, not statistical confidence intervals.
Small differences near 1% need caution. Host power/load observations are saved
by the scripted runs; macOS did not expose thermal warning status. Separate
builds can change code layout and inlining. Historical commit jumps identify a
change set; they do not assign every nanosecond to a single source statement.

The normalization experiment toggles an atomic branch in the **same binary**,
with its environment switch read once outside timing. Its control is compared
with an uninstrumented build to expose instrumentation/layout effects. The
paragraph experiment removes one source-mapping path at compile time, so its
result includes the resulting code-generation changes. The two effects must not
be added or treated as a tested combined optimization.

## Reproducibility

See the [harness instructions](../../../benchmarks/ox-regression/README.md).
`harness/screen-run.py` is the exact runner used by the initial stage/coarse-history
runs; the later runner adds the cross-variant AST gate. Archived build workers
are the exact compiled sources, including historical API adaptations. Absolute
paths in metadata identify the local run and should be changed when replaying
on another checkout. Binaries and build caches remain outside Git; hashes,
source revisions, patches, dependency lock, build logs, inputs, and raw results
are retained here or in the linked native reference archive.

Archived verification JSON uses lossless XZ compression (`verification.json.xz`)
to avoid repeatedly storing large, identical AST observations inefficiently.
Its decompressed bytes match the runner's original gzip output exactly. Timing
windows and corpus files keep their original gzip format.

The checked-in runner subsequently received provenance hardening after review:
it now records the warmup, source corpus/reference hashes, and selection mode,
and rechecks archived source hashes before timing. The measured runner retained
here predates those additions: its warmup is the literal 10 ms in its archived
source, and its reference path is likewise explicit. Recorded runs were not
rewritten to imply they used the newer runner.
