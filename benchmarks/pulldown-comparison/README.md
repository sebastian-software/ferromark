# Ferromark / pulldown-cmark parity benchmarks

This isolated harness compares the two leading Rust implementations without
building md4c or comrak. It uses three explicit feature intersections:

| Lane | Shared features | Raw HTML |
| --- | --- | --- |
| `commonmark` | CommonMark only | preserved |
| `gfm_overlap` | CommonMark, tables, strikethrough, task lists | preserved |
| `extended_overlap` | GFM overlap, footnotes, math, superscript, callouts | preserved |

These are benchmark configurations, not Ferromark user profiles. Ferromark's
secure default performs additional URL and raw-HTML safety work and is measured
separately in the main benchmark suite.

Run semantic guardrails before timing:

```bash
cargo test --manifest-path benchmarks/pulldown-comparison/Cargo.toml
```

Run every comparison:

```bash
cargo bench --manifest-path benchmarks/pulldown-comparison/Cargo.toml
```

Filter to one feature intersection:

```bash
cargo bench --manifest-path benchmarks/pulldown-comparison/Cargo.toml -- \
  '^parity/gfm_overlap/50k/'
```

The dependency is pinned to pulldown-cmark 0.13.4. Published runs must also
record the Ferromark commit, Rust/LLVM version, machine, and Criterion settings.

## Comprehensive profiling

This crate also owns the profiling harness so parity options have one source of
truth. It pins Rust 1.93.0 locally and keeps generated results under the ignored
`results/` directory.

List every parser, configuration, and corpus selector:

```bash
cd benchmarks/pulldown-comparison
cargo run --release --bin profile_driver -- --list
```

Run one allocation-counted release diagnostic:

```bash
scripts/run-diagnostic.sh \
  ferromark extended-secure commonmark-50k 10 portable release
```

Use `counters` instead of `release` to include feature-gated block, inline,
event, mark, capacity, and paragraph-copy counters. Counter builds are diagnostic
only and must not be compared to normal release throughput.

Allocation counters cover parser-internal work inside the measured render loop.
Corpus construction, warmup, environment discovery, and JSON serialization run
outside the counter window.

Run the bounded primary matrix:

```bash
scripts/run-primary-matrix.sh 5 portable
```

## Publication baseline

Use the publication runner when a result will decide whether a production
experiment may begin or when a performance claim needs an auditable baseline:

```bash
scripts/run-publication-baseline.sh
```

It refuses a dirty checkout, uses portable non-PGO code generation, alternates
the parity parser order across its three repetitions, and runs every Criterion
lane with 80 samples, a five-second measurement window, and a three-second
warmup. Each run retains its
`estimates.json` files plus an environment probe under a unique ignored
`results/publication-<random>/` directory. A process-wide lock prevents a
second runner in the same harness checkout from sharing Criterion output.

The run covers trusted CommonMark parity at 5, 20, 50, and 250 KB; secure-default
Extended rendering at 5, 20, and 50 KB; and the Essentials, Extended, and Full
profile-cost lanes at 50 KB. Trusted parity and secure-default results answer
different questions and must never be merged into one comparative claim.

Publish the exact result directory as durable CI or release storage before
cleaning the worktree. Commit or link an audit-ready estimate artifact alongside
any publication summary; an ignored local path alone is not evidence another
checkout can inspect. Do not update the README benchmark headline until the
publication protocol has passed.

The CPU mode must be named explicitly:

- `portable`: `target-cpu=generic`, used for portable source comparisons;
- `apple-m1`: the repository's existing Apple Silicon baseline plus NEON;
- `native`: a local hardware ceiling, never a portable claim;
- `pgo`: requires `PGO_PROFDATA` and remains separate from non-PGO results.

Capture a long-running CPU profile with one of the locally available tools:

```bash
scripts/capture-cpu-profile.sh sample extended-secure commonmark-50k 30 portable
scripts/capture-cpu-profile.sh samply extended-secure commonmark-50k 30 portable
scripts/capture-cpu-profile.sh xctrace extended-secure commonmark-50k 30 portable
```

Use AC power and stable machine conditions. Screening runs use at least 80
Criterion samples, a 5-second measurement window, and a 3-second warmup.
Promising or surprising results require three alternating-order repetitions.
