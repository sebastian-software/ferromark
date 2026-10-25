# Optional Ferrigura integration

This candidate adds a private Rust adapter and an optional Node factory for
published Ferrigura 0.2.0. It extends the earlier
[native plugin host](../2026-10-25-native-code-plugins/README.md), whose retained
source hashes and no-Ferrigura scope describe the preceding implementation.
Nothing in this report represents a published addon or adapter package.

The adapter owns a reusable renderer and emits HTML SVG or intrinsic JSX. Node
selects it once through `CodeCompiler`; rendering does not call JavaScript per
diagram. Real-engine checks cover nested parsed fences, IDs across repeated
renders, literal text and CSS, JSX modules, error propagation, explicit fallback,
plugin ordering, and retained prepared documents. Both Mermaid-only and
Ferriki-plus-Ferrigura native addons were loaded successfully.

Local validation on macOS arm64, Rust 1.95.0, Node.js 24.19.0:

- 1,429 Rust tests in 103 suites and 221 regular Node tests pass, with no failures
  or ignored tests. The Node suite also verifies panic unwinding, input
  conversion, and the separate fixture-only plugin addon.
- Workspace Clippy, standalone-Ferrigura and engine-free Clippy, formatting,
  strict rustdoc, HTML-only adapter compilation, core-only compilation, and
  benchmark builds pass. Node lint, types, formatting, packaging, clean
  installation, and 16 release-package tests pass.
- Ferrigura builds stay outside package artifacts. Guard checks reject package
  output paths and unsupported PGO combinations.

## Dependency and payload cost

The [normal/build graphs](dependency-graphs.json) count unique package
name/version pairs on this host, including workspace packages:

| Native build | Name/version pairs |
| --- | ---: |
| Default compatibility addon | 134 |
| Engine-free addon | 85 |
| Ferrigura only | 153 |
| Ferriki and Ferrigura | 195 |

The default and engine-free lists exactly match the preceding host report.
The lockfile adds 59 registry entries without replacing any existing registry
name/version/checksum entry. Ferrigura 0.2.0 still uses registry Ferromark for
labels; source-distinct packages with the same name and version count once in
this table. The dependency graph is acyclic because the adapter is outside the
core. Ferrigura's separate label-parser replacement is not required by this
candidate.

Local `release-node` binaries, without PGO, measured before compression:

| Native build | Bytes |
| --- | ---: |
| Default compatibility addon | 5,848,144 |
| Ferriki and Ferrigura | 18,531,856 |
| Additional optional payload | 12,683,712 |

[Binary hashes](binary-sizes.json) identify these measured files. The optional
engine, layout, and font payload is substantial; the standard addon excludes it.
These are local binary sizes, not npm download sizes, a cross-platform estimate,
or a runtime memory measurement. No diagram rendering latency or memory
benchmark was performed. The earlier code-hook timings do not measure this
adapter or engine.

[Validation metadata](validation.json) records source hashes and completed
checks. Compressed logs and checksums are retained alongside it. See the
[integration guide](../../ferrigura.md) for use, errors, ID namespaces, engine
version, attribution, and remaining package distribution work.
