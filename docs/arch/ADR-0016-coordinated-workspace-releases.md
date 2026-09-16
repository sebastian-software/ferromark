# ADR-0016: Coordinate Rust and npm releases through one manifest

Status: Superseded on 2026-09-16 by
[ADR-0020](ADR-0020-standards-release-blueprint.md). The decision that Rust and
npm share one version, one manifest entry, one changelog and one tag stands. The
mechanism does not: the `simple` strategy, `version.txt`, the
`x-release-please-version` annotations, the `Cargo.lock` and `pnpm-lock.yaml`
jsonpaths and the separate `release-please.yml` with `skip-github-release` are
all replaced by the native `rust` strategy and the organization's publish
skeleton, where merging the release pull request publishes. The `prerelease`
versioning decision and the `Release-As` rule below are unchanged and still
enforced by the rehearsal. Read this ADR for why the coordination exists; read
ADR-0020 for how it works now.

## Problem

V2 has a virtual Cargo workspace with inherited member versions. Release Please's
Rust package updater requires a root `[package]` and cannot update this root
manifest. The former configuration also omitted the workspace dependency pins
and the six local entries in Cargo.lock.

## Decision

Use Release Please's `simple` strategy with `version.txt` and one root component.
Update the workspace version and five exact dependency requirements through
explicit `x-release-please-version` annotations in Cargo.toml. Generic replacement
preserves the `=` requirement and the existing comments and formatting. All six
Rust packages, including the unpublished Node cdylib, inherit that version.

Target the six local Cargo.lock versions with TOML JSONPath updates. In the pinned
Release Please 17.6.0 implementation, the TOML parser wraps scalar values, so name
predicates use `@.name.value`; `!@.source` excludes registry entries. This detail is
covered by an execution test, not assumed from a JSONPath-shaped config string.
Keep the npm facade, eight native versions, exact optional dependency pins, and
pnpm workspace specifiers in the same release PR. Registry dependency versions,
member inheritance and publication flags must stay unchanged.

The rehearsal uses the official Release Please 17.6.0 Manifest, strategies and
updaters, matching the library locked by release-please-action at
`0dfd8538845b8e92600d271a895a5372865d4062`. Its only substitute is a local read-only
repository/commit source. No GitHub client or publication method is constructed.
Test `dev → rc.1 → rc.2 → stable → patch`, including release notes and manifest
updates. Negative cases remove update rules and must fail validation. Changing
the action/library pin requires repeating this contract.

Use explicit `Release-As` commit footers for RC and stable transitions, rather
than leaving a permanent forced version in configuration. The later publisher
must derive the GitHub prerelease flag and npm dist-tag (`next` versus `latest`)
from the selected version. This configuration alone creates no releases.

## Amendment 2026-09-16: automatic release pull request

The statement above that the configuration "alone creates no releases" described
a repository with no trigger at all: v1's `release-please` job was lost when the
v2 core was initialized and the rebuilt `publish.yml` did not restore it, so the
version bump had to be prepared by hand. That was a regression, not a decision.

`.github/workflows/release-please.yml` restores it on every push to `main` and on
demand, with the same `0dfd8538845b8e92600d271a895a5372865d4062` action pin this
ADR already contracts. It runs with `skip-github-release`, so the action only
calls `createPullRequests()`; the tag and the GitHub release stay with the
verified `publish.yml` ([ADR-0017](ADR-0017-verified-release-candidates.md)).
Subsequent cycles read the last release from the `v*` tags that `publish.yml`
creates, through `releaseIterator` and the `backfillReleasesFromTags` fallback.

Two consequences of that mode are load-bearing:

- `createReleases()` is the only place the library moves `autorelease: pending`
  to `autorelease: tagged`, and `createPullRequests()` returns early while any
  merged pull request still carries the pending label. Skipping releases alone
  would open exactly one release pull request and then stall forever. The
  workflow reconciles the label first: for each merged pending release pull
  request it reads the version its merge commit introduced from
  `.release-please-manifest.json` and promotes the pull request when the tag
  `v<version>` exists. Keying on the released version rather than on the pull
  request title or on the tagged commit keeps that independent of the title
  pattern and of commits that land on `main` between merge and publication.
- The release pull request needs the `RELEASE_PLEASE_TOKEN` personal access
  token. A pull request opened with `GITHUB_TOKEN` starts no further workflow
  runs, so no CI check would ever
  run on it.

Version selection moves to `"versioning": "prerelease"` with
`"prerelease-type": "rc"` and `"prerelease": true`. Inside the candidate series
the proposal is automatic and correct without any footer: on `2.0.0-rc.1` a
breaking change takes `PrereleaseMajorVersionUpdate`, which finds `minor` and
`patch` both zero and increments the existing prerelease to `2.0.0-rc.2` instead
of promoting to `3.0.0`. `prerelease: true` is required; without it the strategy
strips the prerelease and proposes a stable version.

`Release-As` still wins over the strategy, because `determineReleaseType` returns
a `CustomVersionUpdate` for the newest `RELEASE AS` note before it inspects any
commit type. The footer therefore stays mandatory exactly where it was already
prescribed, and now also for the first patch after a stable release: from a
stable `2.0.0` the strategy would otherwise propose `2.0.1-rc`. The rehearsal
encodes both directions.

## Validation and limits

`node --test scripts/test-release-rehearsal.mjs` runs in the existing CI contract
job. `node scripts/rehearse-release.mjs <new-directory>` also writes complete
updated release files and PR text for review. The rehearsal does not create a
tag, PR, registry upload, or update publication permissions. Package assembly and
registry authentication are separate release steps.

The rehearsal covers both selection modes. Its `automatic` case starts from the
repository's own released version and feeds a commit range representative of
`git log --format=%B v2.0.0-rc.1..main` — a breaking `perf(renderer)!` with a
`BREAKING CHANGE:` footer, a `feat(node)` and a `docs` commit, with no
`Release-As` — and must land on `2.0.0-rc.2` with every coordinated file updated.
The forced cases keep checking `dev → rc.1 → rc.2 → stable → patch` by footer,
and the negative cases are unchanged.

The rehearsal substitutes only a local read-only repository and commit source. It
does not exercise the action's own label reconciliation or the `GITHUB_TOKEN`
fallback, which are observable only on GitHub.

The structural reference optimization is deferred in
[issue #320](https://github.com/sebastian-software/ferromark/issues/320) by the
owner's explicit decision. Its measured costs remain documented and do not block
v2.0 release preparation.

## Sources

- [Official simple strategy](https://github.com/googleapis/release-please/blob/v17.6.0/src/strategies/simple.ts)
- [Generic annotations](https://github.com/googleapis/release-please/blob/v17.6.0/src/updaters/generic.ts)
- [TOML updater](https://github.com/googleapis/release-please/blob/v17.6.0/src/updaters/generic-toml.ts)
- [Pinned action lockfile](https://github.com/googleapis/release-please-action/blob/0dfd8538845b8e92600d271a895a5372865d4062/package-lock.json)
- [Pinned action entry point](https://github.com/googleapis/release-please-action/blob/0dfd8538845b8e92600d271a895a5372865d4062/src/index.ts):
  `skip-github-release` guards only `createReleases()`; `createPullRequests()` is
  guarded separately by `skip-github-pull-request`.
- [Prerelease versioning strategy](https://github.com/googleapis/release-please/blob/v17.6.0/src/versioning-strategies/prerelease.ts)
- [Manifest label handling](https://github.com/googleapis/release-please/blob/v17.6.0/src/manifest.ts):
  `createPullRequests` aborts on any merged pull request that still carries
  `autorelease: pending`; `createReleases` is what retires it.
