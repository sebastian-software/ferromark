# Comparison dependency readiness

Review [candidate coverage](candidate-coverage.md) before freezing the scope of
an official campaign, then check every included dependency. The current
23-row executable inventory has 20 default rows and three optional rows; release freshness
and successful execution do not establish complete ecosystem coverage. Pin stable
releases explicitly; do not resolve "latest" while timing or mix versions between
platforms. A successful local preflight proves execution on that host. Each new
managed runner platform still needs its own resource and output checks.

## Current inventory

Latest stable releases checked on 2026-09-30 through npm, crates.io, and upstream
GitHub tags/releases. Supporting Node adapters and extensions are all at their
latest stable direct versions compatible with these projects.

| Track | Project | Current pin | Release authority |
| --- | --- | --- | --- |
| Native | pulldown-cmark | 0.13.4 | [crates.io](https://crates.io/crates/pulldown-cmark) |
| Native | md4c | v0.6.0 / `7fc1815a5eeba2af7d6120a76202bf59f3b6e6e4` | [upstream tag](https://github.com/mity/md4c/tree/v0.6.0) |
| Native | Bun MD | bun-v1.4.2 / `744846f844374847c902b5e7fd59b4342a51ef99` | [release](https://github.com/oven-sh/bun/releases/tag/bun-v1.4.2) |
| Native | OX-Content | v3.2.13 / `616cc793d4d1b2256098d9775d62a3f3baef524f` | [release](https://github.com/ubugeeei-prod/ox-content/releases/tag/v3.2.13) |
| Native | markdown-rs | markdown crate 1.0.0 | [crates.io](https://crates.io/crates/markdown) |
| Native | Comrak | 0.55.0 | [crates.io](https://crates.io/crates/comrak) |
| Native | cmark | 0.31.2 | [release](https://github.com/commonmark/cmark/releases/tag/0.31.2) |
| Native | cmark-gfm | 0.29.0.gfm.13 | [release](https://github.com/github/cmark-gfm/releases/tag/0.29.0.gfm.13) |
| Node.js | marked | 18.0.14 | [npm](https://www.npmjs.com/package/marked) |
| Node.js | markdown-it | 15.0.2 | [npm](https://www.npmjs.com/package/markdown-it) |
| Node.js | remark / unified | remark 15.0.1 | [npm](https://www.npmjs.com/package/remark) |
| Node.js | micromark | 4.0.3 | [release](https://github.com/micromark/micromark/releases/tag/4.0.3) |
| Node.js | Showdown | 2.1.0 | [npm](https://www.npmjs.com/package/showdown) |
| Node.js | commonmark.js | 0.31.2 | [npm](https://www.npmjs.com/package/commonmark) |

Additional current pins: Goldmark v2.1.6 (Go module `github.com/yuin/goldmark/v2`),
Go 1.27.1; Remarkable 2.0.1; markdown-exit 1.3.0; markdown-it-ts 1.1.2;
Sätteri 0.10.5; MD4X 0.0.30 in explicit NAPI and WASM lanes; @ox-content/napi 3.2.13.
TanStack Markdown is pinned to @tanstack/markdown 1.0.0, rechecked on 2026-10-01.
Its direct HTML entry point has no runtime dependencies. It is a documented syntax
subset, not another complete CommonMark implementation.
The [Go module and sum file](../markdown-ecosystem/goldmark/go.mod) and npm lockfile
pin the released implementations. Public API constraints and readiness of this
extension are separate from the immutable original 14-row preflight report.

The 2026-10-01 registry/release recheck found only TanStack changed among the
direct candidate/tooling pins. Its [1.0.0 release](https://github.com/TanStack/markdown/releases/tag/v1.0.0)
documents the supported APIs and adds opt-in inline extension parsing; existing
0.0.16 calls require no migration. The benchmark keeps the same direct HTML API,
trusted-content options and empty extensions. Historical readiness records and
measurements remain tied to 0.0.16; the new pin needs its own full prepare/verify.

The [2026-10-01 balanced readiness report](../../docs/reports/2026-10-01-balanced-comparison-readiness/README.md)
retains that fresh clean-clone build and all-input verification on macOS arm64,
plus a real 1,026-sample/36-control TanStack timing and archive pilot. It records
the exact tested source, updated pins and the managed campaign link. A single
local pair is not a completed homepage campaign or proof of other platforms.

Exact Node extension, conversion pipeline and transitive versions are in
[package-lock.json](../markdown-ecosystem/package-lock.json). In particular,
micromark-extension-gfm-table is 2.1.2; the other direct dependencies needed no
release update. Use `npm view PACKAGE version` to recheck every direct dependency,
not just the visible project names. Keep prereleases separate from stable
registry versions. The historical Ferromark v1 pin is an internal control and
intentionally stays fixed; it is not a homepage competitor.

## What changed in this refresh

- micromark 4.0.2 → 4.0.3 and its table extension 2.1.1 → 2.1.2. The micromark
  release fixes attention/flanking handling and changes token-list processing
  for performance. Existing CommonMark/GFM options and output classification
  must therefore be exercised with the new release.
- md4c moved from development commit `65c6c9d` to the latest released tag.
  [The intervening changes](https://github.com/mity/md4c/compare/65c6c9d72cebd9a731aaa5597414ce04d9ea5de3...7fc1815a5eeba2af7d6120a76202bf59f3b6e6e4)
  include parser, table/pipe handling, allocation error fixes, optional extensions
  and CMake changes. The benchmark keeps its existing shared syntax flags.
- OX-Content v3.2.3 → v3.2.13 includes
  [native parsing/rendering optimizations](https://github.com/ubugeeei-prod/ox-content/compare/v3.2.3...v3.2.13).
  Its public arena/parser/renderer APIs still build with the existing adapter.
  This updates the external competitor only; Ferromark's upstream import and
  attribution remain those recorded in `UPSTREAM.md`.
- Bun moved from a development commit to the released bun-v1.4.2 sources.
  Build version metadata follows that pin. Bun's required nightly-2026-07-20
  stays fixed. Highway and Bun's mimalloc fork retain the upstream native recipe
  and verified archive hashes, rather than substituting unrelated latest versions.
- Both Rust benchmark locks were refreshed with Cargo to their latest compatible
  registry dependencies. The main application lock is unchanged. A dedicated
  [native campaign lock](../native-comparison/Cargo.lock) replaces the old report
  lock as the source for current preparation and CI. Fetching and compilation
  use `--locked`; the native compilation also remains offline and checks its
  registry checksum union.

## Refresh and check

1. Verify every project and Node extension against the authorities above. Read
   release notes for parser, renderer, options, compiler and build changes.
2. Update exact npm pins with npm in `benchmarks/markdown-ecosystem`, then
   regenerate its lock with npm. Update crate pins/lock through Cargo in its
   separate native workspace. Use stable releases supported by the adapters.
3. Update native revisions, release versions and OX archive hash in
   `benchmarks/native-comparison/prepare.py`. Restore with the current
   `restore.py CACHE`, not an old report's script. Historical scripts retain
   their own sources and should never be edited to refresh a new campaign.
4. For a changed shared native workspace, generate a disposable workspace with
   `prepare.py --bun-lock OLD_SEED` without compilation, then use Cargo to
   resolve/update its lock. Copy the generated `bun/Cargo.lock` to the native
   campaign lock, review it, commit, and verify exact replay with `--lockfile`.
   Do not hand-edit lockfile entries or relax the checksum guard to pass a build.
5. Commit the intended benchmark changes in a clean worktree. Run locally:

   ```sh
   ./scripts/benchmark-comparison doctor
   ./scripts/benchmark-comparison prepare /tmp/ferromark-preflight-NEW
   ./scripts/benchmark-comparison verify /tmp/ferromark-preflight-NEW
   ```

   Use a new output directory after a failed preparation. Logs and partial
   attempts stay available for diagnosis. Run the applicable repository checks.
6. Review the recorded versions, source audit, build flags, output disagreements,
   and dependency audits before a Blacksmith verification run. Only complete
   official measurements can replace homepage figures.

Showdown 2.1.0 is still upstream's latest stable release. npm reports three
moderate advisories for that package (GHSA-rmmh-p597-ppvv,
GHSA-cr32-g25g-vxjj, GHSA-22g5-r2x5-97cx), with no available fixed release.
It is a private benchmark dependency using frozen inputs, not a dependency of
the shipped Node facade or homepage. Metadata stays disabled in its adapter.
Recheck this status on subsequent refreshes; `npm audit` currently exits nonzero.

The completed local preflight and its exact tested revision are recorded in
[the readiness report](../../docs/reports/2026-09-30-comparison-readiness/README.md).
This refresh does not replace performance figures or demonstrate Blacksmith
platform compatibility. Existing homepage values identify their original
measured library versions until a full new campaign is published.

## Direct-addition readiness

The [22-row local adapter readiness report](../../docs/reports/2026-09-30-candidate-adapter-readiness/README.md)
retains a fresh clean-clone prepare/verify on macOS arm64, all 57 inputs in both
lifecycles, real Node/NAPI/WASM tests, and short timing-path diagnostics. It verifies
execution on that host and keeps every public API difference visible. Short smokes
fail the publication contract; no official factors or platform selection changed.
The original 14-row readiness report remains immutable. Run the expanded verification
on each managed platform before allocating a complete campaign.

## Main campaign and TanStack readiness

The [20-row main readiness report](../../docs/reports/2026-09-30-main-campaign-readiness/README.md)
records a fresh macOS arm64 prepare/verify with TanStack Markdown 0.0.16 and the
all-document scoring policy. All 57 inputs execute in both lifecycles for every
selected candidate; output agreement is descriptive, never an exclusion filter.
The real TanStack timing-path smoke includes all 57 documents and remains
unpublishable. This adds execution evidence, not official speed values. The three optional
adapters retain their pinned contracts and tests; the previous
22-row readiness record remains immutable.
