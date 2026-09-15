# ADR-0016: Coordinate Rust and npm releases through one manifest

Status: Accepted for release preparation; publication remains disabled.

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

## Validation and limits

`node --test scripts/test-release-rehearsal.mjs` runs in the existing CI contract
job. `node scripts/rehearse-release.mjs <new-directory>` also writes complete
updated release files and PR text for review. The repository's real version stays
`2.0.0-dev.0`; the rehearsal does not create a tag, PR, registry upload, or update
publication permissions. Package assembly and registry authentication are separate
release steps.

The structural reference optimization is deferred in
[issue #320](https://github.com/sebastian-software/ferromark/issues/320) by the
owner's explicit decision. Its measured costs remain documented and do not block
v2.0 release preparation.

## Sources

- [Official simple strategy](https://github.com/googleapis/release-please/blob/v17.6.0/src/strategies/simple.ts)
- [Generic annotations](https://github.com/googleapis/release-please/blob/v17.6.0/src/updaters/generic.ts)
- [TOML updater](https://github.com/googleapis/release-please/blob/v17.6.0/src/updaters/generic-toml.ts)
- [Pinned action lockfile](https://github.com/googleapis/release-please-action/blob/0dfd8538845b8e92600d271a895a5372865d4062/package-lock.json)
