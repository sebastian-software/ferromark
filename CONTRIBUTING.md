# Contributing to ferromark

## Getting started

```bash
git clone https://github.com/sebastian-software/ferromark.git
cd ferromark
cargo test
```

## Running benchmarks

Comparison benchmarks need the md4c C sources:

```bash
git clone --depth 1 https://github.com/mity/md4c.git ../md4c
cargo bench --bench comparison
```

ferromark-only benchmarks work without md4c:

```bash
cargo bench --bench parsing
```

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
2. Run `cargo test` and `cargo clippy` before submitting
3. Keep PRs focused -- one change per PR
