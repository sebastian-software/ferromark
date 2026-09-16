# ADR-0019: Build the published native addons with profile-guided optimization

- Status: Accepted
- Date: 2026-09-16

## Context

The [third Apple Silicon round](../reports/2026-09-16-arm-round-3/README.md)
measured profile-guided optimization as a build experiment rather than a code
change. Trained on 29 broad documents plus 45 authored diagnostics and measured
on the other 28 broad documents, which the training never saw, it reported
**1.204× fresh, 1.240× reuse, 1.255× parse and 1.176× render** — more than the
three code rounds delivered together. The same section records that training on
synthetic diagnostics alone leaves render losses on reference pages, so the
training set has to contain real documents of every category.

The report also names the obstacle: the native matrix produces eight targets,
several of them cross-compiled, and a profile only exists once an instrumented
binary has run. The crates.io crate is a different case entirely: consumers
compile `ferromark` themselves, so no build we control produces their binary.

## Decision

Apply PGO to the eight native Node addons built by the `native` job of
`.github/workflows/ci.yml`, whose artifacts the publish workflow later uploads.
Leave the `ferromark` source crate and local development builds unchanged.

Add `node/native/src/bin/pgo_train.rs`, a training driver in the private
`ferromark-node` crate behind a `pgo-train` feature so `napi build` does not
compile it. It depends only on `ferromark` and the standard library — a `cdylib`
cannot be used as a library target, and the addon's N-API surface is a thin
wrapper around the parser and renderer anyway. It runs every document through
the four option combinations the addon reaches (product defaults, GFM parser
with default rendering, full GFM, MDX) in both addon lifecycles: a fresh
allocator and renderer per document, and a retained allocator reset per document
with a retained renderer writing into its own buffer.

Train on the frozen benchmark corpus already in the repository,
`docs/reports/2026-09-14-simd-round/corpus.json.gz` — 57 real documents plus
link-scan diagnostics. `node/ferromark/scripts/build-native.mjs` expands it into
a working directory under `target/pgo/`; the corpus is not stored twice.

Make the mode opt-in through `FERROMARK_PGO=1`, off by default so local builds
stay fast, and on for the `pnpm build:native` step of the `native` job. Under
that flag the script builds the training binary with `-Cprofile-generate`, runs
it, merges the counters with `llvm-profdata` from the `llvm-tools` rustup
component, and rebuilds the addon with `-Cprofile-use` plus
`-Cllvm-args=-pgo-warn-missing-function`. A missing `llvm-profdata` fails the
build with the component to install; CI never falls back to a non-PGO addon
silently.

The training build must mirror the addon build's Cargo profile (`release-node`)
and pass `--target`, because Cargo's unit hash covers both and a profile only
applies to units whose hash matches. Measured on this checkout: with the
matching profile and triple, 247 of the 297 `ferromark` functions in the addon
receive profile data and the remaining 50 are paths the corpus never executes;
with the default `release` profile instead, all 297 are reported missing and the
build is a silent no-op.

## Coverage limits

Five targets — both Darwin, both GNU Linux and x86-64 Windows — are built on
runners of their own architecture, so the host-trained profile applies to them.
The two musl targets are cross-compiled from `x86_64` GNU runners with
cargo-zigbuild, and `aarch64-unknown-linux-musl` is a different architecture
from its runner. Their unit hash differs from the host's, so those builds carry
the profile flag but receive no profile data; they are otherwise unchanged.
The ARM64 Windows job is built without PGO (`pgo: false` in the CI matrix): on
that runner the pinned toolchain's `llvm-profdata` rejects the counters its own
instrumented binary writes (`malformed instrumentation profile data: symbol
name is empty`), so the addon stays on the plain build until a toolchain
update or a different instrumentation setting is verified there.
Collecting a profile per target would require running an instrumented binary on
each target, which needs emulation or additional runners. That is a separate
decision, not a v2.0 blocker.

## Consequences

The addon binary now depends on counts taken at build time, so two CI runs of
the same commit no longer produce identical binaries. Nothing in the pipeline
requires that: `node/scripts/verify-platform-artifact.mjs`,
`verify-release.mjs` and `verify-pack.mjs` check names, versions, sizes,
contents and the glibc baseline, not hashes. `publish.yml` builds the addons
itself from the release tag and never compares a rebuilt binary with a
published one, so a retry is free to rebuild
([ADR-0020](ADR-0020-standards-release-blueprint.md)).

Training adds an instrumented build plus about twenty seconds of training to
each of the eight native jobs.

## Alternatives

- **No PGO.** Leaves the largest measured gain of the three rounds unclaimed.
- **A profile per target, collected on the target.** Would also cover the musl
  builds, at the cost of emulated or additional runners and a second artifact
  hop through the workflow. Revisit if the musl targets gain weight.
- **PGO for the Rust crate.** Not possible. crates.io ships source; the consumer
  compiles it with their own flags, and a profile in the package would neither
  match their unit hashes nor their workload.
