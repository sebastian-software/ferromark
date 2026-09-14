# Feature-scan optimization harness

This harness compares the definition-list and line-comment scanner changes on
the exact same runtime configuration and input. `prepare.py` archives the
baseline core at `1728355` by default, copies the candidate core (including
uncommitted edits) into a sibling frozen source tree, and builds the same
runtime-configured worker twice. Both builds use Rust 1.95, the same pinned
registry dependencies offline, `target-cpu=generic`, `opt-level=3`, fat LTO, and one
codegen unit. The worker bytes are retained in `worker.rs`, and `build.json`
uses the engine/binary/hash contract consumed by
`benchmarks/optimization-rounds/run.py`.

The corpus generator imports the established runtime-profile definitions. It
freezes the prior study's plain and active definition-list and line-comment
probes at 300 B, 4 KiB, and 64 KiB, 13 mixed documents with the docs profile
plus definition lists and CommonMark controls, and focused near-miss,
late-definition, nesting, comment, URL, CRLF, BOM, source-span, and long
punctuation diagnostics. The 75 cases also include long prose with valid late
markers, long indented bodies, multiline terms, and explicit feature-off plain
controls at all three sizes. Each case records
an absolute JSON `profile`, the same options as `runtime_options`, source bytes,
and a SHA-256 digest. Config files live beside `corpus.json` in the output
directory.

Run the intended workflow with fresh output directories:

```sh
python3 benchmarks/feature-scan-optimization/make_cases.py /tmp/fmv2-feature-cases
python3 benchmarks/feature-scan-optimization/prepare.py \
  --baseline-revision 1728355 \
  --candidate-path "$PWD" \
  --out /tmp/fmv2-feature-build
python3 benchmarks/feature-scan-optimization/run.py \
  /tmp/fmv2-feature-build \
  /tmp/fmv2-feature-cases/corpus.json \
  /tmp/fmv2-feature-results \
  --rounds 3 --pairs 3 --window-ms 50
```

The wrapper accepts either `corpus.json` or `corpus.json.gz`. Before invoking
the existing optimization runner, it validates each input byte count and hash,
validates the `runtime_options` object, and writes canonical configs and a
rewritten corpus to `/tmp/fmv2-feature-results.feature-scan-replay/`. This
sibling replay bundle is persistent and should be archived with the results;
the original corpus' stale absolute profile paths are never used.

The preparation step performs compilation and should be run in isolation from
other builds and measurements. `--reuse-baseline-build PATH` may reuse a
matching baseline engine from another feature-scan build when its baseline
revision, worker hash, frozen source hash, lock hash, and binary hash still
match. No timing claim is implied by corpus generation or preparation; the
optimization-rounds runner remains responsible for output equality, AST and
child-count checks, timing checksums, and measurement summaries.

`verify_generated.py BUILD OUTPUT` adds 400 deterministic small documents in
three feature configurations (1,200 comparisons). These exercise comments,
multiline terms, Unicode, nested/dedented sources, inline backticks, references,
and all three line endings. HTML and complete AST Debug output, including spans,
must agree with the frozen baseline. This is a differential regression check,
not an independent specification oracle or an exhaustive fuzzing campaign.

Preparation also writes `BUILD/runtime-views/{baseline,candidate}`. These are
metadata views of the **same binaries**, accepted by the earlier runtime-profile
runner for same-binary off/on comparisons. For example:

```sh
python3 benchmarks/runtime-profiles/make_cases.py /tmp/fmv2-runtime-cases.json
python3 benchmarks/runtime-profiles/run.py \
  /tmp/fmv2-feature-build/runtime-views/candidate \
  /tmp/fmv2-runtime-cases.json /tmp/fmv2-feature-tax \
  --filter 'parser\.(definition_lists|line_comments)-plain-' \
  --rounds 3 --pairs 3 --window-ms 50
```

The source comparisons keep options enabled on both sides. The off/on study
instead measures the remaining price of enabling a feature whose syntax is
absent. It must keep HTML and AST exactly equal for these plain probes.

The [recorded investigation](../../docs/reports/2026-09-14-feature-scan-optimization/README.md)
contains the variant patches, screening runs, final confirmation, and rejected
approaches. Corpus inputs derived from the mixed-document suite retain their
original attribution in [its corpus manifest and licenses](../broad-comparison/README.md).

Run the focused replay guards without building workers:

```sh
python3 -m unittest discover \
  -s benchmarks/feature-scan-optimization -p 'test_*.py'
```
