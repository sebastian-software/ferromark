# Contributing to ferromark v2

Clone `sebastian-software/ferromark` and check out `main`. The branch combines
the original Ferromark history with the separately developed v2 core. Read
[the integration decision](docs/arch/ADR-0015-v2-repository-integration.md) and
[upstream provenance](UPSTREAM.md) before changing the architecture.

The MSRV is derived from `workspace.package.rust-version` in `Cargo.toml`.
`rust-toolchain.toml` selects that toolchain with Clippy and rustfmt. CI tests
both the MSRV and stable on Linux, macOS, and Windows.

## Required local checks

```sh
cargo test --locked --all-features
cargo clippy --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-features --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo fmt --all --check
cargo bench --workspace --no-run --locked
```

`ferromark` is the repository root package: its `src/`, `tests/`, `benches/`
and `examples/` sit at the top level, and `node/native` is the only workspace
member. A Cargo `include` list decides what the published archive contains, so
compare `cargo package --list` whenever you change it.

Releases are automatic. Every push to `main` updates one release pull request,
and **merging that pull request tags the version and publishes both
registries**, so a change that lands on `main` is a change you are willing to
ship. See [releasing](docs/releasing.md) and
[ADR-0020](docs/arch/ADR-0020-standards-release-blueprint.md).

Node and release changes also require the [package checks](docs/releasing.md#local-package-checks).
Those checks build a plain addon; reproducing the published, profile-guided one
needs the `llvm-tools` rustup component and `FERROMARK_PGO=1`, as described in
[ADR-0019](docs/arch/ADR-0019-profile-guided-native-addon.md).
Website changes require `pnpm install --frozen-lockfile`, `pnpm format:check`,
`pnpm lint`, `pnpm typecheck`, `pnpm run audit`, and `pnpm build` from
`homepage/`. The build checks all 27 prerendered routes, navigation, and v2 content.

The Node and homepage workspaces use the managed Oxfmt configuration. Run
`pnpm format` in the affected workspace before checking formatting. Their seeded
ESLint, Oxlint, and spelling configurations may carry documented local adjustments;
do not change the managed `.oxfmtrc.json`. Homepage's generated Ardo route table
and frozen benchmark guide are excluded through `.prettierignore`; build output
and generated route types are also excluded from linting. The build still checks
the generated navigation and benchmark content.

The Node workspace keeps TypeScript 7 for `tsc` through the `@typescript/native`
alias. The `typescript` alias supplies the TypeScript 6 API required by
`typescript-eslint`, following the [upstream compatibility guidance](https://devblogs.microsoft.com/typescript/announcing-typescript-7-0/#running-side-by-side-with-typescript-6-0).
Both Oxlint and ESLint run in the Node and homepage lint gates.

The NAPI-generated `node/ferromark/native.d.ts` is also excluded from formatting
and linting; native builds regenerate it and TypeScript checks the package types.

## Repository contracts

```sh
(cd scripts && pnpm install --frozen-lockfile && pnpm format:check)
node --test scripts/test-*.mjs
python3 -m unittest discover -s scripts -p 'test_*.py'
```

All `uses:` entries must be full commit SHAs with the version in a trailing
comment. The CI `fmt` job enforces that with the organization's shared
[`check-action-pins`](https://github.com/sebastian-software/standards/tree/main/.github/actions)
action rather than a repository-local copy. Use US English and Conventional
Commits; breaking changes use `!` or a `BREAKING CHANGE:` footer. Managed files
come from standards; run the pinned standards CLI from CI to check or apply them.

## The Ferramenta family block

Edit `README.md.src`, then run `mise run readme:write` and `mise run readme:check`.
The CLI and theme revisions are pinned in `mise.toml`, `mise.lock`, and
`mdtheme.yaml`. See [README themes](docs/readme-theme.md).
The npm README's compact family block is generated separately:

```sh
node ./scripts/check-readme-family.mjs --write
node ./scripts/check-readme-family.mjs
```

## Benchmarks and conformance

Preserve archived measurements and specification fixtures byte for byte.
Do not edit benchmark numbers by hand. Each directory under `benchmarks/`
documents its own measured workload, build settings, and source preparation.
The imported reports retain their original paths and commit references.
The old v1 Criterion comparison gate cannot compare the new suite names;
CI validates all v2 benchmark builds and Python harness tests instead.

## Coverage and dependency policy

CI retains the 90% Rust line coverage gate. It excludes the N-API library,
which is exercised by the Node tests. Run `cargo llvm-cov --workspace --exclude
ferromark-node --all-features --locked --fail-under-lines 90 -- --test-threads=1` for the same gate.

CI uploads the generated `lcov.info` as the `rust-coverage` artifact for 30 days
and links it from the coverage job summary. The upload runs before the coverage
floor check, so the report remains available when coverage falls below 90%; the
summary link is omitted when report generation or upload does not produce an
artifact.

`cargo deny check` and RustSec retain the repository's dependency checks.

Coverage work must improve checked behavior, with a documented reason for any
exclusion. See the [coverage audit](docs/reports/2026-09-15-coverage/README.md)
for measured scope and remaining gaps, and the
[test-quality decision](docs/decisions/2026-09-15-test-quality.md) for oracle and
coverage acceptance rules.
