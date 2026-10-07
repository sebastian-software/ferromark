# Allow Rust API changes in minor releases

- Status: Accepted
- Date: 2026-10-01
- Scope: The public Rust API of the `ferromark` crate from 3.1

## Context

The [API surface decision](2026-09-17-api-surface.md) froze the Rust API under
semver for 2.0.0 and kept `ParserOptions` and `HtmlRendererOptions` exhaustive.
Under that rule every new option field is a major version. Version 3.0 followed
two weeks after 2.0 for exactly that reason, and the JSX work adds option and
result fields again.

The crate is young and has no known external users. Strict semver would raise
the major version every few weeks without protecting anyone.

## Decision

A minor release may add, change, or remove public Rust API. A patch release may
not. The changelog names each such change, and a migration note accompanies a
change that needs more than a mechanical edit.

A major version is a deliberate product decision, not an automatic result of an
API change. Release Please proposes a major version for a commit marked with `!`
or a `BREAKING CHANGE:` footer, so that marker is reserved for releases that are
meant to be major.

The decisions about the shape of the API stay: which types are exhaustive, and
struct-update syntax as the documented way to build options.

## Consequences

A caret requirement such as `ferromark = "3"` can pick up a minor release that
no longer compiles. Consumers who need a stable build use a tilde requirement
such as `~3.1` or an exact version.

Rendered output is not part of this decision. Snapshot output and conformance
baselines remain release gates, and an intended semantic change still needs its
own decision record.

This rule fits a project without adoption. It needs a review when external
users depend on the crate.
