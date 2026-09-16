# Segmented definition pass — 2026-09-16

Paired measurement of the change for
[issue #320](https://github.com/sebastian-software/ferromark/issues/320): the
document-wide definition discovery no longer block-parses a whole document
because it holds one `]:` candidate. It parses only the segments that can hold a
definition, each bounded by a line start where the real parser is provably at
the document root, and runs no structural pass at all when every candidate sits
inside code or raw HTML. The exactness argument and the differential proof are
in [the decision record](../../decisions/2026-09-16-segmented-definition-pass.md);
this report holds the numbers.

Baseline is `main` at `e35e9f64`; the candidate is that commit plus the nine
commits of the change (`80883258`). Same host (Apple M1 Pro), toolchain
(Rust 1.95.0, LLVM 22.1.2), fat-LTO worker built with `-C target-cpu=generic`
and the paired harness of the earlier rounds (`benchmarks/optimization-rounds`),
3 rounds × 5 pairs × 40 ms windows per case and stage. Every case and stage was
verified for exact HTML and AST `Debug` equality between the two workers before
timing. Ratios are baseline time over candidate time, medians of the paired
windows; higher is faster. Load average during the runs was 7–13, driven by
unrelated indexers; the A/A control below was taken under the same conditions.

## Reproduction

The issue attributes the outliers to the structural pass. Measured on the
baseline binary alone, comparing each document with a copy in which every `]:`
is replaced by `]=` (archived as [`corpus-variants.json.gz`](corpus-variants.json.gz);
the copies render differently, so the ratio bounds the discovery cost plus the
work the real definitions cause, and is not itself a target):

Original document time over the time of the same document with every `]:` replaced by `]=`, both measured in the same A/A run (baseline side).

| Document | Fresh | Reuse | Parse | Render |
| --- | ---: | ---: | ---: | ---: |
| `comment-incident` | 1.460 (1.82 vs 1.25 µs) | 1.506 (1.67 vs 1.11 µs) | 1.708 (1.36 vs 0.80 µs) | 1.010 (0.29 vs 0.28 µs) |
| `rust-book-ch00-00-introduction` | 1.357 (17.38 vs 12.80 µs) | 1.327 (16.91 vs 12.74 µs) | 1.430 (14.49 vs 10.13 µs) | 0.975 (2.33 vs 2.39 µs) |
| `typescript-handbook-advanced-types` | 1.206 (52.53 vs 43.55 µs) | 1.302 (54.42 vs 41.79 µs) | 1.508 (32.25 vs 21.39 µs) | 0.987 (16.79 vs 17.00 µs) |

## Result on the affected documents

| Document | Fresh | Reuse | Parse | Render |
| --- | ---: | ---: | ---: | ---: |
| `comment-incident` (1.1 KiB, one `]:` inside a fenced example) | **1.295×** | **1.342×** | **1.443×** | 1.007× |
| `rust-book-ch00-00-introduction` (10.7 KiB, three definitions) | **1.197×** | **1.196×** | **1.235×** | 1.008× |
| `typescript-handbook-advanced-types` (40 KiB, 13 index signatures in fences) | 1.061× | 1.041× | **1.095×** | 1.013× |
| `scan-extension-links` (322 B, footnote and reference definitions) | **1.170×** | **1.172×** | **1.218×** | 0.997× |
| `scan-long-references` (11.8 KiB, 48 definitions back to back) | 0.999× | 1.000× | 1.004× | 0.994× |
| `scan-short-reference` (20 B) | 0.970× | 0.979× | 0.971× | 0.975× |

The three marker-free copies are unchanged (0.986–1.006× on every stage), so
the gain is the pass that no longer runs, not a shift elsewhere. In absolute
terms `comment-incident` drops from 1.78 to 1.38 µs fresh, the Rust
introduction from 16.8 to 14.1 µs, the TypeScript page from 51.3 to 48.3 µs
(its rounds spread 1.02–1.07×; the page is dominated by inline work and the
load was highest during its windows).

`scan-long-references` is reference-dense and is sent to the unsegmented pass
by the density rule before any planning happens. The first candidate of this
work planned it anyway and cost it 0.908–0.916× (+3.0 µs: every opener judged
and 191 lines walked to plan one segment covering the whole document), and
`scan-short-reference` 0.874–0.900× (+60 ns of planning on a 20-byte
document); the second candidate, with the density rule and the jump-based walk
but the opener collection still ahead of the length check, had them at 1.00×
and 0.95–0.97×. Both intermediate builds are archived under
[`results/candidate-1-issue-docs`](results/candidate-1-issue-docs/) and
[`results/candidate-2-issue-docs`](results/candidate-2-issue-docs/) as the
evidence for the two thresholds. In the final build `scan-short-reference`
skips planning by length; the remaining 9–16 ns are consistent with the one
call that is now kept out of line and the verdict it returns, and were not
attributed further.

## The broad and diagnostic sets

| Set | N | Fresh | Reuse | Parse | Render |
| --- | ---: | --- | --- | --- | --- |
| A/A control (same binary twice, 10 documents) | 10 | 1.009× (rounds 1.002, 1.010, 1.010) | 1.006× (rounds 1.005, 1.004, 1.008) | 0.999× (rounds 0.995, 1.001, 1.001) | 1.032× (rounds 1.029, 1.031, 1.028) |
| Original broad documents | 57 | 1.007× (rounds 1.007, 1.008, 1.009) | 1.009× (rounds 1.009, 1.010, 1.008) | 1.009× (rounds 1.008, 1.009, 1.009) | 1.002× (rounds 1.003, 1.002, 1.001) |
| Authored diagnostics | 45 | 1.000× (rounds 0.999, 0.998, 1.001) | 0.997× (rounds 0.997, 0.997, 0.998) | 0.993× (rounds 0.993, 0.993, 0.994) | 0.999× (rounds 1.000, 0.997, 0.998) |

| Category | N | Fresh | Reuse | Parse | Render |
| --- | ---: | ---: | ---: | ---: | ---: |
| comments | 12 | 1.017× | 1.020× | 1.024× | 0.999× |
| encyclopedia | 12 | 1.001× | 1.003× | 0.999× | 1.002× |
| plain-prose | 4 | 0.998× | 0.997× | 1.001× | 1.002× |
| readme | 2 | 0.997× | 0.995× | 0.997× | 1.006× |
| reference | 4 | 1.009× | 1.009× | 1.024× | 1.005× |
| syntax-guard | 1 | 0.999× | 0.997× | 1.001× | 0.998× |
| technical-docs | 22 | 1.008× | 1.011× | 1.007× | 1.003× |

| Stage | Above | Below | Min | Max |
| --- | ---: | ---: | ---: | ---: |
| fresh | 24 | 33 | 0.983 | 1.301 |
| reuse | 29 | 28 | 0.982 | 1.338 |
| parse | 22 | 35 | 0.968 | 1.446 |
| render | 35 | 22 | 0.989 | 1.028 |

Only `comment-ack` (48 ns parse, 0.968×) falls under 0.97× on the broad set,
and on the diagnostics only documents of 18–150 ns do (`scan-empty` 0.908× parse
at 18 ns, the `autolink-*-64` shapes at 0.960–0.968× on 43–50 ns, `scan-short-link`
reuse 0.961× at 146 ns). All of them contain no `]:`, so their code path is the
two byte searches it was before; the 1–2 ns are the fat-LTO placement variance
the earlier rounds documented for unchanged paths. Render moves 0.989–1.028×
on the broad set although no render code changed.

## What the planner does on these corpora

Counted by the differential test in `src/parser/prepass/equivalence.rs`
(`cargo test --lib -- parser::prepass::equivalence --nocapture`): on the 57
broad documents no document falls back to the whole-body pass, and the
structural pass is asked to parse 240 of 914,619 body bytes — all of them in
`rust-book-ch00-00-introduction`. `comment-incident` and
`typescript-handbook-advanced-types` run no structural pass at all. On the
207-case round-3 corpus 7 cases fall back: `scan-long-references` on density
under each option set, and `scan-mdx-links` under MDX, which is excluded by
rule. Of the 6,084 specification checks, 9 fall back on adversarial container
or HTML shapes; every fallback collects identically by construction.

## Files

- [`NUMBERS.md`](NUMBERS.md) — the tables above and the per-document issue
  table, generated from the archived results.
- [`TABLES.md`](TABLES.md) — every case of every set with min/max, per-round
  medians and baseline nanoseconds.
- [`results/`](results/) — `aa-control` (same binary on both sides, 10
  documents), `aa-repro` (the marker-free reproduction), `issue-docs`, `broad`,
  `diagnostics`, plus the two intermediate candidates' issue-document runs; each
  with `summary.json`, `grouped.json`, `run.json`,
  `build.json` (toolchain, worker hash, verified baseline revision) and the
  gzipped raw windows.
- [`corpus-variants.json.gz`](corpus-variants.json.gz) — the three `]=`
  copies; the other cases are byte-identical to
  [`../2026-09-16-arm-round-3/corpus.json.gz`](../2026-09-16-arm-round-3/corpus.json.gz).
- [`harness/`](harness/) — the session scripts that built the pair and ran the
  three sets, and the report generator.

## Not measured here

The six-engine native comparison was not rerun for this change; the other
engines are untouched and v2 moves only by the per-document ratios above. The
next release-native report (the one produced for the `2.0.0-rc.2` release)
carries the competitive position. Profile-guided builds were not measured
either; the change removes work rather than reshaping hot loops, so no
interaction is expected, but that is an expectation, not a number.
