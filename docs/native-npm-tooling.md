# Shared native npm tooling

Ferromark's Node build uses Ferramenta's `ferramenta-native` development package.
`node/native-npm.config.mjs` names the facade, platform packages, binary filenames
and license files. Run `pnpm native:generate` and `pnpm native:check` in `node`
to regenerate or verify the platform manifests and standalone runtime. The
existing npm names, public API, 3.x Ferriki compatibility build, PGO training,
diagnostic-build isolation and release source checks remain supported.

The development dependency pins the published `ferramenta-native@0.1.0` registry
version. The initial bootstrap archive has been removed. No path to a sibling
checkout is required. The end-user facade contains the generated loader and has
no dependency on the tooling package.

## Verification of this migration

The local macOS arm64 checks built all three engines independently, packed each
facade and its host sidecar, and installed all six tarballs together offline with
install scripts disabled. One Node process composed Ferromark's code-renderer
callbacks with a Ferriki highlighter and a Ferrigura renderer. It produced
highlighted, escaped code and an SVG diagram. Each product's separate packed
consumer check also rejects a missing binary.

Ferromark's complete Rust tests, Clippy, formatting, benchmark build, 222 Node
tests, panic containment, input conversion and isolated plugin checks passed.
Ferriki's core compatibility and release recovery checks passed. Ferrigura's
Rust and Node checks and Ferramenta's complete project gate passed. The seven
platform build matrices are configured in CI; the other platforms were not
executed by this local run. This migration makes no new performance claim.

The original starting commit had two failing release contracts. Both fixes are
now included from `main`: the workspace-members assertion covers the private
plugin crates, and release rehearsal preserves registry dependency identities.
The updated infrastructure suite passes all 88 tests. No packages were published
by this migration.
