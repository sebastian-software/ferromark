# Releasing ferromark v2

The `ferromark` core crate publishes to crates.io. The optional
`ferromark-transforms` crate shares its version and depends on that core. The
current publish workflow still lists only `ferromark`; before the transform
crate's first release, that workflow must be approved and changed to publish
the core first and then the extension. The npm facade and eight native platform
packages share the root product version. The Node development workspace and
`ferromark-node` binding crate remain private. Allocator, AST, parser and
renderer are modules inside the core library; consumers can use them
individually through `ferromark`. Since
[ADR-0020](arch/ADR-0020-standards-release-blueprint.md) that crate is the
repository root package, and the release follows the organization's
[release blueprint](https://github.com/sebastian-software/standards/blob/main/reference/release-please/README.md):
**merging the release pull request publishes.**

## The flow

1. Every push to `main` runs `.github/workflows/publish.yml`. Its first job is
   Release Please, which opens or updates one release pull request from the
   commits since the last release tag.
2. Review that pull request. It carries the whole coordinated version bump and
   the changelog section that becomes the release notes.
3. Merge it. Release Please writes the version commit, creates the `v<version>`
   tag and the GitHub release, and sets `releases_created`.
4. The currently gated jobs in the same workflow run from that tag. They gate on Release
   Please alone, so the crates.io publication and the native addon pipeline start
   in parallel: the core crate goes to crates.io while the eight native addons are
   built, assembled, verified and published to npm, sidecars before the facade,
   and the published versions are confirmed on the registry.

The transform extension is not included in that current publish action. Its
first release is blocked until the publish action lists `ferromark` before
`ferromark-transforms` and crates.io Trusted Publishing is configured for the
new crate. The package and version-bump rehearsal do not test registry
credentials or upload an archive.

Nothing else publishes. Merging ordinary source still only opens or updates the
release pull request.

The release pull request is opened with the `RELEASE_PLEASE_TOKEN` organization
secret. A pull request opened with `GITHUB_TOKEN` starts no further workflow
runs, so `ci.yml` would never run on it and the pull request would show no
checks at all.

## What the release pull request changes

`release-please-config.json` selects `release-type: rust` with one root
component. The strategy updates natively, with no template to keep in step:

- the root `Cargo.toml` `[package]` version,
- `transforms/Cargo.toml`'s version and its exact
  `ferromark = { version = "…", path = ".." }` dependency,
- `node/native/Cargo.toml`'s version and its explicit
  `ferromark = { version = "…", path = "../.." }` requirement,
- all local entries in `Cargo.lock`,
- `CHANGELOG.md` and `.release-please-manifest.json`.

Four `extra-files` entries cover the rest: a typed `$.version` for
`node/ferromark/package.json`, one globbed `$.version` for the eight
`node/ferromark/npm/*/package.json` manifests, and the two generic README blocks
marked with `x-release-please-start-version` / `end-version` in `README.md.src`
and the generated `README.md`. Those blocks hold the versioned docs.rs link and
nothing else: the install lines name the package alone, and
`node/ferromark/README.md` carries no version, so it is not a template target.

There is deliberately **no** `version.txt`, no Cargo `extra-files` and no
`pnpm-lock.yaml` jsonpath. The npm facade references its sidecars with
`workspace:*`, which resolves to the sidecar's own version at pack time, so the
pnpm lockfile holds no version at all and cannot go stale. The `node` CI job
proves that in one line with `pnpm install --lockfile-only` and
`git diff --exit-code`.

Because the facade's references are resolved while packing, publishing uses the
**pack-resolved tarballs** in `node/artifacts/` rather than the package
directories: `pnpm pack` rewrites `workspace:*`, `npm pack` would not.

## Version selection

Version selection uses Release Please's default strategy, so the commit range
decides: a `fix:` proposes the next patch, a `feat:` the next minor, and a
breaking change the next major. The candidate series ended with `2.0.0`, and the
`versioning`, `prerelease-type` and `prerelease` keys were removed from
`release-please-config.json` in that same release. A `Release-As: X.Y.Z` footer
still overrides the proposal, which is how `2.0.0` left the candidate series.

Starting a candidate series again is deliberate, never automatic. Land the first
candidate with an explicit `Release-As: X.Y.Z-rc.1` footer. The default strategy
then keeps the suffix but still bumps from the commit range — a `feat:` on
`2.0.0-rc.1` proposes `2.1.0-rc.1` — so a series that has to stay inside one
target version needs `versioning: prerelease`, `prerelease-type: rc` and
`prerelease: true` restored for its duration and removed again by the commit
that carries the stable `Release-As` footer.

`include-component-in-tag` is false, so tags stay `v<version>`, matching the
published `v2.0.0`.

## Channels

The dist-tag is derived from the version by `node/scripts/release-channel.mjs`
and passed to the publishing action explicitly: `X.Y.Z-rc.N` publishes to
`next`, a stable `X.Y.Z` to `latest`. No other version shape is publishable.

Stable v2 is on `latest`, so `npm install ferromark` and `cargo add ferromark`
select it. A future candidate publishes to `next` under an `X.Y.Z-rc.N` version,
and its GitHub release is a prerelease that does not become `latest`.

## Rehearse the version bump

Install the pinned contract dependencies in `scripts/`, then run from the root:

```sh
node --test scripts/test-release-rehearsal.mjs scripts/test-release-channel.mjs
node scripts/rehearse-release.mjs /tmp/ferromark-release-review
```

The output directory must not exist. It writes one directory per case, with the
complete updated files and the release pull request text. The `automatic` case
seeds the stable `2.0.0` release and lets an ordinary maintenance range propose
`2.0.1` on its own; the remaining cases seed an in-memory development version
and check the forced RC1, RC2, stable and patch transitions. All of them
use the real Release Please 17.6.0 Manifest, strategies and updaters, with a
local read-only repository and commit source substituted for the GitHub client.

Each generated candidate is then proved with the real tools rather than only
read:

- `cargo metadata --locked` resolves the generated manifests against the
  generated `Cargo.lock` and fails if either is stale;
- `pnpm install --lockfile-only` re-locks the generated npm manifests and the
  lockfile must come back byte-identical.

`results.json` records both outcomes per case. Run `cargo fetch --locked` first;
the Cargo check is offline.

## Local package checks

Run the Rust checks in [CONTRIBUTING.md](../CONTRIBUTING.md), then from `node/`:

```sh
pnpm install --frozen-lockfile
pnpm audit --audit-level high
pnpm build
pnpm test
pnpm typecheck
pnpm format:check
pnpm lint
pnpm pack:check
pnpm smoke:clean
```

The Node workspace requires 22.13.0; the package supports 22.12.0. CI builds
on the workspace floor and tests the binary on the consumer floor.
`release-node` inherits the optimized release profile with `panic = "unwind"`.
The panic test verifies that Rust panics become JavaScript exceptions.

`pnpm smoke:clean` installs the packed archives for the current host. The same
script takes `--target <platform-target>`, which fails unless the host really is
that target, and `--package-tests`, which copies the package suite into the
installed package and runs it there, against the installed loader and sidecar.
The Alpine job runs
`node ./scripts/clean-install-smoke.mjs --target linux-x64-musl --package-tests`.

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

The output directory must not exist. `cargo package --locked` builds and
verifies the `ferromark` core archive first, then the `ferromark-transforms`
archive. The latter's development manifest uses a local path to the core; Cargo
normalizes that to its exact registry dependency in the package archive. The
rehearsal applies a temporary Cargo patch to the local core because its current
version is not published yet; the package manifest itself still targets the
crates.io version.

The rehearsal checks both packages' metadata, upstream MIT notices and absence
of repository-only paths, then builds and runs an isolated consumer from both
unpacked archives. External versions must remain within the workspace
lockfile. Each README, the upstream `LICENSE` notice, the `LICENSE-MIT` text
and `UPSTREAM.md` ship with the core archive; the extension declares the same
MIT license through workspace metadata and carries its own README. This does
not test registry credentials.

Finally it runs `cargo check --all-targets --locked --offline` inside each
unpacked archive. The consumer links and runs the custom pass against both
packages. `tests/`, `benches/` and `examples/` also ship, so anyone running
`cargo test` on a published package compiles them. `--offline` needs the
dev-dependencies, which the `cargo fetch --locked` above resolves.

The root `Cargo.toml` decides what the core archive contains. Compare
`cargo package --list` before and after any change to it;
`scripts/test_release_package.py` guards both package manifests' allow-lists.
The core allow-list keeps `docs/`, `benchmarks/`, `homepage/`, `node/`,
`scripts/` and `.github/` out as directories. Three regression corpora that
shipped tests embed with `include_str!` are the sole exception, named there as
individual files; the same test checks that no shipped target embeds anything
else from outside the core archive.

## The pre-merge rehearsal

Because merging publishes, `ci.yml` is where a release is proven. On every pull
request it builds all eight native addons with profile-guided optimization,
runs the runtime tests on the six same-architecture targets, inspects the two
cross-compiled musl builds and loads the x64 one on Alpine, assembles the nine
npm packages, checks their contents, performs a clean installation on Linux x64
GNU and rehearses the Cargo archive. It publishes nothing and retains its
verified archives for seven days.

The musl addons are cross-compiled, so inspection alone never proved that they
load. The `node-musl` job closes that gap after the assembly: in a
`node:22-alpine` container it downloads the `npm-package-rehearsal` archives,
installs the packed facade together with the packed `ferromark-linux-x64-musl`
sidecar in a fresh consumer, and runs the package test suite from inside that
installation, so the whole public API — rendering, buffers, renderer reuse,
metadata, the highlighter callback and error propagation — runs through the musl
addon the release would publish. The job names the platform package it expects,
and that check fails unless the container really is that musl host, so it cannot
pass against a glibc addon.

Two limits there are deliberate. `aarch64-unknown-linux-musl` stays inspected
only; loading it needs an arm runner with a musl container. And
`verify-panic-unwind.mjs` and `verify-input-conversion.mjs` do not run in that
container: they build throwaway addons from source with the `panic-test` and
`boundary-bench` Cargo features, which no packed addon carries and which need a
Rust toolchain. Panic unwinding comes from the shared `release-node` profile,
and `pnpm test` verifies it on the same-architecture native jobs. The package
test suite in the container still renders every V8 string representation
through the musl addon.

Seven native jobs set `FERROMARK_PGO=1`, so each published addon is built from a
profile collected on its own runner. The Windows ARM64 job builds without PGO
because the pinned toolchain's `llvm-profdata` rejects the counters written on
that runner; the two cross-compiled musl targets do not receive the host
profile, because a Cargo unit hash covers the target triple. The crates.io crate
is unaffected. See [ADR-0019](arch/ADR-0019-profile-guided-native-addon.md).

`publish.yml` repeats the assembly checks on the release tag —
`verify-release.mjs`, `verify-pack.mjs --all-targets`, `pnpm smoke:clean`, the
same Alpine musl runtime test through `docker run node:22-alpine` on the runner,
and `release-archives.mjs` — before the first registry call, and
`verify-npm-publish.mjs` confirms all nine versions on the registry afterwards.

## Registry authorization

Both registries authenticate through Trusted Publishing (OIDC): no crates.io or
npm token exists in this repository. It is configured for the `ferromark` crate
and for the nine npm packages, repository `sebastian-software/ferromark`,
workflow `publish.yml`. The publishing jobs therefore need
`permissions: id-token: write`, and npm publishing needs npm 11.5.1 or newer,
which the workflow installs explicitly.

Adding a new package name needs an initial credentialed publication before
Trusted Publishing can be enabled for it; a first-ever publish cannot use it,
because the package does not exist yet.

## Retry a failed publish

The release is already tagged, so a retry runs against that tag:

```sh
gh workflow run publish.yml -f tag=v2.0.0
```

Every job checks out that tag rather than `main`, so a delayed retry cannot
publish newer sources under a version that already exists, and an unknown tag
fails the checkout. The crate publisher skips a version that is already in the
crates.io sparse index, and an index lookup that fails outright stops the job
instead of guessing. npm versions are immutable: a version that was already
published cannot be replaced, and a conflict requires a new candidate rather
than a forced upload.

## Release notes and history

The GitHub release body is the `CHANGELOG.md` section Release Please wrote for
that version. Review and, where useful, edit that section in the release pull
request before merging — it lists the conventional commits in the range and is
prepended above the authored `## 2.0.0-rc.1` entry already in Git.
`docs/releases/` keeps the first candidate's hand-written notes as history; no
file is added there for later versions.

The homepage deploys from main. Its deployment is independent of registry
publication and needs no release edit: `homepage/app/version.ts` reads the
version from `node/ferromark/package.json`, which the release pull request
updates, and `homepage/scripts/verify-build.mjs` fails the build unless the
rendered pages carry exactly that version. `deploy-homepage.yml` therefore runs
on a push that changes that manifest as well as on one that changes `homepage/`,
because a release pull request touches no file under `homepage/` at all.
