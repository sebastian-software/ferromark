# Contributing to ferromark

## Getting started

```bash
git clone https://github.com/sebastian-software/ferromark.git
cd ferromark
```

The minimum supported Rust version (MSRV) is Rust 1.94.

`rust-toolchain.toml` tracks stable, so rustup selects and installs the current
stable toolchain in this checkout. CI keeps the MSRV honest with a dedicated
matrix row that overrides the file.

Run the [required local checks](#required-local-checks) below.

## Required local checks

Run these commands from the repository root before opening a pull request:

```bash
cargo test --locked --all-features
cargo clippy --all-targets --all-features --locked -- -D warnings
cargo test -p ferro-byte-search --locked
cargo clippy -p ferro-byte-search --all-targets --locked -- -D warnings
cargo fmt --check
```

Changes to the Node workspace or release process have additional package checks;
follow the [releasing guide](docs/releasing.md) for those commands. Changes to a
README also run the generated-block check in
[The Ferramenta family block](#the-ferramenta-family-block).

## Node.js workspace

The `node/` directory is a pnpm workspace that holds two kinds of package:

- `node/package.json` (`ferromark-workspace`) is private. It is the development
  root and owns the build, test, lint, and packaging scripts.
- `node/ferromark/` is the published `ferromark` npm package. Its eight platform
  packages live in `node/ferromark/npm/*`, one per target triple, and ship the
  binary built from the N-API crate in `node/native/`.

The minimum supported Node.js version is 22.12.0. It is declared once, by the
published package, and the `node-floor` CI job runs the package's tests against
exactly that version. The private workspace root requires Node.js 22.13.0
because the development toolchain does, so contributors need 22.13.0 or newer
while consumers only need 22.12.0.

Workspace commands run from `node/`:

```bash
pnpm install --frozen-lockfile
pnpm build
pnpm test
```

## Running benchmarks

The current README/homepage tables come from the
[five-parser native harness](benchmarks/bun-comparison/README.md), including Bun.
It verifies output and repeats the public cases before generating figures:

```bash
python3 benchmarks/bun-comparison/publish.py --check
python3 -m unittest discover -s benchmarks/bun-comparison -p 'test_*.py'
```

Follow that harness's setup instructions for pinned Bun/md4c sources and the
recorded lockfile. Ordinary library builds do not acquire either dependency.

The [native cmark pair harness](benchmarks/cmark-comparison/README.md) adds
cmark and cmark-gfm through their C APIs. It reuses the workload-review policy
and realistic table fixtures, but measures a separate system-allocator
environment. Its results do not replace or mix with the Bun/mimalloc tables.
See the [expansion plan](docs/plans/2026-09-11-native-comparison-expansion.md)
for the subsequent Goldmark and Sätteri adapters.

The older four-parser Criterion suite remains a diagnostic with historical
adapter policies/lifecycles; its results do not feed the current public tables.
It needs the md4c C sources:

```bash
git clone https://github.com/mity/md4c.git ../md4c
git -C ../md4c checkout --detach 65c6c9d
MD4C_DIR=../md4c cargo bench --locked \
  --manifest-path benchmarks/md4c-comparison/Cargo.toml --bench comparison
```

The benchmark build verifies the md4c revision before compiling its C sources.

The focused ferromark/pulldown-cmark parity harness does not need md4c:

```bash
cargo test --manifest-path benchmarks/pulldown-comparison/Cargo.toml
cargo bench --manifest-path benchmarks/pulldown-comparison/Cargo.toml
```

The table comparison uses plain cells, mixed plain/emphasis/strong cells, and a
Markdown link column, with identical options: CommonMark plus tables. CommonMark
inline parsing is always active. The fixtures live in
`benches/fixtures/tables-plain.md`, `tables-commonmark-inline.md`, and
`tables-links.md`; semantic tests check plain text, rendered emphasis, and links. Strikethrough
is outside this table comparison. Run just these cases:

```bash
cargo bench --locked --manifest-path benchmarks/pulldown-comparison/Cargo.toml --bench comparison -- 'parity/tables-'
cargo run --release --locked --manifest-path benchmarks/pulldown-comparison/Cargo.toml --bin profile_driver -- --parser ferromark --config tables-only --corpus tables-plain --seconds 5
```

Ferromark-only and options-cost benchmarks also work without md4c:

```bash
cargo bench --bench parsing
cargo bench --bench options
```

For pull requests, CI builds and measures the PR base and proposed changes on
the same runner with the same toolchain and sampling settings. The comparison
fails if any existing case takes more than 20% longer or is missing from the
new results. Historical main-branch measurements remain available for trends;
they do not gate PRs because different hosted runners can have different speeds.
The benchmark job summary includes both revisions and all measured ratios.

To focus on large-corpus or fixed-budget pathological parsing cases:

```bash
cargo bench --locked --bench parsing -- commonmark_1m
cargo bench --locked --bench parsing -- pathological
```

The committed synthetic CommonMark fixtures can be regenerated together or by
filename; for example, `python3 scripts/generate_commonmark_50k.py
commonmark-1m.md` refreshes only the 1 MiB corpus.

## Repository contracts

`scripts/` holds executable contracts that the CI `fmt` job runs. They assert
that this document, the README, the workflows, the release configuration, the
profiling scripts, and the Node option documentation still describe what the
code actually does. They are Node test files with a single dependency, so
install it once and then run the contracts that cover what you touched:

```bash
(cd scripts && pnpm install --frozen-lockfile)
node --test ./scripts/test-readme-structure-contract.mjs
python3 -m unittest discover -s scripts -p 'test_ci_benchmarks.py'
./scripts/check-workflow-pins.sh
```

`.github/workflows/ci.yml` lists the full set; each one is a gate. When a
contract legitimately changes, update it in the same pull request as the change
it describes.

## The Ferramenta family block

The `## The Ferramenta family` section of `README.md` and the closing two lines
of `node/ferromark/README.md` are generated from the registry in
[sebastian-software/ferramenta](https://github.com/sebastian-software/ferramenta),
which owns the family's tool names, jobs, groups, and links. Never hand-edit the
text between the `<!-- ferramenta-family:start -->` and
`<!-- ferramenta-family:end -->` markers; regenerate it instead:

```bash
node ./scripts/check-readme-family.mjs           # fails on drift, as CI does
node ./scripts/check-readme-family.mjs --write   # regenerate both READMEs
```

The script runs the generator through `pnpm dlx`, so it needs pnpm, Node.js
22.13 or newer, and network access. It pins the generator to one commit in
`FAMILY_GENERATOR_REVISION`, the only place this repository names a generator
revision. When the registry changes, bump `FAMILY_GENERATOR_REVISION` to the new
ferramenta commit, run the script with `--write`, and commit the regenerated
blocks together with the pin. The company footer below the block belongs to
`@sebastian-software/standards` and is not touched by the generator.

## Coverage and dependency policy

CI measures coverage with `cargo llvm-cov` across all features and fails below a
floor of 90% line coverage. The floor itself lives in the `COVERAGE_FLOOR`
environment variable of the coverage job in `.github/workflows/ci.yml`, and
nowhere else; the job writes `Line coverage: X% (gate: ≥ 90%)` into the run
summary before the gate can fail it, so both numbers are visible on a red run.
`scripts/test-ci-hardening.mjs` checks that this document and the README badge
state the same number, so raise them together. The same gate runs locally:

```bash
cargo llvm-cov --all-features --locked --fail-under-lines 90
```

Dependencies follow `deny.toml`: a permissive license allow-list, crates.io as
the only registry, and `yanked = "deny"`. Duplicate versions are reported but do
not fail the build. Run the policy locally before adding or bumping a
dependency:

```bash
cargo deny check
```

`rustsec/audit-check` keeps reporting advisories as its own check run.

## Commit messages

This project uses [Conventional Commits](https://www.conventionalcommits.org/) for automated changelog generation via Release Please.

- `feat:` new features (minor version bump)
- `fix:` bug fixes (patch version bump)
- `docs:` documentation changes
- `perf:` performance improvements
- `refactor:` code changes that neither fix bugs nor add features
- `test:` adding or updating tests
- `chore:` maintenance tasks

Breaking changes: add `!` after the type (e.g., `feat!:`) or include `BREAKING CHANGE:` in the commit body.

## Pull requests

1. Fork the repo and create a branch from `main`
2. Run the required local checks before submitting
3. Keep PRs focused -- one change per PR
