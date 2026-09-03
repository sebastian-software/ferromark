# Contributing to ferromark

## Getting started

```bash
git clone https://github.com/sebastian-software/ferromark.git
cd ferromark
```

The minimum supported Rust version (MSRV) is Rust 1.88.

Run the [required local checks](#required-local-checks) below.

## Required local checks

Run these commands from the repository root before opening a pull request:

```bash
cargo test --locked --all-features
cargo clippy --all-targets --all-features --locked -- -D warnings
cargo fmt --check
```

Changes to the Node workspace or release process have additional package checks;
follow the [releasing guide](docs/releasing.md) for those commands.

## Running benchmarks

Comparison benchmarks need the md4c C sources:

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

Ferromark-only and options-cost benchmarks also work without md4c:

```bash
cargo bench --bench parsing
cargo bench --bench options
```

To focus on large-corpus or fixed-budget pathological parsing cases:

```bash
cargo bench --locked --bench parsing -- commonmark_1m
cargo bench --locked --bench parsing -- pathological
```

The committed synthetic CommonMark fixtures can be regenerated together or by
filename; for example, `python3 scripts/generate_commonmark_50k.py
commonmark-1m.md` refreshes only the 1 MiB corpus.

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
