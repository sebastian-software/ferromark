# Shared native npm tooling

Ferromark's Node build uses Ferramenta's `ferramenta-native` development package.
`node/native-npm.config.mjs` names the facade, platform packages, binary filenames
and license files. Run `pnpm native:generate` and `pnpm native:check` in `node`
to regenerate or verify the platform manifests and standalone runtime. The
existing npm names, public API, 3.x Ferriki compatibility build, PGO training,
diagnostic-build isolation and release source checks remain supported.

The initial dependency is the reviewed `node/vendor/ferramenta-native-0.1.0-<digest>.tgz`
archive. It is identical to the archive used by Ferriki and Ferrigura. After
Ferramenta publishes that tooling version, replace the file dependency with the
exact registry version, regenerate the lockfile and remove the bootstrap archive.
No path to a sibling checkout is required. The end-user facade contains the
generated loader and has no dependency on the tooling package.

## Verification of this migration

The local macOS arm64 checks built all three engines independently, packed each
facade and its host sidecar, and installed all six tarballs together offline with
install scripts disabled. One Node process composed Ferromark's code-renderer
callbacks with a Ferriki highlighter and a Ferrigura renderer. It produced
highlighted, escaped code and an SVG diagram. Each product's separate packed
consumer check also rejects a missing binary.

Ferromark's complete Rust tests, Clippy, formatting, benchmark build, 221 Node
tests, panic containment, input conversion and isolated plugin checks passed.
Ferriki's core compatibility and release recovery checks passed. Ferrigura's
Rust and Node checks and Ferramenta's complete project gate passed. The seven
platform build matrices are configured in CI; the other platforms were not
executed by this local run. This migration makes no new performance claim.

The infrastructure suite reports 82 passing tests and two failures, with the
same result on the unmodified starting commit: the workspace-members assertion
still expects only `node/native` and `transforms`, and the release rehearsal
changes a registry-sourced Ferromark dependency while checking that external
dependencies stay fixed. Those pre-existing release issues need a separate fix
before a release. No packages were published by this migration.
