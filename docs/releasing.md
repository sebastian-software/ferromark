# Releasing ferromark

Release Please owns the shared Rust and npm version. Its release PR updates
`Cargo.toml`, `node/native/Cargo.toml`, `node/ferromark/package.json`, the native
platform package versions, and their specifiers in `node/pnpm-lock.yaml` together.
The lockfile updater changes the eight local dependency specifiers while retaining
their workspace links. CI checks this contract with
`ruby scripts/test-release-version-sync.rb --self-test`.

When that PR merges, `.github/workflows/publish.yml`:

1. creates the GitHub release,
2. builds and tests native packages for x64 and arm64 on macOS, Windows, and Linux,
3. verifies the complete binary matrix and packed npm contents,
4. runs a clean consumer install,
5. publishes `ferromark` to npm through trusted publishing,
6. publishes the matching Rust crate to crates.io.

GNU Linux binaries use napi-rs's pinned cross-toolchain so their native glibc
symbol floor stays at 2.17 instead of following the current GitHub runner. CI
checks each GNU artifact with `readelf`; musl binaries use cargo-zigbuild.

## npm trusted publisher setup

The npm package must have a GitHub Actions trusted publisher configured with:

- organization or user: `sebastian-software`
- repository: `ferromark`
- workflow: `publish.yml`

The publish job requests `id-token: write` and runs `npm publish --provenance`. It does not read or forward an npm token. Keep the workflow filename and npm trusted-publisher settings aligned.

## crates.io trusted publisher setup

The `ferromark` crate must have a GitHub Actions trusted publisher configured with:

- organization or user: `sebastian-software`
- repository: `ferromark`
- workflow: `publish.yml`
- environment: leave unset (the crate job does not use a GitHub environment)

The `publish-crate` job requests `id-token: write` and exchanges its GitHub OIDC
identity for a temporary crates.io token using the pinned
`rust-lang/crates-io-auth-action`. Only `cargo publish --locked` receives that
token; the action revokes it in its post-job cleanup. The workflow does not use
the repository's `CARGO_REGISTRY_TOKEN` secret.

After an interrupted release, run the Release workflow manually on `main` to
retry crate publication. Manual recovery uses the same trusted publisher and
skips Release Please and npm publication. Check that the version in `Cargo.toml`
is the intended unpublished version before starting recovery. A real OIDC token
exchange requires GitHub Actions; local dry runs cannot validate publisher setup.

## Local package checks

From `node/`:

```sh
pnpm install --frozen-lockfile
pnpm audit --audit-level high
pnpm build
pnpm test
pnpm typecheck
pnpm lint
pnpm pack:check
pnpm smoke:clean
```

`pack:check` rejects unexpected files. `smoke:clean` installs the generated tarball into a temporary project and imports it as a consumer would.
