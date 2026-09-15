# Initial cleanup boundary

The first import establishes a small, testable Markdown-to-HTML baseline before
porting any Ferramenta optimizations. It is not a completed Ferromark v2 release.

## Retained

Four upstream core crates: allocator, AST, parser, and HTML renderer. A small
`ferromark` facade only re-exports these APIs. The arena ownership invariants,
32-byte AST node guard, source-span mapping, recursion bounds, existing byte
scanners, and platform SIMD paths remain intact.

CommonMark, GFM, and existing optional syntax remain available: footnotes,
definition lists, math, MDX syntax, and the other parser options. The existing
HTML rendering behavior also remains, including headings, URL handling, escaping,
render hooks, callouts, code annotations, TOC, and MDX island payloads. These
options are potential later cuts, but removing them needs an explicit behavior
decision. MDX island payloads still require `serde_json`; the framework code
generators are independent and have been removed.

All retained core unit, integration, snapshot, conformance, stress, and doc tests
remain. The specification contents and known-failure lists are unchanged. In
particular, the three CommonMark examples in the GFM baseline document expected
GFM autolink differences; they are not hidden new regressions.

Seven self-contained Criterion suites and the stdin rendering example remain.
The prepass benchmark reads the exact upstream changelog from a fixture, so a
new README or release history cannot silently alter its input.

## Removed

- 23 upstream crates: incremental streaming, mdast/transfer, bindings for Node and
  WASM, Vite, SSG, docs, search, highlighting, transforms, component resolution,
  Mermaid, OG images, language servers, checkers, i18n tools, and profiling tools.
- The HTML renderer's framework generators and their framework-only snapshots.
- Profiler dependencies, features, macros, imports, and instrumentation calls.
- The external corpus benchmark and its download machinery; its measurements
  depended on separately downloaded third-party repositories.
- npm workspaces and lockfiles, website/docs assets, editors, deployment and
  release workflows, theme generation, Nix setup, and unrelated examples/tools.
- Upstream Git history, remotes, release metadata, badges, and publishing setup.

The upstream cargo-fuzz tooling is not imported. The retained deterministic
mutation suite still exercises malformed Markdown in ordinary stable Rust tests.
A focused fuzz harness can be added when a port needs it.

## Mechanical changes

This section records the initial import. The RC later consolidated the core into
one published crate; see [ADR-0018](arch/ADR-0018-single-rust-crate.md).

The four crate names and imports use `ferromark_` instead of `ox_content_`.
Snapshot filenames and source metadata follow those crate names; expected
outputs are preserved. Workspace metadata describes an unpublished local
`2.0.0-dev.0` development baseline. The toolchain is pinned to Rust 1.95 and no
longer installs unrelated Rust components or a WASM target.

The dependency lock shrank from 380 packages to 98, including workspace and dev
packages. All retained registry versions and checksums match the upstream lock.
The 27 upstream crates became four core crates plus a facade. File-size figures
and validation results are recorded in [validation.md](validation.md).

No parser optimization, SIMD transplant, dependency upgrade, or Ferromark v1
compatibility layer is included in this first commit. Later work can be measured
against this clean baseline using [the port roadmap](optimization-roadmap.md).
