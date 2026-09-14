# Investigation log

All experiments below are retained, including bypasses rejected for production.
No parser or renderer changes from this investigation were installed in the
working core. The user requested an explanation of the regression, so this
round stops at measured attribution and a concrete next optimization target.
The performance regression itself is still present in production.

## Ranked hypotheses, stated before instrumentation

1. An unconditional CommonMark normalization scan explains source-length-dependent
   work during parser construction. Prediction: bypassing it on clean input
   removes that construction delta.
2. Compatibility fixes add block/inline recognition cost. Prediction: the parsing
   gap appears at those commits and varies with Markdown structure.
3. New disabled features add state/dispatch costs. Prediction: later feature
   commits regress even simple inputs with their options off.
4. Renderer preparation dominates. Prediction: rendering an already built AST
   reproduces most of the full-pipeline gap.
5. Benchmark/compiler layout explains the observation. Prediction: the gap
   changes substantially when measured with a stage harness that retains the
   original full-pipeline paths.

## R0 — Reproduce the original comparison

Use the unchanged native executable on the same 14 agreeing inputs, with fresh
and reused lifecycles. Result: v2/OX time 1.0935 fresh, 1.1242 reused, close to
the committed 1.0933/1.1207. The observation is reproducible. The initial loop's
raw records are retained under `runs/reproduction/`; its limited provenance is
described in [METHOD.md](METHOD.md).

## S1 — Split initialization, parsing, and rendering

Keep the six-engine worker and append stage entry points. Result: full-pipeline
ratios remain 1.0916/1.1203; parse is 1.1543 and prebuilt-AST rendering 1.0240.
Rendering large chess prose is essentially equal. Hypothesis 4 is not the
primary explanation. Hypothesis 5 does not explain away the original gap,
although smaller code-layout effects remain possible.

## H1 — Historical checkpoints

Build `adf891a`, `4a1e55f`, `db987c8`, `e0eba1d`, and current `33c216b` against
the same frozen dependency setup. All exact HTML gates pass. Before correctness
work, v2 is slightly ahead of OX on this subset. After the compatibility fixes,
the fresh/reuse ratios are only 1.0161/1.0237. At `e0eba1d` they reach
1.0939/1.1195. The larger regression therefore lies after the compatibility
commits. This rejects the initial expectation that normalization alone dominates
the overall gap.

## N1 — Same-binary normalization bypass

Apply [normalization-gate.patch](patches/normalization-gate.patch) only to an
extracted current core. A startup environment variable sets an atomic switch;
both timed variants contain the same switch and machine code. Require exact
HTML and AST equality with each other and the uninstrumented current core.

All 14 clean inputs pass. Skipping normalization reduces fresh time by 1.56%,
reused time by 2.85%, initialization by 26.65%, and parsing by 3.32%. On chess
prose it removes about 1,016 ns from initialization. The full-pipeline gap
remains. This confirms the normalization scan as a real but partial cause.

The instrumented control itself is 1.42% faster fresh than the uninstrumented
build, while reused/parse/render aggregate changes are close to zero. Therefore
only **skip versus its same-binary control** is used to quantify the bypass.
Its slight render-only movement, where no normalization runs inside timing,
also shows why small timing differences should not receive causal explanations.

**Rejected as a production change:** it fails both NUL and leading-BOM fixtures,
including HTML and AST comparisons. Their expected failures are archived in
`normalization-guards/`. A future fix must preserve normalization and source
positions while avoiding a redundant full scan, for example by sharing discovery
with an existing scan.

## H2 — Individual feature checkpoints

Measure each production change from `db987c8` through `e0eba1d`: smart typography
removal (`7010384`), table metadata (`e01c611`), derived columns (`46c761d`), line
comments (`40047f7`), and frontmatter (`e0eba1d`). Include OX and current v2 in
the same run. Results are in [TABLES.md](TABLES.md) and the report's interpretation.
These comparisons localize a commit's net effect, including compiler/layout
changes; they do not isolate each branch added by the commit.

## P1 — Restore direct paragraph source access

Apply [paragraph-map-bypass.patch](patches/paragraph-map-bypass.patch) to an
extracted `33c216b`. Replace only the two paragraph/setext calls to
`without_line_comments_with_first` with the former direct source slice and a
known-absent source map. All other current code, including normalization and
comment dispatch, remains. The compiler can then remove map-result handling
and span-remapping branches from these paths.

Require exact HTML and complete AST equality with unmodified current v2 before
timing. All 14 cases pass. Raw data is retained with H2 under
`runs/history-fine/`; [TABLES.md](TABLES.md) shows candidate/control and per-round
ratios. The direct path reduces fresh time by 5.0%, reused time by 6.3%, and
parsing time by 8.6%, while rendering stays effectively unchanged. Both rounds
agree. Disassembly also shows a smaller paragraph function and stack frame;
see the report for the exact bounds of that evidence.

**Rejected as a production change:** the bypass discards paragraph comment
filtering when the feature is enabled. It is an attribution experiment, not a
valid replacement implementation. Unlike N1, it is a separate compilation and
includes resulting code-generation changes.
