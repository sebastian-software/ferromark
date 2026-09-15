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

## Rust archive rehearsal

From the repository root, with Rust 1.95 and Python 3.12 or later:

```sh
cargo fetch --locked
python3 scripts/rehearse-rust-packages.py /tmp/ferromark-rust-package-review
```

The output directory must not exist. This creates the five actual Cargo archives,
checks normalized version pins, metadata and the unchanged upstream MIT notice,
and builds/runs an isolated consumer using only the unpacked packages. External
registry versions must remain within the workspace lockfile. Member READMEs and
LICENSE files ship inside every archive.

The inter-crate versions are not yet available on crates.io. Stable Cargo cannot
perform its usual registry verification for that combination, so the initial
archive command uses `--no-verify --exclude-lockfile`. The separate consumer then
validates the archives with local crates.io patches. This is a package-content and
build check; it does not test registry credentials, availability or publication.
Publication order is allocator, AST, parser/renderer, then facade; the parser and
renderer have a development-dependency cycle. The publisher must accommodate that
cycle and perform registry checks at the selected release version.

## CI package assembly

Each of the eight native jobs uploads its verified binary. The dependent
`npm-packages` job assembles all nine npm packages, checks versions and exact
archive contents, and performs a clean installation on Linux x64 GNU. Six native
targets also have their own runtime tests; the two musl targets are built and
inspected, not runtime-tested. The `rust-packages` job runs the isolated Cargo
archive rehearsal. CI retains the verified archives for seven days.

These jobs assemble and test artifacts only. They grant no registry publishing
permissions and do not create releases.

## Before enabling publication

Decide and review the public v2 API and migration guide, the publish order of
all five Rust crates, versioned inter-crate dependencies, and prerelease channels.
The local version rehearsal is implemented; validate the first real release PR
against its selected commit history before merging it.
Re-enable trusted publishing only after package checks and the full eight-target
native matrix pass. Keep upstream MIT attribution in every distributed artifact.

The homepage deployment workflow is restricted to `main`, including manual runs.
The v2 branch builds and verifies the site in CI without replacing the live v1 site.
