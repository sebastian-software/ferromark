# Native code plugin validation

This local candidate adds `CodeCompiler`, Rust factory selection for HTML and
JSX, and an optional Ferriki compatibility factory. A separate unpublished Rust
crate supplies the fixture plugin. The isolated Node verification builds and
loads it without Ferriki, checks ordered replacement and fallback, and retains
plugin state through prepared handles after garbage collection of the compiler.

The standard 3.x addon still includes Ferriki. Its legacy `JsxCompiler` delegates
to the same host. Selecting an empty plugin list avoids initialization; only an
engine-free build removes the dependency. No Ferrigura factory or external
adapter package is included in this step. See the
[guide](../../native-code-plugins.md) and
[decision](../../arch/ADR-0024-native-code-plugins.md).

Validation on macOS arm64, Rust 1.95.0, Node.js 24.19.0:

- 1,424 Rust tests across 100 suites; 221 Node tests; no failures or ignored tests.
- Workspace Clippy, engine-free Clippy, formatting, strict rustdoc, and benchmark
  builds pass. Node typecheck, lint, formatting, packaging, and clean installation
  pass. The package/manifest suite passes 16 tests.
- Node tests also run the panic-unwind, input-conversion, and fixture-only native
  plugin verification scripts. Diagnostic builds do not replace package artifacts.
- The main npm archive is 130,447 bytes unpacked and contains no native binary.
  The facade/declaration cap increases from 124,000 to 131,000 bytes for both the
  code-renderer interfaces and native plugin API. No npm dependency is added.
- The audit reports no high or critical findings and one existing moderate finding.

The [normal/build dependency graphs](dependency-graphs.json) contain 134 unique
packages for the compatibility addon and 85 for the engine-free addon on this
host. These counts include workspace packages and do not measure compile time,
binary size, memory, or rendering latency. The native host has not been timed;
the earlier [code-hook characterization](../2026-10-25-code-block-hooks/README.md)
predates it.

[Validation metadata](validation.json) identifies the uncommitted candidate by
source hashes. Compressed command logs and checksums are retained here. Other
platforms and published PGO binaries were not exercised locally.
