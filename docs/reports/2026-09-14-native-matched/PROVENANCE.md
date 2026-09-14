# Source and build provenance

All source revisions were frozen before measurement. V1 and v2 were resolved
from the local repositories, exported with Git archives, and compiled together
with the original pinned competitor cores. Dirty working trees are excluded.
No parser/renderer source was patched for the benchmark.

| Engine | Frozen source | Native entry point |
| --- | --- | --- |
| Ferromark v2 | Local `33c216bf799054912fdf5f014b7cde41814320e5` | Arena → parser → HTML renderer |
| Ferromark v1 | Local `4e151415a15c67e9d3735f719b0bed3e25d8cff8`, package 0.9.0 | `to_html_with_options` / `Renderer::render_into` |
| OX-Content | `a71a58939ffe7f154117cea026f6d6e71a139393`, package 3.2.3 | Original arena → parser → HTML renderer |
| md4c | `65c6c9d72cebd9a731aaa5597414ce04d9ea5de3` | C `md_html`, original HTML callback implementation |
| pulldown-cmark | Registry 0.13.4, checksum in Cargo.lock | Native event iterator → `html::push_html` |
| Bun native `bun_md` | `76e9dcc6ad272a4fb1ee4a4dbbde4809201b71d1` | `root::render_to_html_with_options` |

The OX archive is the original import archive, SHA-256
`7df34e3e2db30678981f6df838eafe3e7950e966453221ae180acfa7938f1cfd`.
It was re-extracted, not copied from an edited fork. Bun's cached sparse clone
was checked against pinned Git blobs; only the disposable workspace membership,
worker, and configure-time metadata were added.

The independent source audit checks 35 v1 files, 372 v2 files, 3,495 Bun files,
10 md4c files, and 393 OX files, plus both native support archives. All match.
See [source-audit.json.gz](source-audit.json.gz). Pulldown and the other registry
dependencies use the exact checksums in the retained lock.

## Common build conditions

- Local macOS arm64 workstation on AC power. This run records `arm64`; the
  earlier host record identified the machine as Apple M1 Pro. CPU and thermal
  isolation are not claimed; host/load/probe output is retained in `run.json`.
- Rust `nightly-2026-07-20`, rustc 1.99.0-nightly, LLVM 22.1.8, used by every
  Rust engine. Generic AArch64 CPU, opt-level 3, fat LTO, one codegen unit,
  panic abort; Bun's line-table debug information retained.
- C/C++ use clang `-O3`, C99 for md4c and C++23 for Highway. Highway retains
  native runtime dispatch. No cross-language LTO or PGO.
- One executable, Bun's mimalloc for Rust and redirected md4c C allocations.
  This controls allocator differences; it is not a stock system-malloc build.
- All 67 registry packages use the exact prior comparison's shared lock:
  SHA-256 `c207bd5aa59eed548e98b03907fe01071480050fed0fe2869b47b58386910040`.
  Build used `--offline --locked`; no dependency was updated. Shared versions
  can differ from each engine's own lock, as documented in the
  [original lock comparison](../2026-09-14-native-engines/PROVENANCE.md#controlled-build-environment).

Bun support remains the original mimalloc and Highway code with the existing
standalone stack boundary/platform adapter. It is the native Markdown core,
not the full Bun runtime or JavaScript-facing `Bun.markdown.html()` API.
The complete native-source details and upstream links remain in the
[original source record](../2026-09-14-native-engines/PROVENANCE.md).

No compiler ran concurrently with timed measurements. The source audit and
workspace tests completed before the timed run. The workstation is shared;
the preserved round/window ranges matter when judging small differences.

The measured executable SHA-256 is
`e674872e6c369e3f696cc33fb90f072f8cd67ddd0a48459457ae5e506d79fd12`.
[Build metadata](build.json) records source trees, native libraries, worker,
adapter, dependency, and binary hashes. The worker and harness used by this
report are frozen under [harness](harness/worker.rs), with source locks and
native adapter notices retained. The build emitted the same pre-existing md4c
C99 typedef warning and unused Bun profile-entry warning as the prior run;
neither required a source change.

Benchmark inputs, outputs, and their original attributions retain the separate
licenses documented by the [broad corpus](../../../benchmarks/broad-comparison/README.md).
The parser's MIT license does not replace third-party corpus or engine licenses.
