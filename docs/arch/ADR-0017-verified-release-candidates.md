# ADR-0017: Verified release candidates

- Status: Release verification accepted; five-crate layout and bootstrap superseded by [ADR-0018](ADR-0018-single-rust-crate.md).
- Date: 2026-09-15

## Context

The owner authorized the first v2 release candidate after platform and archive
rehearsals. This ends the unpublished development phase described in ADR-0015
and ADR-0016. The Rust facade has four normal workspace dependencies; publishing
only the facade with those dependencies marked private would not produce an
installable crates.io package.

## Decision

Retain the five-crate architecture and publish coordinated versions. Keep the
Node build workspace and binding crate private. Converting the core crates into
modules merely to avoid initial registry setup is outside the RC scope.

Use a manual `publish.yml` workflow, bound to a successful main push CI run at
the exact release commit. Publish verified npm archives with explicit channels:
RCs use `next`, stable uses `latest`. Cargo's multi-package publication stages
and verifies Rust packages in dependency order. Repository-only parser/renderer
cross dev dependencies omit versions so Cargo excludes them from published
metadata; they still run in workspace CI.

Reuse the organization's existing authorization. New Rust crate names require
an initial credentialed publication and per-crate Trusted Publishing setup.
Support a maintainer's clean manual publish or a temporary bootstrap secret.
Never infer that organization ownership alone authorizes an unregistered crate.

Verify existing immutable versions on retries. Confirm fresh registry installs
before creating the GitHub release with authored notes and archived packages.

## Consequences

The first RC exercises actual distribution, not just build output. Merging main
does not automatically publish, and an RC cannot silently become npm's stable
release. The five Rust crates need coordinated version pins and initial setup.
A partial registry release can be resumed only from matching source/artifacts.
