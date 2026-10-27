# Shared native npm tooling

Ferromark's Node build uses Ferramenta's `ferramenta-native` development package.
`node/native-npm.config.mjs` names the facade, platform packages, binary filenames
and license files. Run `pnpm native:generate` and `pnpm native:check` in `node`
to regenerate or verify the platform manifests and standalone runtime. The
existing npm names, public API, 3.x Ferriki compatibility build, PGO training,
diagnostic-build isolation and release source checks remain supported.

The development dependency pins a full Ferramenta Git commit through pnpm's
`path:/packages/native` selector. A normal `pnpm install` or frozen install in
`node` fetches the ready-to-run helpers automatically. No toolkit registry
release, submodule setup, sibling checkout or build approval is needed.
The manifest and lockfile own the exact source pin. To adopt a toolkit fix,
update that pin and regenerate the lockfile and generated files.

The same dependency owns archive checks, clean-consumer installation, artifact
receipts, release metadata checks, exact-commit CI decisions and npm release
ordering and verification. Local scripts are entry points; product policy lives
in `node/native-npm.config.mjs`. CI calls the package-local build and release
Actions after installing the pinned dependency. The same release Action rehearses
all packages before merge and publishes only with `publish: "true"`.

Native artifacts include the checked-out source commit and SHA-256 digest. All
seven receipts are verified before any binary is promoted. Packed consumers run
with install scripts disabled and must fail after their native binary is removed.
The release also exercises the x64 musl archives on Alpine. `llvm-tools` and
`FERROMARK_PGO` still train six targets; Windows ARM64 keeps its plain build.

Release retry tooling and configuration come from the immutable workflow commit,
with package paths rebased to the release source. Existing releases through the
configured legacy cutoff retain main-dispatched recovery. Later releases require
tag-dispatched recovery so npm provenance identifies the release commit. Product
OIDC permissions, Rust publishing and functional tests remain in this repository.

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
The updated infrastructure suite passes all 88 tests. No engine packages were published
by this migration.
