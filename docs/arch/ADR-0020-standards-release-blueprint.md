# ADR-0020: Adopt the standards release blueprint

- Status: Accepted
- Date: 2026-09-16
- Supersedes the publication decisions in
  [ADR-0016](ADR-0016-coordinated-workspace-releases.md) and
  [ADR-0017](ADR-0017-verified-release-candidates.md)

## Context

Ferromark v2 prepared a version automatically and published it by hand. A
maintainer merged the release pull request, waited for CI, and dispatched
`publish.yml` with the version and a CI run id; the workflow then verified that
run, downloaded its archives, published both registries and created the tag and
the GitHub release itself. Preparing was automatic, releasing was a second,
different, deliberate act.

The owner has decided that merging the release pull request must publish, and
that the flow must be the same one across the organization's repositories. The
organization's blueprint for that flow is
`reference/release-please/` in
[sebastian-software/standards](https://github.com/sebastian-software/standards):
a real root package with `release-type: rust`, Node versions as typed JSON
`extra-files`, no `version.txt`, no Cargo version `extra-files`, no lockfile
jsonpaths, a lockfile no-diff check, and one publish workflow whose jobs are all
gated on one release signal. The blueprint's own migration table for this
repository asks for two changes: replace the eight `pnpm-lock.yaml` jsonpaths
with workspace references plus the lockfile check, and adopt the shared
`napi-matrix` action for the eight platforms.

The manual flow was not wrong; it was ours alone. Every repository that keeps
its own publisher also keeps its own retry semantics, its own crates.io index
wait and its own platform list, and each of those is a place where one
repository quietly differs from the rest.

## Decision

Adopt the blueprint as written.

**One root package.** `ferromark` moves from `crates/ferromark` to the
repository root, v1's layout, so Cargo discovers its tests and benchmarks and
`release-type: rust` can update the root `[package]`, the members, the published
internal requirement in `node/native` and `Cargo.lock` natively. Package
versions are concrete everywhere: Release Please cannot replace an inherited
`version.workspace = true`, and it deliberately skips a path dependency that
does not spell out a `version`. Workspace inheritance stays for edition,
license, lints, profiles and external dependencies. A root package would offer
the whole repository to crates.io, so the manifest carries a narrow `include`;
the published file set is the one the nested crate had, plus the MIT license
text and `UPSTREAM.md` that the README's attribution section points at.

**No second source of truth for a version.** `version.txt`, the eight Cargo
`x-release-please-version` annotations and the two `Cargo.lock` jsonpaths are
removed. The eight npm sidecars are referenced as `workspace:*`, so the pnpm
lockfile holds no version at all and the eight lockfile jsonpaths are removed
with them. What remains is nine typed `$.version` entries and three README
blocks. The consequence is that publishing goes through pack-resolved tarballs:
only pnpm rewrites `workspace:*` to the sidecar's version while packing, so the
archives `verify-pack.mjs` already produces are what reaches npm, and the
dist-tag is named explicitly rather than derived from a directory.

**One release signal.** `publish.yml` is the blueprint's skeleton. Release
Please runs unconditionally on pushes to `main`; when the release pull request
merges it writes the version commit, creates `v<version>` and the GitHub release
from the changelog section, and the publish jobs run from that tag. crates.io
and npm both authenticate through Trusted Publishing. `workflow_dispatch` takes
the release tag and exists only to retry a failed job against the sources the
release was cut from.

**Shared actions instead of local copies.** `publish-crates`, `napi-matrix` and
`publish-npm` come from the standards repository, pinned by commit. The eight
platform triples, the sidecar/artifact/binary naming and the crates.io index
wait are defined once for the organization. The repository-local
`check-workflow-pins` is replaced by the shared `check-action-pins`.

**Version selection is unchanged.** `versioning: prerelease` with
`prerelease-type: rc` and `prerelease: true` keeps the candidate series
automatic — a breaking change on `2.0.0-rc.1` proposes `2.0.0-rc.2`, not
`3.0.0` — and leaving the series still needs an explicit `Release-As` footer.
`include-component-in-tag` is false, so the tag stays `v<version>` and the
published `v2.0.0-rc.1` remains the anchor for the next cycle. The stable
release amended this paragraph; see the amendment below.

## Consequences

Merging the release pull request publishes. There is no longer a gate between
the merge and the registries, so the release pull request itself is the review,
and the pre-merge CI rehearsal carries the weight the dispatch-time preflight
used to: `ci.yml` still builds all eight profile-guided addons, assembles the
nine npm archives, runs a clean install and rehearses the Cargo archive on every
pull request. `publish.yml` repeats the assembly checks on the release tag
before the first registry call.

Three guarantees change shape rather than disappearing. A publish retry no
longer reuses one CI run's artifacts; it rebuilds from the release tag, which is
sound because the addons were never byte-reproducible (ADR-0019) and the
verification checks names, versions, contents and the glibc baseline rather than
hashes. An already-published crate version is skipped by the composite instead
of being compared against its `.cargo_vcs_info.json`. The release candidate's
channel is enforced by naming `next` explicitly, and confirmed afterwards by
`verify-npm-publish.mjs` across all nine packages.

What we give up for the alignment: this repository's own preflight refused to
publish a version whose CI had not passed at that exact commit. Branch
protection on `main` is what carries that now.

## Validation

`node --test scripts/test-release-rehearsal.mjs` runs the pinned Release Please
17.6.0 Manifest, strategies and updaters against the repository's own files and
a synthetic commit history, with no GitHub client and no publishing method. It
covers the automatic `2.0.0 → 2.0.1` proposal, the forced
`dev → rc.1 → rc.2 → stable → patch` transitions, a missing typed entry for each
npm manifest group, an inherited member version — which the library's Cargo
updater refuses outright — and a path dependency without a version, which it
silently leaves stale.

`node scripts/rehearse-release.mjs <new-directory>` writes every generated
candidate and hands it to the real tools, as the blueprint's "prove the
candidate without publishing" requires: `cargo metadata --locked` over the
generated manifests and lockfile, and `pnpm install --lockfile-only` over the
generated npm manifests, which must leave the lockfile byte-identical.
`python3 -m unittest discover -s scripts` guards the `include` allow-list that
now decides what reaches crates.io, and
`python3 scripts/rehearse-rust-packages.py <new-directory>` builds and verifies
the actual archive from the root package.

Three things are observable only on GitHub and were not rehearsed here: the
crates.io and npm Trusted Publishing exchanges, the two cross-compiled musl
builds, and the `autorelease:` label lifecycle now that `createReleases()` runs
again.

## Amendment (2026-09-18): default versioning for the stable series

The candidate series ended with `2.0.0`. That release removed `versioning`,
`prerelease-type` and `prerelease` from `release-please-config.json`, so version
selection follows Release Please's default strategy from here and the commit
range decides the next stable version: `fix:` a patch, `feat:` a minor, a
breaking change a major.

Keeping the prerelease strategy was not an option. On top of a stable `2.0.0` it
proposes `2.0.1-rc` for an ordinary fix — a shape
`node/scripts/release-channel.mjs` refuses to publish, and it refuses it only
after the tag and the GitHub release already exist. The switch and the
`Release-As: 2.0.0` footer therefore belonged to the same release: under the
default strategy without that footer a candidate series drifts instead of ending
(a `feat:` on `2.0.0-rc.2` proposes `2.1.0-rc.2`).

The GitHub release flag is unaffected. Release Please marks a release as a
prerelease only when the version has a prerelease part or its major is 0, so
`v2.0.0` is an ordinary release under either configuration. Starting a candidate
series again is deliberate and is described in
[releasing.md](../releasing.md).

## Sources

- [One product release with Release Please](https://github.com/sebastian-software/standards/blob/main/reference/release-please/README.md)
- [The publish skeleton](https://github.com/sebastian-software/standards/blob/main/reference/release-please/publish-skeleton.yml)
- [The shared composite actions](https://github.com/sebastian-software/standards/blob/main/.github/actions/README.md)
- [Rust strategy](https://github.com/googleapis/release-please/blob/v17.6.0/src/strategies/rust.ts)
- [Prerelease versioning strategy](https://github.com/googleapis/release-please/blob/v17.6.0/src/versioning-strategies/prerelease.ts)
