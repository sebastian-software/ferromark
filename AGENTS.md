# AGENTS.md

Guidance for coding agents working in the ferromark repository. Humans welcome
too.

## Preflight

Run the [required local checks](CONTRIBUTING.md#required-local-checks) from the
repository root before opening a pull request:

```bash
cargo test --locked --all-features
cargo clippy --all-targets --all-features --locked -- -D warnings
cargo fmt --check
```

Node workspace (`node/`) or release changes need the extra package checks in
[docs/releasing.md](docs/releasing.md). The workspace requires Node.js 22.13.0
(`node/package.json`) while the published package supports 22.12.0
(`node/ferromark/package.json`); the difference is deliberate, and the
`node-floor` CI job builds on the workspace version before running the package
tests on the published floor.

## Contract scripts

The `fmt` job of [.github/workflows/ci.yml](.github/workflows/ci.yml) runs
executable contracts next to `cargo fmt --check`. They run on Node with
`node --test` and need the contract dependencies installed once
(`pnpm install --frozen-lockfile` in `scripts/`). Run the ones covering what you
touched, for example:

```bash
node --test ./scripts/test-readme-structure-contract.mjs
node --test ./scripts/test-contributing-ci-contract.mjs
./scripts/check-workflow-pins.sh
```

The workflow lists the full set; every `scripts/test-*.mjs` invoked there is a
gate, not a suggestion.

## Rules

- Project language is US English: code, comments, commit messages, docs, and
  test names.
- Conventional Commits without exception; Release Please derives versions and
  the changelog from them. Breaking changes need `!` or a `BREAKING CHANGE:`
  footer.
- The README has a structure contract
  (`scripts/test-readme-structure-contract.mjs`): heading uniqueness, the CLI
  section preceding Markdown configuration, the migration-guide link, the
  benchmark disclosures, and the project-structure listing are all asserted.
  When the README legitimately changes, update the contract in the same change.
- Do not edit benchmark numbers by hand. Published figures come from a measured
  run, and the contracts cross-check locked comparison versions plus the pinned
  md4c revision against README and CONTRIBUTING.
- Workflow `uses:` entries must be full 40-character commit SHAs;
  `scripts/check-workflow-pins.sh` enforces it. Keep the trailing `# vN` comment
  for readability.
- Durable decisions live in [docs/arch/](docs/arch) — read the ADRs before
  changing direction.

---

<!-- sebastian-software-consumer-agents:start -->

# Standards-managed repo guardrails

- Do not hand-edit managed files or standards-owned marker sections.
- If `standards check` reports drift, run `standards apply` or update standards.
- `pnpm agent:check` may omit `standards check`; CI can still fail on drift.
- Fix or format every file reported by `oxfmt` whenever practical.
- For generated files, prefer formatting in the generator step.
- If formatting is not viable, use repo-local `.prettierignore`.
- Never add repo-specific ignores to managed `.oxfmtrc.json`.
<!-- sebastian-software-consumer-agents:end -->
