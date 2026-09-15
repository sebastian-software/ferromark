# Releasing ferromark v2

V2 is an unpublished development branch. All Rust crates inherit `publish = false`;
the npm package and its eight native platform packages set `private: true`.
The v1 release workflow is retained in Git history, not enabled on this branch.
Release Please uses a coordinated workspace/npm version mapping; it does not
publish anything by itself. The structural reference optimization is deferred to
[issue #320](https://github.com/sebastian-software/ferromark/issues/320) and is not
a v2.0 release blocker.

## Version rehearsal

Install the pinned contract dependencies with `pnpm install --frozen-lockfile` in
`scripts/`, then run from the repository root:

```sh
node --test scripts/test-release-rehearsal.mjs
node scripts/rehearse-release.mjs /tmp/ferromark-release-review
```

The output directory must not exist. Review the generated RC1, RC2, stable and
patch release files plus PR text. These use the real Release Please updater on
local files and synthetic commits, with no network or publication. Cargo.toml,
Cargo.lock, the Release Please manifest, version.txt, all npm versions and native
dependency pins must agree. External dependencies and publication flags stay
unchanged. See [ADR-0016](arch/ADR-0016-coordinated-workspace-releases.md).

For the real RC and stable transitions, use an explicit `Release-As` commit footer
for the selected version. The pending publisher must route prereleases to npm's
`next` tag and mark their GitHub releases as prereleases. Stable releases use
`latest`. Do not enable automatic publication until that routing is verified.

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
The local version rehearsal is implemented; validate the first real release PR
against its selected commit history before merging it.
Re-enable trusted publishing only after package checks and the full eight-target
native matrix pass. Keep upstream MIT attribution in every distributed artifact.

The homepage deployment workflow is restricted to `main`, including manual runs.
The v2 branch builds and verifies the site in CI without replacing the live v1 site.
