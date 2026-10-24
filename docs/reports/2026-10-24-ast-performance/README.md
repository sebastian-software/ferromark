# AST API performance investigation

This follow-up compares the APIs from #563, #564 and #565 with the previous
revision, then checks two changes retained in the implementation. The original
[alt-content opt-in characterization](../2026-10-24-ast-provenance/README.md)
compares off/on within the initial implementation; it is a different question.

## Scope and method

Baseline: `b4e4377a541a15aa72559cda1ab26fc8e3f322e8`.
Candidate: the local implementation reconstructed by applying patches 01, 02
and 05 in order. Patches 03 and 04 are retained experiments and are not part of
the accepted code. All stages use identical registry versions from Cargo.lock,
Rust 1.95, the `jsx` Cargo feature, opt-level 3, fat LTO, one codegen unit,
`panic=abort`, and `-C target-cpu=generic`.

The repository's existing optimization-round worker and runner perform exact
HTML, AST Debug (including spans), and child-count comparisons before timing
and around every measured batch. All comparisons passed. The [worker](worker.rs)
only adapts the historical harness to the current option fields and adds a
`strict-mdx` profile; it does not change the timed loops. The
[builder](build_pair.py) uses the existing frozen-source builder, enables `jsx`
in both engines and copies the current transforms workspace member.

The final corpus contains the existing 57 mixed documents and eight authored
diagnostics: prose, plain/rich image labels, callouts, strict MDX, nesting,
fragments and deeper nesting beyond the tag stack's inline capacity. The initial
run predates the eighth diagnostic. Existing document provenance is retained
in each compressed corpus and in the original frozen SIMD report.

`parse` reuses a source-sized arena and resets it after each parse. `fresh`
creates/drops an arena and renderer and includes HTML output. These measure
public default behavior; `image_alt_content` and diagnostic collection stay off.
The two binaries alternate in each pair; case order is shuffled deterministically.
Full runs use three rounds, three pairs and 30 ms windows. Focused confirmations
use three rounds, five pairs and 100 ms windows. A/A uses the identical old
binary in both slots, with three rounds, three pairs and 50 ms windows.

Host: macOS 27.0.1, arm64, AC power. CPU-model and thermal probes are unavailable
in the sandbox, and host activity is not controlled. Host observations, exact
sampling configuration, source/binary hashes, samples and verification output
are retained under [raw/](raw/). This is local diagnostic evidence, not a
cross-platform throughput claim.

## Accepted changes

1. Strict MDX tag matching stores source-backed names, with an empty source
   slice identifying each fragment. It reconstructs a conflicting opener's span
   through the existing tag scanner only on failure, using the expression memo.
   The tag-stack container returns from 392 to 264 bytes on this target,
   matching the old parser's footprint. [Layout probe](layout.rs),
   [result](layout.txt). Runtime differences from this change alone were mixed
   and below 1%; its demonstrated benefit is storage.
2. Ordinary `Parser::parse()` keeps its former configuration/completion control
   flow. The new diagnostic entry point owns the helpers it needs. This removes
   extra helper calls from the ordinary path. Some checks are intentionally
   repeated between the two entry points to preserve that path's layout.

The completion-inline experiment improved some focused cases, but did not
remove all short-input differences. Adding another inline hint for configuration
checks had mixed results and regressed some cases. Neither hint is retained.
The accepted path restores the original structure rather than relying on them.

The additional related-span tests cover fragments, multiline openers and `<`/`>`
inside JavaScript attributes and quoted strings. Existing JavaScript validation,
container source mapping and nesting limits remain in effect.

## Broad comparison with the old revision

Values are changes in per-document latency; positive means slower. Each corpus
entry has equal weight in the geometric mean. These small values do not prove
strict neutrality, particularly given the control variation below.

| Implementation | Stage | Documents | Latency change | Per-round changes |
| --- | --- | ---: | ---: | --- |
| Initial APIs | parse | 57 | +0.18% | +0.18% / +0.25% / +0.25% |
| Initial APIs | fresh | 57 | +0.14% | -0.04% / +0.08% / +0.22% |
| Accepted implementation | parse | 57 | +0.08% | +0.19% / +0.12% / +0.06% |
| Accepted implementation | fresh | 57 | +0.07% | -0.15% / +0.14% / -0.03% |

## Authored diagnostics, final full run

| Workload | Parse latency change | Fresh parse + HTML change |
| --- | ---: | ---: |
| prose | +0.60% | +1.15% |
| plain-images | -0.88% | -0.96% |
| rich-images | -1.22% | +2.23% |
| callouts | +0.48% | -0.12% |
| strict-mdx | +0.78% | +0.91% |
| strict-nested | -0.76% | -0.71% |
| strict-fragments | +0.04% | +0.06% |
| strict-deep | -0.82% | -1.28% |

## Longer confirmation of notable cases

The short-comment differences persist at roughly a few nanoseconds. The rich
image workload parses faster but its complete parse/render is around 2% slower.
The volcanic-article result varies across rounds; the longer confirmation does
not retain the short run's roughly 3.5% end-to-end difference. Strict-MDX changes
are comparable to the measured A/A variation. No claim of exact neutrality for
every individual input follows from the broad mean.

The absolute changes below subtract the separately computed median times;
percentage changes use the median paired ratio, so they need not match exactly.

| Case | Stage | Latency change | Median time difference |
| --- | --- | ---: | ---: |
| comment-inline-code | parse | +1.60% | +1.98 ns |
| comment-inline-code | fresh | +1.50% | +3.76 ns |
| comment-reproduction | parse | +1.43% | +3.82 ns |
| comment-reproduction | fresh | +0.78% | +7.66 ns |
| wiki-volcano-article-body | parse | +0.85% | -288.97 ns |
| wiki-volcano-article-body | fresh | +1.32% | +1078.41 ns |
| target-rich-images | parse | -1.37% | -181.47 ns |
| target-rich-images | fresh | +1.99% | +430.50 ns |
| target-strict-mdx | parse | +1.08% | +335.56 ns |
| target-strict-mdx | fresh | +0.46% | +826.50 ns |

## Same-binary A/A control

| Case | Stage | Apparent latency change |
| --- | --- | ---: |
| comment-inline-code | parse | -0.03% |
| comment-inline-code | fresh | -0.32% |
| comment-reproduction | parse | +0.19% |
| comment-reproduction | fresh | +0.33% |
| target-rich-images | parse | -0.39% |
| target-rich-images | fresh | +0.13% |
| target-strict-mdx | parse | +0.83% |
| target-strict-mdx | fresh | +0.48% |

## Remaining costs and limits

`Image` still grows from 64 to 72 bytes on this 64-bit target, even when alt
retention is off. Removing that slot would require a different metadata storage
API. Retaining alt children remains opt-in and has real allocation/parse costs;
this work does not make that additional analysis free. Callout inspection still
borrows the AST, and diagnostic collection still runs only when requested.

The broad default result is close to the old parser on this host, while narrow
inputs retain small differences. No x86-64 run, process-RSS measurement or Node
boundary benchmark was performed. Functional validation covers the complete
Rust workspace and the no-default-features build.

## Reproduction and retained evidence

Create a temporary source archive of the baseline with Cargo.toml, Cargo.lock,
rust-toolchain.toml, README.md, LICENSE, src, tests, benches, examples,
node/native and transforms. Apply [patch 01](patches/01-initial-api.patch) to
reconstruct the initial APIs; apply patches 02 and 05 for the accepted runtime
sources. Keep a separate unmodified baseline archive. Build and run with:

```sh
python3.12 docs/reports/2026-10-24-ast-performance/build_pair.py \
  /tmp/old /tmp/candidate /tmp/ast-build --verify-baseline
python3.12 benchmarks/optimization-rounds/run.py /tmp/ast-build \
  docs/reports/2026-10-24-ast-performance/corpus.json.gz /tmp/ast-run \
  --rounds 3 --pairs 3 --window-ms 30 --modes parse fresh
```

The small layout probe links against the build's SmallVec dependency using the
same panic strategy and fat LTO. It models Span as its two u32 fields. Raw JSON
files compressed by the archiver decompress byte-for-byte to the originals;
[SHA256SUMS.json](SHA256SUMS.json) records artifact integrity. The other
experiments and their patches remain available to explain the acceptance
choice. The Python summarizer generates these tables from retained estimates;
benchmark values were not entered by hand.

## Validation

1,418 tests passed across 97 suites, with no failures or ignored tests.

```sh
cargo fmt --all --check
RUST_TEST_THREADS=1 cargo test --workspace --all-features --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo bench --workspace --no-run --locked
cargo check -p ferromark --no-default-features --locked
```

Every command exited successfully. [Validation record](validation.json); [patch replay](reproduction.json).
