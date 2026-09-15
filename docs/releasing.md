# Releasing ferromark v2

V2 is an unpublished development branch. All Rust crates inherit `publish = false`;
the npm package and its eight native platform packages set `private: true`.
The v1 release workflow is retained in Git history, not enabled on this branch.
Release Please configuration retains the Rust/npm version mapping for a later
release decision; it does not publish anything by itself.

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

The Node workspace requires 22.13.0; the package supports 22.12.0. CI builds
on the workspace floor and tests the resulting binary on the consumer floor.
`release-node` inherits the optimized release profile with `panic = "unwind"`.
The panic test verifies that Rust panics become JavaScript exceptions.

## Before enabling publication

Decide and review the public v2 API and migration guide, the publish order of
all five Rust crates, versioned inter-crate dependencies, and prerelease channels.
Adapt Release Please to the virtual workspace and validate a complete release PR.
Re-enable trusted publishing only after package checks and the full eight-target
native matrix pass. Keep upstream MIT attribution in every distributed artifact.

The homepage deployment workflow is restricted to `main`, including manual runs.
The v2 branch builds and verifies the site in CI without replacing the live v1 site.
