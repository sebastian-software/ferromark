# Releasing ferromark

Release Please owns the shared Rust and npm version. Its release PR updates `Cargo.toml`, `node/native/Cargo.toml`, and `node/ferromark/package.json` together.

When that PR merges, `.github/workflows/release.yml`:

1. creates the GitHub release,
2. builds and tests native packages for x64 and arm64 on macOS, Windows, and glibc Linux,
3. verifies the complete binary matrix and packed npm contents,
4. runs a clean consumer install,
5. publishes `ferromark` to npm through trusted publishing,
6. publishes the matching Rust crate to crates.io.

## npm trusted publisher setup

The npm package must have a GitHub Actions trusted publisher configured with:

- organization or user: `sebastian-software`
- repository: `ferromark`
- workflow: `release.yml`

The publish job requests `id-token: write` and runs `npm publish --provenance`. It does not read or forward an npm token. Keep the workflow filename and npm trusted-publisher settings aligned.

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
