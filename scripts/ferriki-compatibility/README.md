# Ferriki public Rust/Node fixture contract

This lane completes the shared-fixture work from Ferromark [#462] and [#393]
and provides the executable consumer evidence requested in Ferriki [#123].
The inputs in [`cases.json`](../../tests/fixtures/ferriki/cases.json) are authored
MIT fixtures, outside the upstream mirrors. They pin Ferriki 0.7.0 and exercise
14 token cases and 14 Markdown cases: Rust/aliases, TypeScript, TSX, Markdown
embeddings, custom language/theme registrations, Unicode, CRLF and lone CR,
empty lines/blocks, plain text, unknown/missing languages, missing themes,
fence metadata, VitePress annotations, and multiple blocks/documents.

The two peers use the same standard asset release. The Rust consumer uses the
repository's locked dependency versions (including Ferroni 1.6.1); the Node peer
uses the integrity-pinned published `@ferriki/core` and platform binary at 0.7.0.
Neither highlighter is replaced with a fake. Grammar/theme payloads are fetched
for development/CI only and are not added to Ferromark's published packages.

## Run

Prerequisites: the repository Rust toolchain, Python 3.11+, and network access
for the one integrity-pinned npm asset archive. Node 22.13+ is needed only for
the peer comparison. Use a fresh output directory; previous evidence is never
overwritten.

```sh
cargo fetch --locked
python3 scripts/ferriki-compatibility/run.py /tmp/ferriki-rust-contract

npm ci --prefix scripts/ferriki-compatibility --ignore-scripts --no-audit --no-fund
python3 scripts/ferriki-compatibility/run.py /tmp/ferriki-peer-contract --node
```

On a Mac with an older system Python, invoke the installed `python3.12` instead.
The Rust-only command does not invoke Node, compile N-API, or require an npm
installation. Setup downloads a fixed archive, verifies SHA-512 and each
payload's release SHA-256/size, then the tests run against local assets.
The Rust consumer is compiled from the **unpacked Cargo archive** with public
APIs only. Its dependency resolution must retain the repository's registry
versions/checksums and contain no N-API, network/TLS, or bincode dependency.
The Node peer builds the normal addon with default features and uses a temporary
copy of the public facade. It does not change local package binaries.

CI retains stdout, logs, output and summary JSON for 30 days. The output also
contains the Cargo archive and standalone consumer for inspecting the actual
packaging boundary. `snapshots.json` stores both peers' raw output, including
representative HTML, scope paths, theme metadata, token types and errors.
Actual offset/scopes/error mutations are rejected by the Python contract tests.

## Explicit comparison policy

- Rust exposes UTF-8 byte offsets; Node exposes UTF-16 code-unit offsets. The
  Node lane first verifies each original offset selects its token content, then
  converts it to bytes. The Rust lane verifies its offsets against the original
  input independently, including astral Unicode and CRLF.
- Code tokens, styles, theme colors/name, token types and standalone highlighting
  HTML must agree. Every scope path exposed by Rust must agree exactly. When
  style-token merging loses a single scope path, or for plain text, the Node
  facade inserts a synthetic root scope. The comparison permits exactly the
  pinned root for that language; raw snapshots retain the distinction.
- Markdown HTML has **separate** peer snapshots. The existing Node callback
  owns whole-block replacement; the Rust adapter preserves Ferromark wrappers,
  titles, line numbers and annotations. Language-less/indented blocks remain
  plain on Rust and use Node's text fallback highlighter. VitePress fence syntax
  is normalized by the Rust seam; the old Node callback receives the original
  language and can fall back. Unknown-language callbacks are suppressed by the
  Rust adapter and reported as `ERR_UNSUPPORTED` by Node. These differences are
  visible and do not get normalized away or excluded from the corpus.
- The Node metadata/HAST path currently emits different wrapper classes and
  unstyled token spans for the `title="a&b.rs"` case; its raw snapshot exposes
  that existing behavior. This lane establishes compatibility evidence, not
  equality of the two intentionally different Markdown hook contracts or an
  expansion of Node's metadata support.
- Rust's public API is single-theme. Node's supported dual-theme output is
  checked separately for both variants and the dark CSS variables, rather than
  importing Ferriki's private multi-theme bridge into the Rust consumer.

The Rust lane asserts lazy catalog construction, no extra asset reads on a
second pass, and equal output after allocator resets with the same renderer and
highlighter. It rejects missing payloads, corrupt digests, old manifest formats
and old payload formats even with a matching digest, and observes escaped
Ferromark fallback for an asset failure. Node exercises missing/old-format
payloads through the published wrapper and native decoder in fresh package
copies, with typed errors and escaped fallback. Installed packages remain intact.

## Update deliberately

```sh
python3 scripts/ferriki-compatibility/run.py /tmp/ferriki-reviewed-update --node --update-snapshots
```

Review both raw peer snapshots and the comparison policy when updating. Pin the
Rust lock, npm lock, `asset-source.json` and fixture version together for a
Ferriki upgrade. A snapshot update is not permission to silently accept a new
scope, offset, escaping or fallback discrepancy. These checks do not establish
runtime performance, allocation costs, or live CDN behavior. Lifecycle timing is
kept in the existing Criterion benchmark and separate measured reports.

[#462]: https://github.com/sebastian-software/ferromark/issues/462
[#393]: https://github.com/sebastian-software/ferromark/issues/393
[#123]: https://github.com/sebastian-software/ferriki/issues/123
