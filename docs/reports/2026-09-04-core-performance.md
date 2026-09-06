# Core Markdown performance experiments — 2026-09-04

Baseline: `617a29dbe8833e5bdbb207186cfe573051cfd18c`.

Five proposed optimizations were implemented and screened independently against
a retained baseline executable. The candidates were not stacked during initial
screening. Negative percentages below mean less time per render.

## Scope and method

- Local ARM64 macOS 26.6.2, Rust 1.97.1 / LLVM 22.1.6; repository release profile
  and `target-cpu=apple-m1`, no PGO. No x86-64 timing claims.
- `examples/core_performance.rs` covers 13 inputs: tiny, prose, emphasis/code,
  links, entities, late first delimiter, references, lists, CommonMark 5/20/50 KB
  and 1 MB, and tables. The complete matrix uses CommonMark, default and GFM.
- `fresh` allocates parser state and owned HTML on every render; `reused` retains
  a `Renderer` and caller-owned output buffer. Each lane is compared only with
  the same lane of the baseline, not with the other API's allocation contract.
- Three timed windows per case, 16 renders per clock check, median nanoseconds
  per render. The Python driver alternates baseline/candidate order and takes
  the median of paired ratios. Source generation, JSON serialization and output
  comparisons are outside the timed windows.
- Every measured HTML output is compared byte-for-byte across executables and
  APIs. Saved JSON replaces the HTML with SHA-256 to keep evidence compact.
- Initial screening: three pairs of 30 ms windows, CommonMark only. Combined
  screening: three pairs of 50 ms windows, all three presets. These local paired
  measurements are not Criterion confidence intervals or cross-parser rankings.

## Individual experiments

1. **Bracket summary gate:** add a bracket bit to `MarkSummary` and skip bracket
   collection when absent. Ordinary CommonMark fixtures changed by less than
   1%; the targeted emphasis case did not improve. Rejected for insufficient
   benefit. The noisy tiny-input result is not used as evidence of a gain.
2. **Lazy delimiter boundaries:** construct/sort link and HTML boundaries only
   if an enabled delimiter resolver needs them. No useful link-case gain;
   emphasis regressed approximately 2.6–3.3%. Rejected.
3. **Shared text escape/entity screening:** the first escape scan proves both
   that plain text needs no escaping and that it contains no entity. Copy that
   text once. If escapes exist, reuse the known-safe prefix; if an ampersand
   exists, retain the original whole-input entity decoding semantics. Initial
   prose improvement: 3.5% fresh, 7.3% reused; emphasis/links about 2%. Retained
   for combined verification.
4. **Reuse the specials prescan position:** tried exact byte position, matching
   vector start, and returning the position directly instead of through an
   output argument. Late-delimiter input improved about 14% fresh / 27% reused,
   but short formatted input consistently regressed (roughly 2–5%). Ordinary
   CommonMark fixtures showed no broad gain. All three variants rejected.
5. **Reuse block parser scratch:** retain paragraph ranges, comment ranges,
   pending indented-code blanks, and reference parse/label buffers in `Renderer`.
   The new parser still owns fresh cursor, definitions, container state and
   resource limits for every document. Scratch lengths are cleared on return.
   Initial reused tiny input improved 21%, prose 6%, CommonMark 5 KB 2.2%.
   Retained for combined verification.

Raw screening measurements live in
[`2026-09-04-core-performance/`](2026-09-04-core-performance/). Rejected patches
are preserved under `experiments/`; apply them individually to the baseline,
not on top of the final changes.

## Final combined confirmation

The final version retains experiments 3 and 5. An additional guard skips the
second ampersand search when the first escape is already an ampersand. This
reduces the entity-heavy regression seen in the first combined screening.

Five alternating pairs, three 100 ms windows per case per executable; all 78
case/preset/API combinations had identical HTML. The longer run supersedes the
more optimistic initial 1–2% mixed-document screening deltas. Full evidence:
[`final.json`](2026-09-04-core-performance/final.json); retained executable hashes
are in [`executables.json`](2026-09-04-core-performance/executables.json).

CommonMark preset, median paired change in render time:

| Input | Fresh parser + owned HTML | Reused renderer + output |
| --- | ---: | ---: |
| tiny | -0.61% | -21.58% |
| prose | -3.22% | -11.06% |
| emphasis | -0.76% | -0.69% |
| links | -1.80% | -2.52% |
| entities | +1.56% | +0.38% |
| late_special | -2.03% | -7.98% |
| references | +0.28% | -0.17% |
| lists | -0.18% | +0.13% |
| commonmark_5k | -0.18% | -0.86% |
| commonmark_20k | +0.05% | +0.17% |
| commonmark_50k | +0.28% | -0.67% |
| commonmark_1m | -0.73% | -0.16% |

The useful gains are workload-specific: tiny reused documents take about 20–22%
less time across presets; prose takes 6–11% less with reuse and 2–3% less with a
fresh parser. Links improve about 1–2.5%. Default/GFM tables improve about 2–3%.
The ordinary CommonMark 5 KB–1 MB fixture lanes remain within approximately ±1%:
**this is not a demonstrated general mixed-Markdown throughput improvement**.

Measured tradeoff: entity-heavy input takes 1.6–2.0% more time on the fresh API
and 0.4–1.3% more on the reused API, depending on preset. The combined changes are
retained for the substantially larger prose and small-document gains; the
entity cost is explicit rather than reported as a universal win. Changes below
about 1% should not be interpreted as gains from these local measurements.

## Correctness and maintenance

- `cargo test --locked --all-features`: 970 passed, zero failed, three ignored
  before adding the final reused-renderer spec-corpus check. The subsequent
  focused run passed all three new tests, including that additional check.
- `cargo clippy --all-targets --all-features --locked -- -D warnings`: passed.
- `cargo fmt --check` and `git diff --check`: passed.
- The new regression tests cover entity escaping at SIMD length boundaries,
  invalid UTF-8 fallback, multi-codepoint entity fixups, document shape changes,
  and all 652 trusted CommonMark examples in forward and reverse order through
  the same renderer. Existing fresh-render conformance also passes.
- Public APIs and rendering policies are unchanged. Block scratch retains
  capacity until the renderer is dropped, like its existing event and inline
  buffers; definitions, parser state and source references are never retained.

## Reproduction

Build `examples/core_performance.rs` with the baseline source and retain the
executable, then build it again with the candidate source. The harness itself
must be identical in both builds.

```sh
cargo build --locked --release --example core_performance
cp target/release/examples/core_performance /tmp/core-baseline
# Apply candidate source changes, keeping the same example and lockfile.
cargo build --locked --release --example core_performance
python3 scripts/compare-core-performance.py \
  /tmp/core-baseline target/release/examples/core_performance \
  /tmp/core-comparison.json --pairs 5 --window-ms 100
```

Use `--preset commonmark` to isolate the core dialect, and `--filter prose`
or `--filter commonmark_` for focused confirmation. The retained executable
approach prevents a later build from silently changing the baseline.
