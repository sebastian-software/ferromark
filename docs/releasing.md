# Releasing ferromark v2

One Rust crate, `ferromark`, publishes to crates.io. The npm facade and eight
native platform packages share its version. The Node development workspace and
`ferromark-node` binding crate remain private. Allocator, AST, parser and renderer
are modules inside the Rust library; consumers can use them individually through
`ferromark`. See [ADR-0018](arch/ADR-0018-single-rust-crate.md).

The structural reference optimization is deferred to
[issue #320](https://github.com/sebastian-software/ferromark/issues/320) and is not
a v2.0 release blocker.

## Prepare the version

Release Please uses the coordinated mapping in `release-please-config.json`;
there is currently no automatic release PR or publishing trigger. Use an explicit
`Release-As: 2.0.0-rc.1` footer when preparing a candidate with its updater.
Commit the reviewed version changes and authored notes under
`docs/releases/<version>.md`. Never publish the rehearsal's synthetic changelog.

Install the pinned contract dependencies in `scripts/`, then run from the root:

```sh
node --test scripts/test-release-rehearsal.mjs scripts/test-release-channel.mjs scripts/test-publish-packages.mjs
node scripts/rehearse-release.mjs /tmp/ferromark-release-review
```

The output directory must not exist. This seeds an in-memory development version
and checks RC1, RC2, stable and patch transitions with the real Release Please
updater. Cargo.toml, Cargo.lock, version.txt, the Release Please manifest, all npm
versions and native dependency pins must agree. External dependencies and
publication flags stay unchanged. See [ADR-0016](arch/ADR-0016-coordinated-workspace-releases.md).

Only `X.Y.Z-rc.N` and stable `X.Y.Z` are publishable. Candidates use npm's `next`
tag and a GitHub prerelease without replacing `latest`. Stable releases use
`latest`. The publisher tests enforce this distinction.

## Local package checks

Run the Rust checks in [CONTRIBUTING.md](../CONTRIBUTING.md), then from `node/`:

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
on the workspace floor and tests the binary on the consumer floor.
`release-node` inherits the optimized release profile with `panic = "unwind"`.
The panic test verifies that Rust panics become JavaScript exceptions.

`pnpm build` produces a plain addon. To reproduce the published build locally,
install the `llvm-tools` component for the pinned toolchain
(`rustup component add llvm-tools`) and run from `node/`:

```sh
FERROMARK_PGO=1 pnpm build:native
```

That builds an instrumented training binary, trains it on the frozen benchmark
corpus, merges the counters with `llvm-profdata` and rebuilds the addon with the
profile; the work lives under `target/pgo/`. Without `llvm-tools` the build
fails rather than quietly producing an unoptimized addon. See
[ADR-0019](arch/ADR-0019-profile-guided-native-addon.md).

## Rust archive rehearsal

From the repository root, with Rust 1.95 and Python 3.12 or later:

```sh
cargo fetch --locked
python3 scripts/rehearse-rust-packages.py /tmp/ferromark-rust-package-review
```

The output directory must not exist. `cargo package --locked` builds and verifies
the actual `ferromark` archive. There are no internal crate dependencies or
repository-only cross-crate test dependencies to resolve.

The rehearsal checks package metadata, upstream MIT notices and the absence of
internal path dependencies, then builds/runs an isolated consumer from the
unpacked archive. External versions must remain within the workspace lockfile.
The README and LICENSE ship with the package. This does not test registry credentials.

## CI package assembly

Each of eight native jobs uploads its verified binary. The dependent
`npm-packages` job assembles all nine npm packages, checks their contents and
performs a clean installation on Linux x64 GNU. Six native targets have runtime
tests; the two musl targets are built and inspected. The `rust-packages` job runs
the Cargo archive rehearsal. CI retains the verified archives for seven days.
These jobs do not publish. Publication requires a successful **push CI run on
main at the exact release commit**, including every gate.

Seven native jobs set `FERROMARK_PGO=1`, so each published addon is built from
a profile collected on its own runner. Five same-architecture targets receive
that profile; the two cross-compiled musl targets do not, because a Cargo unit
hash covers the target triple, and the Windows ARM64 job builds without PGO
because the pinned toolchain's `llvm-profdata` rejects the counters written on
that runner. The crates.io crate is unaffected. Profile-guided
binaries depend on counts taken at build time and are therefore no longer
byte-identical between runs of the same commit, so a publish retry must reuse
the original `ci_run_id`. See
[ADR-0019](arch/ADR-0019-profile-guided-native-addon.md).

## Registry authorization

Reuse the existing Trusted Publishing configuration for `ferromark`, repository
`sebastian-software/ferromark`, workflow `publish.yml`. No new Rust crate names,
bootstrap token or initial manual publication are required by the consolidation.
The nine existing npm packages keep their Trusted Publishing configuration.

## Publish the reviewed release

Merge the release changes, wait for all main CI gates, and select its run ID.
The workflow is manual so merging source alone never publishes a package:

```sh
gh workflow run publish.yml --ref main -f version=2.0.0-rc.1 -f ci_run_id=SUCCESSFUL_MAIN_CI_RUN_ID
```

The workflow rejects a different commit, branch, workflow, version or failed run.
It downloads that run's npm and Rust archives and validates all npm manifests
and any existing versions before the first upload. It publishes the Rust
crate with Cargo verification, then the eight native npm
packages before the facade. npm uses Trusted Publishing with provenance.

A retry accepts an existing npm version only when the tarball integrity matches;
the existing Rust crate must have the exact clean source commit. A conflicting
immutable version requires investigation and usually a new RC version.
Registry checks confirm all npm versions and tags and preserve the previous
stable tags for an RC. A fresh consumer installs npm from the registry and a
separate Cargo consumer compiles without local patches. Only after both work
does the workflow create the GitHub release and attach all ten archives.

The homepage deploys from main. Its deployment is independent of registry
publication; the candidate documentation must state the selected prerelease.
