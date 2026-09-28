# Drop the macOS Intel native package

- Status: Accepted
- Date: 2026-09-27
- Related: [ADR-0020](../arch/ADR-0020-standards-release-blueprint.md),
  [releasing](../releasing.md)

## Context

Ferromark published eight native npm packages, including
`ferromark-darwin-x64` for macOS on Intel (`x86_64-apple-darwin`). Apple
Silicon has replaced Intel Macs for about five years, Apple no longer sells
Intel machines, and GitHub's `macos-15-intel` runner is the last of its kind.
The target costs a slow CI and release job with profile-guided builds for an
audience the owner judges to be negligible. The sibling Ferriki package is
aligning on the same platform set.

## Decision

Ferromark stops building and publishing the macOS Intel addon. The supported
native targets are macOS arm64, Linux x64/arm64 with glibc or musl, and
Windows x64/arm64 (seven packages). `nativeTarget("darwin", "x64")` now fails
with the same "does not support" error as any other unsupported platform.

The owner decided to ship this as a regular release rather than a major
version: the existing `ferromark-darwin-x64` versions stay on npm for anyone
who pins them, and the change affects no realistic user. The release notes
should still mention it.

## Consequences

- One fewer CI and release job; the release matrix has seven targets.
- Installing a new Ferromark version on an Intel Mac fails at load time with
  the unsupported-platform error. Such users can stay on the last release that
  ships the Intel addon.
- Re-adding the target later is additive: restore the `napi.targets` entry,
  the sidecar package, the loader mapping, the CI row and the release list.
