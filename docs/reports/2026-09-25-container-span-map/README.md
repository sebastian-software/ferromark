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

The [per-document broad results](data/broad-summary.csv), [quote diagnostic
results](data/quote-summary.csv), [broad run metadata](data/broad-run.json) and
[quote run metadata](data/quote-run.json) are archived beside this report.
The paired runner SHA-256 was
`eba0a2038f1c65479a9555705dc923cc8ff6869725832584161c0646529f1c95`, the
worker SHA-256 was
`d0fb282448170d2e43edc79bc7e5bd4e1aaac62f9abab75cd2bfed03de8b1fa7`, and the
corpus SHA-256 was
`f921f7880fc8145dff67c8783b9e2f0577441cd09a20f12dc9331a23f9821ab7`.
