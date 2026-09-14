# Source and build provenance

This comparison freezes source revisions, rather than following moving branches.
Ferromark v1 is the local `main` resolved at the start of this task. OX-Content is
the exact original source used for the v2 fork, before cleanup or optimization.
No remote repository was created or updated.

| Engine | Frozen source | Native entry point |
| --- | --- | --- |
| Ferromark v2 | Local `a44734978cedff133f7dd7e5b3c31781d0fa6952`; core changes through `adf891ad3755f632b44a910b5e6063040400ac17` | Arena → parser → HTML renderer |
| OX-Content | [`a71a58939ffe7f154117cea026f6d6e71a139393`, v3.2.3](https://github.com/ubugeeei-prod/ox-content/tree/a71a58939ffe7f154117cea026f6d6e71a139393) | Original arena → parser → HTML renderer, profiling features off |
| Ferromark v1 | [`143ec2ce151d87d2a3d804a048014afc97733ae0`, package 0.9.0](https://github.com/sebastian-software/ferromark/tree/143ec2ce151d87d2a3d804a048014afc97733ae0) | `to_html_with_options` / retained `Renderer::render_into` |
| md4c | [`65c6c9d72cebd9a731aaa5597414ce04d9ea5de3`](https://github.com/mity/md4c/tree/65c6c9d72cebd9a731aaa5597414ce04d9ea5de3) | C `md_html`, original parser + HTML renderer + entity implementation |
| pulldown-cmark | [0.13.4](https://docs.rs/pulldown-cmark/0.13.4/pulldown_cmark/) | Native event iterator streamed directly to `html::push_html` |
| Bun native `bun_md` | [`76e9dcc6ad272a4fb1ee4a4dbbde4809201b71d1`](https://github.com/oven-sh/bun/tree/76e9dcc6ad272a4fb1ee4a4dbbde4809201b71d1/src/md) | Original `root::render_to_html_with_options` |

The original OX archive SHA-256 is
`7df34e3e2db30678981f6df838eafe3e7950e966453221ae180acfa7938f1cfd`, matching
[the initial import record](../../../UPSTREAM.md). It was freshly extracted.
V1 and V2 were exported with `git archive`, so unrelated changes in their active
working trees were excluded. Bun's existing sparse clone supplied the original
tracked source files; its two root Cargo files were restored from the pinned
commit. Only the disposable workspace membership, worker, and configure-time
metadata were added. Parser and renderer algorithms were not patched.

Bun support dependencies are mimalloc
[`6a64e1ba7f5b2130d4efccb67ec87fd0003f0f6a`](https://github.com/oven-sh/mimalloc/tree/6a64e1ba7f5b2130d4efccb67ec87fd0003f0f6a)
and Highway
[`2607d3b5b0113992fe84d3848859eae13b3b52c1`](https://github.com/google/highway/tree/2607d3b5b0113992fe84d3848859eae13b3b52c1).
The checksummed local source archives were re-extracted and rebuilt. The original
Highway strings implementation and dispatch are retained; `native.h` replaces
only WebKit platform/assertion macros, and `stack.c` supplies the cached pthread
stack boundary used by Bun's original recursion checks. `md4c_alloc.h` redirects
C allocation calls to the shared mimalloc. This is the native Markdown engine,
not the full Bun executable or a JavaScript API benchmark.

## Controlled build environment

- Apple M1 Pro, aarch64 macOS 26.6.2, AC power.
- Rust `nightly-2026-07-20`: `rustc 1.99.0-nightly (9f36de775 2026-07-19)`, LLVM 22.1.8.
- All Rust engines: `-C target-cpu=generic`, release optimization level 3,
  fat LTO, one codegen unit, panic abort; Bun's line-table debug information remains.
- C/C++: Apple clang, `-O3`; C99 md4c and C++23 Highway; no cross-language LTO or PGO.
- One executable with Bun's mimalloc for all Rust allocations and md4c C allocations.
- Three process rounds; six 40 ms windows and 60 ms warmup per engine/document/lifecycle.
  Input/process/IPC/verification costs are outside timing; fresh HTML allocation and destruction are inside.

This is a controlled common environment, **not the unmodified production build
of each package**. In particular, the common Cargo resolution was seeded from
the earlier Bun comparison lock, then frozen. Some dependencies differ from the
individual source lockfiles. These differences are not parser source changes,
but they prevent treating these numbers as a replay of prior Rust-1.95/system-
allocator results.

| Dependency | Shared measured version | V1 source lock | V2 / OX source lock |
| --- | --- | --- | --- |
| memchr | 2.8.0 | 2.8.3 | 2.8.3 |
| rustc-hash | 2.1.2 | 2.1.3 | 2.1.3 |
| smallvec | 1.15.1 | 1.15.2 | 1.16.0 |
| html-escape | 0.2.14 | 0.2.14 | — |
| compact_str | 0.10.0 | — | 0.10.0 |
| bumpalo | 3.20.3 | — | 3.20.3 |
| thiserror | 2.0.18 | — | 2.0.20 |

Bun's original lock has bumpalo 3.20.2 and serde_json 1.0.149; the shared lock
resolves 3.20.3 and 1.0.151 respectively. The complete resolved lock and raw
lock-set differences are archived. Lock-set additions/removals include unrelated
workspace/dev packages; they should not be read as a list of runtime dependency
changes. Replay uses the archived `Cargo.lock` with `--locked`.

The benchmark ran on an active desktop, not an isolated laboratory host. No
compiler or other benchmark was running during timing. Host load, power, and
thermal probe output are retained in `run.json`; macOS did not expose thermal
warning data to this process. Do not infer an absence of throttling from that
unavailable probe or treat small differences as established significance.

## Artifacts and attribution

The [harness](../../../benchmarks/native-comparison/README.md) documents effective
syntax flags, native lifecycles, output checks, and reproduction. `build.json`
records executable/source/native-library/adapter hashes and the resolved
packages. [`source-audit.json.gz`](source-audit.json.gz) records independent checks against Git objects and
the original OX archive. The measured executable SHA-256 is
`009aa0ac8ff7a17f4671af9e0af4a5c64d0f3a20f6f0edfb5ee665d47dcf48bd`.

The benchmark integration derives from Ferromark v1's native Bun comparison;
its MIT notice is retained beside the harness. Third-party engine source remains
in disposable build directories with its original notices and licenses.
Benchmark input and archived HTML retain the separate licenses and attribution
in the [broad corpus source record](../../../benchmarks/broad-comparison/README.md).
Wikipedia views are overlapping samples of four topics, not sixteen independent
articles. Repository documentation includes domain-specific syntax that is
rendered as ordinary Markdown in this shared comparison.
