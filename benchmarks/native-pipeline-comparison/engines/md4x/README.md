# Native MD4X diagnostics

MD4X is called directly through its Zig `md_html` API. The adapter collects the
complete callback stream in a fresh C-allocator buffer and checks the renderer's
return status. There is no Node, NAPI, WASM, AST-only shortcut, or discarded
output. Upstream source is not modified.

The pinned release is **0.0.29**, commit
`b2623bb757ededc9adbc2a422ad1828a0965b2bf`; the compiler is **Zig 0.16.0**.
The build uses ReleaseFast, baseline CPU features and the upstream default
`emoji=false`. Renderer flags are zero: no healing, highlighting, generated
heading IDs or whole-document wrapper.

## Why this is diagnostic rather than a speed comparison

The released `src/abi.zig` explicitly removes the parser flag word. Tables,
strikethrough, tasks, permissive autolinks, math, frontmatter, components,
attributes, alerts, highlighting syntax and footnotes are unconditionally on.
Renderer flags cannot disable those parser features.

Our current lanes require independently selectable CommonMark, tables,
strikethrough and task lists. All eight requested configurations are probed and
the original effective outputs retained, but **none is admitted for timing**.
Even output-equivalent documents do not establish matching feature settings.
The driver rejects timing requests, and the coordinator independently excludes
this fixed dialect. No throughput ratio is published and no earlier MD4X
version is substituted to obtain convenient switches.

## Reproduce

```sh
git clone --branch v0.0.29 https://github.com/unjs/md4x.git /tmp/md4x
python3 benchmarks/native-pipeline-comparison/prepare-engine.py md4x \
  /tmp/md4x-build --source /tmp/md4x
python3 benchmarks/native-pipeline-comparison/run.py /tmp/md4x-build \
  /tmp/md4x-verified --verify-only
python3 benchmarks/native-pipeline-comparison/report.py /tmp/md4x-verified
```

Use new build/result directories. Preparation checks the clean exact upstream
revision, builds and lints an independent native Ferromark baseline, and hashes
sources and executables. Verification captures all 18 existing workloads, all
feature probes, and the 652 stored CommonMark examples. See the parent README
for the shared protocol and ARCH-COMP-002 for workload admission.

A future timing lane needs an explicitly reviewed shared processing contract
for this fixed dialect; changing the parser itself is not an adapter solution.

The [archived native diagnostics](../../../../docs/reports/2026-09-11-native-md4x/REPORT.md)
retain the complete output and configuration evidence for this release.
