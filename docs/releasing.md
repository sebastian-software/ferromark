# Releasing ferromark

The `ferromark` core crate publishes to crates.io. The optional
`ferromark-transforms` crate shares its version and depends on that core. The
publish workflow sends the core first and then the extension. The extension
has already been bootstrapped with its first publication. Both crates must
have a crates.io Trusted Publisher configured and verified before a release
that includes them. The npm facade and seven native platform packages share
the root product version. The Node development workspace and
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
4. The workflow resolves the tag to one commit SHA, verifies its published
   GitHub release and package versions, then starts the publisher jobs. The core
   crate goes to crates.io while the seven native addons are built, assembled,
   verified and published to npm, sidecars before the facade, and the published
   versions are confirmed on the registry.

The package and version-bump rehearsal do not test registry credentials or
upload an archive. The shared publish action receives `ferromark` before
`ferromark-transforms`, so it publishes the dependency first and can safely
skip already-published versions when a release is retried.

Release Please's Rust updater removes Cargo's `=` exact-version operator from
path dependencies. After it creates or updates the release pull request, the
workflow runs `scripts/preserve-exact-transform-pin.mjs` on that branch and
pushes a small correction only when needed. The release rehearsal applies and
checks the same correction.

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
- `transforms/Cargo.toml`'s version and `ferromark = { version = "…", path = ".." }`;
  the workflow then restores the exact `=…` requirement,
- `node/native/Cargo.toml`'s version and its explicit
  `ferromark = { version = "…", path = "../.." }` requirement,
- all local entries in `Cargo.lock`,
- `CHANGELOG.md` and `.release-please-manifest.json`.

Four `extra-files` entries cover the rest: a typed `$.version` for
`node/ferromark/package.json`, one globbed `$.version` for the seven
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

`npm install ferromark` selects the version behind `latest`; `cargo add
ferromark` selects the latest stable core crate available on crates.io. A future
candidate publishes to `next` under an `X.Y.Z-rc.N` version,
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
request it builds all seven native addons, setting `FERROMARK_PGO=1` for six,
runs the runtime tests on the five same-architecture targets, inspects the two
cross-compiled musl builds and loads the x64 one on Alpine, assembles the eight
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

Six native jobs set `FERROMARK_PGO=1`. The Windows ARM64 job builds without PGO
because the pinned toolchain's `llvm-profdata` rejects the counters written on
that runner; the two cross-compiled musl targets cannot use the host profile
because a Cargo unit hash covers the target triple. The crates.io crate is
unaffected. See [ADR-0019](arch/ADR-0019-profile-guided-native-addon.md).

`publish.yml` repeats the assembly checks on the release tag —
`verify-release.mjs`, `verify-pack.mjs --all-targets`, `pnpm smoke:clean`, the
same Alpine musl runtime test through `docker run node:22-alpine` on the runner,
and `release-archives.mjs` — before the first registry call, and
`verify-npm-publish.mjs` confirms all eight versions and their release-channel
dist-tags on the registry afterwards. npm can accept a publication before its
asynchronous processing makes the version available, so verification retries
each package up to 24 times at 15-second intervals. Missing versions and stale
dist-tags still fail when those attempts are exhausted; the verification job
has a 20-minute timeout.

## Registry authorization

Automated releases and retries use Trusted Publishing (OIDC); `publish.yml` has
no registry API-token fallback. The release workflow requests
`permissions: id-token: write`, and it uses crates.io's OIDC authentication
action and npm 11.5.1 or newer with Node 22.14.0 or newer. If registry
authorization fails, stop the publish and fix the publisher configuration,
workflow claim, or OIDC permissions before retrying the same immutable release
tag from `main`. Do not use an emergency registry token. `RELEASE_PLEASE_TOKEN`
is a GitHub release automation token for Release Please and release bookkeeping,
separate from registry credentials.

As of 2026-10-22, registry restrictions are in progress. The restrictive npm
setting has been saved and rechecked for `ferromark`, `ferromark-darwin-arm64`,
and `ferromark-win32-x64-msvc`. The `ferromark-linux-arm64-gnu` change is waiting
for its interactive confirmation; the other four npm packages have not been
updated. Both crates still need their Trusted Publisher and policy settings
checked. Issue #559 records the previous npm option as allowing bypass-2FA
tokens and the crates.io option as allowing API-token publication. Do not treat
the target settings below as the active release policy until every registry
page shows the corresponding value.

For each npm package, the target Trusted Publisher is GitHub Actions repository
`sebastian-software/ferromark` with workflow `publish.yml` and status `Valid`.
The release action uses direct `npm publish`, which must be allowed for this
Trusted Publisher. `npm stage publish` is always allowed. The npm package
settings UI has displayed the target option `Require two-factor
authentication and disallow bypass 2fa tokens (recommended)`. Use that exact
setting as the target.
The current npm docs use the separate label `Require two-factor authentication
and disallow tokens (Recommended)` and describe that option's broader token
behavior. The npm CLI exposes the corresponding restrictive publishing policy
as `mfa=publish`: it requires MFA to publish and disallows automation tokens.
Do not infer additional token restrictions from the package settings UI label.
The Trusted Publishing docs confirm that OIDC authorization remains available
with token restrictions. Confirm each package's selected option and Trusted
Publisher status after saving. See [npm's 2FA
documentation](https://docs.npmjs.com/requiring-2fa-for-package-publishing-and-settings-modification/)
and [Trusted Publishing documentation](https://docs.npmjs.com/trusted-publishers/),
the [`npm access` CLI documentation](https://docs.npmjs.com/cli/v11/commands/npm-access/),
and the [`libnpmaccess` README in the npm CLI
repository](https://github.com/npm/cli/blob/latest/workspaces/libnpmaccess/README.md).

| npm package | Target publishing access option (npm UI label) |
| --- | --- |
| `ferromark` | Require two-factor authentication and disallow bypass 2fa tokens (recommended) |
| `ferromark-darwin-arm64` | Require two-factor authentication and disallow bypass 2fa tokens (recommended) |
| `ferromark-linux-arm64-gnu` | Require two-factor authentication and disallow bypass 2fa tokens (recommended) |
| `ferromark-linux-arm64-musl` | Require two-factor authentication and disallow bypass 2fa tokens (recommended) |
| `ferromark-linux-x64-gnu` | Require two-factor authentication and disallow bypass 2fa tokens (recommended) |
| `ferromark-linux-x64-musl` | Require two-factor authentication and disallow bypass 2fa tokens (recommended) |
| `ferromark-win32-arm64-msvc` | Require two-factor authentication and disallow bypass 2fa tokens (recommended) |
| `ferromark-win32-x64-msvc` | Require two-factor authentication and disallow bypass 2fa tokens (recommended) |

### Batch npm publishing-access updates

Run the interactive review in a terminal with Node.js, Bash, npm, `jq`, and
`shasum` installed:

```bash
node scripts/npm-token-publishing-tui.mjs
```

The tool discovers packages you can write from your npm account. Repeat
`--owner USER_OR_SCOPE` to include additional npm users or organization scopes,
or pass a newline-delimited inventory across repositories with
`--packages-file FILE`. For example:

```bash
node scripts/npm-token-publishing-tui.mjs \
  --owner @team-a \
  --owner @team-b
```

The npm CLI login is separate from a browser login. If `npm whoami` fails, the
tool offers to run `npm login` interactively and retries discovery. It displays
each package's configured Trusted Publisher identity, workflow file, and
permissions, and filters out packages without current write access and a
publish-capable OIDC permission. `createPackage` permits `npm publish`;
`createStagedPackage` permits `npm stage publish`. Multiple configured
publishers are shown separately.

Use `show N` or `review N` to inspect a package's full claims. `review N` asks
you to acknowledge that npm's UI status and current policy are unknown; after
`ACK UNKNOWN`, it selects that package for apply. `toggle N` checks or unchecks
an eligible package. The acknowledgement records your explicit decision to
proceed despite those unknowns; it does not claim a publisher is `Valid`, that
its two-day validation window has not expired, or that token publishing is
currently enabled. The npm CLI and trust API expose configured claims but do
not expose the UI's `Valid` status, publisher expiry, or current Publishing
access option.

`apply` makes no changes unless at least one package is selected and each
selected package has the per-package `ACK UNKNOWN` review. The helper rechecks
account access and publisher claims before each write, then asks for the final
`APPLY UNKNOWN` confirmation. It runs `npm access set mfa=publish <package>`;
npm may prompt for MFA during each update. The tool never accepts an OTP or
token as an argument or writes one to its temporary plan or state. It stops at
the first error and resumes from its state file on the next run. The default
state file is outside the repository under the user's state directory; pass
`--state FILE` to choose another location.

`COMMAND_OK` means only that the npm CLI command returned success, not that the
registry saved the option. Check each package's Publishing access in npm
settings after the batch. If a command fails or its result is unclear, inspect
the package's settings before retrying. If settings show the policy did not
save after a `COMMAND_OK`, use `reset N` and confirm `RESET <package>` to clear
that package's resume marker. `npm access get status <package>` reports
visibility only.
The five-minute 2FA skip is documented for bulk `npm trust` configuration; do
not assume it also covers `npm access`. See the [`npm trust` CLI
documentation](https://docs.npmjs.com/cli/v11/commands/npm-trust/) and the
[`npm access` CLI documentation](https://docs.npmjs.com/cli/v11/commands/npm-access/).

For both crates, the target Trusted Publisher is GitHub Actions repository
`sebastian-software/ferromark` with workflow `publish.yml`. Verify the live
publisher identity and validation status for each crate before enabling
`Require trusted publishing for all new versions`; crates.io then rejects
API-token publication. The `ferromark-transforms` first-publication bootstrap
is complete, but its live Trusted Publisher row remains unverified until its
settings page is checked. See [crates.io's Trusted Publishing
documentation](https://crates.io/docs/trusted-publishing).

| Crates.io crate | Target Trusted Publisher | Target new-version policy |
| --- | --- | --- |
| `ferromark` | GitHub Actions, `sebastian-software/ferromark`, `publish.yml` | Require trusted publishing for all new versions (enabled; API tokens rejected) |
| `ferromark-transforms` | GitHub Actions, `sebastian-software/ferromark`, `publish.yml` | Require trusted publishing for all new versions (enabled; API tokens rejected) |

A new crate or npm package still needs one authorized credentialed publication
to create its registry entry before a Trusted Publisher can be configured. That
bootstrap does not create an emergency publishing path for later versions. The
retry procedure below uses the configured OIDC publishers.

## Retry a failed publish

Dispatch the current workflow from `main`, and pass the immutable release tag
as its `tag` input. For example, retrying `v3.1.0`:

```sh
gh workflow run publish.yml --repo sebastian-software/ferromark --ref main -f tag=v3.1.0
```

The dispatch must run from main. Before any publication job receives OIDC
permissions, the source validation job accepts only a stable or RC version tag
with a published, non-draft GitHub release. It fetches the current main branch
and tag, then requires the tag commit to be on main. A separate version
validation job checks that the tag version matches the Cargo, npm, and platform
package versions at that commit. A branch name, raw commit SHA, missing tag,
draft release, tag outside main, or version mismatch fails validation.

The validation job resolves the tag once to a full commit SHA. Every release
source checkout uses that SHA, so a tag move during a run cannot make the jobs
publish different commits. Native build jobs store the SHA in each platform
artifact; npm promotion verifies all seven artifact records against the same
SHA before assembling packages. Older releases can still be retried while
their tag remains on main and their published GitHub release remains available.
The retry helper is loaded from the commit that defined the current workflow,
so later retry fixes remain available.

The crate publisher skips a version that is already in the crates.io sparse
index, and an index lookup that fails outright stops the job instead of
guessing. The npm preflight skips only exact package versions confirmed by the
registry and fails closed on other responses. npm versions are immutable: a
version that was already published cannot be replaced, and a conflict requires
a new candidate rather than a forced upload.

The active repository ruleset `Ferromark v* release tag protection` (ID
`25623547`) targets `refs/tags/v*` and restricts tag updates and deletion. Its
bypass list is intentionally empty: a bypass could move a release tag between
a partial publish and a retry, causing the same version to refer to different
source commits. Tag creation is not restricted. The Release Please job retains
`contents: write`, so it can create the version tag.

Before publishing, have a ruleset editor verify the active rule and empty
bypass list through these read-only API calls:

```sh
gh api 'repos/sebastian-software/ferromark/rulesets?includes_parents=true'
gh api 'repos/sebastian-software/ferromark/rulesets/25623547?includes_parents=true'
```

Confirm that the rule still targets `refs/tags/v*` and blocks updates and
deletion. The detail response includes `bypass_actors` only for users with
write access to the ruleset. Do not update or delete a live release tag to test
this control.

Before publishing, the release owner must still review and record who can
dispatch this workflow and any environment approvals. See [Registry
authorization](#registry-authorization) for the crates.io and npm Trusted
Publisher claims.

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
