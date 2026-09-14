# Method and replay

The previous [OX regression investigation](../2026-09-14-ox-regression/README.md)
identified disabled line-comment paragraph handling as the largest contributor
on its 14 agreeing documents. This round turns that diagnostic into a valid
implementation. Baseline core `33c216b` is unchanged at starting commit `ec06a8c`.
Source normalization is neither disabled nor modified.

## Candidates and semantic gates

A outlines filtering and remapping behind an observed comment inside the
paragraph content range. B adds a const-generic paragraph loop selected once by
the runtime flag. The original entry point remains available to block parsing
and the math fallback. All candidate patches and the initial B compile failure
are retained in `patches/` and `build-failures/`.

Before native timing, HTML and full AST Debug must agree across baseline/A/B.
The 14-case screens additionally check the existing native reference HTML.
The 57-case confirmation uses `--all-cases --same-ast baseline A B`: it compares
the same v2 options and exact outputs on every case, without requiring agreement
with a different engine. No HTML normalization is used.

The runtime-profile harness independently checks exact HTML, complete AST Debug,
child counts, and agreement across fresh/reuse/parse/render stages. Its 75 cases
cover absent and active comments, definitions, CommonMark, docs profiles, nested
containers, source spans, and punctuation-heavy inputs. Another 400 deterministic
documents in three configurations produce 1,200 differential comparisons per
candidate. These preserve the established baseline; the workspace specification
tests supply the independent correctness checks.

Two new regression tests cover full AST/source-span HTML equality when toggling
comments on inputs without eligible comments, and setext headings with comments
inside versus outside content. They include LF/CRLF/CR, BOM/NUL, heading
attributes, literal container markers, and frontmatter.

## Two controlled build setups

Native workers reuse the frozen six-engine dependency setup from the prior
study: the same nightly compiler, generic CPU target, native support libraries,
allocator, Cargo lock, and exact staged worker bytes. Candidate snapshots use
the baseline revision plus their archived parser patch. Old snapshot test files
do not affect the release library; the accepted regression tests are retained
in the working repository and the final source patch.

Runtime-profile workers use the existing Rust 1.95 harness, generic CPU target,
fat LTO, one codegen unit, and matching pinned registry packages. Baseline and
candidate use identical worker source and runtime options. B reuses the verified
baseline build from A. This setup differs from the native six-engine worker;
absolute times and aggregate ratios from the two setups are not pooled.

`builds/` records source/worker/binary hashes, commands, dependency locks, and
compiler metadata. All production source bytes used by the selected candidate
are checked against the accepted working core. No benchmark-only runtime switch
or disabled-correctness shortcut is included in the accepted implementation.

## Timings

Native screens: two rounds × three 30 ms windows per case/variant/stage, with
10 ms warmups. The full-corpus confirmation uses three rounds × three 40 ms
windows for fresh/reuse/parse. Runtime-profile screens: two rounds × three
20 ms windows per case/engine/stage, with 20 ms warmups. Exact settings and host
observations are retained per run.

After narrowing the helper's reference from `&mut self` to `&self` for Clippy,
the final source form is rebuilt and measured again: all 57 native inputs in
fresh/reuse/parse with two rounds × three 30 ms windows, and all 75 feature cases
in those three stages with two rounds × three 20 ms windows. The final native
run includes OX for the original 14-case subset only; a separate exact-HTML
precheck against the frozen reference passes before timing. OX outputs outside
that subset are retained without receiving an aggregate score.

The same-binary off/on check on three plain-prose sizes uses three rounds ×
three 30 ms windows per lifecycle, with 5 ms warmups. Baseline, prototype B,
and final B have separately recorded runs. This measures enabling the flag
inside each implementation, not the historical cost of adding comment support.

Cases are shuffled with a fixed seed and engine order alternates. Only one
worker executes timed work at a time. Builds, tests, compression, and disassembly
are performed outside the timing runs. Checksums and pre/post-run output checks
guard every timed job. Inputs, parsing results, and I/O are prepared or verified
outside timed loops as specified by each harness.

Tables use the median time per case/variant and the geometric mean of the
per-case time ratios; each document has equal weight. Per-round medians and all
windows remain available. A reduction in elapsed time is reported as
`1 - candidate_time / baseline_time`, not as throughput growth. Stages have
different optimization boundaries and cannot be added. Differences near 1%
are treated cautiously, especially when confined to one small probe.

The 14-case subset matches the published OX comparison. The 57-case set includes
all original mixed inputs, 37–113,609 UTF-8 bytes. Runtime-profile probes use
roughly 300 B, 4 KiB, and 64 KiB inputs plus mixed documents. These are local
macOS arm64 results, not cross-platform performance guarantees.

## Replaying and retained artifacts

Use the existing [native diagnostic harness](../../../benchmarks/ox-regression/README.md)
and [runtime-profile optimization harness](../../../benchmarks/feature-scan-optimization/README.md).
The report includes the exact harness snapshots used. Example native candidate:

```sh
python3 benchmarks/ox-regression/prepare.py /private/tmp/paragraph-candidate \
  --native-build /private/tmp/ferromark-v2-native-matched-build \
  --patch docs/reports/2026-09-14-paragraph-fast-path/patches/B-final.patch \
  --worker docs/reports/2026-09-14-paragraph-fast-path/harness/native-worker.rs
```

Reuse the recorded variant mappings with local build paths adjusted. For feature
comparisons, prepare baseline `ec06a8c` against the accepted checkout, then replay
the archived corpus through the feature harness; `runtime_options` are
authoritative, so old absolute profile paths are safely regenerated.

Verification JSON is stored losslessly as XZ; large corpus/sample JSON uses
gzip. Build caches and executables remain outside Git, while hashes, patches,
source revisions, locks, logs, raw measurements, and original corpus attribution
are retained. The generated differential cases retain their source/config corpus
and a digest of the compared outputs. `SHA256SUMS` covers this report.
