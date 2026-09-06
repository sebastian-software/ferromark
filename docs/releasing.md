# Releasing ferromark

Release Please owns the shared Rust and npm version. Its release PR updates
`Cargo.toml`, `node/native/Cargo.toml`, `node/ferromark/package.json`, the native
platform package versions, and their specifiers in `node/pnpm-lock.yaml` together.
The lockfile updater changes the eight local dependency specifiers while retaining
their workspace links. CI checks this contract with
`node --test scripts/test-release-version-sync.mjs`.

## Why the npm versions stay explicit

The org release-please template prefers `workspace:` references between local
packages "when the package manager's pack and publish behavior fits the
repository". Here it does not. The release publishes the main package with
`npm publish --access public --provenance`, the platform packages with
`npm publish` (`node/scripts/publish-platform-packages.mjs`), and inspects both
with `npm pack` (`node/scripts/verify-pack.mjs`). npm only resolves the
`workspace:` protocol inside an npm workspace, and `node/` is a pnpm workspace,
so `npm pack` copies a `workspace:*` specifier verbatim into the published
`package.json` and ships an uninstallable release. Three checks in this
repository also require the literal version: `verify-package.mjs`,
`verify-release.mjs`, and `publish-platform-packages.mjs`.

The eight `optionalDependencies` and the eight `node/pnpm-lock.yaml` specifiers
therefore stay explicit versions maintained by typed release-please
`extra-files`, and `scripts/test-release-version-sync.mjs` proves that every one
of those fields moves together. CI additionally runs the template's lockfile
no-diff check (`pnpm install --lockfile-only && git diff --exit-code`) in the
`node` job, so a manifest change can no longer outrun the lockfile.

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
