# Linear nested link probing — 2026-09-17

Paired measurement of the change for
[issue #350](https://github.com/sebastian-software/ferromark/issues/350):
bracket text that holds another bracket is parsed once, where it stands,
instead of being probed on its own and then parsed again by the
literal-bracket fallback, and the walk that finds a bracket's `]` keeps the
matches of the openers it passes. The exactness argument is in
[the decision record](../../decisions/2026-09-17-linear-link-probe.md); this
report holds the numbers.

Baseline is `main` at `bdcbf871` (the commit this branch starts from); the
candidate is that commit plus this change. Same host, same toolchain
(Rust 1.95.0), both sides built as the repository's `release` profile
(opt-level 3, fat LTO, one codegen unit) through the same harness, which
parses and renders each input and reports the best of up to 200 runs per
case. **This sandbox is not a quiet benchmark host**: the run-to-run spread
of the Criterion suite below is ±5–7 % on unchanged code, so nothing under
about ×1.1 is resolvable here. The pathological cases move by factors of
100–20,000 and are not in doubt; the unchanged-workload cases are reported
as "no resolvable difference", not as a number.

Every case below was verified byte-identical between the two builds before
timing: the harness prints a hash of the rendered HTML, and all of them
match. The broader equality evidence (1.4 M renders of a generated corpus,
HTML and AST) is in the decision record and in
[`corpus.md`](corpus.md).

## The shapes from the issue

Times are the best of the runs, in seconds. The nesting cap refuses more
than 100 levels (issue #349), so the deep shapes are measured twice: under
the default cap, where the document holds many independent nestings of
depth 100, and with `max_nesting_depth = 0`, which is the only way to see
the growth the issue is about.

| Input | Size | Cap | Before | After | Factor |
| --- | ---: | --- | ---: | ---: | ---: |
| `[`×400 `a` `](u)`×400 | 2 KB | lifted | 0.0438 | 0.000027 | ×1600 |
| `[`×800 `a` `](u)`×800 | 4 KB | lifted | 0.3328 | 0.000060 | ×5500 |
| `[`×1600 `a` `](u)`×1600 | 8 KB | lifted | 2.5607 | 0.000130 | ×19700 |
| `[`×100 `a` `](u)`×100, 50 groups | 25 KB | default | 0.0418 | 0.000488 | ×86 |
| `[`×100 `a` `](u)`×100, 1000 groups | 501 KB | default | 0.9588 | 0.0130 | ×74 |
| `[`×100 `a` `]`×100, 1000 groups | 201 KB | default | 0.8622 | 0.0117 | ×74 |
| `[`×100 `a` `][r]`×100, 250 groups | 125 KB | default | 0.6272 | 0.0122 | ×51 |
| `[`×50000 `a](u)` (one closer) | 50 KB | default | 1.3654 | 0.0051 | ×267 |
| `[![`×40 `a` `](i)](u)`×40, 100 groups | 44 KB | default | 0.00296 | 0.00302 | — |
| `<A>`×40000, `mdx: true` | 120 KB | default | 5.067 | 4.909 | — |

The 25 KB document of depth-100 groups is the acceptance case: 0.49 ms,
well under the 100 ms the issue asks for.

Nested images are the shape that must not pay for the change: they take the
probe path (an image opener gives up the in-place walk immediately) and the
bracket matches are deliberately not recorded for them. Repeated runs put
both builds at 0.0029–0.0034 s, i.e. inside the spread.

The MDX open-tag case is quadratic in `mdx_jsx` for its own reason, shares
nothing with the link probe and is unchanged. It needs its own fix.

## How the cost grows

With the cap lifted, doubling the nesting:

| Depth | Size | Before | After | After, per doubling |
| ---: | ---: | ---: | ---: | ---: |
| 1600 | 8 KB | 2.5607 | 0.000129 | — |
| 3200 | 16 KB | (≈ 20 s, extrapolated ×8) | 0.000316 | ×2.45 |
| 6400 | 32 KB | — | 0.000732 | ×2.32 |
| 12800 | 64 KB | — | 0.001839 | ×2.51 |

Before: ×7.6, ×7.6 and ×7.7 per doubling over 200 → 400 → 800 → 1600, which
is the cubic the issue reported. After: ×2.4 on average, against ×2 for the
input size itself; the remainder is the map that grows with the number of
openers.

A run of openers with one closer, doubling the run:

| Openers | Size | Before | After |
| ---: | ---: | ---: | ---: |
| 12500 | 12.5 KB | 0.1080 | 0.000818 |
| 25000 | 25 KB | 0.4210 (×3.9) | 0.002408 (×2.9) |
| 50000 | 50 KB | 1.3654 (×3.2) | 0.005111 (×2.1) |

## Criterion

`cargo bench --bench parser --locked` and `cargo bench --bench prepass
--locked`, three runs of the parser suite per side and two of the prepass
suite, alternating. Criterion's own mid estimate per run:

| Benchmark | Baseline runs | Candidate runs |
| --- | --- | --- |
| `parse_simple/simple_md` | 822.7, 833.8, 726.4 ns | 832.3, 778.4, 856.1 ns |
| `parse_large/large_md` | 5.701, 5.972, 5.330 µs | 5.675, 5.854, 5.108 µs |
| `parse_lists/single_item` | 124.6, 131.4, 136.2 ns | 131.4, 120.4, 135.2 ns |
| `parse_lists/long_list` | 2.722, 2.535, 2.785 µs | 2.775, 2.602, 2.910 µs |
| `parse_prepass/changelog_decoy` | 218.1, 199.7 µs | 204.6, 222.5 µs |
| `parse_prepass/reference_definitions` | 1.315, 1.301 µs | 1.300, 1.190 µs |

`parse_lists/single_item` is `- one\n` — a document with no bracket in it —
and it moves ±9 % between runs of the *same* build. That is the resolution
of this host. Every candidate value falls inside the baseline's own spread
for its case, so no regression and no gain is resolvable from these suites.
None of them is link-dense enough to exercise the changed path much; the
documents below are.

## Ordinary documents

Repository documents through the same harness (parse + render, best of up to
20,000 runs, three alternating pairs each). These are the link-dense
real-world shape the change must not slow down. Rendered HTML hashes match
between the builds.

| Document | Size | Before | After |
| --- | ---: | ---: | ---: |
| `README.md` | 5.9 KB | 17.6, 14.5, 16.7 µs | 14.5, 14.0, 15.8 µs |
| `CHANGELOG.md` | 6.9 KB | 21.2 µs | 20.6 µs |
| `CONTRIBUTING.md` | 5.6 KB | 8.4, 8.9, 9.0 µs | 7.6, 8.7, 7.6 µs |
| `docs/reports/2026-09-16-definition-segments/README.md` | 8.8 KB | 27.7, 28.6, 28.6 µs | 27.6, 28.2, 24.3 µs |
| `benches/fixtures/upstream-changelog.md` | 82 KB | 333 µs | 291 µs |

All of these are at or slightly below the baseline, but the margins are
inside the host's spread; they are reported as "no regression", not as a
speed-up.

## Stack

The in-place walk recurses three frames per bracket level where the probe
path took four. With `max_nesting_depth = 0` on a 1 MiB worker stack,
`[`×N `a` `]`×N parsed to N = 950 and overflowed by N = 980 before the
change; after it, N = 2400 parses and N = 3000 overflows. The default cap of
100 keeps every ordinary parse three orders of magnitude away from either
bound.

## Not measured here

The six-engine native comparison, the Node addon and profile-guided builds
were not rerun: the change removes work from one inline path and adds no
code to the renderer or the block parser. The paired
`benchmarks/optimization-rounds` harness was not run either — this host
cannot resolve the sub-percent differences it is built for, which is why the
unchanged workloads above are reported as ranges rather than ratios.
