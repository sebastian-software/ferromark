# Working on Ferromark v2

This branch develops Ferromark v2 in the Ferromark repository, based on OX-Content. Read
[UPSTREAM.md](UPSTREAM.md) and [docs/fork.md](docs/fork.md) before widening its scope.

- Use US English for code, documentation, and Conventional Commit messages.
- Keep the core focused on Markdown parsing and HTML rendering; Node bindings and the documentation site live in `node/` and `homepage/`.
- Preserve upstream attribution and the separately licensed specification fixtures.
- Preserve existing snapshot output and conformance baselines during optimization.
  Any intended semantic change needs its own documented decision and review.
- Port one optimization at a time, with a scalar fallback for SIMD and output
  equality verified before timing. Do not publish unmeasured speed claims.
- Publish only reviewed release versions through the verified workflow in [docs/releasing.md](docs/releasing.md). Keep the Node build workspace and native binding crate private.

Run from the repository root after code changes:

```sh
cargo fmt --all --check
cargo test --workspace --all-features --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
```

Use `cargo bench --workspace --no-run --locked` to validate benchmark builds.
The self-contained Criterion benchmarks are the starting point for measured ports.

See [CONTRIBUTING.md](CONTRIBUTING.md) for Node, website, and infrastructure checks.
Durable integration decisions live in [docs/arch/](docs/arch/); retained v2 feature decisions live in [docs/decisions/](docs/decisions/).

<!-- sebastian-software-consumer-agents:start -->

# Standards-managed repo guardrails

- Do not hand-edit managed files or standards-owned marker sections.
- If `standards check` reports drift, run `standards apply` or update standards.
- The repository's own gate may omit `standards check`; CI can still fail on it.

Node repositories:

- Fix or format every file reported by `oxfmt` whenever practical.
- For generated files, prefer formatting in the generator step.
- If formatting is not viable, use repo-local `.prettierignore`.
- Never add repo-specific ignores to managed `.oxfmtrc.json`.

Rust repositories:

- Keep `cargo fmt --all --check` and
  `cargo clippy --workspace --all-targets --all-features -- -D warnings` green.
- Lint levels belong in `[workspace.lints]`, never in managed `rustfmt.toml`.
- `rust-version` in `Cargo.toml` is the only MSRV; every other mention is a
  derived copy.
- Record a cargo-deny finding as a narrow, commented exception in `deny.toml` —
  never by widening the org allow-list.

<!-- sebastian-software-consumer-agents:end -->
