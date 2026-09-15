# Single-crate consolidation

Decision: retain the consolidation in `6ac7837da2819c0dfd03c85c9a4269b69b42a8d2`. Publish one Rust crate with
public allocator, AST, parser and renderer modules. The private Node cdylib remains
separate. No parsing or rendering algorithm, option default, or external dependency
version changes. See [ADR-0018](../../arch/ADR-0018-single-rust-crate.md).

## Correctness and package validation

- 835 Rust tests, including five doctests, pass after consolidation.
- All 168 snapshot output bodies are byte-for-byte identical; paths and unit-test
  module identifiers moved with the source.
- Exact HTML, debug AST (including source spans), and root-child counts match
  before/after, fresh/reused, and before/after timing for every measured input.
- Clippy, rustdoc, formatting, all seven benchmark builds, and Node package checks
  pass. The Node checks include panic conversion and a clean packed installation.
- Cargo builds/verifies the single archive, then an isolated consumer runs from
  its unpacked contents. A full publish dry-run succeeds without uploading.
- External Cargo.lock entries are unchanged. The benchmark runner verifies each
  recorded worker lock hash and compares the external dependency graph in full;
  removing the four local crate entries intentionally changes the overall hash.

## Measurements

Positive deltas mean longer elapsed time. The table reports the median of three
rounds; each round sums the CommonMark/GFM (or targeted-profile) rotating batches.
The broad input set is the existing 57-document suite, with the same options as
its previous refactoring runs. Individual documents were also measured.

| Corpus | Lifecycle | Complete-suite delta | Individual round range |
| --- | --- | ---: | ---: |
| targeted (15 inputs) | fresh | +0.90% | +0.47% to +1.72% |
| targeted (15 inputs) | reuse | +0.91% | +0.86% to +1.13% |
| broad (57 inputs) | fresh | +0.33% | +0.17% to +1.26% |
| broad (57 inputs) | reuse | +0.71% | -0.08% to +1.73% |

The aggregate broad-corpus cost is below one percent in this run. This is not
uniform zero overhead: the largest per-document median increases are below.

| Document | Lifecycle | Before, µs | After, µs | Elapsed-time delta |
| --- | --- | ---: | ---: | ---: |
| `wiki-chess-lead` | reuse | 3.56 | 3.69 | +3.82% |
| `wiki-tea-lead` | reuse | 7.04 | 7.30 | +3.62% |
| `wiki-rainbow-first-paragraph` | fresh | 1.15 | 1.18 | +3.27% |
| `wiki-rainbow-first-paragraph` | reuse | 0.83 | 0.85 | +3.20% |
| `wiki-volcano-first-paragraph` | reuse | 2.03 | 2.10 | +3.19% |
| `wiki-rainbow-lead` | reuse | 1.37 | 1.42 | +3.16% |

These are small but measured costs, accepted in exchange for one package identity,
one publication, fewer version pins, and removal of the bootstrap and cross-crate
test-dependency workarounds. This report makes no new cross-library speed claim.
Build time, other CPU architectures, and different compiler/LTO settings were
not compared. Existing published benchmark figures remain unchanged.

## Reproduce

Baseline: `50d175e5133ba0fd33b08efe5aa6456e96057b59`. Candidate: the frozen working
tree whose library sources and Cargo build inputs match `6ac7837da2819c0dfd03c85c9a4269b69b42a8d2` byte-for-byte.
Subsequent test regeneration hints and documentation do not change those inputs.
The recorded worker source, binary, source-tree and lock hashes identify both
builds. Both use Rust 1.95, generic CPU, fat LTO, one codegen unit and panic abort.
Builds completed before timing. Each of three independent rounds alternates the
before/after order across five pairs of 40 ms windows, after a 20 ms warmup.

```sh
python3 benchmarks/runtime-profiles/prepare.py /tmp/single-before --revision 50d175e
python3 benchmarks/runtime-profiles/prepare.py /tmp/single-after --revision 6ac7837da2819c0dfd03c85c9a4269b69b42a8d2
python3 benchmarks/refactoring/fixtures.py > /tmp/single-targeted.json
python3 benchmarks/suite-regression/run.py /tmp/single-before /tmp/single-after /tmp/single-targeted.json /tmp/single-targeted
python3 benchmarks/suite-regression/run.py /tmp/single-before /tmp/single-after docs/reports/2026-09-15-suite-regression/corpus.json.gz /tmp/single-broad
```

[Targeted results](targeted/summary.json), [broad results](broad/summary.json),
[raw timing windows](broad/samples.json.gz), [verified outputs](broad/verification.json.gz),
[build identities](broad/builds.json), and [harness hashes](harness-sha256.json)
retain the evidence. The compressed corpus files include each input and its
attribution; timing does not modify or regenerate the archived baseline reports.
