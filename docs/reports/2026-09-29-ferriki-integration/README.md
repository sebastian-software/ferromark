# Ferriki adapter lifecycle: initial local measurement

This is a small integration check, not a throughput comparison or a CDN result.
The benchmark uses one short Rust code fence, a custom in-memory grammar and
theme, and a parsed Ferromark document. It excludes standard catalog loading,
network fetches, persistent cache behavior, and parser time. Escaping and
fallback behavior are covered by `tests/ferriki_integration.rs`.

- Base commit: `7f0772d63b1101e4e580c48d74f1c65a97d47f79`, plus the adapter
  changes in this worktree.
- Ferriki: crates.io `0.4.1`; Rust: `1.95.0`.
- Host: macOS 27.0, arm64. The processor model was unavailable in the sandbox.
- Profile: repository `bench` profile, optimized with fat LTO and one codegen unit.
- Command: `cargo bench -p ferromark --features ferriki --bench ferriki --locked -- --quick`.
- One Criterion quick run; the ranges below are Criterion's reported estimates,
  not a stability claim across hosts or larger workloads.

| Phase | Time |
| --- | ---: |
| Highlighter build and custom registrations | 4.3097–4.3773 µs |
| Build plus first highlighted render | 21.464–21.551 µs |
| Repeated highlighted render with one highlighter and renderer | 7.6111–7.6135 µs |

The second row includes the first row's construction work. Standard assets and
the planned CDN source need separate measurement after Ferriki issue #141 lands;
these numbers do not establish their cost or a general Ferromark speed claim.
