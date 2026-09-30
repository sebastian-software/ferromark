# Ferriki 0.7.0: shared consumer contract and lifecycle diagnostic

## Identity and method

- Base: `f8ebc39a5f2d811dfd7299557808f4501adc829e`, plus this PR's fixture
  infrastructure. Parser/renderer and the lifecycle benchmark source are unchanged.
- Ferriki Rust and published Node package: 0.7.0. Rust lock: Ferroni 1.6.1.
- Rust 1.95.0, LLVM 22.1.2, aarch64-apple-darwin; macOS 27.0; Node 24.21.0.
  The processor model was unavailable in the sandbox.
- [Source identities](source-identities.json) pin the lock, benchmark, public
  consumer, authored corpus, npm lock and both peer snapshots by SHA-256.
- [Fixture protocol](../../../scripts/ferriki-compatibility/README.md) and
  [raw peer snapshots](../../../scripts/ferriki-compatibility/snapshots.json).

The local peer command is:

```sh
python3.12 scripts/ferriki-compatibility/run.py /private/tmp/ferromark-ferriki-shared-final --node
```

The consumer compiles the public example source against the unpacked Cargo
archive, preserving the repository's registry versions/checksums. It uses no
Node/N-API, HTTP/TLS or bincode dependency. The Node peer uses the published
highlighter and a default-feature addon with the public Ferromark facade.
The archive's development assets are SHA-512 pinned, with every payload checked
against the release digest and byte size before constructing the test cache.

## Functional results

All 14 code cases and 14 Markdown cases passed on both peers. After converting
Node UTF-16 offsets to UTF-8 bytes, token contents/positions, styles, theme
metadata, token types and standalone highlighting HTML agree. Scope paths agree
where exposed; the permitted synthetic-root distinction is documented and both
raw outputs are retained. There are no omitted incompatible cases.

Markdown wrappers have separate snapshots. The Node whole-block callback and
Rust per-line seam keep their existing behavior, including plain-block fallback,
metadata, VitePress normalization and error-observer differences. The Node
metadata/HAST path's unstyled spans in the title case are visible in the snapshot.
This evidence does not claim complete Markdown HTML equality.

The Rust catalog reads no payload during construction and performs no extra
asset reads on the second pass. Repeated output remains equal after allocator
reset while retaining the same renderer/highlighter. The public consumer rejects
missing assets (`AssetUnavailable`), corrupted payloads (`AssetIntegrity`), old
manifest versions and old payload versions even with a matching digest
(`AssetFormat`), and verifies escaped adapter fallback with the typed error.
Node rejects missing and old-format themes through the published wrapper/native
decoder (`ERR_ASSET`) and verifies the same escaped fallback in fresh package
copies. Its supported dual-theme result is checked separately; Rust's public
single-theme API is respected.

The five Python gate tests include deliberate offset, scope and error-code
mutations and a missing case; each regression fails instead of disappearing from
the peer comparison. The other required workspace checks, 76 Node repository
contracts and 50 Python repository tests passed locally.

CI adds independent Rust-only and shared-peer jobs and retains their logs/results
for 30 days. No production default feature, dependency, parser algorithm or
renderer output is changed by this PR.

## Current lifecycle timing

```sh
cargo bench -p ferromark --features ferriki --bench ferriki --locked -- --quick
```

This is one local Criterion quick diagnostic, run after the repository checks
finished. It uses the existing short Rust fence and custom in-memory grammar and
theme, with repository `bench` settings: optimized, fat LTO, one codegen unit.
The document is parsed outside measurement. These numbers characterize that
small adapter lifecycle and are not a standard-catalog or end-to-end benchmark.

| Phase | Criterion's reported interval |
| --- | ---: |
| Build highlighter and register custom assets | 4.1599–4.1730 µs |
| Build plus first highlighted render | 20.925–20.950 µs |
| Repeated render with reusable highlighter and renderer | 7.6543–7.6811 µs |

[Raw log](criterion.log) and per-phase [estimates and samples](criterion/) are
retained. There is no performance pass/fail threshold in CI. The initial 0.4.1
report remains historical and is not relabeled as a 0.7.0 measurement. This run
does not establish cross-version speedup, cross-platform stability, memory/RSS,
allocation counts, standard grammar/theme initialization costs, or CDN latency.
