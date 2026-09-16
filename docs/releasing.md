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

Every push to `main` runs `.github/workflows/release-please.yml`, which opens or
updates one coordinated release pull request from the commits since the last
release tag. It applies the mapping in `release-please-config.json`: Cargo.toml,
Cargo.lock, version.txt, `.release-please-manifest.json`, the npm facade and its
eight native manifests, the exact optional dependency pins, the pnpm workspace
specifiers, the version-bearing README blocks (marked with
`x-release-please-start-version` / `end-version` comments in `README.md.src`,
the generated `README.md` and `node/ferromark/README.md`), and a new
CHANGELOG.md section. It creates no tag and publishes
nothing; the action runs with `skip-github-release`, so only `publish.yml` below
tags a release. Merging source therefore still never publishes a package.

The release pull request must be opened with the `RELEASE_PLEASE_TOKEN`
organization secret. A pull request opened with `GITHUB_TOKEN` starts no further
workflow runs, so `ci.yml` would never run on it and the pull request would
show no checks at all.

Version selection uses the `prerelease` strategy pinned to `rc`. Inside the
candidate series the proposal is automatic: any commit range on top of
`2.0.0-rc.1` proposes `2.0.0-rc.2`, including a breaking `feat!` or a
`BREAKING CHANGE:` footer, which never promotes a candidate out of its series.
Leaving the series is deliberate, because the strategy never proposes a bare
stable version on its own. Land the transition with an explicit footer —
`Release-As: 2.0.0` for the stable release, `Release-As: 2.0.1` for the first
patch after it — as [ADR-0016](arch/ADR-0016-coordinated-workspace-releases.md)
prescribes. Without that footer a stable `2.0.0` would propose `2.0.1-rc`.

The CHANGELOG.md section that the release pull request adds is the release
text: `scripts/finish-github-release.py` uses exactly that section as the GitHub
release body, and `publish.yml` refuses to publish a version whose section is
missing. Review and, where useful, edit that section in the release pull request
before merging — it lists the conventional commits in the range and is prepended
above the authored `## 2.0.0-rc.1` entry already in Git. `docs/releases/` keeps
the first candidate's hand-written notes as history; no file is added there for
later versions. Never publish the rehearsal's synthetic changelog.

After the merge, the release commit is an ordinary `main` push: wait for its CI
run and publish it with `gh workflow run publish.yml` as described below. Once
`publish.yml` has created the `v<version>` tag, the next push to `main` starts
the next cycle from that tag.

Because `skip-github-release` suppresses the library's own tagging step, it also
never retires the `autorelease: pending` label from the merged release pull
request, and the library refuses to open a new one while a merged pending pull
request exists. The workflow reconciles this before each run: a merged release
pull request whose version (read from `.release-please-manifest.json` at its
merge commit) has a `v<version>` tag becomes `autorelease: tagged`, however many
commits landed on `main` before the publication was dispatched. A merged release
pull request that was deliberately never
published stays pending on purpose and blocks the next proposal; remove its label
by hand to release that block.

Install the pinned contract dependencies in `scripts/`, then run from the root:

```sh
node --test scripts/test-release-rehearsal.mjs scripts/test-release-channel.mjs scripts/test-publish-packages.mjs
node scripts/rehearse-release.mjs /tmp/ferromark-release-review
```

The output directory must not exist. It writes one directory per case. The
`automatic` case runs the repository's real released version through the
prerelease strategy and must land on `2.0.0-rc.2`; the remaining cases seed an
in-memory development version and check the forced RC1, RC2, stable and patch
transitions. All of them use the real Release Please updater. Cargo.toml,
Cargo.lock, version.txt, the Release Please manifest, all npm versions and native
dependency pins must agree. External dependencies and publication flags stay
unchanged. See [ADR-0016](arch/ADR-0016-coordinated-workspace-releases.md).

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

Merge the release pull request, wait for all main CI gates on the resulting
commit, and select its run ID. The workflow is manual so merging source, or the
release pull request itself, never publishes a package:

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
