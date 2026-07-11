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
