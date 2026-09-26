# Container span-map endpoint lookup

This report records a rejected implementation experiment for
[issue #432](https://github.com/sebastian-software/ferromark/issues/432).
The migration scope and its behavior and benchmark gates remain in
[ADR-0021](../../arch/ADR-0021-source-preserving-container-parsing.md).

## Question

Can block-quote span remapping avoid searching the `SourceMap` twice for every
ordered, non-empty span whose endpoints lie in the same map entry, without
changing parser behavior?

The existing mapper finds the source-map entry for each endpoint separately.
The candidate reused the start-entry lookup for the end only when that entry
also contained the end. Empty, reversed, out-of-range, boundary and
cross-entry spans retained the existing path.

## Correctness checks

The candidate passed an exhaustive differential mapping test against the
previous two-search implementation across empty, contiguous, blank-run,
zero-length-entry and gapped maps. Focused quote span and nested error tests,
the full AST/span/HTML comparator, and the required Cargo checks passed:

- `cargo fmt --all --check`
- `cargo test --workspace --all-features --locked`
- `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`
- `cargo bench --workspace --no-run --locked`

The paired comparator found byte-identical HTML, exact AST debug output
(including spans) and child counts for all 57 broad documents in fresh, reuse,
parse and render modes.

## Measurements

The baseline was `eb506d6968dc292d91e60ea51762d62bc98edc52`, the merged #450
revision. The run used Rust 1.95.0, fat LTO, an Apple Silicon host, three rounds
and three paired windows per round. Ratios are baseline time divided by
candidate time; values above 1.000 are faster.

| Broad corpus, 57 documents | Geometric mean | Slowest case ratio |
| --- | ---: | ---: |
| Fresh | 1.0041× | 0.9727× |
| Reuse | 1.0066× | 0.9809× |
| Parse | 1.0101× | 0.9938× |
| Render | 0.9936× | 0.9394× |

The isolated `container-quote-4096` diagnostic improved fresh (1.0495×), reuse
(1.0307×) and parse (1.0422×); render was 0.9964×. Those diagnostic gains did
not carry through to the broad fresh and reuse measurements. The broad render
regression also needs an explanation before accepting the change. No x86-64
paired run was made.

A sampling profile of the same 4 KB quote diagnostic attributed about 37.4% of
parse samples to `SourceMap::map_with_indent`, 4.6% to recursive node-span
remapping and 20.7% to inline parsing. This identifies a hotspot in the
diagnostic; it does not establish a broad-corpus gain.

## Outcome and next constraint

The candidate was reverted. Parse-only clears 1.010×, but complete fresh and
reuse processing reach only 1.0041× and 1.0066×. Render measures 0.9936×, with
one unexplained 0.9394× case on Apple Silicon, and no x86-64 result was
collected. The candidate was not retained and no performance improvement is
claimed.

The profile supports investigating source-aware parsing, but the current parser
and AST assume contiguous strings: parser routines slice one `&str`, and
`Text.value` borrows one `&str`. Removing quote prefixes from multiple lines
produces discontiguous logical content. A general source-view implementation
therefore needs to define how scanners traverse those segments and when leaf
content is materialized, or it needs an AST representation change. A
direct single-line block-quote path was measured earlier: broad ratios were
1.0063× fresh, 1.0046× reuse, 1.0098× parse and 1.0010× render. It also lost
about 10% on repeated-quote diagnostics and was reverted. Any new candidate
still has to satisfy the exact-parity and two-architecture requirements in
ADR-0021.

## Follow-up experiment: monotonic start-index hint

A second candidate cached the previous generated start and its `SourceMap`
partition point. It searched only the remaining suffix for nondecreasing
queries, used the original full lookup for backward queries, and cleared the
hint when the map changed. Differential lookup tests covered ascending,
descending and interleaved query orders; empty, contiguous, gapped, blank-run,
zero-length and indented maps; and map growth. The paired comparator found
exact HTML and AST/span equality for all 57 broad and 26 container cases across
fresh, reuse, parse and render.

The runs used the same baseline, Rust 1.95.0, fat LTO, generic CPU target, 3
rounds and 3 paired windows per round on Apple Silicon. Ratios are baseline
time divided by candidate time.

| Broad corpus, 57 documents | Geometric mean | Round ratios | Cases faster | Cases below 0.970× |
| --- | ---: | --- | ---: | --- |
| Fresh | 1.0062× | 1.003 / 1.009 / 1.006 | 44 | none |
| Reuse | 1.0033× | 0.998 / 1.003 / 1.004 | 40 | none |
| Parse | 1.0063× | 1.006 / 1.007 / 1.006 | 40 | none |
| Render | 1.0012× | 1.002 / 1.002 / 1.000 | 28 | none |

| Container diagnostics, 26 cases | Geometric mean | Round ratios | Cases faster | Cases below 0.970× |
| --- | ---: | --- | ---: | --- |
| Fresh | 1.0516× | 1.054 / 1.050 / 1.047 | 12 | 5 |
| Reuse | 1.0469× | 1.047 / 1.051 / 1.048 | 10 | 5 |
| Parse | 1.0504× | 1.054 / 1.048 / 1.047 | 12 | 5 |
| Render | 0.9994× | 1.000 / 0.999 / 1.001 | 10 | none |

The diagnostic gains do not meet the broad 1.010× retention bar. In parse
mode, five quote-related diagnostics regress below 0.970×: nested quotes reach
0.798×, plain quotes 0.843×, quote tabs 0.903×, lazy quotes 0.914× and mixed
quote/list 0.954×. The variant was rejected; no x86-64 run was dispatched and
no performance improvement is claimed.

The candidate passed `cargo fmt --all --check`,
`cargo test --workspace --all-features --locked`,
`cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`
and `cargo bench --workspace --no-run --locked`. The benchmark harness tests
also passed (10 tests).

The measurement data is archived beside this report:

- Original [broad results](data/broad-summary.csv), [quote results](data/quote-summary.csv), [broad metadata](data/broad-run.json) and [quote metadata](data/quote-run.json).
- Start-hint [broad results](data/map-hint-broad-summary.csv), [container results](data/map-hint-containers-summary.csv), [broad metadata](data/map-hint-broad-run.json) and [container metadata](data/map-hint-containers-run.json).
The paired runner SHA-256 was
`eba0a2038f1c65479a9555705dc923cc8ff6869725832584161c0646529f1c95`, the
worker SHA-256 was
`d0fb282448170d2e43edc79bc7e5bd4e1aaac62f9abab75cd2bfed03de8b1fa7`, and the
corpus SHA-256 was
`f921f7880fc8145dff67c8783b9e2f0577441cd09a20f12dc9331a23f9821ab7`.
