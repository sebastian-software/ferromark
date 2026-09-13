# Working on Ferromark v2

This repository is a local, history-free OX-Content core fork. Read
[UPSTREAM.md](UPSTREAM.md) and [docs/fork.md](docs/fork.md) before widening its scope.

- Use US English for code, documentation, and Conventional Commit messages.
- Keep the workspace focused on Markdown parsing and HTML rendering.
- Preserve upstream attribution and the separately licensed specification fixtures.
- Preserve existing snapshot output and conformance baselines during optimization.
  Any intended semantic change needs its own documented decision and review.
- Port one optimization at a time, with a scalar fallback for SIMD and output
  equality verified before timing. Do not publish unmeasured speed claims.
- Keep all packages unpublished while this is a local development baseline.

Run from the repository root after code changes:

```sh
cargo fmt --all --check
cargo test --workspace --all-features --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
```

Use `cargo bench --workspace --no-run --locked` to validate benchmark builds.
The self-contained Criterion benchmarks are the starting point for measured ports.
