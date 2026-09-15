# Releasing ferromark v2

All five Rust crates share one version and publish to crates.io. The npm facade
and eight native platform packages share that version. The Node development
workspace and `ferromark-node` binding crate remain private: npm distributes the
compiled binding, while Rust consumers compile the four core dependencies.
See [ADR-0017](arch/ADR-0017-verified-release-candidates.md).

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

## Rust archive rehearsal

From the repository root, with Rust 1.95 and Python 3.12 or later:

```sh
cargo fetch --locked
python3 scripts/rehearse-rust-packages.py /tmp/ferromark-rust-package-review
```

The output directory must not exist. Cargo's multi-package
`publish --dry-run` stages the new workspace dependencies and builds all five
packages without uploading them. Parser/renderer cross-dependencies used only
by repository tests and benchmarks are path-only dev dependencies and are
omitted from published metadata. Production dependencies retain exact pins.

The rehearsal also checks archive metadata, version pins and upstream MIT
notices, then builds/runs an isolated consumer from the unpacked packages.
External versions must remain within the workspace lockfile. Member READMEs and
LICENSE files ship in every archive. This does not test registry credentials.

## CI package assembly

Each of eight native jobs uploads its verified binary. The dependent
`npm-packages` job assembles all nine npm packages, checks their contents and
performs a clean installation on Linux x64 GNU. Six native targets have runtime
tests; the two musl targets are built and inspected. The `rust-packages` job runs
the Cargo archive rehearsal. CI retains the verified archives for seven days.
These jobs do not publish. Publication requires a successful **push CI run on
main at the exact release commit**, including every gate.

## First publication of a new crate

Keep the existing organization authorization. Trusted Publishing must additionally
be configured for each new crate after its first publication; it cannot bootstrap
an unregistered name. See the [crates.io documentation](https://crates.io/docs/trusted-publishing).

For the first v2 release, an authorized maintainer can publish the four new crates
from a clean checkout of the reviewed main commit, using their local crates.io
credentials:

```sh
cargo publish --locked -p ferromark_allocator -p ferromark_ast -p ferromark_parser -p ferromark_renderer
```

Configure their ownership and Trusted Publishing for
`sebastian-software/ferromark`, workflow `publish.yml`, matching the existing
`ferromark` configuration. The workflow then verifies the already-published
crates' source commit and publishes the facade through the existing authorization.
Do not publish from a dirty or different checkout: retry validation rejects it.

Alternatively, an authorized maintainer may temporarily provide
`CRATES_IO_BOOTSTRAP_TOKEN` through GitHub Actions secrets, with rights to create
and publish the five crate names. The workflow can perform that initial publish;
remove the token after configuring Trusted Publishing. Never commit or paste
credentials into release notes, issues or chat. No new token is needed when the
crates already exist and their Trusted Publishing configuration is complete.

## Publish the reviewed release

Merge the release changes, wait for all main CI gates, and select its run ID.
The workflow is manual so merging source alone never publishes a package:

```sh
gh workflow run publish.yml --ref main -f version=2.0.0-rc.1 -f ci_run_id=SUCCESSFUL_MAIN_CI_RUN_ID
```

The workflow rejects a different commit, branch, workflow, version or failed run.
It downloads that run's npm and Rust archives and validates all npm manifests
and any existing versions before the first upload. It publishes missing Rust
crates in dependency order with Cargo verification, then the eight native npm
packages before the facade. npm uses Trusted Publishing with provenance.

A retry accepts an existing npm version only when the tarball integrity matches;
existing Rust crates must have the exact clean source commit. A conflicting
immutable version requires investigation and usually a new RC version.
Registry checks confirm all npm versions and tags and preserve the previous
stable tags for an RC. A fresh consumer installs npm from the registry and a
separate Cargo consumer compiles without local patches. Only after both work
does the workflow create the GitHub release and attach all fourteen archives.

The homepage deploys from main. Its deployment is independent of registry
publication; the candidate documentation must state the selected prerelease.
