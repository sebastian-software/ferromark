# Linear nested link probing — 2026-09-17

Paired measurement of the change for
[issue #350](https://github.com/sebastian-software/ferromark/issues/350):
bracket text that holds another bracket is parsed once, where it stands,
instead of being probed on its own and then parsed again by the
literal-bracket fallback, and the walk that finds a bracket's `]` keeps the
matches of the openers it passes. The exactness argument, and the one verdict
at the nesting cap that this moves, are in
[the decision record](../../decisions/2026-09-17-linear-link-probe.md); this
report holds the numbers.

Baseline is `main` at `e8439fd` (this branch is merged with it); the
candidate is that commit plus this change. Same host, same toolchain
(Rust 1.95.0), both sides built as the repository's `release` profile
(opt-level 3, fat LTO, one codegen unit) through the same harness, which
parses and renders each input and reports the best of up to 200 runs per
case. **This sandbox is not a quiet benchmark host**: the Criterion suite
below moves ±10 % between runs of the *same* build, so nothing under about
×1.2 is resolvable here. The pathological cases move by factors of 100 to
15,000 and are not in doubt; the unchanged workloads are reported as "no
resolvable difference", not as a number.

Every case below was verified byte-identical between the two builds before
timing: the harness prints a hash of the rendered HTML and all of them match.
The corpus behind that claim is in [`corpus.md`](corpus.md).

## The shapes from the issue

Times are the best of the runs, in seconds. The nesting cap refuses more than
100 levels (issue #349), so the deep shapes are measured twice: under the
default cap, where the document holds many independent nestings of depth 100,
and with `max_nesting_depth = 0`, which is the only way to see the growth the
issue is about.

| Input | Size | Cap | Before | After | Factor |
| --- | ---: | --- | ---: | ---: | ---: |
| `[`×400 `a` `](u)`×400 | 2 KB | lifted | 0.0435 | 0.000034 | ×1280 |
| `[`×800 `a` `](u)`×800 | 4 KB | lifted | 0.3011 | 0.000075 | ×4000 |
| `[`×1600 `a` `](u)`×1600 | 8 KB | lifted | 2.5492 | 0.000168 | ×15000 |
| `[`×100 `a` `](u)`×100, 50 groups | 25 KB | default | 0.0463 | 0.000651 | ×71 |
| `[`×100 `a` `](u)`×100, 1000 groups | 501 KB | default | 1.0668 | 0.0171 | ×62 |
| `[`×100 `a` `]`×100, 1000 groups | 201 KB | default | 0.8125 | 0.0160 | ×51 |
| `[`×100 `a` `][r]`×100, 250 groups | 125 KB | default | 0.6386 | 0.0122 | ×52 |
| `[`×50000 `a](u)` (one closer) | 50 KB | default | 1.7032 | 0.0058 | ×294 |
| `[![`×40 `a` `](i)](u)`×40, 100 groups | 44 KB | default | 0.00315 | 0.00282 | — |
| `<A>`×40000, `mdx: true` | 120 KB | default | 5.384 | 5.571 | — |

The 25 KB document of depth-100 groups is the acceptance case: 0.65 ms, well
under the 100 ms the issue asks for. At 501 KB the same shape is 17 ms.

Nested images are the shape that must not pay for the change: they take the
probe path (an image opener is refused before the walk starts) and no bracket
matches are recorded for them. Both builds measure 0.0028–0.0034 s over
repeated runs, i.e. inside the spread.

The MDX open-tag case is quadratic in `mdx_jsx` for its own reason, shares
nothing with the link probe and is unchanged. It needs its own fix.

## How the cost grows

With the cap lifted, doubling the nesting:

| Depth | Size | Before | After | After, per doubling |
| ---: | ---: | ---: | ---: | ---: |
| 800 | 4 KB | 0.3011 | 0.000075 | — |
| 1600 | 8 KB | 2.5492 (×8.5) | 0.000166 | ×2.2 |
| 3200 | 16 KB | (≈ 20 s, extrapolated ×8) | 0.000445 | ×2.7 |
| 6400 | 32 KB | — | 0.001022 | ×2.3 |

Before: ×6.9, ×8.5 per doubling over 400 → 800 → 1600, the cubic the issue
reported. After: ×2.2, ×2.7 and ×2.3 (best of three series each), against ×2
for the input size itself. The remainder is the two maps, which grow with the
number of openers and leave cache-resident size at these depths: the cost per
byte is 19 ns at 4 KB and 32 ns at 32 KB. With the cap lifted the stack becomes the limit before the
time does (see below), which is what the cap exists to prevent.

A run of openers with one closer, doubling the run:

| Openers | Size | Before | After |
| ---: | ---: | ---: | ---: |
| 12500 | 12.5 KB | 0.1067 | 0.000847 |
| 25000 | 25 KB | 0.4352 (×4.1) | 0.002283 (×2.7) |
| 50000 | 50 KB | 1.7032 (×3.9) | 0.005173 (×2.3) |

## Criterion

`cargo bench --bench parser --locked` and `cargo bench --bench prepass
--locked`, three runs of the parser suite per side and two of the prepass
suite, alternating. Criterion's own mid estimate per run:

| Benchmark | Baseline runs | Candidate runs |
| --- | --- | --- |
| `parse_simple/simple_md` | 842.1, 824.7, 851.3 ns | 854.5, 907.4, 731.3 ns |
| `parse_large/large_md` | 5.766, 5.691, 5.732 µs | 6.026, 5.269, 5.543 µs |
| `parse_lists/single_item` | 119.3, 138.4, 133.6 ns | 129.5, 128.6, 116.2 ns |
| `parse_lists/long_list` | 2.942, 3.095, 2.616 µs | 2.773, 2.841, 2.871 µs |
| `parse_prepass/changelog_decoy` | 210.7, 192.0 µs | 219.8, 199.8 µs |
| `parse_prepass/reference_definitions` | 1.281, 1.296 µs | 1.307, 1.261 µs |

`parse_lists/single_item` is `- one\n` — a document with no bracket in it —
and it moves ±8 % between runs of the *same* build; `parse_lists/long_list`
moves ±9 % and `parse_simple/simple_md` ±3 %. That is the resolution of this
host. Every candidate value falls inside the baseline's own spread for its
case, so neither a regression nor a gain is resolvable from these suites.
None of them is link-dense enough to exercise the changed path much; the
documents below are.

## Ordinary documents

Repository documents through the same harness (parse + render, best of up to
20,000 runs, three alternating pairs each). These are the link-dense
real-world shape the change must not slow down. Rendered HTML hashes match
between the builds.

| Document | Size | Before | After |
| --- | ---: | ---: | ---: |
| `README.md` | 5.9 KB | 17.5, 17.0, 14.1 µs | 14.6, 16.6, 14.6 µs |
| `CHANGELOG.md` | 6.9 KB | 21.8, 21.5, 19.7 µs | 21.2, 18.3, 18.4 µs |
| `CONTRIBUTING.md` | 5.6 KB | 7.9, 9.5, 9.5 µs | 7.7, 9.4, 9.1 µs |
| `docs/reports/2026-09-16-definition-segments/README.md` | 8.8 KB | 28.1, 29.0, 28.3 µs | 28.6, 27.7, 24.4 µs |
| `benches/fixtures/upstream-changelog.md` | 82 KB | 346, 341, 292 µs | 339, 334, 338 µs |

All of these are at or slightly below the baseline, but the margins are
inside the host's spread; they are reported as "no regression", not as a
speed-up.

## Stack

The in-place walk recurses less than the probe path it replaces: three frames
per bracket level where the probe took four, and no `parse_inline` frame for
the text it keeps. With `max_nesting_depth = 0` on a 1 MiB worker stack,
`[`×N `a` `]`×N parsed to N = 900 and overflowed by N = 950 before the
change; after it, N = 1200 parses and N = 1600 overflows. The default cap of
100 keeps every ordinary parse an order of magnitude away from either bound.

## Not measured here

The six-engine native comparison, the Node addon and profile-guided builds
were not rerun: the change removes work from one inline path and adds no code
to the renderer or the block parser. The paired
`benchmarks/optimization-rounds` harness was not run either — this host
cannot resolve the sub-percent differences it is built for, which is why the
unchanged workloads above are reported as ranges rather than ratios.
