# Ferriki public Rust/Node fixture contract

This lane exercises the shared authored fixtures from Ferromark [#462] and
[#393] against the published Ferriki 0.10.0 crates and Node package, providing
the executable consumer evidence requested in Ferriki [#123]. The cases pin
that release and cover Rust/aliases, TypeScript, TSX, Markdown embeddings,
custom language/theme registrations, Unicode, CRLF and lone CR, empty lines and
blocks, plain text, unknown and missing languages, missing themes, fence
metadata, VitePress annotations, and multiple blocks/documents.

The Rust peer retains its public token, UTF-8 offset, scope, token-type, asset,
and lifecycle contract. Ferriki 0.10.0's Node API exposes HTML highlighting;
this lane does not call removed tokenization APIs or synthesize Node tokens or
HAST. The Node peer uses the same published release's public HTML and Markdown
adapter APIs. Its package archive pins the catalogs and release manifest; the
release commit pins CDN payloads by digest. Payloads are downloaded into a
fresh output directory and checked against the manifest's SHA-256 and size
before either consumer reads them.

## Run

Prerequisites: the repository Rust toolchain, Python 3.11+, Node 22.13+ for the
peer comparison, and network access to npm and the Ferriki asset CDN.

```sh
cargo fetch --locked
python3 scripts/ferriki-compatibility/run.py /tmp/ferriki-rust-contract

npm ci --prefix scripts/ferriki-compatibility --ignore-scripts --no-audit --no-fund
python3 scripts/ferriki-compatibility/run.py /tmp/ferriki-peer-contract --node
```

Use a fresh output directory; previous evidence is never overwritten. The Rust
only command does not invoke Node, compile N-API, or install npm packages. It
verifies the release-pinned npm archive, downloads each release-manifest asset,
and stores verified payloads both in their manifest paths and in the digest
cache. The Rust consumer compiles from the **unpacked Cargo archive** using
public APIs. Its dependency resolution must retain the repository's locked
registry versions/checksums and contain no N-API, network/TLS, or bincode
dependency. The Node peer builds the normal Ferromark addon and uses the
integrity-pinned published Ferriki package with remote downloads disabled and
the verified digest cache supplied explicitly.

CI retains stdout, logs, output, and summary JSON for 30 days. The output also
contains the Cargo archive and standalone consumer for inspecting the actual
packaging boundary. `snapshots.json` keeps the frozen Rust token output and
independent raw Rust/Node HTML output, errors, and reuse results.

## Comparison policy

- Rust snapshots remain the evidence for token contents, byte offsets, scope
  paths, token types, style metadata, and tokenization HTML. The Rust consumer
  independently checks offsets against the original UTF-8 source, including
  astral Unicode and CRLF. A Ferriki upgrade does not silently rewrite these
  snapshots.
- The Node peer records only public `codeToHtml` and Markdown HTML results,
  public errors, dual-theme HTML, and reuse behavior. No Node token offsets,
  scopes, token types, or HAST are invented or compared.
- For standalone highlighting, both peers' HTML must decode to the authored
  code text. Unsupported language/theme errors map to Node's public
  `ERR_UNSUPPORTED`. Exact renderer markup and styles remain separately pinned
  in each peer's raw snapshot.
- Markdown HTML also has separate exact snapshots. The contract compares its
  visible text and matching public errors. The Rust adapter deliberately
  suppresses unsupported-language and VitePress fence-metadata callbacks so
  Ferromark can preserve its own fallback and annotations; the Node adapter
  reports `ERR_UNSUPPORTED` for those two cases. This difference is explicit.
- The only reviewed 0.7-to-0.10 Markdown HTML change is `fence-metadata`:
  0.10 now emits its Shiki class and token colors, where the prior Node snapshot
  had unstyled spans. The other 13 Markdown HTML/error cases and all 14
  standalone HTML/error cases remain identical.
- The Node-only dual-theme HTML check verifies the public dark CSS variables;
  Rust's public API accepts one theme at a time, so this is not a parity claim.

The Rust lane asserts lazy catalog construction, no extra asset reads on a
second pass, and equal output after allocator resets with the same renderer and
highlighter. It rejects missing payloads, corrupt digests, old manifest
formats, and old payload formats even with a matching digest, and observes
escaped Ferromark fallback for an asset failure. The Node lane checks the
published offline cache contract with one missing and one corrupt cached
payload. Both failures must surface as `ERR_ASSET` and preserve escaped fallback;
the verified cache must continue to highlight normally. Neither peer performs
runtime performance, allocation, or live-CDN measurements.

## Updating the published peer

```sh
python3 scripts/ferriki-compatibility/run.py /tmp/ferriki-reviewed-update --node --update-node-snapshot
```

The update command first checks the Rust output against its frozen snapshot and
updates only the Node snapshot. Review public HTML, escaping, errors, and the
comparison policy before accepting that update. Pin the Ferriki Cargo lock,
npm lock, `asset-source.json`, and fixture version together for an upgrade.
Rust output changes require separate review and must not be accepted by the
Node snapshot update command.

[#462]: https://github.com/sebastian-software/ferromark/issues/462
[#393]: https://github.com/sebastian-software/ferromark/issues/393
[#123]: https://github.com/sebastian-software/ferriki/issues/123
