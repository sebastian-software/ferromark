# Upstream provenance

Ferromark v2 starts from the MIT-licensed OX-Content source snapshot below.
It has a new Git history; the original project is credited here and in LICENSE.

- Repository: https://github.com/ubugeeei-prod/ox-content
- Revision: `a71a58939ffe7f154117cea026f6d6e71a139393`
- Release / commit subject: `v3.2.3` / `chore(release): v3.2.3`
- Commit time: `2026-09-13T15:49:22Z`
- Import date: `2026-09-13`
- Archive: https://api.github.com/repos/ubugeeei-prod/ox-content/tarball/a71a58939ffe7f154117cea026f6d6e71a139393
- SHA-256 of the downloaded archive: `7df34e3e2db30678981f6df838eafe3e7950e966453221ae180acfa7938f1cfd`

The commit identifies the source even if GitHub later changes archive compression
or packaging. No upstream `.git` directory was copied or created during cleanup.

| Original path | Local path |
| --- | --- |
| `crates/ox_content_allocator/` | `src/allocator/` |
| `crates/ox_content_ast/` | `src/ast/` |
| `crates/ox_content_parser/` | `src/parser/` |
| `crates/ox_content_renderer/` | `src/renderer/` |
| `CHANGELOG.md` | `benches/fixtures/upstream-changelog.md` |
| `LICENSE` | `LICENSE` (unchanged) |

The changelog is retained solely as frozen input for the existing prepass
benchmark. Its upstream history is fixture data, not this project's changelog.
`rustfmt.toml`, `clippy.toml`, workspace lint settings, and build profile settings
also derive from upstream. See [the cleanup record](docs/fork.md) for local changes.

`Cargo.lock` was pruned from the imported lockfile. Every retained registry package
has the same version and checksum as upstream; no dependency upgrades were mixed
into the import. At import, the workspace packages were renamed, carried `2.0.0-dev.0`,
and all set `publish = false`. Today the root `ferromark` package is published to
crates.io (`publish = ["crates-io"]`) and `node/native` is the only workspace
package that keeps `publish = false`.

CommonMark and GFM specification text has separate CC-BY-SA 4.0 attribution in
[the fixture README](tests/spec_fixtures/README.md).
The original MIT copyright notice remains intact. Upstream references in source
comments and literal HTML such as `data-ox-island` are intentionally retained when
changing them could obscure provenance or change rendering behavior.
